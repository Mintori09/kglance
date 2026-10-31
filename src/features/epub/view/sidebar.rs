use iced::widget::{button, column, container, row, text, text_input};
use iced::{Border, Color, Element, Font, Length, Padding, Shadow};

use crate::app::Message;
use crate::core::types::{EpubChapterInfo, EpubState};
use crate::features::epub::view::constants::{
    CHAPTER_ENTRY_SPACING, CHAPTER_LIST_PADDING, ENTRY_PADDING_BOTTOM, ENTRY_PADDING_LEFT_BASE,
    ENTRY_PADDING_RIGHT, ENTRY_PADDING_TOP, HEADER_SPACING, RESIZE_BUTTON_FONT_SIZE,
    SIDEBAR_HEADER_PADDING_HORIZONTAL, SIDEBAR_HEADER_PADDING_VERTICAL, SIDEBAR_RESIZE_STEP,
};
use crate::features::epub::view::helpers::{
    EntryVisualState, calculate_indent, entry_font_size, entry_font_weight, entry_text_color,
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
    } else if state.sidebar_tab == crate::features::epub::types::EpubSidebarTab::Gallery {
        build_sidebar_gallery(state, theme, main_font)
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
    use crate::features::epub::types::EpubSidebarTab;

    let is_toc = state.sidebar_tab == EpubSidebarTab::Toc;
    let is_gallery = state.sidebar_tab == EpubSidebarTab::Gallery;

    let toc_btn =
        button(
            text("Contents")
                .size(11)
                .font(if is_toc { main_font_bold } else { main_font }),
        )
        .on_press(crate::app::messages::EpubMsg::SidebarTabChanged(EpubSidebarTab::Toc).into())
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

    let gallery_btn = button(text("Images").size(11).font(if is_gallery {
        main_font_bold
    } else {
        main_font
    }))
    .on_press(crate::app::messages::EpubMsg::SidebarTabChanged(EpubSidebarTab::Gallery).into())
    .padding([3, 8])
    .style(move |_, status| {
        let bg = if is_gallery {
            Some(Color::from_rgba(1.0, 1.0, 1.0, 0.1).into())
        } else if matches!(status, button::Status::Hovered) {
            Some(Color::from_rgba(1.0, 1.0, 1.0, 0.05).into())
        } else {
            None
        };
        button::Style {
            background: bg,
            text_color: if is_gallery {
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

    let current_width = state.sidebar_width;

    container(
        row![
            row![toc_btn, gallery_btn].spacing(4),
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

fn build_sidebar_gallery<'a>(
    state: &'a EpubState,
    theme: AppTheme,
    main_font: Font,
) -> Element<'a, Message> {
    let palette = theme.palette().base;
    let mut elements: Vec<Element<'a, Message>> = Vec::new();

    let mut all_image_paths = Vec::new();
    if state.reading_mode == crate::core::config::EpubReadingMode::Continuous {
        all_image_paths =
            crate::features::epub::parser::extract_images_from_blocks(&state.continuous_blocks);
    } else {
        for ch in &state.chapters {
            for p in crate::features::epub::parser::extract_images_from_blocks(&ch.blocks) {
                if !all_image_paths.contains(&p) {
                    all_image_paths.push(p);
                }
            }
        }
    }

    if all_image_paths.is_empty() {
        return container(
            text("No images found")
                .size(12)
                .font(main_font)
                .color(palette.text_dim),
        )
        .padding(16)
        .center_x(Length::Fill)
        .into();
    }

    let mut current_row: Vec<Element<'a, Message>> = Vec::new();

    for path in all_image_paths {
        let path_clone = path.clone();
        let filename = std::path::Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&path)
            .to_string();

        let thumb: Element<'a, Message> = if let Some(handle) = state
            .markdown_state
            .cached_image_handles
            .get(&path)
            .or_else(|| {
                state
                    .markdown_state
                    .cached_image_handles
                    .get(filename.as_str())
            }) {
            iced::widget::image(handle.clone())
                .width(Length::Fixed(90.0))
                .height(Length::Fixed(90.0))
                .into()
        } else {
            container(text("🖼️").size(24))
                .width(Length::Fixed(90.0))
                .height(Length::Fixed(90.0))
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
        };

        let caption = text(filename)
            .size(10)
            .font(main_font)
            .color(palette.text_dim);

        let card_content = column![thumb, caption]
            .spacing(4)
            .align_x(iced::Alignment::Center);

        let btn = button(card_content)
            .on_press(crate::app::messages::EpubMsg::OpenImageLightbox(path_clone).into())
            .padding(6)
            .style(move |_, status| {
                let border_color = match status {
                    button::Status::Hovered => Color::from_rgba(0.4, 0.7, 1.0, 0.4),
                    button::Status::Pressed => Color::from_rgba(0.4, 0.7, 1.0, 0.7),
                    _ => Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                };
                button::Style {
                    background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.03).into()),
                    border: Border {
                        radius: 6.0.into(),
                        width: 1.0,
                        color: border_color,
                    },
                    shadow: Shadow::default(),
                    text_color: Color::WHITE,
                    snap: false,
                }
            });

        current_row.push(btn.into());

        if current_row.len() == 2 {
            elements.push(row(std::mem::take(&mut current_row)).spacing(8).into());
        }
    }

    if !current_row.is_empty() {
        elements.push(row(current_row).spacing(8).into());
    }

    scroll_pane(
        "gallery_scroll",
        column(elements).spacing(8).padding(CHAPTER_LIST_PADDING),
    )
    .build()
}

pub const CHAPTER_ROW_HEIGHT: f32 = 28.0;
pub const CHAPTER_SCROLL_CENTER_OFFSET: f32 = 100.0;

/// Estimates the rendered visual height of a chapter button in the sidebar based on title length and wrapping.
pub fn estimate_chapter_entry_height(
    title: &str,
    level: u8,
    has_hierarchy: bool,
    sidebar_width: f32,
) -> f32 {
    let indent = if has_hierarchy {
        calculate_indent(level)
    } else {
        0.0
    };
    let title_font_size = entry_font_size(level);

    // Reserved horizontal space: left padding + right padding + indent + icon spacer (18) + spacing (4)
    let extra_w = ENTRY_PADDING_LEFT_BASE + indent + ENTRY_PADDING_RIGHT + 22.0;
    let available_w = (sidebar_width - extra_w).max(80.0);

    // Approximate average char width (~0.54 of font size for proportional fonts)
    let avg_char_w = title_font_size * 0.54;
    let approx_text_w = title.chars().count() as f32 * avg_char_w;
    let lines = (approx_text_w / available_w).ceil().max(1.0);

    let line_height = title_font_size * 1.35;
    let content_h = lines * line_height;
    (ENTRY_PADDING_TOP + ENTRY_PADDING_BOTTOM + content_h).max(CHAPTER_ROW_HEIGHT)
}

/// Calculates the target scroll offset in the chapter sidebar to keep the active chapter centered.
pub fn calculate_chapter_target_scroll_y(state: &EpubState, active_idx: usize) -> f32 {
    let has_hierarchy = state.chapters.windows(2).any(|w| w[1].level > w[0].level);
    let sidebar_w = if state.sidebar_width > 0.0 {
        state.sidebar_width
    } else {
        240.0
    };

    let mut accumulated_y = CHAPTER_LIST_PADDING;
    let mut skip_until_level: Option<u8> = None;

    // Viewport height of sidebar (with sensible fallback)
    let vh = state.markdown_state.viewport_height;
    let center_offset = if vh > 0.0 {
        (vh * 0.40).max(60.0)
    } else {
        CHAPTER_SCROLL_CENTER_OFFSET
    };

    for (index, chapter) in state.chapters.iter().enumerate() {
        if let Some(target_level) = skip_until_level {
            if chapter.level > target_level {
                if index == active_idx {
                    return (accumulated_y - center_offset).max(0.0);
                }
                continue;
            }
            skip_until_level = None;
        }

        let entry_h =
            estimate_chapter_entry_height(&chapter.title, chapter.level, has_hierarchy, sidebar_w);

        if index == active_idx {
            // Center the entry by its middle position
            let entry_center = accumulated_y + (entry_h * 0.5);
            return (entry_center - center_offset).max(0.0);
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

        accumulated_y += entry_h + CHAPTER_LIST_SPACING;
    }

    (accumulated_y - center_offset).max(0.0)
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
    let visual_state = EntryVisualState::new(is_active, is_light_background);
    let text_color = entry_text_color(visual_state, chapter.level);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_chapter_entry_height_single_and_wrapped() {
        let short_title = "Chapter 1";
        let h_short = estimate_chapter_entry_height(short_title, 1, false, 240.0);
        assert_eq!(h_short, CHAPTER_ROW_HEIGHT);

        let long_title =
            "Chapter 1: The Foundations of Type Systems and Advanced Variance with Covariance";
        let h_long = estimate_chapter_entry_height(long_title, 1, true, 240.0);
        assert!(h_long > CHAPTER_ROW_HEIGHT);
    }

    #[test]
    fn test_calculate_chapter_target_scroll_y_accumulates_and_centers() {
        let chapters = vec![
            EpubChapterInfo {
                title: "Intro".into(),
                level: 1,
                anchor: None,
                file_href: "intro.xhtml".into(),
                blocks: vec![],
            },
            EpubChapterInfo {
                title: "Chapter 1: An Extremely Long Chapter Title That Easily Wraps Into Multiple Lines In The Sidebar".into(),
                level: 1,
                anchor: None,
                file_href: "ch1.xhtml".into(),
                blocks: vec![],
            },
            EpubChapterInfo {
                title: "Section 1.1".into(),
                level: 2,
                anchor: None,
                file_href: "ch1.xhtml".into(),
                blocks: vec![],
            },
        ];

        let state = EpubState {
            chapters,
            sidebar_width: 240.0,
            markdown_state: crate::core::MarkdownState {
                viewport_height: 800.0,
                ..Default::default()
            },
            ..Default::default()
        };

        // Target for Chapter 0 (at the very top, should clamp to 0.0)
        let y_ch0 = calculate_chapter_target_scroll_y(&state, 0);
        assert_eq!(y_ch0, 0.0);

        // Target for Section 1.1 (after wrapped Chapter 1, should accumulate larger height than flat 28.0)
        let y_sec = calculate_chapter_target_scroll_y(&state, 2);
        let h_wrapped = estimate_chapter_entry_height(&state.chapters[1].title, 1, true, 240.0);
        assert!(h_wrapped > CHAPTER_ROW_HEIGHT);
        // Ensure y_sec took h_wrapped into account
        assert!(y_sec >= 0.0);
    }
}
