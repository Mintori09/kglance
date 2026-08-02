use crate::app::Message;
use iced::widget::{container, text};
use iced::{Element, Length};

use super::constants::{EMPTY_LABEL, EMPTY_LABEL_FONT_SIZE};

pub fn render_placeholder<'a>() -> Element<'a, Message> {
    container(text(EMPTY_LABEL).size(EMPTY_LABEL_FONT_SIZE))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
