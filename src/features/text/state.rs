use crate::core::types::KglanceState;
use crate::features::text::anchor::ViewportAnchor;
use crate::features::text::display_map::{DisplayMap, WrapMode};
use crate::features::text::document::CodeDocument;
use crate::features::text::highlight::SyntaxCache;
use crate::features::text::policy::PerformancePolicy;
use crate::ui::theme::AppTheme;
use std::path::Path;

pub fn create_text_state(
    content: String,
    extension: &str,
    font_size: f32,
    wrap: bool,
    theme: AppTheme,
    viewport_width: f32,
) -> crate::core::types::TextState {
    let doc = CodeDocument::new(content);
    let total_lines = doc.total_lines();
    let words = if total_lines <= 10_000 {
        doc.as_str().split_whitespace().count()
    } else {
        doc.as_str().len() / 6
    };
    let chars = doc.as_str().chars().count();
    let mins = (words as f32 / 200.0).ceil() as usize;

    let policy = PerformancePolicy::default();
    let checkpoint_interval = policy.recommended_checkpoint_interval(total_lines);
    let mut syntax_cache = SyntaxCache::with_capacity(extension, checkpoint_interval, total_lines);

    let wrap_mode = if wrap { WrapMode::Word } else { WrapMode::None };
    let viewport_w = viewport_width.max(100.0);
    let display_map = DisplayMap::new(&doc, font_size, wrap_mode, viewport_w);
    let total_content_height = display_map.total_content_height();

    // Cache whole file for files <= 3,000 lines for instant highlighting without scroll lag,
    // or prefetch a generous initial viewport for larger files.
    let prefetch_count = if total_lines <= 3000 {
        total_lines
    } else {
        total_lines.min(150)
    };
    let initial_spans = if prefetch_count > 0 {
        syntax_cache.get_or_tokenize_range(0, prefetch_count - 1, &doc, theme)
    } else {
        Vec::new()
    };

    let symbols = if total_lines <= 100_000 {
        crate::features::text::extract_symbols(doc.as_str(), extension)
    } else {
        Vec::new()
    };

    crate::core::types::TextState {
        document: doc,
        syntax_cache,
        cached_tokens: initial_spans,
        cached_tokens_start_line: 0,
        cached_tokens_version: 0,
        extension: extension.to_string(),
        wrap,
        selection: None,
        anchor: ViewportAnchor::new(0, 0.0),
        display_map,
        indexer: crate::features::text::BackgroundIndexer::default(),
        policy,
        search_visible: false,
        search_query: String::new(),
        search_matches: Vec::new(),
        search_match_index: 0,
        search_info: String::new(),
        goto_line_visible: false,
        goto_line_query: String::new(),
        scroll_y: 0.0,
        viewport_height: 800.0,
        total_content_height,
        word_count: words,
        char_count: chars,
        reading_time_mins: mins,
        symbols,
        outline_visible: false,
        sidebar_width: 250.0,
        sidebar_resizing: false,
        sidebar_drag_start_x: None,
        sidebar_drag_start_width: 250.0,
        scroll_controller: crate::core::scroll::ScrollController::default(),
        smooth_scroll: crate::core::types::SmoothScrollState::default(),
        is_dragging_selection: false,
        drag_start: None,
        drag_last_cursor: iced::Point::ORIGIN,
        auto_scroll_delta: None,
    }
}

pub fn populate_state(
    state: &mut KglanceState,
    content: String,
    _line_numbers: String,
    language: &str,
) {
    let path_ext = if !state.file_name.is_empty() {
        Path::new(&state.file_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or(language)
    } else {
        language
    };

    let old_outline_visible = state.text.outline_visible;
    let old_sidebar_width = state.text.sidebar_width;
    let win_w = if state.current_window_size.width > 0.0 {
        state.current_window_size.width
    } else if state.window_width > 0.0 {
        state.window_width
    } else {
        1024.0
    };
    let sidebar_w = if old_outline_visible && old_sidebar_width > 0.0 {
        old_sidebar_width
    } else {
        0.0
    };
    let available_w = (win_w - sidebar_w).max(200.0);

    let mut text_state = create_text_state(
        content,
        path_ext,
        state.font_size,
        state.word_wrap,
        state.app_theme,
        available_w,
    );
    text_state.outline_visible = old_outline_visible;
    if old_sidebar_width > 0.0 {
        text_state.sidebar_width = old_sidebar_width;
    }
    state.text = text_state;
    state.file_type_text = language.to_string();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_text_state_precaches_full_file_under_threshold() {
        let lines: Vec<String> = (0..150)
            .map(|i| format!("fn func_{i}() {{ let x = {i}; }}\n"))
            .collect();
        let content = lines.concat();
        let state = create_text_state(content, "rs", 14.0, false, AppTheme::Dark, 800.0);

        // Entire file (150 lines) should be pre-cached immediately
        assert_eq!(state.document.total_lines(), 150);
        assert_eq!(state.cached_tokens.len(), 150);
        assert_eq!(state.cached_tokens_start_line, 0);
        assert!(!state.cached_tokens[0].is_empty());
        assert!(!state.cached_tokens[149].is_empty());
    }

    #[test]
    fn test_create_text_state_large_file_prefetches_initial_viewport() {
        let lines: Vec<String> = (0..4000)
            .map(|i| format!("let line_{i} = {i};\n"))
            .collect();
        let content = lines.concat();
        let state = create_text_state(content, "rs", 14.0, false, AppTheme::Dark, 800.0);

        assert_eq!(state.document.total_lines(), 4000);
        assert_eq!(state.cached_tokens.len(), 150);
        assert_eq!(state.cached_tokens_start_line, 0);
    }
}
