use crate::app::Message;
use crate::core::types::JsonState;
use crate::parsers::json::JsonNode;
use crate::ui::components::code_editor::code_editor;
use crate::ui::theme::tokens::spacing;
use iced::widget::{button, container, row, text};
use iced::{Element, Length, Padding};
use std::collections::HashMap;

use super::style::{dim_color, link_color, text_color};

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

    Some(
        container(
            row(parts)
                .align_y(iced::Alignment::Center)
                .padding(Padding {
                    left: spacing::S,
                    right: spacing::S,
                    top: spacing::XXS,
                    bottom: spacing::XXS,
                }),
        )
        .width(Length::Fill)
        .into(),
    )
}

pub fn render_schema<'a>(
    state: &'a JsonState,
    theme: AppTheme,
    font_size: f32,
) -> Option<Element<'a, Message>> {
    if !state.schema_visible || state.nodes.is_empty() {
        return None;
    }

    let mut depth_types: Vec<HashMap<&str, usize>> = Vec::new();
    for node in &state.nodes {
        while depth_types.len() <= node.depth {
            depth_types.push(HashMap::new());
        }
        *depth_types[node.depth].entry(node.value_type).or_insert(0) += 1;
    }

    let mut parts: Vec<Element<'a, Message>> = Vec::new();
    for (d, types) in depth_types.iter().enumerate() {
        let summary: String = types
            .iter()
            .map(|(t, c)| format!("{}×{}", c, t))
            .collect::<Vec<_>>()
            .join(" ");
        parts.push(
            text(format!("L{}: {}", d, summary))
                .size(font_size * 0.75)
                .color(dim_color(theme))
                .into(),
        );
    }

    Some(
        container(
            row(parts)
                .align_y(iced::Alignment::Center)
                .spacing(spacing::S)
                .padding(Padding {
                    left: spacing::S,
                    right: spacing::S,
                    top: spacing::XXS,
                    bottom: spacing::XXS,
                }),
        )
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
    let mono_font = match font_family_mono {
        Some(name) => iced::Font::with_name(Box::leak(name.to_string().into_boxed_str())),
        None => iced::Font::MONOSPACE,
    };

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
