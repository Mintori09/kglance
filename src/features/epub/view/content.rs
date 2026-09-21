use iced::widget::container;
use iced::{Element, Length, Padding};

use crate::app::Message;
use crate::core::types::EpubState;
use crate::features::epub::view::constants::CONTENT_SPACING;
use crate::ui::components::content_layout::scrollable_content;
use crate::ui::types::RenderContext;

pub(crate) fn build_epub_content<'a>(
    state: &'a EpubState,
    active_chapter: usize,
    ctx: &RenderContext<'_>,
    max_text_width: Option<f32>,
) -> Element<'a, Message> {
    let chapter_blocks: &[crate::parsers::markdown::Block] = state
        .chapters
        .get(active_chapter)
        .map(|ch| ch.blocks.as_slice())
        .unwrap_or(&[]);

    let chapter_offset: usize = state
        .chapters
        .iter()
        .take(active_chapter)
        .map(|ch| ch.blocks.len())
        .sum();

    const VIRTUAL_THRESHOLD: usize = 20;

    let offsets = &state.markdown_state.block_y_offsets;
    let use_virtual =
        chapter_blocks.len() > VIRTUAL_THRESHOLD && offsets.len() == chapter_blocks.len();

    let elements: Vec<Element<'a, Message>> = if use_virtual {
        let md_state = &state.markdown_state;
        let min_buf = md_state.viewport_height.max(600.0);
        let max_buf = (md_state.viewport_height * 4.0).max(3000.0);
        let buffer = (md_state.viewport_height * 2.0).clamp(min_buf, max_buf);
        const CHUNK_SIZE: usize = 32;

        let view_top = (md_state.scroll_y - buffer).max(0.0);
        let view_bottom = md_state.scroll_y + md_state.viewport_height + buffer;

        let raw_first = offsets.partition_point(|&y| y < view_top).saturating_sub(1);
        let raw_last = offsets
            .partition_point(|&y| y <= view_bottom)
            .min(chapter_blocks.len());

        let first_visible = (raw_first / CHUNK_SIZE) * CHUNK_SIZE;
        let last_visible = (raw_last.div_ceil(CHUNK_SIZE) * CHUNK_SIZE).min(chapter_blocks.len());

        let top_height = if first_visible > 0 {
            offsets[first_visible] - offsets[0]
        } else {
            0.0
        };

        let bottom_height = if last_visible < chapter_blocks.len() {
            (md_state.total_content_height - offsets[last_visible]).max(0.0)
        } else {
            0.0
        };

        let visible_count = last_visible.saturating_sub(first_visible);
        let mut els: Vec<Element<'a, Message>> = Vec::with_capacity(visible_count + 2);

        if top_height > 0.0 {
            els.push(
                iced::widget::Space::new()
                    .width(Length::Fill)
                    .height(top_height)
                    .into(),
            );
        }

        for (i, block) in chapter_blocks
            .iter()
            .enumerate()
            .skip(first_visible)
            .take(visible_count)
        {
            let global_index = chapter_offset + i;
            let block_ctx = RenderContext {
                block_index: global_index * 1000,
                selection_range: state.markdown_state.selection_range,
                drag_active: state.markdown_state.is_dragging_selection
                    || state.markdown_state.is_mouse_held,
                ..*ctx
            };
            let inner = crate::features::markdown::view::render_block(
                global_index,
                block,
                &state.markdown_state,
                &block_ctx,
            );
            let margin_bottom = crate::features::markdown::view::block_margin(block, ctx.font_size);
            els.push(
                container(inner)
                    .padding(Padding {
                        top: 0.0,
                        right: 0.0,
                        bottom: margin_bottom,
                        left: 0.0,
                    })
                    .width(Length::Fill)
                    .into(),
            );
        }

        if bottom_height > 0.0 {
            els.push(
                iced::widget::Space::new()
                    .width(Length::Fill)
                    .height(bottom_height)
                    .into(),
            );
        }

        els
    } else {
        chapter_blocks
            .iter()
            .enumerate()
            .map(|(i, block)| {
                let global_index = chapter_offset + i;
                let block_ctx = RenderContext {
                    block_index: global_index * 1000,
                    selection_range: state.markdown_state.selection_range,
                    drag_active: state.markdown_state.is_dragging_selection
                        || state.markdown_state.is_mouse_held,
                    ..*ctx
                };
                let inner = crate::features::markdown::view::render_block(
                    global_index,
                    block,
                    &state.markdown_state,
                    &block_ctx,
                );
                let margin_bottom =
                    crate::features::markdown::view::block_margin(block, ctx.font_size);
                container(inner)
                    .padding(Padding {
                        top: 0.0,
                        right: 0.0,
                        bottom: margin_bottom,
                        left: 0.0,
                    })
                    .width(Length::Fill)
                    .into()
            })
            .collect()
    };

    let content_padding = crate::ui::theme::scale_size(CONTENT_SPACING, ctx.font_size);
    scrollable_content(elements, max_text_width, content_padding, "content_scroll")
        .filter_wheel(true)
        .on_scroll(|v| {
            crate::app::messages::MarkdownMsg::Scrolled {
                y: v.absolute_offset().y,
                viewport_height: v.bounds().height,
            }
            .into()
        })
        .build()
}
