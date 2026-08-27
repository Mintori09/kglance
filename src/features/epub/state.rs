use crate::core::types::{EpubChapterInfo, EpubState, KglanceState, MarkdownState};
use std::collections::HashMap;

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

        if let crate::parsers::markdown::Block::Image { path: img_path, .. } = block {
            let filename = std::path::Path::new(img_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(img_path);
            if let Some(bytes) = images.get(img_path).or_else(|| images.get(filename)) {
                let handle = iced::widget::image::Handle::from_bytes(bytes.clone());
                markdown_state
                    .cached_image_handles
                    .insert(block_global_idx, handle);
                if let Ok(img) = image::load_from_memory(bytes) {
                    markdown_state
                        .cached_image_sizes
                        .insert(block_global_idx, (img.width(), img.height()));
                }
            }
        }
    }
}

pub fn populate_state(
    state: &mut KglanceState,
    title: &str,
    author: &str,
    chapters: &[EpubChapterInfo],
    active_chapter: usize,
    images: &HashMap<String, Vec<u8>>,
) {
    let old_sidebar = state.epub.sidebar_visible;
    let old_scroll = state.epub.scroll_y;
    let old_collapsed = std::mem::take(&mut state.epub.collapsed_chapters);
    let mut markdown_state = MarkdownState::default();

    // Lazy / On-demand: only decode images for the active chapter
    ensure_chapter_images(&mut markdown_state, chapters, active_chapter, images);

    let old_sidebar_width = state.epub.sidebar_width;
    state.epub = EpubState {
        title: title.to_string(),
        author: author.to_string(),
        chapters: chapters.to_vec(),
        active_chapter,
        sidebar_visible: old_sidebar,
        sidebar_width: if old_sidebar_width > 0.0 {
            old_sidebar_width
        } else {
            240.0
        },
        sidebar_resizing: false,
        sidebar_drag_start_x: None,
        sidebar_drag_start_width: 240.0,
        scroll_y: old_scroll,
        collapsed_chapters: old_collapsed,
        markdown_state,
    };
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
                blocks: vec![Block::Image {
                    path: "img_ch0.png".to_string(),
                    alt: "Image 0".to_string(),
                }],
            },
            EpubChapterInfo {
                title: "Ch 1".to_string(),
                level: 1,
                anchor: None,
                blocks: vec![Block::Image {
                    path: "img_ch1.png".to_string(),
                    alt: "Image 1".to_string(),
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
}
