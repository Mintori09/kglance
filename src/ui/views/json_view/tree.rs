use crate::app::Message;
use crate::core::types::JsonState;
use crate::parsers::json::JsonNode;
use crate::ui::components::button as ui_btn;
use crate::ui::theme::breeze_text_input;
use crate::ui::theme::tokens::spacing;
use iced::widget::{button, column, container, row, text, text_input};
use iced::{Color, Element, Length, Padding};
use std::collections::HashSet;

use super::style::{dim_color, text_color, type_color};

pub fn visible_node_indices(state: &JsonState) -> Vec<usize> {
    let query = state.search_query.trim().to_lowercase();
    let filter_active = state.search_visible && !query.is_empty();

    let filter_set: HashSet<usize> = if filter_active {
        let mut set = HashSet::new();
        let mut stack: Vec<(usize, usize)> = Vec::new();

        for (i, node) in state.nodes.iter().enumerate() {
            while stack.last().is_some_and(|&(d, _)| d >= node.depth) {
                stack.pop();
            }

            let key_match = node
                .key
                .as_ref()
                .is_some_and(|k| k.to_lowercase().contains(&query));
            let val_match = node.value_preview.to_lowercase().contains(&query);
            if key_match || val_match {
                for &(_, anc_idx) in &stack {
                    set.insert(anc_idx);
                }
                set.insert(i);
                for j in (i + 1)..=(i + node.skip_count) {
                    set.insert(j);
                }
            }

            stack.push((node.depth, i));
        }
        set
    } else {
        HashSet::new()
    };

    let mut visible = Vec::new();
    let mut i = 0;
    while i < state.nodes.len() {
        if filter_active {
            if filter_set.contains(&i) {
                visible.push(i);
                i += 1;
            } else {
                i += 1 + state.nodes[i].skip_count;
            }
        } else {
            visible.push(i);
            let skip = state.nodes[i].skip_count;
            if skip > 0 && !state.expanded.contains(&i) {
                i += 1 + skip;
            } else {
                i += 1;
            }
        }
    }
    visible
}

pub fn render_tree_node<'a>(
    index: usize,
    node: &'a JsonNode,
    is_expanded: bool,
    is_dark: bool,
    font_size: f32,
    indent: f32,
    is_active: bool,
) -> Element<'a, Message> {
    let arrow = if node.children_count > 0 {
        if is_expanded { "▼ " } else { "▶ " }
    } else {
        "  "
    };

    let arrow_text = text(arrow)
        .size(font_size * 0.8)
        .color(dim_color(is_dark))
        .font(iced::Font {
            weight: iced::font::Weight::Bold,
            ..iced::Font::MONOSPACE
        });

    let key_part: Element<'a, Message> = if let Some(ref k) = node.key {
        text(format!("{}: ", k))
            .size(font_size)
            .color(text_color(is_dark))
            .into()
    } else {
        text("").into()
    };

    let type_tag = text(format!("{} ", node.value_type))
        .size(font_size * 0.85)
        .color(type_color(node.value_type, is_dark))
        .font(iced::Font {
            weight: iced::font::Weight::Bold,
            ..iced::Font::MONOSPACE
        });

    let value_text = text(&node.value_preview)
        .size(font_size)
        .color(if node.value_type == "String" {
            if is_dark {
                Color::from_rgb(0.6, 0.9, 0.4)
            } else {
                Color::from_rgb(0.2, 0.6, 0.1)
            }
        } else {
            text_color(is_dark)
        })
        .font(iced::Font::MONOSPACE);

    let children_badge: Element<'a, Message> = if node.children_count > 0 {
        text(format!(" [{}]", node.children_count))
            .size(font_size * 0.75)
            .color(dim_color(is_dark))
            .into()
    } else {
        text("").into()
    };

    let copy_btn = button(text("C").size(font_size * 0.7).font(iced::Font::MONOSPACE))
        .on_press(Message::JsonCopyPath(index))
        .padding([0, 3])
        .style(ui_btn::breeze_tool);

    let active_bg = if is_active {
        Some(
            (if is_dark {
                Color::from_rgba(1.0, 1.0, 1.0, 0.08)
            } else {
                Color::from_rgba(0.0, 0.0, 0.0, 0.05)
            })
            .into(),
        )
    } else {
        None
    };

    let content = row![
        arrow_text,
        key_part,
        type_tag,
        value_text,
        children_badge,
        copy_btn
    ]
    .align_y(iced::Alignment::Center)
    .spacing(2);

    let padded = container(content)
        .padding(Padding {
            left: indent + node.depth as f32 * (spacing::L + spacing::XS),
            right: spacing::XS,
            top: 1.0,
            bottom: 1.0,
        })
        .width(Length::Fill)
        .style(move |_theme: &iced::Theme| container::Style {
            background: active_bg,
            ..container::Style::default()
        });

    if node.children_count > 0 {
        button(padded)
            .on_press(Message::JsonToggleNode(index))
            .style(
                |_theme: &iced::Theme, _status: button::Status| button::Style {
                    background: None,
                    text_color: Color::TRANSPARENT,
                    border: iced::Border {
                        width: 0.0,
                        color: Color::TRANSPARENT,
                        radius: 0.0.into(),
                    },
                    shadow: iced::Shadow::default(),
                    snap: false,
                },
            )
            .padding(0)
            .width(Length::Fill)
            .into()
    } else {
        padded.into()
    }
}

pub fn render_tree<'a>(
    state: &'a JsonState,
    is_dark: bool,
    font_size: f32,
    editing_view: bool,
) -> Element<'a, Message> {
    let indices = visible_node_indices(state);

    let nodes: Vec<Element<'a, Message>> = indices
        .iter()
        .map(|&i| {
            let node = &state.nodes[i];
            let expanded = state.expanded.contains(&i);
            let is_active = state.active_node == Some(i);
            let is_editing = state.editing_node == Some(i);

            let row_elem: Element<'a, Message> =
                render_tree_node(i, node, expanded, is_dark, font_size, 0.0, is_active);

            if is_editing && editing_view {
                let edit_input: Element<'a, Message> =
                    text_input(&node.value_preview, &state.edit_value)
                        .on_input(Message::JsonEditValue)
                        .on_submit(Message::JsonEditSave)
                        .style(breeze_text_input)
                        .width(Length::Fill)
                        .into();

                column![row_elem, edit_input].spacing(2).into()
            } else if is_active && !editing_view {
                button(row_elem)
                    .on_press(Message::JsonEditStart(i))
                    .style(|_theme, _status| button::Style {
                        background: None,
                        text_color: Color::TRANSPARENT,
                        border: iced::Border {
                            width: 0.0,
                            color: Color::TRANSPARENT,
                            radius: 0.0.into(),
                        },
                        shadow: iced::Shadow::default(),
                        snap: false,
                    })
                    .padding(0)
                    .width(Length::Fill)
                    .into()
            } else {
                row_elem
            }
        })
        .collect();

    column(nodes)
        .spacing(0)
        .padding(Padding {
            left: spacing::XS,
            right: spacing::XS,
            top: spacing::XS,
            bottom: spacing::XS,
        })
        .into()
}
