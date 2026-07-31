use iced::widget::{button, column, container, row, text};
use iced::{Border, Color, Element, Font, Length};

use crate::app::Message;
use crate::core::types::EpubState;
use crate::ui::components::sidebar::SIDEBAR_BORDER_WIDTH as BORDER_WIDTH;
use crate::ui::views::epub_view::constants::{
    AUTHOR_FONT_SCALE, HEADER_FONT_SCALE, HEADER_PADDING_HORIZONTAL, HEADER_PADDING_VERTICAL,
    SIDEBAR_TOGGLE_FONT_SIZE, TITLE_AUTHOR_SPACING, TOGGLE_BUTTON_PADDING_HORIZONTAL,
    TOGGLE_BUTTON_PADDING_VERTICAL,
};

pub fn build_epub_header<'a>(
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
    .on_press(crate::app::messages::EpubMsg::SidebarToggled.into())
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
