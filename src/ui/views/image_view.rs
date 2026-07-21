use crate::app::Message;
use crate::core::ImageState;
use iced::{Element, Length};

pub fn view_image<'a>(state: &'a ImageState) -> Element<'a, Message> {
    if state.handle.is_some() {
        crate::preview::image::ImageCanvas::new(state, &state.camera)
            .width(Length::Fill)
            .height(Length::Fill)
            .on_drag(Message::ImagePanDelta)
            .on_double_click(|| Message::ImageDoubleClick)
            .into()
    } else {
        iced::widget::container(iced::widget::text("No image loaded").size(18))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}
