use crate::app::Message;
use crate::core::preview::{ContentType, PreviewContent};
use crate::core::types::KglanceState;
use iced::Element;

pub struct FontContent {
    pub name: String,
    pub metadata: String,
    pub sample: Vec<u8>,
    pub sample_width: u32,
    pub sample_height: u32,
}

impl PreviewContent<Message> for FontContent {
    fn populate_state(&self, state: &mut KglanceState) {
        let content = format!("Font: {}\n\n{}", self.name, self.metadata);
        state.text.content = iced::widget::text_editor::Content::with_text(&content);
        state.text.extension = format!("Font ({})", self.name);
        state.file_type_text = "Font File".to_string();
    }

    fn view<'a>(&'a self, state: &'a KglanceState) -> Element<'a, Message> {
        crate::ui::views::view_text(&state.text, state.theme_dark, state.font_size)
    }

    fn content_type(&self) -> ContentType {
        ContentType::Font
    }
}
