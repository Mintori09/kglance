pub mod constants;
pub mod content;
pub mod empty;
pub mod helpers;

use crate::app::Message;
use crate::core::ImageState;
use iced::Element;

use content::render_image;
use empty::render_placeholder;
use helpers::is_loaded;

pub fn view_image<'a>(state: &'a ImageState) -> Element<'a, Message> {
    if is_loaded(state) {
        render_image(state)
    } else {
        render_placeholder()
    }
}
