use crate::core::config::EpubReadingMode;
use crate::core::types::{EpubChapterInfo, EpubState, KglanceState, MarkdownState};
use crate::parsers::markdown::Block;
use std::collections::HashMap;

pub fn ensure_blocks_images(
    markdown_state: &mut MarkdownState,
    blocks: &[Block],
    images: &HashMap<String, Vec<u8>>,
) {
    for (i, block) in blocks.iter().enumerate() {
        if markdown_state.cached_image_handles.contains_key(&i) {
            continue;
        }

        if let Block::Image { path: img_path, .. } = block {
            let filename = std::path::Path::new(img_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(img_path);
            if let Some(bytes) = images.get(img_path).or_else(|| images.get(filename)) {
                let handle = iced::widget::image::Handle::from_bytes(bytes.clone());
                markdown_state.cached_image_handles.insert(i, handle);
                if let Ok(reader) =
                    image::ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format()
                    && let Ok(dims) = reader.into_dimensions()
                {
                    markdown_state.cached_image_sizes.insert(i, dims);
                }
            }
        }
    }
}

pub fn ensure_chapter_images(
    markdown_state: &mut MarkdownState,
    chapters: &[EpubChapterInfo],
    active_chapter: usize,
    images: &HashMap<String, Vec<u8>>,
) {
    let Some(chapter) = chapters.get(active_chapter) else {
        return;
    };

    let chapter_offset: usize = chapters
        .iter()
        .take(active_chapter)
        .map(|ch| ch.blocks.len())
        .sum();

    for (i, block) in chapter.blocks.iter().enumerate() {
        let block_global_idx = chapter_offset + i;
        if markdown_state
            .cached_image_handles
            .contains_key(&block_global_idx)
        {
            continue;
        }

        if let Block::Image { path: img_path, .. } = block {
            let filename = std::path::Path::new(img_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(img_path);
            if let Some(bytes) = images.get(img_path).or_else(|| images.get(filename)) {
                let handle = iced::widget::image::Handle::from_bytes(bytes.clone());
                markdown_state
                    .cached_image_handles
                    .insert(block_global_idx, handle);
                if let Ok(reader) =
                    image::ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format()
                    && let Ok(dims) = reader.into_dimensions()
                {
                    markdown_state
                        .cached_image_sizes
                        .insert(block_global_idx, dims);
                }
            }
        }
    }
}

pub fn build_continuous_blocks(chapters: &[EpubChapterInfo]) -> (Vec<Block>, Vec<usize>) {
    let mut all_blocks = Vec::new();
    let mut chapter_block_offsets = Vec::with_capacity(chapters.len());

    for (ch_idx, ch) in chapters.iter().enumerate() {
        let start_idx = all_blocks.len();
        chapter_block_offsets.push(start_idx);

        if ch.blocks.is_empty() {
            continue;
        }

        let first_is_heading = matches!(ch.blocks.first(), Some(Block::Heading { .. }));
        if !first_is_heading && !ch.title.is_empty() {
            if ch_idx > 0 {
                all_blocks.push(Block::HorizontalRule);
            }
            all_blocks.push(Block::Heading {
                level: ch.level.clamp(1, 6),
                content: vec![crate::parsers::markdown::Inline::Text(ch.title.clone())],
            });
        }

        all_blocks.extend(ch.blocks.iter().cloned());
    }

    (all_blocks, chapter_block_offsets)
}

pub fn populate_state(
    state: &mut KglanceState,
    title: &str,
    author: &str,
    chapters: &[EpubChapterInfo],
    active_chapter: usize,
    images: &HashMap<String, Vec<u8>>,
) {
    let reading_mode = state.epub_reading_mode;
    let old_sidebar = state.epub.sidebar_visible;
    let old_scroll = state.epub.scroll_y;
    let old_collapsed = std::mem::take(&mut state.epub.collapsed_chapters);
    let mut markdown_state = MarkdownState::default();

    let (loaded_chapters, chapter_block_offsets, effective_blocks) = match reading_mode {
        EpubReadingMode::Continuous => {
            let mut chs = chapters.to_vec();
            let mut imgs = images.clone();
            if chs.iter().any(|c| c.blocks.is_empty())
                && !state.file_name.is_empty()
                && let Ok((full_chs, new_imgs)) =
                    crate::features::epub::parser::load_all_chapters_and_images_from_epub(
                        std::path::Path::new(&state.file_name),
                        chapters,
                    )
            {
                chs = full_chs;
                for (k, v) in new_imgs {
                    imgs.insert(k, v);
                }
            }

            let (all_blocks, offsets) = build_continuous_blocks(&chs);
            ensure_blocks_images(&mut markdown_state, &all_blocks, &imgs);
            let content_width = state.epub_content_width();
            crate::features::markdown::recompute_markdown_layout(
                &mut markdown_state,
                &all_blocks,
                state.font_size,
                content_width,
            );
            (chs, offsets, all_blocks)
        }
        EpubReadingMode::SingleChapter => {
            ensure_chapter_images(&mut markdown_state, chapters, active_chapter, images);
            let active_blocks = chapters
                .get(active_chapter)
                .map(|ch| ch.blocks.as_slice())
                .unwrap_or(&[]);
            let content_width = state.epub_content_width();
            crate::features::markdown::recompute_markdown_layout(
                &mut markdown_state,
                active_blocks,
                state.font_size,
                content_width,
            );
            (chapters.to_vec(), Vec::new(), Vec::new())
        }
    };

    if state.window_height > 0.0 {
        markdown_state.viewport_height = state.window_height;
    } else if state.epub.markdown_state.viewport_height > 0.0 {
        markdown_state.viewport_height = state.epub.markdown_state.viewport_height;
    }

    let old_sidebar_width = state.epub.sidebar_width;
    let mut scroll_y = old_scroll;
    if scroll_y == 0.0 {
        if reading_mode == EpubReadingMode::Continuous {
            if active_chapter > 0
                && let Some(&start_block) = chapter_block_offsets.get(active_chapter)
                && let Some(&y) = markdown_state.block_y_offsets.get(start_block)
            {
                scroll_y = y;
                markdown_state.scroll_y = y;
            }
        } else if let Some(active_ch) = loaded_chapters.get(active_chapter)
            && let Some(b_idx) = crate::features::epub::parser::find_block_index(
                &active_ch.blocks,
                active_ch.anchor.as_deref(),
                Some(&active_ch.title),
            )
            && let Some(&y) = markdown_state.block_y_offsets.get(b_idx)
        {
            scroll_y = y;
            markdown_state.scroll_y = y;
        }
    }

    state.epub = EpubState {
        title: title.to_string(),
        author: author.to_string(),
        chapters: loaded_chapters,
        active_chapter,
        reading_mode,
        chapter_block_offsets,
        continuous_blocks: effective_blocks,
        sidebar_visible: old_sidebar,
        sidebar_width: if old_sidebar_width > 0.0 {
            old_sidebar_width
        } else {
            240.0
        },
        sidebar_resizing: false,
        sidebar_drag_start_x: None,
        sidebar_drag_start_width: 240.0,
        scroll_y,
        collapsed_chapters: old_collapsed,
        markdown_state,
    };
    state
        .epub
        .markdown_state
        .scroll_controller
        .apply_config(&state.scroll_config);
    state
        .epub
        .markdown_state
        .smooth_scroll
        .apply_config(&state.scroll_config);
    state.file_type_text = format!("EPUB E-Book ({} chapters)", chapters.len());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::markdown::Block;

    #[test]
    fn test_ensure_chapter_images_lazy_loads_only_active_chapter() {
        let mut images = HashMap::new();
        // 1x1 transparent PNG
        let sample_png = vec![
            137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1,
            8, 6, 0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 10, 73, 68, 65, 84, 120, 156, 99, 0, 1, 0, 0,
            5, 0, 1, 13, 10, 45, 180, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
        ];
        images.insert("img_ch0.png".to_string(), sample_png.clone());
        images.insert("img_ch1.png".to_string(), sample_png.clone());

        let chapters = vec![
            EpubChapterInfo {
                title: "Ch 0".to_string(),
                level: 1,
                anchor: None,
                file_href: "ch0.xhtml".to_string(),
                blocks: vec![Block::Image {
                    path: "img_ch0.png".to_string(),
                    alt: "Image 0".to_string(),
                    link_url: None,
                }],
            },
            EpubChapterInfo {
                title: "Ch 1".to_string(),
                level: 1,
                anchor: None,
                file_href: "ch1.xhtml".to_string(),
                blocks: vec![Block::Image {
                    path: "img_ch1.png".to_string(),
                    alt: "Image 1".to_string(),
                    link_url: None,
                }],
            },
        ];

        let mut markdown_state = MarkdownState::default();
        // Activate chapter 0
        ensure_chapter_images(&mut markdown_state, &chapters, 0, &images);
        assert!(markdown_state.cached_image_handles.contains_key(&0));
        assert!(!markdown_state.cached_image_handles.contains_key(&1));

        // Switch to chapter 1
        ensure_chapter_images(&mut markdown_state, &chapters, 1, &images);
        assert!(markdown_state.cached_image_handles.contains_key(&1));
    }

    #[test]
    fn test_build_continuous_blocks() {
        let chapters = vec![
            EpubChapterInfo {
                title: "Chapter 1".to_string(),
                level: 1,
                anchor: None,
                file_href: "ch1.xhtml".to_string(),
                blocks: vec![Block::Paragraph(vec![
                    crate::parsers::markdown::Inline::Text("Content 1".to_string()),
                ])],
            },
            EpubChapterInfo {
                title: "Chapter 2".to_string(),
                level: 1,
                anchor: None,
                file_href: "ch2.xhtml".to_string(),
                blocks: vec![Block::Paragraph(vec![
                    crate::parsers::markdown::Inline::Text("Content 2".to_string()),
                ])],
            },
        ];

        let (blocks, offsets) = build_continuous_blocks(&chapters);
        assert_eq!(offsets.len(), 2);
        assert_eq!(offsets[0], 0);
        // Chapter 1 has 1 heading block + 1 paragraph block = 2 blocks, plus a HorizontalRule before Chapter 2
        assert!(offsets[1] > offsets[0]);
        assert!(blocks.len() >= 4);
    }
}
