use super::style::STYLE;
use crate::app::Message;
use iced::widget::{column, container, image};
use iced::{Element, Length};

pub(crate) fn render_inline_image<'a>(
    index: usize,
    state: &'a crate::core::MarkdownState,
) -> Element<'a, Message> {
    let Some(handle) = state.cached_image_handles.get(&index) else {
        return container(column![]).into();
    };

    let img = image(handle.clone()).height(Length::Shrink);
    let img = match state.cached_image_sizes.get(&index) {
        Some((width, _)) if *width as f32 > STYLE.image.max_width => {
            img.width(Length::Fixed(STYLE.image.max_width))
        }
        _ => img.width(Length::Shrink),
    };

    container(img)
        .center_x(Length::Fill)
        .padding(STYLE.image.padding)
        .width(Length::Fill)
        .into()
}
