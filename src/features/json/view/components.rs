use crate::app::Message;
use crate::core::types::JsonState;
use crate::parsers::json::JsonNode;
use crate::ui::components::code_editor::code_editor;
use crate::ui::theme::tokens::spacing;
use iced::widget::{button, container, row, text};
use iced::{Element, Length, Padding};

use super::style::{dim_color, link_color, text_color};

pub fn build_json_path(nodes: &[JsonNode], index: usize) -> String {
    let mut path_segments: Vec<String> = Vec::new();
    let mut current = Some(index);
    while let Some(idx) = current {
        if let Some(node) = nodes.get(idx) {
            if let Some(ref k) = node.key {
                path_segments.push(k.clone());
            }
            current = node.parent_index;
        } else {
            break;
        }
    }
    path_segments.reverse();
    if path_segments.is_empty() {
        "$".to_string()
    } else {
        let mut full_path = String::from("$");
        for seg in path_segments {
            if seg.starts_with('[') {
                full_path.push_str(&seg);
            } else {
                full_path.push('.');
                full_path.push_str(&seg);
            }
        }
        full_path
    }
}

pub fn build_breadcrumbs(nodes: &[JsonNode], index: usize) -> Vec<(usize, String)> {
    let mut crumbs: Vec<(usize, String)> = Vec::new();
    let mut current = Some(index);
    while let Some(idx) = current {
        if let Some(node) = nodes.get(idx) {
            let label = match &node.key {
                Some(k) => k.clone(),
                None => "root".to_string(),
            };
            crumbs.push((idx, label));
            current = node.parent_index;
        } else {
            break;
        }
    }
    crumbs.reverse();
    crumbs
}

use crate::ui::theme::AppTheme;

pub fn render_breadcrumbs<'a>(
    state: &'a JsonState,
    theme: AppTheme,
    font_size: f32,
) -> Option<Element<'a, Message>> {
    let active = state.active_node?;
    let crumbs = build_breadcrumbs(&state.nodes, active);
    if crumbs.len() <= 1 {
        return None;
    }

    let mut parts: Vec<Element<'a, Message>> = Vec::new();
    for (i, (idx, label)) in crumbs.iter().enumerate() {
        if i > 0 {
            parts.push(Element::from(
                text(" › ").size(font_size * 0.8).color(dim_color(theme)),
            ));
        }
        if *idx == active {
            parts.push(Element::from(
                text(label.clone())
                    .size(font_size * 0.8)
                    .color(text_color(theme)),
            ));
        } else {
            parts.push(Element::from(
                button(
                    text(label.clone())
                        .size(font_size * 0.8)
                        .color(link_color(theme)),
                )
                .on_press(crate::app::messages::JsonMsg::BreadcrumbClicked(*idx).into())
                .padding(0)
                .style(crate::ui::components::button::transparent),
            ));
        }
    }

    let path_str = build_json_path(&state.nodes, active);

    let copy_path_btn = button(
        text("Copy Path")
            .size(font_size * 0.7)
            .color(dim_color(theme)),
    )
    .on_press(crate::app::messages::JsonMsg::CopyPath(active).into())
    .padding([2, 6])
    .style(crate::ui::components::button::breeze_tool);

    let active_node = state.nodes.get(active);
    let is_container = active_node.is_some_and(|n| n.children_count > 0);
    let has_key = active_node.is_some_and(|n| n.key.is_some());

    let mut actions: Vec<Element<'a, Message>> = Vec::new();
    if has_key {
        actions.push(
            button(
                text("Copy Key")
                    .size(font_size * 0.7)
                    .color(dim_color(theme)),
            )
            .on_press(crate::app::messages::JsonMsg::CopyKey(active).into())
            .padding([2, 6])
            .style(crate::ui::components::button::breeze_tool)
            .into(),
        );
    }

    if is_container {
        actions.push(
            button(
                text("Copy JSON")
                    .size(font_size * 0.7)
                    .color(dim_color(theme)),
            )
            .on_press(crate::app::messages::JsonMsg::CopySubtree(active).into())
            .padding([2, 6])
            .style(crate::ui::components::button::breeze_tool)
            .into(),
        );
    } else {
        actions.push(
            button(
                text("Copy Value")
                    .size(font_size * 0.7)
                    .color(dim_color(theme)),
            )
            .on_press(crate::app::messages::JsonMsg::CopyValue(active).into())
            .padding([2, 6])
            .style(crate::ui::components::button::breeze_tool)
            .into(),
        );
    }
    actions.push(copy_path_btn.into());

    let breadcrumbs_row = row(parts).align_y(iced::Alignment::Center).spacing(0);

    let mut bar_items: Vec<Element<'a, Message>> = vec![
        breadcrumbs_row.into(),
        iced::widget::Space::new().width(Length::Fill).into(),
        text(path_str)
            .size(font_size * 0.7)
            .color(dim_color(theme))
            .font(iced::Font::MONOSPACE)
            .into(),
    ];
    bar_items.extend(actions);

    let bar = row(bar_items)
        .align_y(iced::Alignment::Center)
        .spacing(spacing::XS);

    Some(
        container(bar)
            .padding(Padding {
                left: spacing::S,
                right: spacing::S,
                top: spacing::XXS,
                bottom: spacing::XXS,
            })
            .width(Length::Fill)
            .into(),
    )
}

pub fn render_raw<'a>(
    state: &'a JsonState,
    theme: AppTheme,
    font_size: f32,
    font_family_mono: Option<&str>,
    word_wrap: bool,
) -> Element<'a, Message> {
    let mono_font = crate::ui::theme::font::get_code_font(font_family_mono);

    code_editor(
        &state.raw_editor,
        "json",
        theme,
        font_size,
        mono_font,
        word_wrap,
        |action| crate::app::messages::JsonMsg::RawEdit(action).into(),
    )
}
