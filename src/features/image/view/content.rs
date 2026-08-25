use crate::app::Message;
use crate::core::ImageState;
use crate::features::image::ImageCanvas;
use iced::{Element, Length};

pub fn render_image<'a>(state: &'a ImageState) -> Element<'a, Message> {
    ImageCanvas::new(state, &state.camera)
        .width(Length::Fill)
        .height(Length::Fill)
        .on_zoom(|factor, cursor| crate::app::messages::ImageMsg::Zoom { factor, cursor }.into())
        .on_drag(|dx, dy| crate::app::messages::ImageMsg::PanDelta(dx, dy).into())
        .on_double_click(|| crate::app::messages::ImageMsg::FitToWindow.into())
        .into()
}
