use crate::parsers::markdown::BlockLayout;
use crate::ui::components::measured::Measured;
use iced::widget::container;
use iced::{Element, Length, Padding};

use crate::app::Message;
use crate::core::types::EpubState;
use crate::features::epub::view::constants::CONTENT_SPACING;
use crate::ui::components::content_layout::scrollable_content;
use crate::ui::types::RenderContext;

pub const VIRTUAL_THRESHOLD: usize = 60;

pub(crate) fn build_epub_content<'a>(
    state: &'a EpubState,
    _active_chapter: usize,
    ctx: &RenderContext<'_>,
    max_text_width: Option<f32>,
) -> Element<'a, Message> {
    let chapter_blocks = state.active_blocks();

    let offsets = &state.markdown_state.block_y_offsets;
    let use_virtual =
        chapter_blocks.len() > VIRTUAL_THRESHOLD && offsets.len() == chapter_blocks.len();

    let content_padding = CONTENT_SPACING;

    let elements: Vec<Element<'a, Message>> = if use_virtual {
        let md_state = &state.markdown_state;
        let window = crate::features::markdown::view::window::render_range(
            &md_state.virtual_window,
            offsets,
            md_state.scroll_y,
            md_state.viewport_height,
        );
        let (first_visible, last_visible) = (window.start, window.end);

        let padding_v = content_padding * 2.0;
        let blocks_total_height = (md_state.total_content_height - padding_v).max(0.0);

        let top_height = if first_visible > 0 {
            offsets[first_visible]
        } else {
            0.0
        };

        let bottom_height = if last_visible < chapter_blocks.len() {
            (blocks_total_height - offsets[last_visible]).max(0.0)
        } else {
            0.0
        };

        let visible_count = last_visible.saturating_sub(first_visible);
        let mut els: Vec<Element<'a, Message>> = Vec::with_capacity(visible_count + 2);

        els.push(
            iced::widget::Space::new()
                .width(Length::Fill)
                .height(top_height)
                .into(),
        );

        for (i, block) in chapter_blocks
            .iter()
            .enumerate()
            .skip(first_visible)
            .take(visible_count)
        {
            let block_ctx = RenderContext {
                block_index: i * 1000,
                selection_range: state.markdown_state.selection_range,
                drag_active: state.markdown_state.is_dragging_selection
                    || state.markdown_state.is_mouse_held,
                ..*ctx
            };
            let inner = crate::features::markdown::view::render_block(
                i,
                block,
                &state.markdown_state,
                &block_ctx,
            );
            let margin_bottom = crate::features::markdown::view::block_margin(block, ctx.font_size);
            let block = container(inner)
                .padding(Padding {
                    top: 0.0,
                    right: 0.0,
                    bottom: margin_bottom,
                    left: 0.0,
                })
                .width(Length::Fill);
            els.push(
                Measured::new(
                    block,
                    i,
                    state
                        .markdown_state
                        .block_layouts
                        .get(i)
                        .map_or(0.0, BlockLayout::effective_height),
                    |block_index, height| {
                        crate::app::messages::MarkdownMsg::BlockMeasured {
                            block_index,
                            height,
                        }
                        .into()
                    },
                )
                .into(),
            );
        }

        els.push(
            iced::widget::Space::new()
                .width(Length::Fill)
                .height(bottom_height)
                .into(),
        );

        els
    } else {
        chapter_blocks
            .iter()
            .enumerate()
            .map(|(i, block)| {
                let block_ctx = RenderContext {
                    block_index: i * 1000,
                    selection_range: state.markdown_state.selection_range,
                    drag_active: state.markdown_state.is_dragging_selection
                        || state.markdown_state.is_mouse_held,
                    ..*ctx
                };
                let inner = crate::features::markdown::view::render_block(
                    i,
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

    scrollable_content(elements, max_text_width, content_padding, "content_scroll")
        .filter_wheel(true)
        .on_wheel(|delta| crate::app::messages::MarkdownMsg::SmoothWheelScrolled(delta).into())
        .on_scroll(|v| {
            crate::app::messages::MarkdownMsg::Scrolled {
                y: v.absolute_offset().y,
                viewport_height: v.bounds().height,
                content_height: v.content_bounds().height,
            }
            .into()
        })
        .build()
}
