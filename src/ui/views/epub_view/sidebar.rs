use iced::widget::{button, column, container, row, text};
use iced::{Border, Color, Element, Font, Length, Padding, Shadow};

use crate::app::Message;
use crate::core::types::{EpubChapterInfo, EpubState};
use crate::ui::components::scroll_pane::scroll_pane;
use crate::ui::components::sidebar::{
    SIDEBAR_BORDER_WIDTH as BORDER_WIDTH, SIDEBAR_ITEM_SPACING as CHAPTER_LIST_SPACING,
    collapse_arrow, sidebar_entry_style,
};
use crate::ui::theme::color::base::BaseColors;
use crate::ui::theme::color::primitive;
use crate::ui::views::epub_view::constants::{
    CHAPTER_ENTRY_SPACING, CHAPTER_LIST_PADDING, ENTRY_PADDING_BOTTOM, ENTRY_PADDING_LEFT_BASE,
    ENTRY_PADDING_RIGHT, ENTRY_PADDING_TOP, HEADER_SPACING, RESIZE_BUTTON_FONT_SIZE,
    SIDEBAR_HEADER_PADDING_HORIZONTAL, SIDEBAR_HEADER_PADDING_VERTICAL, SIDEBAR_RESIZE_STEP,
    SIDEBAR_TOGGLE_FONT_SIZE,
};
use crate::ui::views::epub_view::helpers::{
    calculate_indent, entry_font_size, entry_font_weight, entry_text_color,
};

pub fn render_chapter_sidebar<'a>(state: &'a EpubState, is_dark: bool) -> Element<'a, Message> {
    let palette = *BaseColors::palette_for(is_dark);
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

fn build_sidebar_chapter_list<'a>(state: &'a EpubState, is_dark: bool) -> Element<'a, Message> {
    let palette = *BaseColors::palette_for(is_dark);
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
    chapter: &'a EpubChapterInfo,
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
        let collapse_message = crate::app::messages::EpubMsg::ChapterToggleCollapse(index).into();
        let arrow = collapse_arrow(is_collapsed, is_dark, collapse_message);
        row_content = row_content.push(arrow);
    }

    row_content = row_content.push(label);

    let entry_button = button(row_content)
        .on_press(crate::app::messages::EpubMsg::ChapterClicked(index).into())
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
