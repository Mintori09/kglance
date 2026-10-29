use iced::widget::{button, column, container, row, text, text_input};
use iced::{Border, Color, Element, Font, Length, Padding, Shadow};

use crate::app::Message;
use crate::core::types::{EpubChapterInfo, EpubState};
use crate::features::epub::view::constants::{
    CHAPTER_ENTRY_SPACING, CHAPTER_LIST_PADDING, ENTRY_PADDING_BOTTOM, ENTRY_PADDING_LEFT_BASE,
    ENTRY_PADDING_RIGHT, ENTRY_PADDING_TOP, HEADER_SPACING, RESIZE_BUTTON_FONT_SIZE,
    SIDEBAR_HEADER_PADDING_HORIZONTAL, SIDEBAR_HEADER_PADDING_VERTICAL, SIDEBAR_RESIZE_STEP,
    SIDEBAR_TOGGLE_FONT_SIZE,
};
use crate::features::epub::view::helpers::{
    calculate_indent, entry_font_size, entry_font_weight, entry_text_color,
};
use crate::ui::components::scroll_pane::scroll_pane;
use crate::ui::components::sidebar::{
    SIDEBAR_BORDER_WIDTH as BORDER_WIDTH, SIDEBAR_ITEM_SPACING as CHAPTER_LIST_SPACING,
    collapse_arrow, sidebar_entry_style,
};
use crate::ui::theme::color::primitive;
use crate::ui::theme::default_text_input;

use crate::ui::theme::AppTheme;

pub fn render_chapter_sidebar<'a>(
    state: &'a EpubState,
    theme: AppTheme,
    font_family: Option<&str>,
) -> Element<'a, Message> {
    let main_font = crate::ui::theme::font::get_main_font(font_family);
    let main_font_bold = crate::ui::theme::font::get_main_font_bold(font_family);
    let palette = theme.palette().base;
    let sidebar_header = build_sidebar_header(state, palette.text, main_font, main_font_bold);
    let search_bar = build_sidebar_search_bar(state, main_font, theme);

    let body_list = if !state.search.is_empty() {
        build_sidebar_search_results(state, theme, main_font)
    } else {
        build_sidebar_chapter_list(state, theme, main_font)
    };

    iced::widget::opaque(
        container(
            column![sidebar_header, search_bar, body_list]
                .width(state.sidebar_width)
                .height(Length::Fill),
        )
        .width(state.sidebar_width)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(palette.bg.into()),
            border: Border {
                width: BORDER_WIDTH,
                color: palette.border,
                radius: 0.0.into(),
            },
            ..Default::default()
        }),
    )
}

fn build_sidebar_search_bar<'a>(
    state: &'a EpubState,
    main_font: Font,
    theme: AppTheme,
) -> Element<'a, Message> {
    let palette = theme.palette().base;
    let is_searching = !state.search.is_empty();

    let input = text_input("Search in book...", &state.search.query)
        .id("epub_sidebar_search_input")
        .on_input(|q| crate::app::messages::EpubMsg::SearchQueryChanged(q).into())
        .on_submit(crate::app::messages::EpubMsg::SearchNext.into())
        .size(12)
        .font(main_font)
        .padding([5, 8])
        .style(default_text_input);

    let mut row_items = row![input].spacing(4).align_y(iced::Alignment::Center);

    if is_searching {
        let total = state.search.total_matches();
        let count_str = if total >= crate::features::epub::search::MAX_SEARCH_RESULTS {
            format!("{}/{}+", state.search.active_match_index + 1, total)
        } else {
            format!("{}/{}", state.search.active_match_index + 1, total)
        };
        let count_badge = text(count_str)
            .size(11)
            .font(main_font)
            .color(palette.text_dim);

        let text_color = palette.text;
        let text_dim_color = palette.text_dim;

        let prev_btn = button(text("▲").size(10).font(main_font))
            .on_press(crate::app::messages::EpubMsg::SearchPrev.into())
            .padding([2, 5])
            .style(move |_, _| button::Style {
                background: None,
                text_color,
                ..Default::default()
            });

        let next_btn = button(text("▼").size(10).font(main_font))
            .on_press(crate::app::messages::EpubMsg::SearchNext.into())
            .padding([2, 5])
            .style(move |_, _| button::Style {
                background: None,
                text_color,
                ..Default::default()
            });

        let clear_btn = button(text("✕").size(11).font(main_font))
            .on_press(crate::app::messages::EpubMsg::SearchClear.into())
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
    state: &'a EpubState,
    theme: AppTheme,
    main_font: Font,
) -> Element<'a, Message> {
    let palette = theme.palette().base;
    let mut elements: Vec<Element<'a, Message>> = Vec::new();

    if state.search.matches.is_empty() {
        elements.push(
            container(
                text("No matches found")
                    .size(12)
                    .font(main_font)
                    .color(palette.text_dim),
            )
            .padding(16)
            .into(),
        );
    } else {
        let mut last_chapter: Option<usize> = None;

        for (match_idx, m) in state.search.matches.iter().enumerate() {
            if last_chapter != Some(m.chapter_index) {
                last_chapter = Some(m.chapter_index);
                let ch_title = state
                    .chapters
                    .get(m.chapter_index)
                    .map(|c| c.title.as_str())
                    .unwrap_or("Chapter");

                elements.push(
                    container(
                        text(ch_title)
                            .size(11)
                            .font(crate::ui::theme::font::get_main_font_bold(None))
                            .color(palette.text_dim),
                    )
                    .padding(Padding {
                        top: 8.0,
                        right: 8.0,
                        bottom: 2.0,
                        left: 8.0,
                    })
                    .into(),
                );
            }

            let is_active = match_idx == state.search.active_match_index;
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
            .on_press(crate::app::messages::EpubMsg::SearchResultClicked(match_idx).into())
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

        if state.search.matches.len() >= crate::features::epub::search::MAX_SEARCH_RESULTS {
            elements.push(
                container(
                    text(format!(
                        "Showing first {} matches. Refine query for more.",
                        crate::features::epub::search::MAX_SEARCH_RESULTS
                    ))
                    .size(11)
                    .font(main_font)
                    .color(palette.text_dim),
                )
                .padding(Padding {
                    top: 10.0,
                    right: 8.0,
                    bottom: 10.0,
                    left: 8.0,
                })
                .into(),
            );
        }
    }

    scroll_pane(
        "search_scroll",
        column(elements).spacing(4).padding(CHAPTER_LIST_PADDING),
    )
    .build()
}

fn build_sidebar_header<'a>(
    state: &'a EpubState,
    text_color: Color,
    main_font: Font,
    main_font_bold: Font,
) -> Element<'a, Message> {
    let title = text("Chapters")
        .size(SIDEBAR_TOGGLE_FONT_SIZE)
        .font(main_font_bold)
        .color(text_color);

    let current_width = state.sidebar_width;

    container(
        row![
            title,
            iced::widget::Space::new().width(Length::Fill),
            build_resize_button("−", current_width - SIDEBAR_RESIZE_STEP, main_font),
            build_resize_button("+", current_width + SIDEBAR_RESIZE_STEP, main_font),
        ]
        .spacing(HEADER_SPACING)
        .align_y(iced::Alignment::Center)
        .padding([
            SIDEBAR_HEADER_PADDING_VERTICAL,
            SIDEBAR_HEADER_PADDING_HORIZONTAL,
        ]),
    )
    .into()
}

fn build_resize_button<'a>(
    label: &'a str,
    new_width: f32,
    main_font: Font,
) -> Element<'a, Message> {
    button(text(label).size(RESIZE_BUTTON_FONT_SIZE).font(main_font))
        .on_press(crate::app::messages::EpubMsg::SidebarResized(new_width).into())
        .padding([1, 4])
        .style(|_, _| button::Style {
            background: None,
            text_color: primitive::SIDEBAR_DARK_ARROW_TEXT,
            border: Border::default(),
            shadow: Shadow::default(),
            snap: false,
        })
        .into()
}

pub const CHAPTER_ROW_HEIGHT: f32 = 28.0;
pub const CHAPTER_SCROLL_CENTER_OFFSET: f32 = 100.0;

/// Calculates the target scroll offset in the chapter sidebar to keep the active chapter centered.
pub fn calculate_chapter_target_scroll_y(state: &EpubState, active_idx: usize) -> f32 {
    let mut visible_row_count = 0usize;
    let mut skip_until_level: Option<u8> = None;

    for (index, chapter) in state.chapters.iter().enumerate() {
        if let Some(target_level) = skip_until_level {
            if chapter.level > target_level {
                if index == active_idx {
                    // Even if collapsed, return current visible row position
                    return (visible_row_count as f32 * CHAPTER_ROW_HEIGHT
                        - CHAPTER_SCROLL_CENTER_OFFSET)
                        .max(0.0);
                }
                continue;
            }
            skip_until_level = None;
        }

        if index == active_idx {
            return (visible_row_count as f32 * CHAPTER_ROW_HEIGHT - CHAPTER_SCROLL_CENTER_OFFSET)
                .max(0.0);
        }

        let has_children = state
            .chapters
            .get(index + 1)
            .map(|next| next.level > chapter.level)
            .unwrap_or(false);

        let is_collapsed = state.collapsed_chapters.contains(&index);
        if is_collapsed && has_children {
            skip_until_level = Some(chapter.level);
        }

        visible_row_count += 1;
    }

    (visible_row_count as f32 * CHAPTER_ROW_HEIGHT - CHAPTER_SCROLL_CENTER_OFFSET).max(0.0)
}

fn build_sidebar_chapter_list<'a>(
    state: &'a EpubState,
    theme: AppTheme,
    main_font: Font,
) -> Element<'a, Message> {
    let palette = theme.palette().base;
    let is_light_background = palette.bg.r > 0.5;

    let mut entries: Vec<Element<'a, Message>> = Vec::new();
    let mut skip_until_level: Option<u8> = None;

    let has_hierarchy = state.chapters.windows(2).any(|w| w[1].level > w[0].level);

    for (index, chapter) in state.chapters.iter().enumerate() {
        if let Some(target_level) = skip_until_level {
            if chapter.level > target_level {
                continue;
            }
            skip_until_level = None;
        }

        let has_children = state
            .chapters
            .get(index + 1)
            .map(|next| next.level > chapter.level)
            .unwrap_or(false);

        let is_collapsed = state.collapsed_chapters.contains(&index);
        if is_collapsed && has_children {
            skip_until_level = Some(chapter.level);
        }

        let is_active = index == state.active_chapter;
        let entry = build_chapter_entry(
            index,
            chapter,
            is_active,
            has_children,
            is_collapsed,
            has_hierarchy,
            is_light_background,
            theme,
            main_font,
        );
        entries.push(entry);
    }

    scroll_pane(
        "chapter_scroll",
        column(entries)
            .spacing(CHAPTER_LIST_SPACING)
            .padding(CHAPTER_LIST_PADDING),
    )
    .build()
}

#[allow(clippy::too_many_arguments)]
fn build_chapter_entry<'a>(
    index: usize,
    chapter: &'a EpubChapterInfo,
    is_active: bool,
    has_children: bool,
    is_collapsed: bool,
    has_hierarchy: bool,
    is_light_background: bool,
    theme: AppTheme,
    main_font: Font,
) -> Element<'a, Message> {
    let indent = if has_hierarchy {
        calculate_indent(chapter.level)
    } else {
        0.0
    };
    let font_weight = entry_font_weight(chapter.level);
    let title_font_size = entry_font_size(chapter.level);
    let text_color = entry_text_color(is_active, is_light_background, chapter.level);

    let label = text(&chapter.title).size(title_font_size).font(Font {
        weight: font_weight,
        ..main_font
    });

    let mut row_content = row![]
        .spacing(CHAPTER_ENTRY_SPACING)
        .align_y(iced::Alignment::Center);

    if has_hierarchy {
        if has_children {
            let collapse_message =
                crate::app::messages::EpubMsg::ChapterToggleCollapse(index).into();
            let arrow = collapse_arrow(is_collapsed, theme, collapse_message);
            row_content = row_content.push(arrow);
        } else {
            // Reserve exact same width as collapse_arrow (ARROW_FONT_SIZE + padding) for alignment
            let spacer = iced::widget::Space::new().width(Length::Fixed(18.0));
            row_content = row_content.push(spacer);
        }
    }

    row_content = row_content.push(label);

    let entry_button = button(row_content)
        .on_press(crate::app::messages::EpubMsg::ChapterClicked(index).into())
        .width(Length::Fill)
        .style(move |iced_theme, status| {
            let mut style = sidebar_entry_style(iced_theme, status, is_active, theme);
            if let Some(color) = text_color {
                style.text_color = color;
            }
            style
        })
        .padding(Padding {
            top: ENTRY_PADDING_TOP,
            right: ENTRY_PADDING_RIGHT,
            bottom: ENTRY_PADDING_BOTTOM,
            left: ENTRY_PADDING_LEFT_BASE + indent,
        });

    container(entry_button).width(Length::Fill).into()
}
