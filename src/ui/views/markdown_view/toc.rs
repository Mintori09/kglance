use crate::app::Message;
use crate::core::{MarkdownState, TocEntry};
use crate::ui::components::scroll_pane::scroll_pane;
use crate::ui::components::sidebar::{collapse_arrow, sidebar_entry_style};
use crate::ui::theme::color::base::BaseColors;
use crate::ui::views::markdown_view::components::style::STYLE;
use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Border, Element, Length, Padding};

pub fn render_toc_sidebar<'a>(
    toc: &'a [TocEntry],
    state: &'a MarkdownState,
    scroll_y: f32,
    is_dark: bool,
) -> Element<'a, Message> {
    let active_block_index = find_active_block_index(toc, scroll_y);
    let visible_entries = filter_visible_entries(toc, state);

    let entries: Vec<Element<'a, Message>> = visible_entries
        .into_iter()
        .map(|entry| {
            let is_active = active_block_index == Some(entry.block_index);
            let has_children = entry_has_children(toc, entry);
            let is_collapsed = state.collapsed_headings.contains(&entry.block_index);

            render_toc_entry(entry, is_active, has_children, is_collapsed, is_dark)
        })
        .collect();

    let (background_color, border_color) = get_sidebar_theme_colors(is_dark);

    let content = column![
        scroll_pane(
            "toc_scroll",
            column(entries)
                .spacing(STYLE.toc.item_spacing)
                .padding(STYLE.toc.container_padding)
        )
        .build()
    ]
    .width(state.sidebar_width)
    .height(Length::Fill);

    container(content)
        .width(state.sidebar_width)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(background_color.into()),
            border: Border {
                width: STYLE.toc.sidebar_border_width,
                color: border_color,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn find_active_block_index(toc: &[TocEntry], scroll_y: f32) -> Option<usize> {
    toc.iter()
        .rposition(|entry| entry.y_offset <= scroll_y + STYLE.toc.scroll_offset_margin)
        .map(|index| toc[index].block_index)
}

fn filter_visible_entries<'a>(toc: &'a [TocEntry], state: &'a MarkdownState) -> Vec<&'a TocEntry> {
    let mut visible_entries = Vec::new();
    let mut current_collapsed_depth: Option<u8> = None;

    for entry in toc {
        if matches!(current_collapsed_depth, Some(depth) if entry.level <= depth) {
            current_collapsed_depth = None;
        }

        if current_collapsed_depth.is_none() {
            visible_entries.push(entry);
            if state.collapsed_headings.contains(&entry.block_index) {
                current_collapsed_depth = Some(entry.level);
            }
        }
    }

    visible_entries
}

fn entry_has_children(toc: &[TocEntry], entry: &TocEntry) -> bool {
    toc.iter()
        .skip_while(|e| e.block_index != entry.block_index)
        .nth(1)
        .map(|next_entry| next_entry.level > entry.level)
        .unwrap_or(false)
}

fn render_toc_entry<'a>(
    entry: &'a TocEntry,
    is_active: bool,
    has_children: bool,
    is_collapsed: bool,
    is_dark: bool,
) -> Element<'a, Message> {
    let indent_amount = (entry.level as f32 - 1.0) * STYLE.toc.indent_per_level;

    let leading_control: Element<'a, Message> = if has_children {
        collapse_arrow(
            is_collapsed,
            is_dark,
            crate::app::messages::MarkdownMsg::TocToggleCollapse(entry.block_index).into(),
        )
    } else {
        Space::new()
            .width(STYLE.toc.chevron_placeholder_width)
            .into()
    };

    let heading_label = text(&entry.text).size(STYLE.toc.entry_font_size);
    let heading_button = button(heading_label)
        .on_press(crate::app::messages::MarkdownMsg::TocHeadingClicked(entry.block_index).into())
        .width(Length::Fill)
        .style(move |theme, status| sidebar_entry_style(theme, status, is_active, is_dark))
        .padding(STYLE.toc.entry_padding);

    let item_row = row![leading_control, heading_button]
        .align_y(Alignment::Center)
        .spacing(STYLE.toc.item_spacing);

    container(item_row)
        .padding(Padding {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: indent_amount,
        })
        .width(Length::Fill)
        .into()
}

fn get_sidebar_theme_colors(is_dark: bool) -> (iced::Color, iced::Color) {
    let p = BaseColors::palette_for(is_dark);
    (p.bg, p.border)
}
