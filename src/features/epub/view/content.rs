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

    let elements = chapter_blocks.iter().enumerate().map(|(i, block)| {
        let global_index = chapter_offset + i;
        let block_ctx = RenderContext {
            block_index: global_index * 1000,
            selection_range: state.markdown_state.selection_range,
            drag_active: state.markdown_state.is_dragging_selection,
            ..*ctx
        };
        let inner = crate::features::markdown::view::render_block(
            global_index,
            block,
            &state.markdown_state,
            &block_ctx,
        );
        let margin_bottom = crate::features::markdown::view::block_margin(block);
        container(inner)
            .padding(Padding {
                top: 0.0,
                right: 0.0,
                bottom: margin_bottom,
                left: 0.0,
            })
            .width(Length::Fill)
            .into()
    });

    scrollable_content(elements, max_text_width, CONTENT_SPACING, "content_scroll")
        .on_scroll(|v| crate::app::messages::MarkdownMsg::Scrolled(v.absolute_offset().y).into())
        .build()
}
