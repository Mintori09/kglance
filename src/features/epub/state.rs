use crate::core::types::{EpubChapterInfo, EpubState, KglanceState, MarkdownState};
use std::collections::HashMap;

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
    let mut block_global_idx = 0;
    for ch in chapters {
        for block in &ch.blocks {
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
            block_global_idx += 1;
        }
    }
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
