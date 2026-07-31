use crate::app::Message;
use crate::ui::components::scroll_pane::{ScrollPaneBuilder, scroll_pane};
use iced::{Element, Length};

pub fn scrollable_content<'a>(
    content_elements: impl IntoIterator<Item = Element<'a, Message>>,
    max_text_width: Option<f32>,
    content_padding: f32,
    scroll_id: &'static str,
) -> ScrollPaneBuilder<'a, Message> {
    let inner_column = iced::widget::column(content_elements)
        .spacing(0)
        .padding(content_padding);

    let text_container = match max_text_width {
        Some(width) if width > 0.0 => iced::widget::container(inner_column).max_width(width),
        _ => iced::widget::container(inner_column).width(Length::Fill),
    };

    let centered = iced::widget::container(text_container)
        .center_x(Length::Fill)
        .width(Length::Fill);

    scroll_pane(scroll_id, centered)
}
