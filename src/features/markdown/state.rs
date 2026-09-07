use crate::core::types::{KglanceState, MarkdownState};
use crate::features::markdown::view::STYLE;
use crate::parsers::markdown::{Block, estimated_block_height, extract_toc, flatten_inlines};
use std::collections::HashMap;
use std::sync::Arc;

pub fn effective_content_width(
    max_text_width: Option<f32>,
    window_width: f32,
    sidebar_visible: bool,
    sidebar_width: f32,
) -> f32 {
    max_text_width.unwrap_or_else(|| {
        let win_w = if window_width > 0.0 {
            window_width
        } else {
            1000.0
        };
        let sb = if sidebar_visible { sidebar_width } else { 0.0 };
        (win_w - sb - STYLE.general.content_padding * 2.0).max(300.0)
    })
}

pub fn recompute_markdown_layout(
    state: &mut MarkdownState,
    blocks: &[Block],
    font_size: f32,
    content_width: f32,
) {
    let (offsets, total_h) =
        compute_block_y_offsets(blocks, font_size, &state.cached_image_sizes, content_width);
    state.toc = extract_toc(blocks, font_size, &state.cached_image_sizes, content_width);
    state.block_y_offsets = offsets;
    state.total_content_height = total_h;
}

pub fn rescale_and_update_markdown_layout(
    state: &mut MarkdownState,
    blocks: &[Block],
    old_size: f32,
    new_size: f32,
    content_width: f32,
) -> f32 {
    recompute_markdown_layout(state, blocks, new_size, content_width);
    let new_scroll_y = crate::parsers::markdown::rescale_markdown_scroll_y(
        blocks,
        state.scroll_y,
        old_size,
        new_size,
        &state.cached_image_sizes,
        content_width,
    );
    state.scroll_y = new_scroll_y;
    new_scroll_y
}

pub fn compute_block_y_offsets(
    blocks: &[Block],
    font_size: f32,
    image_sizes: &HashMap<usize, (u32, u32)>,
    content_width: f32,
) -> (Vec<f32>, f32) {
    let mut offsets = Vec::with_capacity(blocks.len());
    let mut y: f32 = 15.0;
    for (i, block) in blocks.iter().enumerate() {
        offsets.push(y);
        y += estimated_block_height(block, font_size, i, image_sizes, content_width);
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

    let old_toc_visible = state.markdown.toc_visible;
    let old_collapsed = std::mem::take(&mut state.markdown.collapsed_headings);
    let old_mermaid = std::mem::take(&mut state.markdown.cached_mermaid_handles);
    let old_image_h = std::mem::take(&mut state.markdown.cached_image_handles);
    let old_image_s = std::mem::take(&mut state.markdown.cached_image_sizes);

    let old_sidebar_w = state.markdown.sidebar_width;
    let old_gen = Arc::clone(&state.markdown.generation_id);

    let sidebar_w = if old_sidebar_w > 0.0 {
        old_sidebar_w
    } else {
        220.0
    };
    let content_width = effective_content_width(
        state.max_text_width,
        state.window_width,
        old_toc_visible,
        sidebar_w,
    );

    // Compute cumulative Y offsets for virtual rendering (one pass over blocks).
    let (block_y_offsets, total_content_height) =
        compute_block_y_offsets(blocks, fs, &old_image_s, content_width);

    state.markdown = MarkdownState {
        toc: extract_toc(blocks, fs, &old_image_s, content_width),
        toc_visible: old_toc_visible,
        sidebar_width: sidebar_w,
        sidebar_resizing: false,
        sidebar_drag_start_x: None,
        sidebar_drag_start_width: 220.0,
        collapsed_headings: old_collapsed,
        scroll_y: 0.0,
        cached_mermaid_handles: old_mermaid,
        cached_image_handles: old_image_h,
        cached_image_sizes: old_image_s,
        word_count: words,
        char_count: chars,
        reading_time_mins: mins,
        block_y_offsets,
        total_content_height,
        viewport_height: 800.0,
        generation_id: old_gen,
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
            state
                .markdown
                .cached_mermaid_handles
                .insert(i, iced::widget::image::Handle::from_bytes(png.clone()));
        }
    }
    state.file_type_text = "Markdown Document".to_string();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effective_content_width_max_text_width_override() {
        // When max_text_width is set, it overrides window and sidebar calculations
        let w = effective_content_width(Some(800.0), 1600.0, true, 250.0);
        assert_eq!(w, 800.0);

        let w_narrow = effective_content_width(Some(500.0), 400.0, false, 0.0);
        assert_eq!(w_narrow, 500.0);
    }

    #[test]
    fn test_effective_content_width_sidebar_behavior() {
        let padding = STYLE.general.content_padding * 2.0;

        // Sidebar visible
        let w_with_sb = effective_content_width(None, 1200.0, true, 220.0);
        assert_eq!(w_with_sb, 1200.0 - 220.0 - padding);

        // Sidebar hidden
        let w_no_sb = effective_content_width(None, 1200.0, false, 220.0);
        assert_eq!(w_no_sb, 1200.0 - padding);
    }

    #[test]
    fn test_effective_content_width_clamping_and_fallback() {
        // Very small window respects minimum 300px
        let w_min = effective_content_width(None, 200.0, true, 100.0);
        assert_eq!(w_min, 300.0);

        // Window width 0.0 triggers 1000.0 fallback
        let padding = STYLE.general.content_padding * 2.0;
        let w_zero = effective_content_width(None, 0.0, false, 0.0);
        assert_eq!(w_zero, 1000.0 - padding);
    }
}
