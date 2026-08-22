use crate::core::types::{KglanceState, MarkdownState};
use crate::parsers::markdown::{Block, estimated_block_height, extract_toc, flatten_inlines};
use std::collections::HashMap;

pub fn compute_block_y_offsets(
    blocks: &[Block],
    font_size: f32,
    image_sizes: &HashMap<usize, (u32, u32)>,
) -> (Vec<f32>, f32) {
    let mut offsets = Vec::with_capacity(blocks.len());
    let mut y: f32 = 15.0;
    for (i, block) in blocks.iter().enumerate() {
        offsets.push(y);
        y += estimated_block_height(block, font_size, i, image_sizes);
    }
    (offsets, y)
}

pub fn populate_state(state: &mut KglanceState, blocks: &[Block]) {
    let fs = state.font_size;
    let full_text: String = blocks
        .iter()
        .map(|b| match b {
            Block::Heading { content, .. } | Block::Paragraph(content) => flatten_inlines(content),
            Block::CodeBlock { code, .. } => code.clone(),
            Block::Math(code) => code.clone(),
            _ => String::new(),
        })
        .collect::<Vec<_>>()
        .join(" ");

    let words = full_text.split_whitespace().count();
    let chars = full_text.chars().count();
    let mins = (words as f32 / 200.0).ceil() as usize;

    let old_scroll_y = state.markdown.scroll_y;
    let old_toc_visible = state.markdown.toc_visible;
    let old_collapsed = std::mem::take(&mut state.markdown.collapsed_headings);
    let old_mermaid = std::mem::take(&mut state.markdown.cached_mermaid_handles);
    let old_image_h = std::mem::take(&mut state.markdown.cached_image_handles);
    let old_image_s = std::mem::take(&mut state.markdown.cached_image_sizes);

    let old_sidebar_w = state.markdown.sidebar_width;

    // Compute cumulative Y offsets for virtual rendering (one pass over blocks).
    let (block_y_offsets, total_content_height) = compute_block_y_offsets(blocks, fs, &old_image_s);

    state.markdown = MarkdownState {
        toc: extract_toc(blocks, fs, &old_image_s),
        toc_visible: old_toc_visible,
        sidebar_width: if old_sidebar_w > 0.0 {
            old_sidebar_w
        } else {
            220.0
        },
        sidebar_resizing: false,
        sidebar_drag_start_x: None,
        sidebar_drag_start_width: 220.0,
        collapsed_headings: old_collapsed,
        scroll_y: old_scroll_y,
        cached_mermaid_handles: old_mermaid,
        cached_image_handles: old_image_h,
        cached_image_sizes: old_image_s,
        word_count: words,
        char_count: chars,
        reading_time_mins: mins,
        block_y_offsets,
        total_content_height,
        viewport_height: 800.0,
        search_visible: false,
        search_query: String::new(),
        search_match_count: 0,
        search_match_index: 0,
        search_match_blocks: Vec::new(),
        search_info: String::new(),
        selected_text: None,
        selection_range: None,
        is_dragging_selection: false,
        is_mouse_held: false,
        auto_scroll_delta: None,
        drag_last_y: 0.0,
    };

    for (i, block) in blocks.iter().enumerate() {
        if let Block::Mermaid {
            rendered: Some(png),
            ..
        } = block
        {
            let handle = match image::load_from_memory(png) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let (w, h) = rgba.dimensions();
                    iced::widget::image::Handle::from_rgba(w, h, rgba.into_raw())
                }
                Err(_) => iced::widget::image::Handle::from_bytes(png.clone()),
            };
            state.markdown.cached_mermaid_handles.insert(i, handle);
        }
    }
    state.file_type_text = "Markdown Document".to_string();
}
