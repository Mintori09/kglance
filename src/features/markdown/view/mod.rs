pub(crate) mod blocks;
pub mod components;
pub(crate) mod highlight;
pub mod toc;

use std::cell::Cell;

use crate::app::Message;
use crate::core::MarkdownState;
use crate::features::markdown::view::toc::render_toc_sidebar;
use crate::parsers::markdown::Block;
use crate::parsers::markdown::BlockLayout;
use crate::ui::components::content_layout::scrollable_content;
use crate::ui::components::measured::Measured;
use crate::ui::components::search_bar::{SearchKind, search_bar};
use crate::ui::components::selectable_text::SelectableText;
use crate::ui::components::sidebar::drag_handle;
use crate::ui::types::RenderContext;
use iced::widget::{column, container, row};
use iced::{Element, Length, Padding};

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_selectable<'a>(
    spans: Vec<iced::widget::text::Span<'a, (), iced::Font>>,
    font_size: f32,
    block_index: Option<usize>,
    width: Length,
    default_text_color: iced::Color,
    selection_range: Option<crate::core::SelectionRange>,
    drag_active: bool,
    result_popup: bool,
) -> SelectableText<'a, Message> {
    let widget = SelectableText::new(spans, font_size)
        .default_text_color(default_text_color)
        .width(width)
        .selection_range(selection_range)
        .drag_active(drag_active);

    attach_selection_callbacks(widget, block_index, result_popup)
}

fn attach_selection_callbacks<'a>(
    mut widget: SelectableText<'a, Message>,
    block_index: Option<usize>,
    result_popup: bool,
) -> SelectableText<'a, Message> {
    if let Some(blk) = block_index {
        widget = widget.block_index(blk);
        if result_popup {
            widget
                .on_selection_change(|s| {
                    crate::app::messages::SelectionMenuMsg::ResultSelectionChanged(s).into()
                })
                .on_drag_start(|block, offset| {
                    crate::app::messages::SelectionMenuMsg::ResultDragStart { block, offset }.into()
                })
                .on_drag_update(|block, offset| {
                    crate::app::messages::SelectionMenuMsg::ResultDragUpdate { block, offset }
                        .into()
                })
                .on_drag_end(|| crate::app::messages::SelectionMenuMsg::ResultDragEnd.into())
                .on_clear_selection(|| {
                    crate::app::messages::SelectionMenuMsg::ResultSelectionClear.into()
                })
        } else {
            widget
                .on_selection_change(|s| {
                    crate::app::messages::MarkdownMsg::SelectionChanged(s).into()
                })
                .on_drag_start(|block, offset| {
                    crate::app::messages::MarkdownMsg::SelectionDragStart { block, offset }.into()
                })
                .on_drag_update(|block, offset| {
                    crate::app::messages::MarkdownMsg::SelectionDragUpdate { block, offset }.into()
                })
                .on_drag_end(|| crate::app::messages::MarkdownMsg::SelectionDragEnd.into())
                .on_clear_selection(|| crate::app::messages::MarkdownMsg::SelectionClear.into())
        }
    } else {
        widget
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_selectable_with_links<'a>(
    spans: Vec<iced::widget::text::Span<'a, (), iced::Font>>,
    links: Vec<crate::ui::components::selectable_text::TextLink>,
    rubies: Vec<crate::ui::components::selectable_text::TextRuby>,
    tooltips: Vec<crate::ui::components::selectable_text::TextTooltip>,
    font_size: f32,
    block_index: Option<usize>,
    width: Length,
    default_text_color: iced::Color,
    selection_range: Option<crate::core::SelectionRange>,
    drag_active: bool,
    theme: crate::ui::theme::AppTheme,
    result_popup: bool,
    align_x: iced::alignment::Horizontal,
) -> SelectableText<'a, Message> {
    let widget = SelectableText::new(spans, font_size)
        .links(links)
        .rubies(rubies)
        .tooltips(tooltips)
        .app_theme(theme)
        .default_text_color(default_text_color)
        .width(width)
        .align_x(align_x)
        .selection_range(selection_range)
        .drag_active(drag_active)
        .on_link_click(|url| crate::app::messages::SystemMsg::OpenLink(url).into());

    attach_selection_callbacks(widget, block_index, result_popup)
}

pub(crate) use blocks::{block_margin, render_block};
pub(crate) use components::STYLE;
pub(crate) mod window;
pub const VIRTUAL_THRESHOLD: usize = 60;
const SCROLL_PANE_ID: &str = "content_scroll";

#[allow(clippy::too_many_arguments)]
pub fn view_markdown<'a>(
    blocks: &'a [Block],
    state: &'a MarkdownState,
    font_size: f32,
    theme: crate::ui::theme::AppTheme,
    font_family: Option<&'a str>,
    font_family_mono: Option<&'a str>,
    max_text_width: Option<f32>,
    ruby_display_mode: crate::core::config::RubyDisplayMode,
) -> Element<'a, Message> {
    let search_counter = Cell::new(0);
    let block_counter = Cell::new(0);
    let footnotes: std::collections::HashMap<String, String> = blocks
        .iter()
        .filter_map(|b| {
            if let Block::FootnoteDefinition { label, content } = b {
                let mut parts = Vec::new();
                for block in content {
                    if let Block::Paragraph(inlines) = block {
                        parts.push(crate::parsers::markdown::flatten_inlines_visual(inlines));
                    }
                }
                Some((label.clone(), parts.join(" ")))
            } else {
                None
            }
        })
        .collect();

    let ctx = RenderContext {
        next_block_index: &block_counter,
        selection_range: state.selection_range,
        drag_active: state.is_dragging_selection || state.is_mouse_held,
        search_query: &state.search_query,
        active_match: state.search_match_index,
        counter: &search_counter,
        theme,
        font_size,
        font_family,
        font_family_mono,
        ruby_display_mode,
        footnotes: Some(&footnotes),
        result_popup: false,
        show_heading_divider: true,
        is_epub: false,
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
    let offsets = &state.block_y_offsets;
    let use_virtual = blocks.len() > VIRTUAL_THRESHOLD && offsets.len() == blocks.len();

    let content_padding = STYLE.general.content_padding;
    let elements: Vec<Element<'a, Message>> = if use_virtual {
        let window = window::render_range(
            &state.virtual_window,
            offsets,
            state.scroll_y,
            state.viewport_height,
        );
        let (first_visible, last_visible) = (window.start, window.end);

        let padding_v = content_padding * 2.0;
        let blocks_total_height = (state.total_content_height - padding_v).max(0.0);

        let top_height = if first_visible > 0 {
            offsets[first_visible]
        } else {
            0.0
        };

        let bottom_height = if last_visible < blocks.len() {
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

        for (i, block) in blocks
            .iter()
            .enumerate()
            .skip(first_visible)
            .take(visible_count)
        {
            let block_cell = Cell::new(i * 1000);
            let block_ctx = RenderContext {
                next_block_index: &block_cell,
                ..*ctx
            };
            let element = render_block(i, block, state, &block_ctx);
            let margin = block_margin(block, ctx.font_size);
            let block = container(element)
                .padding(Padding {
                    top: 0.0,
                    right: 0.0,
                    bottom: margin,
                    left: 0.0,
                })
                .width(Length::Fill);
            els.push(
                Measured::new(
                    block,
                    i,
                    state
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
        blocks
            .iter()
            .enumerate()
            .map(|(index, block)| {
                let block_cell = Cell::new(index * 1000);
                let block_ctx = RenderContext {
                    next_block_index: &block_cell,
                    ..*ctx
                };
                let element = render_block(index, block, state, &block_ctx);
                let margin = block_margin(block, ctx.font_size);
                container(element)
                    .padding(Padding {
                        top: 0.0,
                        right: 0.0,
                        bottom: margin,
                        left: 0.0,
                    })
                    .width(Length::Fill)
                    .into()
            })
            .collect()
    };

    scrollable_content(elements, max_text_width, content_padding, SCROLL_PANE_ID)
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

fn build_content_area<'a>(
    state: &'a MarkdownState,
    scroll_content: Element<'a, Message>,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    if state.toc_visible && !state.toc.is_empty() {
        let sidebar = render_toc_sidebar(
            &state.toc,
            state,
            state.scroll_y,
            ctx.theme,
            ctx.font_family,
        );
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
