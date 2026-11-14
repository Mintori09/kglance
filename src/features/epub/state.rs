use crate::core::config::EpubReadingMode;
use crate::core::types::{EpubChapterInfo, EpubState, KglanceState, MarkdownState};
use crate::parsers::markdown::Block;
use std::collections::{HashMap, HashSet};

fn collect_image_entries<'a>(block: &'a Block, out: &mut Vec<(&'a str, Option<u32>, Option<u32>)>) {
    match block {
        Block::Image {
            path,
            width,
            height,
            ..
        } => {
            out.push((path.as_str(), *width, *height));
        }
        Block::Paragraph(inlines)
        | Block::Heading {
            content: inlines, ..
        } => {
            collect_image_entries_from_inlines(inlines, out);
        }
        Block::List { items, .. } => {
            for item in items {
                collect_image_entries_from_inlines(&item.content, out);
                for sb in &item.sub_blocks {
                    collect_image_entries(sb, out);
                }
            }
        }
        Block::Quote(blocks)
        | Block::Alert {
            content: blocks, ..
        }
        | Block::FootnoteDefinition {
            content: blocks, ..
        } => {
            for b in blocks {
                collect_image_entries(b, out);
            }
        }
        Block::Table(table) => {
            for header in &table.headers {
                collect_image_entries_from_inlines(&header.content, out);
            }
            for row in &table.rows {
                for cell in row {
                    collect_image_entries_from_inlines(&cell.content, out);
                }
            }
        }
        _ => {}
    }
}

fn collect_image_entries_from_inlines<'a>(
    inlines: &'a [crate::parsers::markdown::Inline],
    out: &mut Vec<(&'a str, Option<u32>, Option<u32>)>,
) {
    for inline in inlines {
        match inline {
            crate::parsers::markdown::Inline::Image {
                url, width, height, ..
            } => {
                out.push((url.as_str(), *width, *height));
            }
            crate::parsers::markdown::Inline::Bold(sub)
            | crate::parsers::markdown::Inline::Italic(sub)
            | crate::parsers::markdown::Inline::Strikethrough(sub)
            | crate::parsers::markdown::Inline::Underline(sub)
            | crate::parsers::markdown::Inline::Link { text: sub, .. } => {
                collect_image_entries_from_inlines(sub, out);
            }
            _ => {}
        }
    }
}

fn is_svg_data(path: &str, bytes: &[u8]) -> bool {
    if path.ends_with(".svg") {
        return true;
    }
    let trimmed = bytes
        .iter()
        .position(|&b| !b.is_ascii_whitespace())
        .map(|i| &bytes[i..])
        .unwrap_or(bytes);
    trimmed.starts_with(b"<?xml") || trimmed.starts_with(b"<svg")
}

pub fn parse_svg_dimensions(svg_bytes: &[u8]) -> Option<(u32, u32)> {
    let header_len = svg_bytes.len().min(4096);
    let header = std::str::from_utf8(&svg_bytes[..header_len]).ok()?;
    let svg_tag_start = header.find("<svg")?;
    let rest = &header[svg_tag_start..];
    let tag_end = rest.find('>')?;
    let tag = &rest[..tag_end];

    fn extract_attr<'a>(tag: &'a str, attr: &str) -> Option<&'a str> {
        let mut search = 0;
        while let Some(pos) = tag[search..].find(attr) {
            let abs_pos = search + pos;
            let is_boundary = abs_pos == 0
                || tag.as_bytes()[abs_pos - 1].is_ascii_whitespace()
                || tag.as_bytes()[abs_pos - 1] == b':';
            let after = &tag[abs_pos + attr.len()..].trim_start();
            if is_boundary && after.starts_with('=') {
                let val_part = after[1..].trim_start();
                let quote = val_part.chars().next()?;
                if quote == '"' || quote == '\'' {
                    let rest_val = &val_part[1..];
                    let end = rest_val.find(quote)?;
                    return Some(&rest_val[..end]);
                }
            }
            search = abs_pos + attr.len();
        }
        None
    }

    fn parse_num(s: &str) -> Option<f32> {
        let trimmed = s.trim();
        let end_idx = trimmed
            .find(|c: char| !c.is_ascii_digit() && c != '.')
            .unwrap_or(trimmed.len());
        trimmed[..end_idx].parse::<f32>().ok()
    }

    if let (Some(w_str), Some(h_str)) = (extract_attr(tag, "width"), extract_attr(tag, "height"))
        && let (Some(w), Some(h)) = (parse_num(w_str), parse_num(h_str))
        && w > 0.0
        && h > 0.0
    {
        return Some((w.round() as u32, h.round() as u32));
    }

    if let Some(vb) = extract_attr(tag, "viewBox") {
        let parts: Vec<f32> = vb
            .split(|c: char| c.is_ascii_whitespace() || c == ',')
            .filter(|s| !s.is_empty())
            .filter_map(|p| p.parse().ok())
            .collect();
        if parts.len() == 4 && parts[2] > 0.0 && parts[3] > 0.0 {
            return Some((parts[2].round() as u32, parts[3].round() as u32));
        }
    }

    None
}

pub fn rasterize_svg_to_png(svg_bytes: &[u8]) -> Option<(Vec<u8>, (u32, u32))> {
    let opt = resvg::usvg::Options::default();
    let rtree = resvg::usvg::Tree::from_data(svg_bytes, &opt).ok()?;
    let pixmap_size = rtree.size().to_int_size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())?;
    pixmap.fill(resvg::tiny_skia::Color::WHITE);
    resvg::render(
        &rtree,
        resvg::usvg::Transform::default(),
        &mut pixmap.as_mut(),
    );
    let png_data = pixmap.encode_png().ok()?;
    Some((png_data, (pixmap_size.width(), pixmap_size.height())))
}

type RasterizedBlockImage = (String, Option<Vec<u8>>, Option<(u32, u32)>);

pub fn ensure_blocks_images_parallel(
    markdown_state: &mut MarkdownState,
    blocks: &[Block],
    images: &HashMap<String, Vec<u8>>,
) {
    use rayon::prelude::*;

    let mut entries = Vec::new();
    for block in blocks {
        collect_image_entries(block, &mut entries);
    }

    let mut needed: Vec<(&str, Option<u32>, Option<u32>)> = Vec::new();
    let mut seen = HashSet::new();
    for (img_path, exp_w, exp_h) in entries {
        if seen.insert(img_path) && !markdown_state.cached_image_handles.contains_key(img_path) {
            needed.push((img_path, exp_w, exp_h));
        }
    }

    let results: Vec<RasterizedBlockImage> = needed
        .into_par_iter()
        .map(|(img_path, explicit_w, explicit_h)| {
            let filename = std::path::Path::new(img_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(img_path);

            if let Some(bytes) = images.get(img_path).or_else(|| images.get(filename)) {
                if is_svg_data(img_path, bytes) {
                    if let Some((png_bytes, (w, h))) = rasterize_svg_to_png(bytes) {
                        let final_dims = (explicit_w.unwrap_or(w), explicit_h.unwrap_or(h));
                        (img_path.to_string(), Some(png_bytes), Some(final_dims))
                    } else {
                        (img_path.to_string(), None, None)
                    }
                } else {
                    let dims = if let (Some(w), Some(h)) = (explicit_w, explicit_h) {
                        Some((w, h))
                    } else if let Ok(reader) =
                        image::ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format()
                        && let Ok(d) = reader.into_dimensions()
                    {
                        Some(d)
                    } else {
                        None
                    };
                    (img_path.to_string(), Some(bytes.clone()), dims)
                }
            } else {
                (img_path.to_string(), None, None)
            }
        })
        .collect();

    for (path, img_bytes, dims) in results {
        if let Some(bytes) = img_bytes {
            let handle = iced::widget::image::Handle::from_bytes(bytes);
            if let Some(filename) = std::path::Path::new(&path)
                .file_name()
                .and_then(|n| n.to_str())
            {
                markdown_state
                    .cached_image_handles
                    .insert(filename.to_string(), handle.clone());
            }
            markdown_state
                .cached_image_handles
                .insert(path.clone(), handle);
        }
        if let Some(d) = dims {
            if let Some(filename) = std::path::Path::new(&path)
                .file_name()
                .and_then(|n| n.to_str())
            {
                markdown_state
                    .cached_image_sizes
                    .insert(filename.to_string(), d);
            }
            markdown_state.cached_image_sizes.insert(path, d);
        }
    }
}

pub fn ensure_blocks_image_sizes_fast(
    markdown_state: &mut MarkdownState,
    blocks: &[Block],
    images: &HashMap<String, Vec<u8>>,
) {
    use rayon::prelude::*;

    let mut entries = Vec::new();
    for block in blocks {
        collect_image_entries(block, &mut entries);
    }

    let mut needed: Vec<(&str, Option<u32>, Option<u32>)> = Vec::new();
    let mut seen = HashSet::new();
    for (img_path, exp_w, exp_h) in entries {
        if seen.insert(img_path) && !markdown_state.cached_image_sizes.contains_key(img_path) {
            needed.push((img_path, exp_w, exp_h));
        }
    }

    let results: Vec<(String, Option<(u32, u32)>)> = needed
        .into_par_iter()
        .map(|(img_path, explicit_w, explicit_h)| {
            if let (Some(w), Some(h)) = (explicit_w, explicit_h) {
                return (img_path.to_string(), Some((w, h)));
            }

            let filename = std::path::Path::new(img_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(img_path);

            if let Some(bytes) = images.get(img_path).or_else(|| images.get(filename)) {
                if is_svg_data(img_path, bytes) {
                    let dims = parse_svg_dimensions(bytes)
                        .map(|(w, h)| (explicit_w.unwrap_or(w), explicit_h.unwrap_or(h)));
                    (img_path.to_string(), dims)
                } else if let Ok(reader) =
                    image::ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format()
                    && let Ok(d) = reader.into_dimensions()
                {
                    (img_path.to_string(), Some(d))
                } else {
                    (img_path.to_string(), None)
                }
            } else {
                (img_path.to_string(), None)
            }
        })
        .collect();

    for (path, dims) in results {
        if let Some(d) = dims {
            markdown_state.cached_image_sizes.insert(path, d);
        }
    }
}

pub fn ensure_blocks_images(
    markdown_state: &mut MarkdownState,
    blocks: &[Block],
    images: &HashMap<String, Vec<u8>>,
) {
    ensure_blocks_images_parallel(markdown_state, blocks, images);
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

    markdown_state.cached_image_handles.clear();
    markdown_state.cached_image_sizes.clear();

    ensure_blocks_images(markdown_state, &chapter.blocks, images);
}

pub fn build_continuous_blocks(chapters: &[EpubChapterInfo]) -> (Vec<Block>, Vec<usize>) {
    let total_block_count: usize = chapters.iter().map(|ch| ch.blocks.len() + 2).sum();
    let mut all_blocks = Vec::with_capacity(total_block_count);
    let mut chapter_block_offsets = Vec::with_capacity(chapters.len());
    let mut seen_file_offsets: HashMap<&str, usize> = HashMap::new();

    for (ch_idx, ch) in chapters.iter().enumerate() {
        if !ch.file_href.is_empty()
            && let Some(&existing_offset) = seen_file_offsets.get(ch.file_href.as_str())
        {
            let local_anchor_offset = crate::features::epub::parser::find_block_index(
                &ch.blocks,
                ch.anchor.as_deref(),
                Some(&ch.title),
            )
            .unwrap_or(0);
            let target_block_idx = existing_offset + local_anchor_offset;
            chapter_block_offsets.push(target_block_idx);
            continue;
        }

        let start_idx = all_blocks.len();
        if ch.blocks.is_empty() {
            chapter_block_offsets.push(start_idx);
            continue;
        }

        if !ch.file_href.is_empty() {
            seen_file_offsets.insert(&ch.file_href, start_idx);
        }

        if ch_idx > 0
            && crate::features::epub::anchor::chapter_header_blocks_count(chapters, ch_idx) > 0
        {
            all_blocks.push(Block::HorizontalRule);
        }

        chapter_block_offsets.push(start_idx);
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
    let reading_mode = state
        .read_positions
        .get(&state.file_name)
        .and_then(|p| p.reading_mode)
        .unwrap_or(state.epub_reading_mode);
    let old_sidebar = state.epub.sidebar_visible;
    let old_scroll = state.epub.markdown_state.scroll_y;
    let old_collapsed = std::mem::take(&mut state.epub.collapsed_chapters);
    let mut old_stylesheets = std::mem::take(&mut state.epub.cached_stylesheets);
    let mut markdown_state = MarkdownState::default();

    let (loaded_chapters, chapter_block_offsets, effective_blocks) = match reading_mode {
        EpubReadingMode::Continuous => {
            let mut chs = chapters.to_vec();
            let mut imgs = images.clone();
            if chs.iter().any(|c| c.blocks.is_empty())
                && !state.file_name.is_empty()
                && let Ok((full_chs, new_imgs, full_styles)) =
                    crate::features::epub::parser::load_all_chapters_and_images_from_epub(
                        std::path::Path::new(&state.file_name),
                        chapters,
                    )
            {
                chs = full_chs;
                for (k, v) in new_imgs {
                    imgs.insert(k, v);
                }
                markdown_state.block_styles = full_styles;
            }

            let (all_blocks, offsets) = build_continuous_blocks(&chs);
            ensure_blocks_image_sizes_fast(&mut markdown_state, &all_blocks, &imgs);

            // Pre-rasterize images for initial viewport (up to VIRTUAL_THRESHOLD * 2 blocks)
            let initial_block_count = all_blocks.len().min(120);
            ensure_blocks_images(
                &mut markdown_state,
                &all_blocks[..initial_block_count],
                &imgs,
            );

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

            let cached_css = if !old_stylesheets.is_empty() {
                Some(&old_stylesheets)
            } else {
                None
            };

            if !state.file_name.is_empty()
                && let Some(ch) = chapters.get(active_chapter)
                && !ch.file_href.is_empty()
                && let Ok(((_, _, styles), parsed_css)) =
                    crate::features::epub::parser::load_chapter_content_and_images_cached(
                        std::path::Path::new(&state.file_name),
                        &ch.file_href,
                        ch.anchor.as_deref(),
                        cached_css,
                    )
            {
                markdown_state.block_styles = styles;
                if let Some(css) = parsed_css {
                    old_stylesheets = css;
                }
            }

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

    let chapter_target_offsets = if reading_mode == EpubReadingMode::Continuous {
        let mut prev_target = 0;
        let mut seen_file_hrefs = HashSet::new();
        loaded_chapters
            .iter()
            .enumerate()
            .map(|(idx, ch)| {
                let start_idx = chapter_block_offsets.get(idx).copied().unwrap_or(0);
                let target =
                    if !ch.file_href.is_empty() && !seen_file_hrefs.insert(ch.file_href.as_str()) {
                        start_idx.max(prev_target)
                    } else {
                        let local_offset = crate::features::epub::parser::find_block_index(
                            &ch.blocks,
                            ch.anchor.as_deref(),
                            Some(&ch.title),
                        )
                        .unwrap_or(0);
                        (start_idx + local_offset).max(prev_target)
                    };
                prev_target = target;
                target
            })
            .collect()
    } else {
        Vec::new()
    };

    let gallery_image_paths = if reading_mode == EpubReadingMode::Continuous {
        crate::features::epub::parser::extract_images_from_blocks(&effective_blocks)
    } else {
        let mut paths = Vec::new();
        for ch in &loaded_chapters {
            for p in crate::features::epub::parser::extract_images_from_blocks(&ch.blocks) {
                if !paths.contains(&p) {
                    paths.push(p);
                }
            }
        }
        paths
    };

    state.epub = EpubState {
        title: title.to_string(),
        author: author.to_string(),
        chapters: loaded_chapters,
        active_chapter,
        reading_mode,
        chapter_block_offsets,
        chapter_target_offsets,
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
        search: super::search::EpubSearchState::default(),
        markdown_state,
        sidebar_tab: state.epub.sidebar_tab,
        lightbox: None,
        chapter_y_offsets: Vec::new(),
        gallery_image_paths,
        cached_stylesheets: old_stylesheets,
    };
    crate::features::epub::view::sidebar::rebuild_chapter_y_offsets(&mut state.epub);
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
                    width: None,
                    height: None,
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
                    width: None,
                    height: None,
                }],
            },
        ];

        let mut markdown_state = MarkdownState::default();
        // Activate chapter 0
        ensure_chapter_images(&mut markdown_state, &chapters, 0, &images);
        assert!(
            markdown_state
                .cached_image_handles
                .contains_key("img_ch0.png")
        );
        assert!(
            markdown_state
                .cached_image_sizes
                .contains_key("img_ch0.png")
        );

        // Switch to chapter 1 (replaces cached images with chapter 1's images)
        ensure_chapter_images(&mut markdown_state, &chapters, 1, &images);
        assert!(
            markdown_state
                .cached_image_handles
                .contains_key("img_ch1.png")
        );
        assert!(
            markdown_state
                .cached_image_sizes
                .contains_key("img_ch1.png")
        );
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
        // Chapter 1 has 1 paragraph block = 1 block, plus a HorizontalRule before Chapter 2 (offset at index 1)
        assert!(offsets[1] > offsets[0]);
        assert_eq!(offsets[1], 1);
        assert_eq!(blocks.len(), 3); // ch1 para + HR + ch2 para

        // Test deduplication when chapter 2 starts with HorizontalRule
        let chapters_with_hr = vec![
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
                blocks: vec![
                    Block::HorizontalRule,
                    Block::Paragraph(vec![crate::parsers::markdown::Inline::Text(
                        "Content 2".to_string(),
                    )]),
                ],
            },
        ];

        let (blocks_dedup, offsets_dedup) = build_continuous_blocks(&chapters_with_hr);
        assert_eq!(offsets_dedup.len(), 2);
        assert_eq!(offsets_dedup[0], 0);
        assert_eq!(offsets_dedup[1], 1);
        // Only 3 blocks: ch1 para, then ch2's own HR and ch2 para (no synthetic double HR)
        assert_eq!(blocks_dedup.len(), 3);
        assert!(matches!(blocks_dedup[1], Block::HorizontalRule));

        // Test empty first chapter pointing to same file does not poison seen_file_offsets
        let chapters_empty_first = vec![
            EpubChapterInfo {
                title: "Cover / Empty Chapter".to_string(),
                level: 1,
                anchor: None,
                file_href: "ch1.xhtml".to_string(),
                blocks: Vec::new(),
            },
            EpubChapterInfo {
                title: "Chapter 1 Real".to_string(),
                level: 1,
                anchor: None,
                file_href: "ch1.xhtml".to_string(),
                blocks: vec![Block::Paragraph(vec![
                    crate::parsers::markdown::Inline::Text("Real Content".to_string()),
                ])],
            },
        ];
        let (blocks_empty, offsets_empty) = build_continuous_blocks(&chapters_empty_first);
        assert_eq!(offsets_empty.len(), 2);
        assert_eq!(offsets_empty[0], 0);
        assert_eq!(offsets_empty[1], 0);
        assert_eq!(blocks_empty.len(), 1);
    }

    #[test]
    fn test_populate_state_continuous_subchapters_no_double_offset() {
        let mut state = KglanceState {
            epub_reading_mode: EpubReadingMode::Continuous,
            ..Default::default()
        };

        let sample_blocks = vec![
            Block::Heading {
                level: 1,
                content: vec![crate::parsers::markdown::Inline::Text("Chapter 1".into())],
            }, // block 0
            Block::Paragraph(vec![crate::parsers::markdown::Inline::Text("Intro".into())]), // block 1
            Block::Heading {
                level: 2,
                content: vec![crate::parsers::markdown::Inline::Text("Section 1".into())],
            }, // block 2
            Block::Paragraph(vec![crate::parsers::markdown::Inline::Text(
                "Body 1".into(),
            )]), // block 3
            Block::Heading {
                level: 2,
                content: vec![crate::parsers::markdown::Inline::Text("Section 2".into())],
            }, // block 4
            Block::Paragraph(vec![crate::parsers::markdown::Inline::Text(
                "Body 2".into(),
            )]), // block 5
        ];

        let chapters = vec![
            EpubChapterInfo {
                title: "Chapter 1".to_string(),
                level: 1,
                anchor: None,
                file_href: "ch1.xhtml".to_string(),
                blocks: sample_blocks.clone(),
            },
            EpubChapterInfo {
                title: "Section 1".to_string(),
                level: 2,
                anchor: Some("sec1".to_string()),
                file_href: "ch1.xhtml".to_string(),
                blocks: sample_blocks.clone(),
            },
            EpubChapterInfo {
                title: "Section 2".to_string(),
                level: 2,
                anchor: Some("sec2".to_string()),
                file_href: "ch1.xhtml".to_string(),
                blocks: sample_blocks.clone(),
            },
            EpubChapterInfo {
                title: "Chapter 2".to_string(),
                level: 1,
                anchor: None,
                file_href: "ch2.xhtml".to_string(),
                blocks: vec![Block::Heading {
                    level: 1,
                    content: vec![crate::parsers::markdown::Inline::Text("Chapter 2".into())],
                }],
            },
        ];

        populate_state(
            &mut state,
            "Test Book",
            "Author",
            &chapters,
            0,
            &HashMap::new(),
        );

        // Section 1 should match anchor block 2, not double-counted (4)
        // Section 2 should match anchor block 4, not double-counted (8)
        assert_eq!(state.epub.chapter_block_offsets, vec![0, 2, 4, 6]);
        assert_eq!(state.epub.chapter_target_offsets, vec![0, 2, 4, 6]);
    }
}
