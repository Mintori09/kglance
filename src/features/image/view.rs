use crate::app::Message;
use crate::core::ImageState;
use crate::features::image::ImageCanvas;
use iced::Element;
use iced::Length;
use iced::Size;
use iced::widget::{container, text};

const EMPTY_LABEL: &str = "No image loaded";
const EMPTY_LABEL_FONT_SIZE: f32 = 18.0;
const HEADER_HEIGHT: f32 = 50.0;

pub fn is_loaded(state: &ImageState) -> bool {
    state.handle.is_some() || state.preview_handle.is_some() || state.display_handle.is_some()
}

pub fn view_image<'a>(state: &'a ImageState) -> Element<'a, Message> {
    render_image(state)
}

pub fn calculate_window_size(
    window_w: f32,
    window_h: f32,
    img_width: u32,
    img_height: u32,
) -> Size {
    if img_width == 0 || img_height == 0 {
        return Size::new(window_w, window_h);
    }

    let img_w = img_width as f32;
    let img_h = img_height as f32;

    let scale = (window_w / img_w).min(window_h / img_h).min(1.0);

    Size::new(img_w * scale, img_h * scale + HEADER_HEIGHT)
}

pub fn render_image<'a>(state: &'a ImageState) -> Element<'a, Message> {
    ImageCanvas::new(state, &state.camera)
        .width(Length::Fill)
        .height(Length::Fill)
        .on_zoom(|factor, cursor| crate::app::messages::ImageMsg::Zoom { factor, cursor }.into())
        .on_drag(|dx, dy| crate::app::messages::ImageMsg::PanDelta(dx, dy).into())
        .on_double_click(|| crate::app::messages::ImageMsg::FitToWindow.into())
        .into()
}

pub fn render_placeholder<'a>() -> Element<'a, Message> {
    container(text(EMPTY_LABEL).size(EMPTY_LABEL_FONT_SIZE))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
