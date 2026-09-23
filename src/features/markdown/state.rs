use crate::core::types::{KglanceState, MarkdownState};
use crate::parsers::markdown::layout_constants as lc;
use crate::parsers::markdown::{
    Block, BlockLayout, estimated_block_height, extract_toc, flatten_inlines,
};
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
        (win_w - sb - lc::CONTENT_PADDING * 2.0).max(300.0)
    })
}

pub fn compute_block_layouts(
    blocks: &[Block],
    font_size: f32,
    image_sizes: &HashMap<usize, (u32, u32)>,
    content_width: f32,
) -> (Vec<BlockLayout>, Vec<f32>, f32) {
    let padding_v = crate::ui::theme::scale_size(
        crate::features::markdown::view::components::STYLE
            .general
            .content_padding,
        font_size,
    ) * 2.0;
    let mut layouts = Vec::with_capacity(blocks.len());
    let mut offsets = Vec::with_capacity(blocks.len());
    let mut y: f32 = 0.0;
    for (i, block) in blocks.iter().enumerate() {
        offsets.push(y);
        let est_h = estimated_block_height(block, font_size, i, image_sizes, content_width);
        layouts.push(BlockLayout::new(est_h));
        y += est_h;
    }
    (layouts, offsets, y + padding_v)
}

pub fn rebuild_block_offsets(layouts: &[BlockLayout], font_size: f32) -> (Vec<f32>, f32) {
    let padding_v = crate::ui::theme::scale_size(
        crate::features::markdown::view::components::STYLE
            .general
            .content_padding,
        font_size,
    ) * 2.0;
    let mut offsets = Vec::with_capacity(layouts.len());
    let mut y: f32 = 0.0;
    for layout in layouts {
        offsets.push(y);
        y += layout.effective_height();
    }
    (offsets, y + padding_v)
}

pub fn invalidate_markdown_geometry(state: &mut MarkdownState) {
    for layout in &mut state.block_layouts {
        layout.invalidate_measurement();
    }
}

pub fn apply_measured_block_heights(
    state: &mut MarkdownState,
    measurements: &[(usize, f32)],
    font_size: f32,
) -> f32 {
    if measurements.is_empty() || state.block_layouts.is_empty() {
        return state.scroll_y;
    }

    // 1. Identify Anchor Block (first block at or immediately before current scroll_y)
    let old_offsets = &state.block_y_offsets;
    let anchor_idx = if !old_offsets.is_empty() {
        old_offsets
            .partition_point(|&y| y <= state.scroll_y)
            .saturating_sub(1)
    } else {
        0
    };
    let old_anchor_y = old_offsets.get(anchor_idx).copied().unwrap_or(0.0);

    // 2. Batch update measured heights into BlockLayouts
    let mut changed = false;
    for &(idx, measured_h) in measurements {
        if let Some(layout) = state.block_layouts.get_mut(idx)
            && layout.measured_height != Some(measured_h)
        {
            layout.measured_height = Some(measured_h);
            changed = true;
        }
    }

    if !changed {
        return state.scroll_y;
    }

    // 3. Batched Prefix Rebuild
    let (new_offsets, new_total_h) = rebuild_block_offsets(&state.block_layouts, font_size);
    let new_anchor_y = new_offsets.get(anchor_idx).copied().unwrap_or(0.0);
    let delta = new_anchor_y - old_anchor_y;

    state.block_y_offsets = new_offsets;
    state.total_content_height = new_total_h;

    // 4. Scroll Anchoring Invariant: Anchor screen Y remains unchanged (Δ_anchor ≈ 0)
    if delta.abs() > 0.001 {
        let max_y = crate::core::scroll::max_scroll_y(new_total_h, state.viewport_height);
        let new_scroll_y = (state.scroll_y + delta).clamp(0.0, max_y);
        state.scroll_y = new_scroll_y;

        if state.scroll_controller.is_animating() {
            let cur_pos = state.scroll_controller.position_y();
            let cur_tgt = state.scroll_controller.target_y();
            state
                .scroll_controller
                .set_position_y((cur_pos + delta).clamp(0.0, max_y));
            state.scroll_controller.target_y = (cur_tgt + delta).clamp(0.0, max_y);
        } else {
            state.scroll_controller.set_position_y(new_scroll_y);
        }

        if state.smooth_scroll.is_animating {
            let cur_pos = state.smooth_scroll.position_y();
            let cur_tgt = state.smooth_scroll.target_y();
            state
                .smooth_scroll
                .set_position_y((cur_pos + delta).clamp(0.0, max_y));
            state.smooth_scroll.target_y = (cur_tgt + delta).clamp(0.0, max_y);
        } else {
            state.smooth_scroll.set_position_y(new_scroll_y);
        }
    }

    state.scroll_y
}

pub fn recompute_markdown_layout(
    state: &mut MarkdownState,
    blocks: &[Block],
    font_size: f32,
    content_width: f32,
) {
    let (layouts, offsets, total_h) =
        compute_block_layouts(blocks, font_size, &state.cached_image_sizes, content_width);
    state.toc = extract_toc(blocks, font_size, &state.cached_image_sizes, content_width);
    state.block_layouts = layouts;
    state.block_y_offsets = offsets;
    state.total_content_height = total_h;
    state.layout_content_width = content_width;
    state.layout_font_size = font_size;
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
    let (_, offsets, total_h) =
        compute_block_layouts(blocks, font_size, image_sizes, content_width);
    (offsets, total_h)
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

    // Compute cumulative Y offsets and block layouts for virtual rendering (one pass over blocks).
    let (block_layouts, block_y_offsets, total_content_height) =
        compute_block_layouts(blocks, fs, &old_image_s, content_width);

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
        block_layouts,
        block_y_offsets,
        total_content_height,
        viewport_height: if state.window_height > 0.0 {
            state.window_height
        } else if state.markdown.viewport_height > 0.0 {
            state.markdown.viewport_height
        } else {
            800.0
        },
        layout_content_width: content_width,
        layout_font_size: fs,
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
        smooth_scroll: crate::core::types::SmoothScrollState::default(),
        touchpad_tracker: crate::core::scroll::TouchpadGestureTracker::default(),
        scroll_controller: crate::core::scroll::ScrollController::default(),
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
        let padding = lc::CONTENT_PADDING * 2.0;

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
        let padding = lc::CONTENT_PADDING * 2.0;
        let w_zero = effective_content_width(None, 0.0, false, 0.0);
        assert_eq!(w_zero, 1000.0 - padding);
    }

    #[test]
    fn test_block_layout_effective_height_and_invalidation() {
        let mut layout = BlockLayout::new(50.0);
        assert_eq!(layout.effective_height(), 50.0);
        assert_eq!(layout.measured_height, None);

        layout.measured_height = Some(72.5);
        assert_eq!(layout.effective_height(), 72.5);

        layout.invalidate_measurement();
        assert_eq!(layout.effective_height(), 50.0);
        assert_eq!(layout.measured_height, None);
    }

    #[test]
    fn test_apply_measured_block_heights_scroll_anchoring() {
        let mut state = MarkdownState {
            block_layouts: vec![
                BlockLayout::new(100.0), // block 0: [0, 100)
                BlockLayout::new(100.0), // block 1: [100, 200)
                BlockLayout::new(100.0), // block 2: [200, 300)
                BlockLayout::new(100.0), // block 3: [300, 400)
                BlockLayout::new(100.0), // block 4: [400, 500)
                BlockLayout::new(100.0), // block 5: [500, 600)
            ],
            block_y_offsets: vec![0.0, 100.0, 200.0, 300.0, 400.0, 500.0],
            total_content_height: 600.0,
            scroll_y: 250.0, // viewport is scrolled into block 2
            viewport_height: 200.0,
            ..Default::default()
        };

        // Screen Y of anchor (block 2) before update:
        // anchor_screen_y = offset[2] - scroll_y = 200.0 - 250.0 = -50.0
        let anchor_screen_y_before = state.block_y_offsets[2] - state.scroll_y;

        // Suppose blocks 0 and 1 actually measured 120px each (+20px each = +40px before anchor)
        let measurements = vec![(0, 120.0), (1, 120.0)];
        let new_scroll_y = apply_measured_block_heights(&mut state, &measurements, 14.0);

        // Anchor is block 2, whose offset is now 0 + 120 + 120 = 240.0 (+40px)
        assert_eq!(state.block_y_offsets[2], 240.0);
        // new_scroll_y should be 250.0 + 40.0 = 290.0
        assert_eq!(new_scroll_y, 290.0);
        assert_eq!(state.scroll_y, 290.0);

        // Verify Invariant: Screen Y of anchor block after update must match before update
        let anchor_screen_y_after = state.block_y_offsets[2] - state.scroll_y;
        assert!(
            (anchor_screen_y_after - anchor_screen_y_before).abs() < 0.001,
            "Anchor screen Y must be preserved: before={anchor_screen_y_before}, after={anchor_screen_y_after}"
        );
    }

    #[test]
    fn test_invalidate_markdown_geometry() {
        let mut state = MarkdownState {
            block_layouts: vec![
                BlockLayout {
                    estimated_height: 100.0,
                    measured_height: Some(150.0),
                },
                BlockLayout {
                    estimated_height: 200.0,
                    measured_height: Some(250.0),
                },
            ],
            ..Default::default()
        };

        invalidate_markdown_geometry(&mut state);
        assert_eq!(state.block_layouts[0].measured_height, None);
        assert_eq!(state.block_layouts[1].measured_height, None);
        assert_eq!(state.block_layouts[0].effective_height(), 100.0);
        assert_eq!(state.block_layouts[1].effective_height(), 200.0);
    }
}
