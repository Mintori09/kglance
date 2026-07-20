use crate::app::Message;
use crate::core::ImageState;
use iced::widget::{container, image, scrollable};
use iced::{Element, Length, Theme};

pub fn view_image<'a>(state: &'a ImageState) -> Element<'a, Message> {
    let handle = image::Handle::from_bytes(state.image_bytes.clone());
    
    let w = state.image_width as f32 * state.zoom;
    let h = state.image_height as f32 * state.zoom;
    
    let img = image(handle)
        .width(Length::Fixed(w))
        .height(Length::Fixed(h))
        .content_fit(iced::ContentFit::Contain);

    let scroll = scrollable(
        container(img)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
    )
    .id("content_scroll")
    .width(Length::Fill)
    .height(Length::Fill);

    container(scroll)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(palette.background.strong.color.into()),
                text_color: None,
                border: Default::default(),
                shadow: Default::default(),
                snap: false,
            }
        })
        .into()
}

