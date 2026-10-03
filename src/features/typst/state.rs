use crate::core::types::KglanceState;
use crate::features::pdf::PdfTocEntry;
use crate::features::pdf::types::PageDimensions;

pub fn populate_state(
    state: &mut KglanceState,
    page_count: usize,
    source: &str,
    error: Option<String>,
    outline: &[PdfTocEntry],
    page_dimensions: &[PageDimensions],
) {
    crate::features::pdf::populate_state(
        state,
        page_count,
        outline.to_vec(),
        page_dimensions.to_vec(),
    );

    let win_w = if state.current_window_size.width > 0.0 {
        state.current_window_size.width
    } else if state.window_width > 0.0 {
        state.window_width
    } else {
        1024.0
    };

    let source_text = crate::features::text::create_text_state(
        source.to_string(),
        "typ",
        state.font_size,
        state.word_wrap,
        state.app_theme,
        win_w,
    );

    state.typst = crate::core::TypstState {
        source_text,
        show_source: error.is_some(),
        error,
    };
    state.file_type_text = "Typst Document".to_string();
}
