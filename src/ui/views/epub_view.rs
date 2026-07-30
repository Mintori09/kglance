use std::cell::Cell;

use crate::app::Message;
use crate::core::types::EpubState;
use crate::ui::components::scroll_pane::scroll_pane;
use crate::ui::components::sidebar::{collapse_arrow, drag_handle, sidebar_entry_style};
use crate::ui::theme::glass;
use crate::ui::views::markdown_view::blocks::RenderContext;
use crate::ui::views::shared::content_layout::scrollable_content;
use iced::widget::{button, column, container, row, text};
use iced::{Border, Color, Element, Font, Length, Padding, Shadow};

const HEADER_FONT_SCALE: f32 = 1.15;
const AUTHOR_FONT_SCALE: f32 = 0.85;
const SIDEBAR_TOGGLE_FONT_SIZE: f32 = 12.0;
const RESIZE_BUTTON_FONT_SIZE: f32 = 11.0;
const CHAPTER_TITLE_SIZE_LEVEL_ONE: f32 = 12.0;
const CHAPTER_TITLE_SIZE_OTHER: f32 = 11.0;
const SIDEBAR_RESIZE_STEP: f32 = 30.0;
const INDENT_PER_LEVEL: f32 = 12.0;
const MAX_INDENT: f32 = 36.0;
const CONTENT_SPACING: f32 = 15.0;
const HEADER_SPACING: f32 = 4.0;
const CHAPTER_LIST_SPACING: f32 = 2.0;
const CHAPTER_LIST_PADDING: f32 = 6.0;
const CHAPTER_ENTRY_SPACING: f32 = 4.0;
const ENTRY_PADDING_TOP: f32 = 4.0;
const ENTRY_PADDING_RIGHT: f32 = 6.0;
const ENTRY_PADDING_BOTTOM: f32 = 4.0;
const ENTRY_PADDING_LEFT_BASE: f32 = 6.0;
const SIDEBAR_HEADER_PADDING_VERTICAL: f32 = 8.0;
const SIDEBAR_HEADER_PADDING_HORIZONTAL: f32 = 12.0;
const TOGGLE_BUTTON_PADDING_VERTICAL: f32 = 4.0;
const TOGGLE_BUTTON_PADDING_HORIZONTAL: f32 = 10.0;
const HEADER_PADDING_VERTICAL: f32 = 6.0;
const HEADER_PADDING_HORIZONTAL: f32 = 16.0;
const TITLE_AUTHOR_SPACING: f32 = 1.0;
const BORDER_WIDTH: f32 = 1.0;

pub fn view_epub<'a>(
    state: &'a EpubState,
    font_size: f32,
    is_dark: bool,
    font_family: Option<&str>,
    font_family_mono: Option<&str>,
    max_text_width: Option<f32>,
) -> Element<'a, Message> {
    let main_font = resolve_main_font(font_family);
    let active_chapter = clamp_active_chapter(state);
    let search_counter = Cell::new(0);

    let ctx = RenderContext {
        search_query: &state.markdown_state.search_query,
        active_match: state.markdown_state.search_match_index,
        counter: &search_counter,
        is_dark,
        font_size,
        font_family,
        font_family_mono,
    };

    let palette = *glass::palette_for(is_dark);
    let header_bar = build_epub_header(
        state,
        main_font,
        font_size,
        palette.text,
        palette.text_dim,
        palette.bg,
        palette.border,
    );
    let main_content = build_epub_content(state, active_chapter, &ctx, max_text_width);
    let main_view = column![header_bar, main_content].height(Length::Fill);

    if state.sidebar_visible && !state.chapters.is_empty() {
        let sidebar = render_chapter_sidebar(state, is_dark);
        let drag = drag_handle(
            state.sidebar_resizing,
            is_dark,
            Message::SidebarDragStarted(0.0),
        );
        row![sidebar, drag, main_view]
            .spacing(0)
            .height(Length::Fill)
            .into()
    } else {
        main_view.into()
    }
}

fn resolve_main_font(font_family: Option<&str>) -> Font {
    match font_family {
        Some(name) => Font::with_name(Box::leak(
            crate::ui::views::shared::font::resolve_font_name(name).into_boxed_str(),
        )),
        None => Font::DEFAULT,
    }
}

fn clamp_active_chapter(state: &EpubState) -> usize {
    state
        .active_chapter
        .min(state.chapters.len().saturating_sub(1))
}

fn build_epub_header<'a>(
    state: &'a EpubState,
    main_font: Font,
    font_size: f32,
    text_color: Color,
    dim_color: Color,
    bg_color: Color,
    border_color: Color,
) -> Element<'a, Message> {
    let title = text(&state.title)
        .size(font_size * HEADER_FONT_SCALE)
        .font(Font {
            weight: iced::font::Weight::Bold,
            ..main_font
        })
        .color(text_color);

    let author = text(format!("by {}", state.author))
        .size(font_size * AUTHOR_FONT_SCALE)
        .font(Font::DEFAULT)
        .color(dim_color);

    let toggle_button = button(
        text(if state.sidebar_visible {
            "Hide Chapters"
        } else {
            "Chapters"
        })
        .size(SIDEBAR_TOGGLE_FONT_SIZE)
        .font(Font::DEFAULT),
    )
    .on_press(Message::EpubSidebarToggled)
    .padding([
        TOGGLE_BUTTON_PADDING_VERTICAL,
        TOGGLE_BUTTON_PADDING_HORIZONTAL,
    ]);

    container(
        row![
            column![title, author].spacing(TITLE_AUTHOR_SPACING),
            iced::widget::Space::new().width(Length::Fill),
            toggle_button,
        ]
        .align_y(iced::Alignment::Center)
        .padding([HEADER_PADDING_VERTICAL, HEADER_PADDING_HORIZONTAL]),
    )
    .style(move |_| container::Style {
        background: Some(bg_color.into()),
        border: Border {
            width: BORDER_WIDTH,
            color: border_color,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn build_epub_content<'a>(
    state: &'a EpubState,
    active_chapter: usize,
    ctx: &RenderContext<'_>,
    max_text_width: Option<f32>,
) -> Element<'a, Message> {
    let chapter_blocks: &[crate::parsers::markdown::Block] = state
        .chapters
        .get(active_chapter)
        .map(|ch| ch.blocks.as_slice())
        .unwrap_or(&[]);

    let chapter_offset: usize = state
        .chapters
        .iter()
        .take(active_chapter)
        .map(|ch| ch.blocks.len())
        .sum();

    let elements = chapter_blocks.iter().enumerate().map(|(i, block)| {
        let global_index = chapter_offset + i;
        let inner = crate::ui::views::markdown_view::render_block(
            global_index,
            block,
            &state.markdown_state,
            ctx,
        );
        let margin_bottom = crate::ui::views::markdown_view::block_margin(block);
        container(inner)
            .padding(Padding {
                top: 0.0,
                right: 0.0,
                bottom: margin_bottom,
                left: 0.0,
            })
            .width(Length::Fill)
            .into()
    });

    scrollable_content(elements, max_text_width, CONTENT_SPACING, "content_scroll").build()
}

fn render_chapter_sidebar<'a>(state: &'a EpubState, is_dark: bool) -> Element<'a, Message> {
    let palette = *glass::palette_for(is_dark);
    let sidebar_header = build_sidebar_header(state, palette.text);
    let chapter_list = build_sidebar_chapter_list(state, is_dark);

    container(
        column![sidebar_header, chapter_list]
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
    })
    .into()
}

fn build_sidebar_header<'a>(state: &'a EpubState, text_color: Color) -> Element<'a, Message> {
    let title = text("Chapters")
        .size(SIDEBAR_TOGGLE_FONT_SIZE)
        .font(Font {
            weight: iced::font::Weight::Bold,
            ..Font::DEFAULT
        })
        .color(text_color);

    let current_width = state.sidebar_width;

    container(
        row![
            title,
            iced::widget::Space::new().width(Length::Fill),
            build_resize_button("−", current_width - SIDEBAR_RESIZE_STEP),
            build_resize_button("+", current_width + SIDEBAR_RESIZE_STEP),
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

fn build_resize_button<'a>(label: &'a str, new_width: f32) -> Element<'a, Message> {
    button(
        text(label)
            .size(RESIZE_BUTTON_FONT_SIZE)
            .font(Font::DEFAULT),
    )
    .on_press(Message::EpubSidebarResized(new_width))
    .padding([1, 4])
    .style(|_, _| button::Style {
        background: None,
        text_color: Color::from_rgb(0.6, 0.65, 0.7),
        border: Border::default(),
        shadow: Shadow::default(),
        snap: false,
    })
    .into()
}

fn build_sidebar_chapter_list<'a>(state: &'a EpubState, is_dark: bool) -> Element<'a, Message> {
    let palette = *glass::palette_for(is_dark);
    let is_light_background = palette.bg.r > 0.5;

    let mut entries: Vec<Element<'a, Message>> = Vec::new();
    let mut skip_until_level: Option<u8> = None;

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
            is_light_background,
            is_dark,
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

fn build_chapter_entry<'a>(
    index: usize,
    chapter: &'a crate::core::types::EpubChapterInfo,
    is_active: bool,
    has_children: bool,
    is_collapsed: bool,
    is_light_background: bool,
    is_dark: bool,
) -> Element<'a, Message> {
    let indent = calculate_indent(chapter.level);
    let font_weight = entry_font_weight(chapter.level);
    let title_font_size = entry_font_size(chapter.level);
    let text_color = entry_text_color(is_active, is_light_background, chapter.level);

    let label = text(&chapter.title).size(title_font_size).font(Font {
        weight: font_weight,
        ..Font::DEFAULT
    });

    let mut row_content = row![]
        .spacing(CHAPTER_ENTRY_SPACING)
        .align_y(iced::Alignment::Center);

    if has_children {
        let collapse_message = Message::EpubChapterToggleCollapse(index);
        let arrow = collapse_arrow(is_collapsed, is_dark, collapse_message);
        row_content = row_content.push(arrow);
    }

    row_content = row_content.push(label);

    let entry_button = button(row_content)
        .on_press(Message::EpubChapterClicked(index))
        .width(Length::Fill)
        .style(move |theme, status| {
            let mut style = sidebar_entry_style(theme, status, is_active, is_dark);
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

fn calculate_indent(level: u8) -> f32 {
    ((level.saturating_sub(1)) as f32 * INDENT_PER_LEVEL).min(MAX_INDENT)
}

fn entry_font_weight(level: u8) -> iced::font::Weight {
    if level == 1 {
        iced::font::Weight::Bold
    } else {
        iced::font::Weight::Normal
    }
}

fn entry_font_size(level: u8) -> f32 {
    if level == 1 {
        CHAPTER_TITLE_SIZE_LEVEL_ONE
    } else {
        CHAPTER_TITLE_SIZE_OTHER
    }
}

fn entry_text_color(is_active: bool, is_light_background: bool, level: u8) -> Option<Color> {
    if is_active {
        return None;
    }
    if is_light_background {
        if level == 1 {
            Some(Color::from_rgb(0.2, 0.22, 0.25))
        } else {
            Some(Color::from_rgb(0.4, 0.42, 0.45))
        }
    } else if level == 1 {
        Some(Color::from_rgb(0.9, 0.92, 0.95))
    } else {
        Some(Color::from_rgb(0.75, 0.78, 0.82))
    }
}
