use crate::app::Message;
use crate::core::{MarkdownState, TocEntry};
use crate::features::markdown::types::MarkdownSidebarTab;
use crate::features::markdown::view::components::style::STYLE;
use crate::ui::components::scroll_pane::scroll_pane;
use crate::ui::components::sidebar::{collapse_arrow, sidebar_entry_style};
use crate::ui::theme::{AppTheme, default_text_input};
use iced::widget::{Space, button, column, container, row, text, text_input};
use iced::{Alignment, Border, Color, Element, Font, Length, Padding, Shadow};

pub fn render_toc_sidebar<'a>(
    toc: &'a [TocEntry],
    state: &'a MarkdownState,
    scroll_y: f32,
    theme: AppTheme,
    font_family: Option<&str>,
) -> Element<'a, Message> {
    let main_font = crate::ui::theme::font::get_main_font(font_family);
    let main_font_bold = crate::ui::theme::font::get_main_font_bold(font_family);
    let palette = theme.palette().base;

    let header = build_sidebar_header(state, palette.text, main_font, main_font_bold);
    let search_bar = build_sidebar_search_bar(state, main_font, theme);

    let body: Element<'a, Message> = if state.sidebar_tab == MarkdownSidebarTab::Search {
        build_sidebar_search_results(state, theme, main_font)
    } else {
        build_sidebar_toc_list(toc, state, scroll_y, theme, main_font)
    };

    let (background_color, border_color) = get_sidebar_theme_colors(theme);

    let content = column![header, search_bar, body]
        .width(state.sidebar_width)
        .height(Length::Fill);

    iced::widget::opaque(
        container(content)
            .width(state.sidebar_width)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(background_color.into()),
                border: Border {
                    color: border_color,
                    width: 1.0,
                    ..Default::default()
                },
                ..Default::default()
            }),
    )
}

fn build_sidebar_header<'a>(
    state: &'a MarkdownState,
    text_color: Color,
    main_font: Font,
    main_font_bold: Font,
) -> Element<'a, Message> {
    let is_toc = state.sidebar_tab == MarkdownSidebarTab::Toc;
    let is_search = state.sidebar_tab == MarkdownSidebarTab::Search;

    let toc_btn =
        button(
            text("Contents")
                .size(11)
                .font(if is_toc { main_font_bold } else { main_font }),
        )
        .on_press(
            crate::app::messages::MarkdownMsg::SidebarTabChanged(MarkdownSidebarTab::Toc).into(),
        )
        .padding([3, 8])
        .style(move |_, status| {
            let bg = if is_toc {
                Some(Color::from_rgba(1.0, 1.0, 1.0, 0.1).into())
            } else if matches!(status, button::Status::Hovered) {
                Some(Color::from_rgba(1.0, 1.0, 1.0, 0.05).into())
            } else {
                None
            };
            button::Style {
                background: bg,
                text_color: if is_toc {
                    text_color
                } else {
                    Color::from_rgba(0.7, 0.75, 0.8, 0.7)
                },
                border: Border {
                    radius: 4.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                shadow: Shadow::default(),
                snap: false,
            }
        });

    let search_title = if state.search_match_count > 0 {
        format!("Search ({})", state.search_match_count)
    } else {
        "Search".to_string()
    };

    let search_btn = button(text(search_title).size(11).font(if is_search {
        main_font_bold
    } else {
        main_font
    }))
    .on_press(
        crate::app::messages::MarkdownMsg::SidebarTabChanged(MarkdownSidebarTab::Search).into(),
    )
    .padding([3, 8])
    .style(move |_, status| {
        let bg = if is_search {
            Some(Color::from_rgba(1.0, 1.0, 1.0, 0.1).into())
        } else if matches!(status, button::Status::Hovered) {
            Some(Color::from_rgba(1.0, 1.0, 1.0, 0.05).into())
        } else {
            None
        };
        button::Style {
            background: bg,
            text_color: if is_search {
                text_color
            } else {
                Color::from_rgba(0.7, 0.75, 0.8, 0.7)
            },
            border: Border {
                radius: 4.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            snap: false,
        }
    });

    container(row![toc_btn, search_btn].spacing(4))
        .padding(Padding {
            top: 8.0,
            right: 8.0,
            bottom: 4.0,
            left: 8.0,
        })
        .width(Length::Fill)
        .into()
}

fn build_sidebar_search_bar<'a>(
    state: &'a MarkdownState,
    main_font: Font,
    theme: AppTheme,
) -> Element<'a, Message> {
    let palette = theme.palette().base;
    let is_searching = !state.search_query.trim().is_empty();

    let input = text_input("Search in document...", &state.search_query)
        .id("md_sidebar_search_input")
        .on_input(|q| crate::app::messages::MarkdownMsg::SearchQueryChanged(q).into())
        .on_submit(crate::app::messages::MarkdownMsg::SearchNext.into())
        .size(12)
        .font(main_font)
        .padding([5, 8])
        .style(default_text_input);

    let mut row_items = row![input].spacing(4).align_y(iced::Alignment::Center);

    if is_searching {
        let total = state.search_matches.len();
        let count_str = if total == 0 {
            "0/0".to_string()
        } else if total >= crate::features::markdown::search::MAX_SEARCH_RESULTS {
            format!("{}/{}+", state.search_match_index + 1, total)
        } else {
            format!("{}/{}", state.search_match_index + 1, total)
        };
        let count_badge = text(count_str)
            .size(11)
            .font(main_font)
            .color(palette.text_dim);

        let text_color = palette.text;
        let text_dim_color = palette.text_dim;

        let mut prev_btn = button(text("▲").size(10).font(main_font))
            .padding([2, 5])
            .style(move |_, _| button::Style {
                background: None,
                text_color: if total > 0 {
                    text_color
                } else {
                    text_dim_color
                },
                ..Default::default()
            });
        if total > 0 {
            prev_btn = prev_btn.on_press(crate::app::messages::MarkdownMsg::SearchPrev.into());
        }

        let mut next_btn = button(text("▼").size(10).font(main_font))
            .padding([2, 5])
            .style(move |_, _| button::Style {
                background: None,
                text_color: if total > 0 {
                    text_color
                } else {
                    text_dim_color
                },
                ..Default::default()
            });
        if total > 0 {
            next_btn = next_btn.on_press(crate::app::messages::MarkdownMsg::SearchNext.into());
        }

        let clear_btn = button(text("✕").size(11).font(main_font))
            .on_press(crate::app::messages::MarkdownMsg::SearchClosed.into())
            .padding([2, 5])
            .style(move |_, _| button::Style {
                background: None,
                text_color: text_dim_color,
                ..Default::default()
            });

        row_items = row_items
            .push(count_badge)
            .push(prev_btn)
            .push(next_btn)
            .push(clear_btn);
    }

    container(row_items)
        .padding(Padding {
            top: 4.0,
            right: 8.0,
            bottom: 8.0,
            left: 8.0,
        })
        .width(Length::Fill)
        .into()
}

fn build_sidebar_search_results<'a>(
    state: &'a MarkdownState,
    theme: AppTheme,
    main_font: Font,
) -> Element<'a, Message> {
    let palette = theme.palette().base;
    let mut elements: Vec<Element<'a, Message>> = Vec::new();

    if state.search_matches.is_empty() {
        let msg = if state.search_query.trim().is_empty() {
            "Type above to search"
        } else {
            "No matches found"
        };
        elements.push(
            container(text(msg).size(12).font(main_font).color(palette.text_dim))
                .padding(16)
                .into(),
        );
    } else {
        for (match_idx, m) in state.search_matches.iter().enumerate() {
            let is_active = match_idx == state.search_match_index;
            let card_bg = if is_active {
                Color::from_rgba(0.23, 0.51, 0.96, 0.2)
            } else {
                Color::from_rgba(palette.surface.r, palette.surface.g, palette.surface.b, 0.4)
            };

            let card = button(
                text(&m.snippet)
                    .size(11)
                    .font(main_font)
                    .color(if is_active {
                        palette.text
                    } else {
                        palette.text_dim
                    }),
            )
            .on_press(crate::app::messages::MarkdownMsg::SearchResultClicked(match_idx).into())
            .padding([6, 8])
            .width(Length::Fill)
            .style(move |_, status| button::Style {
                background: Some(match status {
                    button::Status::Hovered => Color::from_rgba(0.23, 0.51, 0.96, 0.15).into(),
                    _ => card_bg.into(),
                }),
                text_color: if is_active {
                    palette.text
                } else {
                    palette.text_dim
                },
                border: Border {
                    width: if is_active { 1.0 } else { 0.5 },
                    color: if is_active {
                        Color::from_rgb(0.23, 0.51, 0.96)
                    } else {
                        palette.border
                    },
                    radius: 4.0.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            });

            elements.push(card.into());
        }
    }

    scroll_pane(
        "md_search_results_scroll",
        column(elements).spacing(4).padding(Padding {
            top: 4.0,
            right: 8.0,
            bottom: 8.0,
            left: 8.0,
        }),
    )
    .build()
}

fn build_sidebar_toc_list<'a>(
    toc: &'a [TocEntry],
    state: &'a MarkdownState,
    scroll_y: f32,
    theme: AppTheme,
    main_font: Font,
) -> Element<'a, Message> {
    let active_block_index = find_active_block_index(toc, scroll_y);
    let visible_entries = filter_visible_entries(toc, state);
    let has_hierarchy = toc.iter().any(|entry| entry_has_children(toc, entry));

    let entries: Vec<Element<'a, Message>> = visible_entries
        .into_iter()
        .map(|entry| {
            let is_active = active_block_index == Some(entry.block_index);
            let has_children = entry_has_children(toc, entry);
            let is_collapsed = state.collapsed_headings.contains(&entry.block_index);

            render_toc_entry(
                entry,
                is_active,
                has_children,
                is_collapsed,
                has_hierarchy,
                theme,
                main_font,
            )
        })
        .collect();

    scroll_pane(
        "toc_scroll",
        column(entries)
            .spacing(STYLE.toc.item_spacing)
            .padding(STYLE.toc.container_padding),
    )
    .build()
}

fn find_active_block_index(toc: &[TocEntry], scroll_y: f32) -> Option<usize> {
    let detection_offset_y = 60.0;
    let effective_scroll_y = scroll_y + detection_offset_y;

    toc.iter()
        .filter(|entry| entry.y_offset <= effective_scroll_y)
        .max_by(|a, b| {
            a.y_offset
                .partial_cmp(&b.y_offset)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .or_else(|| toc.first())
        .map(|entry| entry.block_index)
}

fn filter_visible_entries<'a>(toc: &'a [TocEntry], state: &MarkdownState) -> Vec<&'a TocEntry> {
    let mut visible = Vec::new();
    let mut hide_until_level: Option<u8> = None;

    for entry in toc {
        if let Some(level) = hide_until_level {
            if entry.level > level {
                continue;
            } else {
                hide_until_level = None;
            }
        }

        visible.push(entry);

        if state.collapsed_headings.contains(&entry.block_index) {
            hide_until_level = Some(entry.level);
        }
    }

    visible
}

fn entry_has_children(toc: &[TocEntry], parent: &TocEntry) -> bool {
    let mut found_parent = false;
    for entry in toc {
        if !found_parent {
            if entry.block_index == parent.block_index {
                found_parent = true;
            }
            continue;
        }

        if entry.level <= parent.level {
            return false;
        }

        if entry.level == parent.level + 1 {
            return true;
        }
    }
    false
}

fn render_toc_entry<'a>(
    entry: &'a TocEntry,
    is_active: bool,
    has_children: bool,
    is_collapsed: bool,
    has_subheadings: bool,
    theme: AppTheme,
    main_font: Font,
) -> Element<'a, Message> {
    let indent_amount = if has_subheadings {
        (entry.level.saturating_sub(1) as f32) * STYLE.toc.indent_per_level
    } else {
        0.0
    };

    let heading_label = text(&entry.text)
        .size(STYLE.toc.entry_font_size)
        .font(main_font);
    let heading_button = button(heading_label)
        .on_press(crate::app::messages::MarkdownMsg::TocHeadingClicked(entry.block_index).into())
        .width(Length::Fill)
        .style(move |iced_theme, status| sidebar_entry_style(iced_theme, status, is_active, theme))
        .padding(STYLE.toc.entry_padding);

    let item_row: Element<'a, Message> = if has_subheadings {
        let leading_control: Element<'a, Message> = if has_children {
            collapse_arrow(
                is_collapsed,
                theme,
                crate::app::messages::MarkdownMsg::TocToggleCollapse(entry.block_index).into(),
            )
        } else {
            Space::new()
                .width(STYLE.toc.chevron_placeholder_width)
                .into()
        };

        row![leading_control, heading_button]
            .align_y(Alignment::Center)
            .spacing(STYLE.toc.item_spacing)
            .into()
    } else {
        heading_button.into()
    };

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

fn get_sidebar_theme_colors(theme: AppTheme) -> (iced::Color, iced::Color) {
    let p = theme.palette().base;
    (p.bg, p.border)
}
