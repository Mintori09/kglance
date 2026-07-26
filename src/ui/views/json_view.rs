use crate::app::Message;
use crate::core::types::JsonState;
use crate::parsers::json::JsonNode;
use crate::ui::components::code_editor::code_editor;
use crate::ui::components::scroll_pane::scroll_pane;
use crate::ui::components::search_bar::{SearchKind, search_bar};
use crate::ui::theme::breeze_text_input;
use iced::widget::container;
use iced::widget::tooltip;
use iced::widget::{button, column, row, text, text_input};
use iced::{Color, Element, Length, Padding};
use std::collections::{HashMap, HashSet};

fn visible_node_indices(state: &JsonState) -> Vec<usize> {
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

fn build_breadcrumbs(nodes: &[JsonNode], index: usize) -> Vec<(usize, String)> {
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

fn render_tree_node<'a>(
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
                _ => None,
            };
            button::Style {
                background: bg,
                text_color: dim_color(is_dark),
                border: iced::Border {
                    width: 0.0,
                    color: Color::TRANSPARENT,
                    radius: 3.0.into(),
                },
                shadow: iced::Shadow::default(),
                snap: false,
            }
        });

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
            left: indent + node.depth as f32 * 20.0,
            right: 4.0,
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

fn render_breadcrumbs<'a>(
    state: &'a JsonState,
    is_dark: bool,
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
                text(" › ").size(font_size * 0.8).color(dim_color(is_dark)),
            ));
        }
        if *idx == active {
            parts.push(Element::from(
                text(label.clone())
                    .size(font_size * 0.8)
                    .color(text_color(is_dark)),
            ));
        } else {
            parts.push(Element::from(
                button(text(label.clone()).size(font_size * 0.8).color(if is_dark {
                    Color::from_rgb(0.4, 0.7, 1.0)
                } else {
                    Color::from_rgb(0.0, 0.3, 0.8)
                }))
                .on_press(Message::JsonBreadcrumbClicked(*idx))
                .padding(0)
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
                }),
            ));
        }
    }

    Some(
        container(
            row(parts)
                .align_y(iced::Alignment::Center)
                .padding(Padding {
                    left: 8.0,
                    right: 8.0,
                    top: 2.0,
                    bottom: 2.0,
                }),
        )
        .width(Length::Fill)
        .into(),
    )
}

fn render_schema<'a>(
    state: &'a JsonState,
    is_dark: bool,
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
                .color(dim_color(is_dark))
                .into(),
        );
    }

    Some(
        container(
            row(parts)
                .align_y(iced::Alignment::Center)
                .spacing(8)
                .padding(Padding {
                    left: 8.0,
                    right: 8.0,
                    top: 2.0,
                    bottom: 2.0,
                }),
        )
        .width(Length::Fill)
        .into(),
    )
}

fn render_tree<'a>(
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

    let content = if state.raw_pretty {
        &state.pretty_content
    } else {
        &state.minified_content
    };
    let line_numbers = (1..=content.lines().count())
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    code_editor(
        &state.raw_editor,
        &line_numbers,
        "json",
        is_dark,
        font_size,
        mono_font,
        Message::JsonRawEdit,
    )
}

fn header_button_style() -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |theme: &iced::Theme, status: button::Status| {
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
    }
}

fn small_btn_style() -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |theme: &iced::Theme, status: button::Status| {
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
            _ => None,
        };
        button::Style {
            background: bg,
            text_color: text_color(is_dark),
            border: iced::Border {
                width: 0.0,
                color: Color::TRANSPARENT,
                radius: 3.0.into(),
            },
            shadow: iced::Shadow::default(),
            snap: false,
        }
    }
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
    } else {
        if let Some(fb) = format_btn {
            header_items.push(
                tooltip(
                    fb,
                    "Toggle Format (Ctrl+Shift+P)",
                    tooltip::Position::Bottom,
                )
                .into(),
            );
        }
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
        ))
    } else {
        None
    };

    let content: Element<'a, Message> = if state.tree_mode {
        let editing_view = state.editing_node.is_some();
        render_tree(state, is_dark, font_size, editing_view)
    } else {
        render_raw(state, is_dark, font_size, font_family_mono)
    };

    let scroll = scroll_pane("json_scroll", content).container_padding(4);

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
    col_parts.push(scroll.build());

    column(col_parts).height(Length::Fill).into()
}
