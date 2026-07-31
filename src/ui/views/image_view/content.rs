use crate::app::Message;
use crate::core::ImageState;
use crate::preview::image::ImageCanvas;
use iced::{Element, Length};

pub fn render_image<'a>(state: &'a ImageState) -> Element<'a, Message> {
    ImageCanvas::new(state, &state.camera)
        .width(Length::Fill)
        .height(Length::Fill)
        .on_drag(Message::ImagePanDelta)
        .on_double_click(|| Message::ImageDoubleClick)
        .into()
}
