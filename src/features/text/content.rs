use crate::app::Message;
use crate::core::preview::{ContentType, PreviewContent};
use crate::core::types::KglanceState;
use iced::Element;

pub struct TextContent {
    pub content: String,
    pub language: String,
    pub line_count: usize,
    pub highlighted_html: Option<String>,
}

impl PreviewContent<Message> for TextContent {
    fn populate_state(&self, state: &mut KglanceState) {
        state.text.content = iced::widget::text_editor::Content::with_text(&self.content);
        state.text.extension = self.language.clone();
        state.text.line_numbers = (1..=self.line_count)
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        state.file_type_text = self.language.clone();
    }

    fn view<'a>(&'a self, state: &'a KglanceState) -> Element<'a, Message> {
        crate::ui::views::view_text(&state.text, state.theme_dark, state.font_size)
    }

    fn content_type(&self) -> ContentType {
        ContentType::Text
    }
    fn supports_text_operations(&self) -> bool {
        true
    }
}
