pub mod components;
pub mod style;
pub mod tree;

use crate::app::Message;
use crate::app::messages::JsonMsg;
use crate::core::types::JsonState;
use crate::ui::components::scroll_pane::scroll_pane;
use crate::ui::components::search_bar::{SearchKind, search_bar};
use crate::ui::theme::AppTheme;
use crate::ui::theme::tokens::spacing;

use components::{render_breadcrumbs, render_raw};
use iced::font::Weight;
use iced::widget::{Space, button, column, container, row, text, tooltip};
use iced::{Alignment, Element, Font, Length, Padding};
use style::{error_color, header_button_style, small_btn_style};
use tree::render_tree;

const CONTENT_SCROLL_ID: &str = "content_scroll";
const CONTENT_CONTAINER_PADDING: u16 = 4;

const STATUS_TEXT_SIZE: f32 = 11.0;
const BUTTON_TEXT_SIZE: f32 = 12.0;
const SMALL_BUTTON_TEXT_SIZE: f32 = 11.0;

const TOGGLE_BUTTON_PADDING: [u16; 2] = [3, 10];
const ACTION_BUTTON_PADDING: [u16; 2] = [2, 6];

pub fn view_json<'a>(
    state: &'a JsonState,
    font_size: f32,
    theme: AppTheme,
    font_family_mono: Option<&str>,
    word_wrap: bool,
) -> Element<'a, Message> {
    let header = build_header(state, theme);
    let search_bar = build_search_bar(state);
    let breadcrumbs = build_breadcrumbs(state, theme, font_size);
    let content = build_content(state, theme, font_size, font_family_mono, word_wrap);

    let mut layout = Vec::new();
    layout.push(header);

    if let Some(search) = search_bar {
        layout.push(search);
    }

    if let Some(crumbs) = breadcrumbs {
        layout.push(crumbs);
    }

    layout.push(content);

    column(layout).height(Length::Fill).into()
}

fn build_header<'a>(state: &'a JsonState, theme: AppTheme) -> Element<'a, Message> {
    let status_text = if state.has_parse_error {
        "⚠ JSON Parse Error — showing raw"
    } else {
        ""
    };

    let mut header_items: Vec<Element<'a, Message>> = vec![
        text(status_text)
            .size(STATUS_TEXT_SIZE)
            .color(error_color(theme))
            .into(),
        Space::new().width(Length::Fill).into(),
    ];

    if state.tree_mode {
        header_items.push(build_expand_button());
        header_items.push(build_collapse_button());
    } else {
        header_items.push(build_format_button(state.raw_pretty));
    }

    header_items.push(build_mode_toggle_button(state.tree_mode));

    let header_padding = Padding {
        left: spacing::S,
        right: spacing::S,
        top: spacing::XS,
        bottom: spacing::XS,
    };

    container(
        row(header_items)
            .align_y(Alignment::Center)
            .spacing(spacing::XS)
            .padding(header_padding),
    )
    .width(Length::Fill)
    .into()
}

fn build_mode_toggle_button<'a>(is_tree_mode: bool) -> Element<'a, Message> {
    let label = if is_tree_mode { "Raw" } else { "Tree" };

    let button_font = Font {
        weight: Weight::Bold,
        ..Font::DEFAULT
    };

    button(text(label).size(BUTTON_TEXT_SIZE).font(button_font))
        .on_press(JsonMsg::ToggleMode.into())
        .padding(TOGGLE_BUTTON_PADDING)
        .style(header_button_style())
        .into()
}

fn build_expand_button<'a>() -> Element<'a, Message> {
    let button = button(text("+").size(BUTTON_TEXT_SIZE))
        .on_press(JsonMsg::ExpandAll.into())
        .padding(ACTION_BUTTON_PADDING)
        .style(small_btn_style());

    tooltip(button, "Expand All (Ctrl+E)", tooltip::Position::Bottom).into()
}

fn build_collapse_button<'a>() -> Element<'a, Message> {
    let button = button(text("−").size(BUTTON_TEXT_SIZE))
        .on_press(JsonMsg::CollapseAll.into())
        .padding(ACTION_BUTTON_PADDING)
        .style(small_btn_style());

    tooltip(
        button,
        "Collapse All (Ctrl+Shift+E)",
        tooltip::Position::Bottom,
    )
    .into()
}

fn build_format_button<'a>(is_pretty: bool) -> Element<'a, Message> {
    let label = if is_pretty { "Minify" } else { "Pretty" };

    let button = button(text(label).size(SMALL_BUTTON_TEXT_SIZE))
        .on_press(JsonMsg::ToggleFormat.into())
        .padding(ACTION_BUTTON_PADDING)
        .style(small_btn_style());

    tooltip(
        button,
        "Toggle Format (Ctrl+Shift+P)",
        tooltip::Position::Bottom,
    )
    .into()
}

fn build_search_bar<'a>(state: &'a JsonState) -> Option<Element<'a, Message>> {
    if !state.search_visible || !state.tree_mode {
        return None;
    }

    let search_info = if state.search_info.is_empty() {
        None
    } else {
        Some(state.search_info.as_str())
    };

    Some(search_bar(
        SearchKind::Json,
        &state.search_query,
        search_info,
    ))
}

fn build_breadcrumbs<'a>(
    state: &'a JsonState,
    theme: AppTheme,
    font_size: f32,
) -> Option<Element<'a, Message>> {
    if state.tree_mode {
        render_breadcrumbs(state, theme, font_size)
    } else {
        None
    }
}

fn build_content<'a>(
    state: &'a JsonState,
    theme: AppTheme,
    font_size: f32,
    font_family_mono: Option<&str>,
    word_wrap: bool,
) -> Element<'a, Message> {
    let view_content = if state.tree_mode {
        render_tree(state, theme, font_size)
    } else {
        render_raw(state, theme, font_size, font_family_mono, word_wrap)
    };

    scroll_pane(CONTENT_SCROLL_ID, view_content)
        .container_padding(CONTENT_CONTAINER_PADDING)
        .on_scroll(|viewport| JsonMsg::Scrolled(viewport.absolute_offset().y).into())
        .build()
}
