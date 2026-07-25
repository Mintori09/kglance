use crate::app::Message;
use crate::core::types::JsonState;
use crate::parsers::json::JsonNode;
use iced::widget::{button, column, container, row, scrollable, text, text_editor};
use iced::{Color, Element, Length, Padding};

fn visible_node_indices(state: &JsonState) -> Vec<usize> {
    let mut visible = Vec::new();
    let mut i = 0;
    while i < state.nodes.len() {
        visible.push(i);
        let skip = state.nodes[i].skip_count;
        if skip > 0 && !state.expanded.contains(&i) {
            i += 1 + skip;
        } else {
            i += 1;
        }
    }
    visible
}

fn transparent_container(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: None,
        ..Default::default()
    }
}

fn type_color(value_type: &str, is_dark: bool) -> Color {
    match value_type {
        "String" => {
            if is_dark {
                Color::from_rgb(0.6, 0.9, 0.4)
            } else {
                Color::from_rgb(0.2, 0.6, 0.1)
            }
        }
        "Number" => {
            if is_dark {
                Color::from_rgb(0.8, 0.6, 0.3)
            } else {
                Color::from_rgb(0.7, 0.4, 0.0)
            }
        }
        "Bool" => {
            if is_dark {
                Color::from_rgb(0.4, 0.7, 1.0)
            } else {
                Color::from_rgb(0.0, 0.3, 0.8)
            }
        }
        "Null" => {
            if is_dark {
                Color::from_rgb(0.6, 0.6, 0.6)
            } else {
                Color::from_rgb(0.5, 0.5, 0.5)
            }
        }
        _ => {
            if is_dark {
                Color::from_rgb(0.7, 0.7, 0.9)
            } else {
                Color::from_rgb(0.3, 0.3, 0.6)
            }
        }
    }
}

fn text_color(is_dark: bool) -> Color {
    if is_dark {
        Color::from_rgb(0.85, 0.85, 0.85)
    } else {
        Color::from_rgb(0.15, 0.15, 0.15)
    }
}

fn dim_color(is_dark: bool) -> Color {
    if is_dark {
        Color::from_rgb(0.5, 0.5, 0.5)
    } else {
        Color::from_rgb(0.6, 0.6, 0.6)
    }
}

fn render_tree_node<'a>(
    index: usize,
    node: &'a JsonNode,
    is_expanded: bool,
    is_dark: bool,
    font_size: f32,
    indent: f32,
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

    let content = row![arrow_text, key_part, type_tag, value_text, children_badge,]
        .align_y(iced::Alignment::Center)
        .spacing(2);

    let padded = container(content)
        .padding(Padding {
            left: indent + node.depth as f32 * 20.0,
            right: 8.0,
            top: 1.0,
            bottom: 1.0,
        })
        .width(Length::Fill)
        .style(transparent_container);

    // Make entire row clickable to toggle expand/collapse if node has children
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

fn render_tree<'a>(state: &'a JsonState, is_dark: bool, font_size: f32) -> Element<'a, Message> {
    let indices = visible_node_indices(state);

    let nodes: Vec<Element<'a, Message>> = indices
        .iter()
        .map(|&i| {
            let node = &state.nodes[i];
            let expanded = state.expanded.contains(&i);
            render_tree_node(i, node, expanded, is_dark, font_size, 0.0)
        })
        .collect();

    column(nodes)
        .spacing(0)
        .padding(Padding {
            left: 4.0,
            right: 4.0,
            top: 4.0,
            bottom: 4.0,
        })
        .into()
}

fn render_raw<'a>(
    state: &'a JsonState,
    is_dark: bool,
    font_size: f32,
    font_family_mono: Option<&str>,
) -> Element<'a, Message> {
    let mono_font = match font_family_mono {
        Some(name) => iced::Font::with_name(Box::leak(name.to_string().into_boxed_str())),
        None => iced::Font::MONOSPACE,
    };

    let theme = if is_dark {
        iced::highlighter::Theme::Base16Mocha
    } else {
        iced::highlighter::Theme::InspiredGitHub
    };

    text_editor(&state.raw_editor)
        .highlight("json", theme)
        .font(mono_font)
        .size(font_size)
        .on_action(Message::JsonRawEdit)
        .style(|theme: &iced::Theme, _status| {
            let is_dark = matches!(theme, iced::Theme::Dark);
            text_editor::Style {
                background: Color::TRANSPARENT.into(),
                border: iced::Border {
                    width: 0.0,
                    color: Color::TRANSPARENT,
                    radius: 0.0.into(),
                },
                placeholder: Color::TRANSPARENT,
                value: text_color(is_dark),
                selection: if is_dark {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.15)
                } else {
                    Color::from_rgba(0.0, 0.0, 0.0, 0.15)
                },
            }
        })
        .into()
}

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
    .style(move |theme: &iced::Theme, status: button::Status| {
        let is_dark = matches!(theme, iced::Theme::Dark);
        let bg = match status {
            button::Status::Hovered | button::Status::Pressed => Some(
                (if is_dark {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.12)
                } else {
                    Color::from_rgba(0.0, 0.0, 0.0, 0.08)
                })
                .into(),
            ),
            _ => Some(
                (if is_dark {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.06)
                } else {
                    Color::from_rgba(0.0, 0.0, 0.0, 0.04)
                })
                .into(),
            ),
        };
        let border_color = if is_dark {
            Color::from_rgba(1.0, 1.0, 1.0, 0.15)
        } else {
            Color::from_rgba(0.0, 0.0, 0.0, 0.12)
        };
        button::Style {
            background: bg,
            text_color: text_color(is_dark),
            border: iced::Border {
                width: 1.0,
                color: border_color,
                radius: 4.0.into(),
            },
            shadow: iced::Shadow::default(),
            snap: false,
        }
    });

    let header = container(
        row![
            text(if state.has_parse_error {
                "⚠ JSON Parse Error — showing raw"
            } else {
                ""
            })
            .size(11)
            .color(if is_dark {
                Color::from_rgb(1.0, 0.5, 0.3)
            } else {
                Color::from_rgb(0.8, 0.2, 0.0)
            }),
            iced::widget::Space::new().width(Length::Fill),
            toggle_btn,
        ]
        .align_y(iced::Alignment::Center)
        .padding(Padding {
            left: 8.0,
            right: 8.0,
            top: 4.0,
            bottom: 4.0,
        }),
    )
    .width(Length::Fill);

    let content: Element<'a, Message> = if state.tree_mode {
        render_tree(state, is_dark, font_size)
    } else {
        render_raw(state, is_dark, font_size, font_family_mono)
    };

    let scroll = scrollable(
        container(content)
            .width(Length::Fill)
            .padding(4)
            .style(crate::ui::theme::breeze_container),
    )
    .id("json_scroll")
    .direction(scrollable::Direction::Vertical(
        scrollable::Scrollbar::new().width(4).margin(2),
    ))
    .style(crate::ui::theme::glass_scrollable)
    .width(Length::Fill)
    .height(Length::Fill);

    column![header, scroll].height(Length::Fill).into()
}
