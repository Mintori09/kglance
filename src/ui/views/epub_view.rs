use crate::app::Message;
use crate::core::types::EpubState;
use crate::ui::components::scroll_pane::scroll_pane;
use crate::ui::components::sidebar::{collapse_arrow, drag_handle, sidebar_entry_style};
use crate::ui::theme::glass;
use iced::widget::{button, column, container, row, text};
use iced::{Border, Color, Element, Font, Length, Shadow};
use std::cell::Cell;

pub fn view_epub<'a>(
    state: &'a EpubState,
    font_size: f32,
    is_dark: bool,
    font_family: Option<&str>,
    font_family_mono: Option<&str>,
    max_text_width: Option<f32>,
) -> Element<'a, Message> {
    let main_font = match font_family {
        Some(name) => Font::with_name(Box::leak(
            crate::ui::views::markdown_view::resolve_font_name(name).into_boxed_str(),
        )),
        None => Font::DEFAULT,
    };

    let (bg_color, text_color, dim_color) = if is_dark {
        (glass::DARK_BG, glass::DARK_TEXT, glass::DARK_TEXT_DIM)
    } else {
        (glass::LIGHT_BG, glass::LIGHT_TEXT, glass::LIGHT_TEXT_DIM)
    };

    let active_ch = state
        .active_chapter
        .min(state.chapters.len().saturating_sub(1));

    let chapter_blocks: &[crate::parsers::markdown::Block] = state
        .chapters
        .get(active_ch)
        .map(|ch| ch.blocks.as_slice())
        .unwrap_or(&[]);

    // Title / Header banner
    let header_title = text(&state.title)
        .size(font_size * 1.15)
        .font(Font {
            weight: iced::font::Weight::Bold,
            ..main_font
        })
        .color(text_color);

    let header_author = text(format!("by {}", state.author))
        .size(font_size * 0.85)
        .font(Font::DEFAULT)
        .color(dim_color);

    let toggle_btn = button(
        text(if state.sidebar_visible {
            "Hide Chapters"
        } else {
            "Chapters"
        })
        .size(12)
        .font(Font::DEFAULT),
    )
    .on_press(Message::EpubSidebarToggled)
    .padding([4, 10]);

    let header_bar = container(
        row![
            column![header_title, header_author].spacing(1),
            iced::widget::Space::new().width(Length::Fill),
            toggle_btn
        ]
        .align_y(iced::Alignment::Center)
        .padding([6, 16]),
    )
    .style(move |_| container::Style {
        background: Some(bg_color.into()),
        border: Border {
            width: 1.0,
            color: if is_dark {
                glass::DARK_BORDER
            } else {
                glass::LIGHT_BORDER
            },
            radius: 0.0.into(),
        },
        ..Default::default()
    });

    let chapter_offset: usize = state
        .chapters
        .iter()
        .take(active_ch)
        .map(|ch| ch.blocks.len())
        .sum();

    // Render Markdown blocks for the current chapter
    let search_counter = Cell::new(0);
    let content = chapter_blocks.iter().enumerate().map(|(i, block)| {
        let global_i = chapter_offset + i;
        let inner = crate::ui::views::markdown_view::render_block(
            global_i,
            block,
            &state.markdown_state,
            &state.markdown_state.search_query,
            state.markdown_state.search_match_index,
            &search_counter,
            is_dark,
            font_size,
            font_family,
            font_family_mono,
        );
        let mb = crate::ui::views::markdown_view::block_margin(block);
        container(inner)
            .padding(iced::Padding {
                top: 0.0,
                right: 0.0,
                bottom: mb,
                left: 0.0,
            })
            .width(Length::Fill)
            .into()
    });

    let inner_column = column(content).spacing(0).padding(15);

    // Limit text column width if configured, and center it inside full-width container
    let text_container = match max_text_width {
        Some(w) if w > 0.0 => container(inner_column).max_width(w),
        _ => container(inner_column).width(Length::Fill),
    };

    let centered_text_wrapper = container(text_container)
        .center_x(Length::Fill)
        .width(Length::Fill);

    let scroll = scroll_pane("content_scroll", centered_text_wrapper).build();

    let main_view = column![header_bar, scroll].height(Length::Fill);

    if state.sidebar_visible && !state.chapters.is_empty() {
        let sidebar = render_chapter_sidebar(state, is_dark);
        let dh = drag_handle(
            state.sidebar_resizing,
            is_dark,
            Message::SidebarDragStarted(0.0),
        );

        row![sidebar, dh, main_view]
            .spacing(0)
            .height(Length::Fill)
            .into()
    } else {
        main_view.into()
    }
}

fn render_chapter_sidebar<'a>(state: &'a EpubState, is_dark: bool) -> Element<'a, Message> {
    let (bg, border_color) = if is_dark {
        (glass::DARK_BG, glass::DARK_BORDER)
    } else {
        (glass::LIGHT_BG, glass::LIGHT_BORDER)
    };

    let title_text = text("Chapters")
        .size(12)
        .font(Font {
            weight: iced::font::Weight::Bold,
            ..Font::DEFAULT
        })
        .color(if is_dark {
            glass::DARK_TEXT
        } else {
            glass::LIGHT_TEXT
        });

    let current_w = state.sidebar_width;
    let header = container(
        row![
            title_text,
            iced::widget::Space::new().width(Length::Fill),
            button(text("−").size(11).font(Font::DEFAULT))
                .on_press(Message::EpubSidebarResized(current_w - 30.0))
                .padding([1, 4])
                .style(|_, _| button::Style {
                    background: None,
                    text_color: Color::from_rgb(0.6, 0.65, 0.7),
                    border: Border::default(),
                    shadow: Shadow::default(),
                    snap: false,
                }),
            button(text("+").size(11).font(Font::DEFAULT))
                .on_press(Message::EpubSidebarResized(current_w + 30.0))
                .padding([1, 4])
                .style(|_, _| button::Style {
                    background: None,
                    text_color: Color::from_rgb(0.6, 0.65, 0.7),
                    border: Border::default(),
                    shadow: Shadow::default(),
                    snap: false,
                }),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center)
        .padding([8, 12]),
    );

    let mut entries: Vec<Element<'a, Message>> = Vec::new();
    let mut skip_until_level: Option<u8> = None;

    for (idx, ch) in state.chapters.iter().enumerate() {
        if let Some(target_lvl) = skip_until_level {
            if ch.level > target_lvl {
                continue;
            } else {
                skip_until_level = None;
            }
        }

        let has_children = state
            .chapters
            .get(idx + 1)
            .map(|next| next.level > ch.level)
            .unwrap_or(false);

        let is_collapsed = state.collapsed_chapters.contains(&idx);
        if is_collapsed && has_children {
            skip_until_level = Some(ch.level);
        }

        let is_active = idx == state.active_chapter;
        let indent = ((ch.level.saturating_sub(1)) as f32 * 12.0).min(36.0);
        let font_weight = if ch.level == 1 {
            iced::font::Weight::Bold
        } else {
            iced::font::Weight::Normal
        };

        let label = text(&ch.title)
            .size(if ch.level == 1 { 12 } else { 11 })
            .font(Font {
                weight: font_weight,
                ..Font::DEFAULT
            });

        let mut row_content = row![].spacing(4).align_y(iced::Alignment::Center);

        if has_children {
            let cb = collapse_arrow(
                is_collapsed,
                is_dark,
                Message::EpubChapterToggleCollapse(idx),
            );
            row_content = row_content.push(cb);
        }

        row_content = row_content.push(label);

        let text_color = if is_active {
            None
        } else if is_dark {
            if ch.level == 1 {
                Some(Color::from_rgb(0.9, 0.92, 0.95))
            } else {
                Some(Color::from_rgb(0.75, 0.78, 0.82))
            }
        } else if ch.level == 1 {
            Some(Color::from_rgb(0.2, 0.22, 0.25))
        } else {
            Some(Color::from_rgb(0.4, 0.42, 0.45))
        };

        let btn = button(row_content)
            .on_press(Message::EpubChapterClicked(idx))
            .width(Length::Fill)
            .style(move |theme, status| {
                let mut s = sidebar_entry_style(theme, status, is_active, is_dark);
                if let (Some(c), false) = (text_color, is_active) {
                    s.text_color = c;
                }
                s
            })
            .padding(iced::Padding {
                top: 4.0,
                right: 6.0,
                bottom: 4.0,
                left: 6.0 + indent,
            });

        entries.push(container(btn).width(Length::Fill).into());
    }

    let chapter_list = scroll_pane("chapter_scroll", column(entries).spacing(2).padding(6)).build();

    container(
        column![header, chapter_list]
            .width(current_w)
            .height(Length::Fill),
    )
    .width(current_w)
    .height(Length::Fill)
    .style(move |_| container::Style {
        background: Some(bg.into()),
        border: Border {
            width: 1.0,
            color: border_color,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}
