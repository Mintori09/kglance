use crate::app::Message;
use crate::core::ImageState;
use iced::widget::{container, image, scrollable};
use iced::{Element, Length, Theme};

pub fn view_image<'a>(state: &'a ImageState) -> Element<'a, Message> {
    let content: Element<'a, Message> = if let Some(handle) = &state.handle {
        let img = if state.zoom == 1.0 {
            image(handle.clone())
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(iced::ContentFit::Contain)
        } else {
            let w = state.image_width as f32 * state.zoom;
            let h = state.image_height as f32 * state.zoom;

            image(handle.clone())
                .width(Length::Fixed(w))
                .height(Length::Fixed(h))
                .content_fit(iced::ContentFit::Contain)
        };

        container(img)
            .width(if state.zoom == 1.0 {
                Length::Fill
            } else {
                Length::Shrink
            })
            .height(if state.zoom == 1.0 {
                Length::Fill
            } else {
                Length::Shrink
            })
            .into()
    } else {
        container(iced::widget::text("No images!")).into()
    };

    let content_wrapper = container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill);

    let scroll = scrollable(content_wrapper)
        .id("content_scroll")
        .width(Length::Fill)
        .height(Length::Fill);

    container(scroll)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(palette.background.strong.color.into()),
                ..Default::default()
            }
        })
        .into()
}
