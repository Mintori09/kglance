use crate::core::types::KglanceState;
use std::path::Path;

pub fn populate_state(
    state: &mut KglanceState,
    content: String,
    line_numbers: String,
    language: &str,
) {
    let words = content.split_whitespace().count();
    let chars = content.chars().count();
    let mins = (words as f32 / 200.0).ceil() as usize;

    state.text.content = iced::widget::text_editor::Content::with_text(&content);
    let path_ext = if !state.file_name.is_empty() {
        Path::new(&state.file_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or(language)
    } else {
        language
    };
    state.text.extension = path_ext.to_string();
    state.text.line_numbers.clone_from(&line_numbers);
    state.text.word_count = words;
    state.text.char_count = chars;
    state.text.reading_time_mins = mins;
    let lines_count = content.lines().count().max(1);
    let line_height = state.font_size * 1.35;
    state.text.total_content_height = (lines_count as f32) * line_height + 40.0;
    state.text.scroll_y = 0.0;
    state.text.scroll_controller.reset();
    state.text.smooth_scroll.stop(0.0);
    state.text.symbols = crate::features::text::extract_symbols(&content, path_ext);
    state.file_type_text = language.to_string();
}
