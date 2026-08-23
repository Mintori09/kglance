use crate::app::Message;
use crate::core::types::JsonState;
use crate::parsers::json::JsonNode;
use crate::ui::components::button as ui_btn;
use crate::ui::theme::tokens::spacing;
use iced::widget::{button, column, container, row, text};
use iced::{Color, Element, Length, Padding};
use std::collections::HashSet;

use super::style::{dim_color, selection_color, string_color, text_color, type_color};

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

use crate::ui::theme::AppTheme;

fn render_highlighted_text<'a>(
    content: &'a str,
    suffix: &'static str,
    query: &str,
    base_color: Color,
    font_size: f32,
    font: iced::Font,
) -> Element<'a, Message> {
    let q = query.trim().to_lowercase();
    if q.is_empty() || !content.to_lowercase().contains(&q) {
        let display = if suffix.is_empty() {
            text(content)
        } else {
            text(format!("{}{}", content, suffix))
        };
        return display.size(font_size).color(base_color).font(font).into();
    }

    let mut spans: Vec<Element<'a, Message>> = Vec::new();
    let lower_content = content.to_lowercase();
    let query_len = q.len();
    let mut last_idx = 0;

    for (start_byte, _) in lower_content.match_indices(&q) {
        if start_byte > last_idx {
            spans.push(
                text(&content[last_idx..start_byte])
                    .size(font_size)
                    .color(base_color)
                    .font(font)
                    .into(),
            );
        }
        let end_byte = start_byte + query_len;
        let matched_slice = &content[start_byte..end_byte];
        spans.push(
            container(
                text(matched_slice)
                    .size(font_size)
                    .color(Color::BLACK)
                    .font(font),
            )
            .style(|_t| container::Style {
                background: Some(Color::from_rgb8(255, 220, 80).into()),
                ..container::Style::default()
            })
            .padding([0, 1])
            .into(),
        );
        last_idx = end_byte;
    }

    if last_idx < content.len() {
        spans.push(
            text(&content[last_idx..])
                .size(font_size)
                .color(base_color)
                .font(font)
                .into(),
        );
    }

    if !suffix.is_empty() {
        spans.push(
            text(suffix)
                .size(font_size)
                .color(base_color)
                .font(font)
                .into(),
        );
    }

    row(spans)
        .spacing(0)
        .align_y(iced::Alignment::Center)
        .into()
}

#[derive(Clone, Copy)]
pub struct TreeNodeOptions<'a> {
    pub theme: AppTheme,
    pub font_size: f32,
    pub is_expanded: bool,
    pub is_active: bool,
    pub search_query: &'a str,
}

pub fn render_tree_node<'a>(
    index: usize,
    node: &'a JsonNode,
    options: TreeNodeOptions<'a>,
) -> Element<'a, Message> {
    let arrow_str = if node.children_count > 0 {
        if options.is_expanded { "▾ " } else { "▸ " }
    } else {
        "  "
    };

    let arrow_text = text(arrow_str)
        .size(options.font_size * 0.9)
        .color(dim_color(options.theme))
        .font(iced::Font {
            weight: iced::font::Weight::Bold,
            ..iced::Font::MONOSPACE
        });

    let mut row_items: Vec<Element<'a, Message>> = Vec::new();
    row_items.push(arrow_text.into());

    if let Some(ref k) = node.key {
        let is_array_idx = k.starts_with('[') && k.ends_with(']');
        let key_color = if is_array_idx {
            dim_color(options.theme)
        } else {
            text_color(options.theme)
        };

        let key_font = iced::Font {
            weight: if is_array_idx {
                iced::font::Weight::Normal
            } else {
                iced::font::Weight::Medium
            },
            ..iced::Font::MONOSPACE
        };

        row_items.push(render_highlighted_text(
            k.as_str(),
            ": ",
            options.search_query,
            key_color,
            options.font_size,
            key_font,
        ));
    }

    if node.children_count == 0 {
        let is_string = node.value_type == "String";
        let val_color = if is_string {
            string_color(options.theme)
        } else {
            type_color(node.value_type, options.theme)
        };

        row_items.push(render_highlighted_text(
            &node.value_preview,
            "",
            options.search_query,
            val_color,
            options.font_size,
            iced::Font::MONOSPACE,
        ));
    } else {
        row_items.push(
            text(&node.value_preview)
                .size(options.font_size * 0.9)
                .color(dim_color(options.theme))
                .font(iced::Font::MONOSPACE)
                .into(),
        );
    }

    let active_bg = if options.is_active {
        Some(selection_color(options.theme).into())
    } else {
        None
    };

    let content = row(row_items).align_y(iced::Alignment::Center).spacing(2);

    let padded = container(content)
        .padding(Padding {
            left: node.depth as f32 * (spacing::M + spacing::XS),
            right: spacing::S,
            top: 2.0,
            bottom: 2.0,
        })
        .width(Length::Fill)
        .style(move |_theme: &iced::Theme| container::Style {
            background: active_bg,
            ..container::Style::default()
        });

    if node.children_count > 0 {
        button(padded)
            .on_press(crate::app::messages::JsonMsg::ToggleNode(index).into())
            .style(ui_btn::transparent)
            .padding(0)
            .width(Length::Fill)
            .into()
    } else {
        button(padded)
            .on_press(crate::app::messages::JsonMsg::NodeClicked(index).into())
            .style(ui_btn::transparent)
            .padding(0)
            .width(Length::Fill)
            .into()
    }
}

pub fn render_tree<'a>(
    state: &'a JsonState,
    theme: AppTheme,
    font_size: f32,
) -> Element<'a, Message> {
    let indices = visible_node_indices(state);
    let search_q = if state.search_visible {
        state.search_query.as_str()
    } else {
        ""
    };

    let nodes: Vec<Element<'a, Message>> = indices
        .iter()
        .map(|&i| {
            let node = &state.nodes[i];
            let expanded = state.expanded.contains(&i);
            let is_active = state.active_node == Some(i);

            render_tree_node(
                i,
                node,
                TreeNodeOptions {
                    theme,
                    font_size,
                    is_expanded: expanded,
                    is_active,
                    search_query: search_q,
                },
            )
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
