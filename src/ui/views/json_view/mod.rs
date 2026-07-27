pub mod components;
pub mod style;
pub mod tree;

use crate::app::Message;
use crate::core::types::JsonState;
use crate::ui::components::scroll_pane::scroll_pane;
use crate::ui::components::search_bar::{SearchKind, search_bar};
use iced::widget::{button, column, container, row, text, tooltip};
use iced::{Color, Element, Length, Padding};

use components::{render_breadcrumbs, render_raw, render_schema};
use style::{header_button_style, small_btn_style};
use tree::render_tree;

pub fn view_json<'a>(
    state: &'a JsonState,
    font_size: f32,
    is_dark: bool,
    font_family_mono: Option<&str>,
) -> Element<'a, Message> {
    let toggle_label = if state.tree_mode { "Raw" } else { "Tree" };

    let toggle_btn = button(text(toggle_label).size(12).font(iced::Font {
        weight: iced::font::Weight::Bold,
        ..iced::Font::DEFAULT
    }))
    .on_press(Message::JsonToggleMode)
    .padding([3, 10])
    .style(header_button_style());

    let expand_btn = button(text("+").size(12))
        .on_press(Message::JsonExpandAll)
        .padding([2, 6])
        .style(small_btn_style());

    let collapse_btn = button(text("−").size(12))
        .on_press(Message::JsonCollapseAll)
        .padding([2, 6])
        .style(small_btn_style());

    let format_btn = if !state.tree_mode {
        let lbl = if state.raw_pretty { "Minify" } else { "Pretty" };
        Some(
            button(text(lbl).size(11))
                .on_press(Message::JsonToggleFormat)
                .padding([2, 6])
                .style(small_btn_style()),
        )
    } else {
        None
    };

    let schema_btn = button(text("Σ").size(12).font(iced::Font {
        weight: iced::font::Weight::Bold,
        ..iced::Font::MONOSPACE
    }))
    .on_press(Message::JsonSchemaToggle)
    .padding([2, 6])
    .style(small_btn_style());

    let err_text = if state.has_parse_error {
        "⚠ JSON Parse Error — showing raw"
    } else {
        ""
    };

    let mut header_items: Vec<Element<'a, Message>> = vec![
        text(err_text)
            .size(11)
            .color(if is_dark {
                Color::from_rgb(1.0, 0.5, 0.3)
            } else {
                Color::from_rgb(0.8, 0.2, 0.0)
            })
            .into(),
        iced::widget::Space::new().width(Length::Fill).into(),
    ];

    if state.tree_mode {
        header_items
            .push(tooltip(expand_btn, "Expand All (Ctrl+E)", tooltip::Position::Bottom).into());
        header_items.push(
            tooltip(
                collapse_btn,
                "Collapse All (Ctrl+Shift+E)",
                tooltip::Position::Bottom,
            )
            .into(),
        );
    } else if let Some(fb) = format_btn {
        header_items.push(
            tooltip(
                fb,
                "Toggle Format (Ctrl+Shift+P)",
                tooltip::Position::Bottom,
            )
            .into(),
        );
    }
    header_items.push(tooltip(schema_btn, "Schema (Ctrl+I)", tooltip::Position::Bottom).into());
    header_items.push(toggle_btn.into());

    let header = container(
        row(header_items)
            .align_y(iced::Alignment::Center)
            .spacing(4)
            .padding(Padding {
                left: 8.0,
                right: 8.0,
                top: 4.0,
                bottom: 4.0,
            }),
    )
    .width(Length::Fill);

    let search_bar: Option<Element<'a, Message>> = if state.search_visible && state.tree_mode {
        Some(search_bar(
            SearchKind::Json,
            &state.search_query,
            None,
            "Search key or value...",
            "json_search_input",
        ))
    } else {
        None
    };

    let content: Element<'a, Message> = if state.tree_mode {
        let editing_view = state.editing_node.is_some();
        let tree = render_tree(state, is_dark, font_size, editing_view);
        scroll_pane("json_scroll", tree)
            .container_padding(4)
            .build()
    } else {
        let raw = render_raw(state, is_dark, font_size, font_family_mono);
        scroll_pane("json_raw_scroll", raw)
            .container_padding(4)
            .build()
    };

    let breadcrumbs = if state.tree_mode {
        render_breadcrumbs(state, is_dark, font_size)
    } else {
        None
    };

    let schema_bar = render_schema(state, is_dark, font_size);

    let mut col_parts: Vec<Element<'a, Message>> = Vec::new();
    col_parts.push(header.into());
    if let Some(sb) = search_bar {
        col_parts.push(sb);
    }
    if let Some(sb) = schema_bar {
        col_parts.push(sb);
    }
    if let Some(bc) = breadcrumbs {
        col_parts.push(bc);
    }
    col_parts.push(content);

    column(col_parts).height(Length::Fill).into()
}
