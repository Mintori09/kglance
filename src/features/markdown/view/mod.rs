pub(crate) mod blocks;
pub(crate) mod components;
pub(crate) mod highlight;
pub mod toc;

use std::cell::Cell;

use crate::app::Message;
use crate::core::MarkdownState;
use crate::features::markdown::view::toc::render_toc_sidebar;
use crate::parsers::markdown::Block;
use crate::ui::components::content_layout::scrollable_content;
use crate::ui::components::search_bar::{SearchKind, search_bar};
use crate::ui::components::selectable_text::SelectableText;
use crate::ui::components::sidebar::drag_handle;
use crate::ui::types::RenderContext;
use iced::widget::{column, container, row};
use iced::{Element, Length, Padding};

pub(crate) fn build_selectable<'a>(
    spans: Vec<iced::widget::text::Span<'a, (), iced::Font>>,
    font_size: f32,
    block_index: usize,
    width: Length,
    default_text_color: iced::Color,
    selection_range: Option<crate::core::SelectionRange>,
    drag_active: bool,
) -> SelectableText<'a, Message> {
    SelectableText::new(spans, font_size)
        .default_text_color(default_text_color)
        .width(width)
        .block_index(block_index)
        .selection_range(selection_range)
        .drag_active(drag_active)
        .on_selection_change(|s| crate::app::messages::MarkdownMsg::SelectionChanged(s).into())
        .on_drag_start(|block, offset| {
            crate::app::messages::MarkdownMsg::SelectionDragStart { block, offset }.into()
        })
        .on_drag_update(|block, offset| {
            crate::app::messages::MarkdownMsg::SelectionDragUpdate { block, offset }.into()
        })
        .on_drag_end(|| crate::app::messages::MarkdownMsg::SelectionDragEnd.into())
        .on_clear_selection(|| crate::app::messages::MarkdownMsg::SelectionClear.into())
}

pub(crate) use blocks::{block_margin, render_block};
use components::style::STYLE;
const SCROLL_PANE_ID: &str = "content_scroll";

pub fn view_markdown<'a>(
    blocks: &'a [Block],
    state: &'a MarkdownState,
    font_size: f32,
    theme: crate::ui::theme::AppTheme,
    font_family_mono: Option<&str>,
    max_text_width: Option<f32>,
) -> Element<'a, Message> {
    let search_counter = Cell::new(0);
    let ctx = RenderContext {
        block_index: 0,
        selection_range: state.selection_range,
        drag_active: state.is_dragging_selection,
        search_query: &state.search_query,
        active_match: state.search_match_index,
        counter: &search_counter,
        theme,
        font_size,
        font_family: None,
        font_family_mono,
    };
    let scroll = build_scrollable_content(blocks, state, &ctx, max_text_width);
    let content_area = build_content_area(state, scroll, &ctx);

    if state.search_visible {
        let search = search_bar(
            SearchKind::Markdown,
            &state.search_query,
            if state.search_info.is_empty() {
                None
            } else {
                Some(state.search_info.as_str())
            },
        );
        column![search, content_area].height(Length::Fill).into()
    } else {
        content_area
    }
}

fn build_scrollable_content<'a>(
    blocks: &'a [Block],
    state: &'a MarkdownState,
    ctx: &RenderContext<'_>,
    max_text_width: Option<f32>,
) -> Element<'a, Message> {
    let elements = blocks.iter().enumerate().map(|(index, block)| {
        let block_ctx = RenderContext {
            block_index: index * 1000,
            ..*ctx
        };
        let element = render_block(index, block, state, &block_ctx);
        let margin = block_margin(block);
        container(element)
            .padding(Padding {
                top: 0.0,
                right: 0.0,
                bottom: margin,
                left: 0.0,
            })
            .width(Length::Fill)
            .into()
    });

    scrollable_content(
        elements,
        max_text_width,
        STYLE.general.content_padding,
        SCROLL_PANE_ID,
    )
    .on_scroll(|v| crate::app::messages::MarkdownMsg::Scrolled(v.absolute_offset().y).into())
    .build()
}

fn build_content_area<'a>(
    state: &'a MarkdownState,
    scroll_content: Element<'a, Message>,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    if state.toc_visible && !state.toc.is_empty() {
        let sidebar = render_toc_sidebar(&state.toc, state, state.scroll_y, ctx.theme);
        let drag_handle = drag_handle(
            state.sidebar_resizing,
            ctx.theme,
            Message::SidebarDragStarted,
        );
        row![sidebar, drag_handle, scroll_content]
            .spacing(0)
            .height(Length::Fill)
            .into()
    } else {
        scroll_content
    }
}
