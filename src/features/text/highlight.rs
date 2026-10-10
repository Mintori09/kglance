use crate::features::text::document::CodeDocument;
use crate::features::text::syntax::{
    find_syntax_for_extension, global_syntax_set, global_theme_set,
};
use crate::ui::theme::AppTheme;
use crate::ui::theme::color::primitive::syntect_to_iced_color;
use iced::{Color, Font};
use lru::LruCache;
use std::num::NonZeroUsize;
use syntect::highlighting::{HighlightIterator, HighlightState, Highlighter};
use syntect::parsing::{ParseState, ScopeStack, SyntaxReference};

/// Structure representing a syntax-highlighted span.
#[derive(Debug, Clone, PartialEq)]
pub struct HighlightedSpan {
    /// Starting byte index in the line.
    pub start_byte: usize,
    /// Ending byte index in the line.
    pub end_byte: usize,
    /// Color from theme.
    pub color: Color,
    /// Font style.
    pub font: Font,
}

/// Checkpoint storing Syntect parse state for fast arbitrary line seeking.
#[derive(Clone)]
pub struct SyntaxCheckpoint {
    pub line_idx: usize,
    pub parse_state: ParseState,
    pub highlight_state: HighlightState,
}

/// Cache managing syntax highlighting tokens and checkpoints.
pub struct SyntaxCache {
    extension: String,
    checkpoint_interval: usize,
    checkpoints: Vec<SyntaxCheckpoint>,
    token_cache: LruCache<usize, Vec<HighlightedSpan>>,
    current_theme: Option<AppTheme>,
}

impl std::fmt::Debug for SyntaxCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyntaxCache")
            .field("extension", &self.extension)
            .field("checkpoint_interval", &self.checkpoint_interval)
            .field("checkpoints_count", &self.checkpoints.len())
            .field("token_cache_len", &self.token_cache.len())
            .field("current_theme", &self.current_theme)
            .finish()
    }
}

impl Clone for SyntaxCache {
    fn clone(&self) -> Self {
        let capacity = NonZeroUsize::new(1024).unwrap_or(NonZeroUsize::MIN);
        let mut new_cache = LruCache::new(capacity);
        for (k, v) in self.token_cache.iter() {
            new_cache.put(*k, v.clone());
        }
        Self {
            extension: self.extension.clone(),
            checkpoint_interval: self.checkpoint_interval,
            checkpoints: self.checkpoints.clone(),
            token_cache: new_cache,
            current_theme: self.current_theme,
        }
    }
}

impl SyntaxCache {
    /// Initializes a new SyntaxCache with default capacity.
    pub fn new(extension: impl Into<String>, checkpoint_interval: usize) -> Self {
        Self::with_capacity(extension, checkpoint_interval, 4096)
    }

    /// Initializes a new SyntaxCache with custom capacity.
    pub fn with_capacity(
        extension: impl Into<String>,
        checkpoint_interval: usize,
        capacity: usize,
    ) -> Self {
        let cap = NonZeroUsize::new(capacity.max(4096)).expect("4096 is non-zero");
        Self {
            extension: extension.into(),
            checkpoint_interval: checkpoint_interval.max(16),
            checkpoints: Vec::new(),
            token_cache: LruCache::new(cap),
            current_theme: None,
        }
    }

    /// Returns the currently active theme, if any.
    pub fn current_theme(&self) -> Option<AppTheme> {
        self.current_theme
    }

    /// Sets active theme, clearing caches if changed.
    pub fn set_theme(&mut self, theme: AppTheme) {
        if self.current_theme != Some(theme) {
            self.current_theme = Some(theme);
            self.clear();
        }
    }

    /// Changes file extension and clears cache.
    pub fn set_extension(&mut self, ext: impl Into<String>) {
        let new_ext = ext.into();
        if self.extension != new_ext {
            self.extension = new_ext;
            self.clear();
        }
    }

    /// Clears all cached tokens and checkpoints.
    pub fn clear(&mut self) {
        self.checkpoints.clear();
        self.token_cache.clear();
    }

    /// Retrieves or tokenizes highlight spans for a contiguous line range in a single pass O(N).
    pub fn get_or_tokenize_range(
        &mut self,
        start_line: usize,
        end_line: usize,
        doc: &CodeDocument,
        theme: AppTheme,
    ) -> Vec<Vec<HighlightedSpan>> {
        let total_lines = doc.total_lines();
        if total_lines == 0 || start_line > end_line {
            return Vec::new();
        }

        if self.current_theme != Some(theme) {
            self.current_theme = Some(theme);
            self.clear();
        }

        let end_line = end_line.min(total_lines.saturating_sub(1));
        let count = end_line.saturating_sub(start_line) + 1;

        // If all lines in range are already cached, return immediately O(1)
        let all_cached = (start_line..=end_line).all(|l| self.token_cache.contains(&l));
        if all_cached {
            let mut result = Vec::with_capacity(count);
            for l in start_line..=end_line {
                if let Some(spans) = self.token_cache.get(&l) {
                    result.push(spans.clone());
                } else {
                    result.push(Vec::new());
                }
            }
            return result;
        }

        let syntax = find_syntax_for_extension(&self.extension);
        let ts = global_theme_set();
        let syntect_theme = ts
            .themes
            .get(theme.syntect_theme())
            .or_else(|| ts.themes.get("base16-ocean.dark"))
            .unwrap_or_else(|| ts.themes.values().next().unwrap());

        let highlighter = Highlighter::new(syntect_theme);
        let ss = global_syntax_set();

        // Find closest checkpoint <= start_line
        let (checkpoint_line, mut current_state, mut highlight_state) =
            self.find_closest_checkpoint(start_line, syntax, &highlighter);

        // Iterate sequentially from checkpoint to end_line
        for idx in checkpoint_line..=end_line {
            let raw_line = doc.get_line_raw(idx);

            // Save checkpoint if interval is reached
            if idx > 0 && idx % self.checkpoint_interval == 0 {
                let has_checkpoint = self.checkpoints.iter().any(|cp| cp.line_idx == idx);
                if !has_checkpoint {
                    self.checkpoints.push(SyntaxCheckpoint {
                        line_idx: idx,
                        parse_state: current_state.clone(),
                        highlight_state: highlight_state.clone(),
                    });
                }
            }

            let ops = current_state.parse_line(raw_line, ss).unwrap_or_default();

            if idx >= start_line {
                // Collect spans only for visible lines in requested range
                let ranges =
                    HighlightIterator::new(&mut highlight_state, &ops[..], raw_line, &highlighter);

                let mut spans = Vec::new();
                let mut current_byte = 0;

                for (style, text) in ranges {
                    let text_bytes = text.len();
                    let trimmed_text = text.trim_end_matches(['\r', '\n']);
                    if !trimmed_text.is_empty() {
                        let color = syntect_to_iced_color(style.foreground);
                        spans.push(HighlightedSpan {
                            start_byte: current_byte,
                            end_byte: current_byte + trimmed_text.len(),
                            color,
                            font: Font::MONOSPACE,
                        });
                    }
                    current_byte += text_bytes;
                }

                self.token_cache.put(idx, spans);
            } else {
                // For lines before start_line, run iterator to update highlight_state (Zero-Allocation)
                for _ in
                    HighlightIterator::new(&mut highlight_state, &ops[..], raw_line, &highlighter)
                {
                }
            }
        }

        let mut result = Vec::with_capacity(count);
        for l in start_line..=end_line {
            if let Some(spans) = self.token_cache.get(&l) {
                result.push(spans.clone());
            } else {
                result.push(Vec::new());
            }
        }
        result
    }

    /// Retrieves or tokenizes highlight spans for a specific line.
    pub fn get_or_tokenize_line(
        &mut self,
        line_idx: usize,
        doc: &CodeDocument,
        theme: AppTheme,
    ) -> &[HighlightedSpan] {
        if self.current_theme != Some(theme) || !self.token_cache.contains(&line_idx) {
            let _ = self.get_or_tokenize_range(line_idx, line_idx, doc, theme);
        }

        self.token_cache
            .get(&line_idx)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Finds closest checkpoint at or before `target_line`.
    fn find_closest_checkpoint(
        &self,
        target_line: usize,
        syntax: &SyntaxReference,
        highlighter: &Highlighter,
    ) -> (usize, ParseState, HighlightState) {
        if self.checkpoints.is_empty() || target_line < self.checkpoint_interval {
            return (
                0,
                ParseState::new(syntax),
                HighlightState::new(highlighter, ScopeStack::new()),
            );
        }

        let mut best_cp: Option<&SyntaxCheckpoint> = None;
        for cp in &self.checkpoints {
            if cp.line_idx <= target_line {
                if let Some(current) = best_cp {
                    if cp.line_idx > current.line_idx {
                        best_cp = Some(cp);
                    }
                } else {
                    best_cp = Some(cp);
                }
            }
        }

        if let Some(cp) = best_cp {
            (
                cp.line_idx,
                cp.parse_state.clone(),
                cp.highlight_state.clone(),
            )
        } else {
            (
                0,
                ParseState::new(syntax),
                HighlightState::new(highlighter, ScopeStack::new()),
            )
        }
    }
}

impl Default for SyntaxCache {
    fn default() -> Self {
        Self::new("rs", 128)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_cache_tokenization() {
        let code = "fn main() {\n    let x = 42;\n}";
        let doc = CodeDocument::new(code);
        let mut cache = SyntaxCache::new("rs", 64);

        let spans = cache.get_or_tokenize_line(0, &doc, AppTheme::Dark).to_vec();
        assert!(!spans.is_empty());
        assert_eq!(spans[0].start_byte, 0);

        let spans_cached = cache.get_or_tokenize_line(0, &doc, AppTheme::Dark);
        assert_eq!(spans.as_slice(), spans_cached);
    }

    #[test]
    fn test_syntax_checkpoints_generation() {
        let lines: Vec<String> = (0..300).map(|i| format!("let var_{i} = {i};\n")).collect();
        let full_code = lines.concat();
        let doc = CodeDocument::new(full_code);
        let mut cache = SyntaxCache::new("rs", 64);

        // Access line 200 => must generate checkpoints at 64, 128, 192
        let spans = cache.get_or_tokenize_line(200, &doc, AppTheme::Dark);
        assert!(!spans.is_empty());
        assert!(!cache.checkpoints.is_empty());
    }

    #[test]
    fn test_json_highlight() {
        let code = "{\n  \"key\": \"value\",\n  \"number\": 123,\n  \"bool\": true\n}";
        let doc = CodeDocument::new(code);
        let mut cache = SyntaxCache::new("json", 64);
        let spans_line_1 = cache.get_or_tokenize_line(1, &doc, AppTheme::Dark);
        let line_1 = doc.get_line(1);

        let key_span = spans_line_1
            .iter()
            .find(|s| &line_1[s.start_byte..s.end_byte] == "key")
            .expect("key span found");
        let val_span = spans_line_1
            .iter()
            .find(|s| &line_1[s.start_byte..s.end_byte] == "value")
            .expect("value span found");

        assert_ne!(
            key_span.color, val_span.color,
            "JSON key and string value should have distinct highlight colors"
        );
    }

    #[test]
    fn test_typescript_highlight() {
        let code = "interface User<T> {\n    id: string;\n    data: T;\n}\nconst user: User<number> = { id: \"123\", data: 42 };";
        let doc = CodeDocument::new(code);
        let mut cache = SyntaxCache::new("ts", 64);
        let spans_line_0 = cache.get_or_tokenize_line(0, &doc, AppTheme::Dark);
        assert!(!spans_line_0.is_empty());

        let spans_line_4 = cache.get_or_tokenize_line(4, &doc, AppTheme::Dark);
        assert!(!spans_line_4.is_empty());

        let line_4 = doc.get_line(4);
        let const_span = spans_line_4
            .iter()
            .find(|s| &line_4[s.start_byte..s.end_byte] == "const")
            .expect("const keyword span found");
        let string_span = spans_line_4
            .iter()
            .find(|s| &line_4[s.start_byte..s.end_byte] == "123")
            .expect("string literal span found");
        let num_span = spans_line_4
            .iter()
            .find(|s| &line_4[s.start_byte..s.end_byte] == "42")
            .expect("numeric literal span found");

        assert_ne!(
            string_span.color, num_span.color,
            "TypeScript string and number literals should have distinct colors"
        );
        assert_ne!(
            const_span.color, string_span.color,
            "TypeScript keyword and string literals should have distinct colors"
        );
    }

    #[test]
    fn test_syntax_cache_theme_invalidation() {
        let code = "const message = \"hello\";";
        let doc = CodeDocument::new(code);
        let mut cache = SyntaxCache::new("ts", 64);

        let spans_dracula = cache
            .get_or_tokenize_line(0, &doc, AppTheme::Dracula)
            .to_vec();
        assert!(!spans_dracula.is_empty());

        let spans_latte = cache
            .get_or_tokenize_line(0, &doc, AppTheme::CatppuccinLatte)
            .to_vec();
        assert!(!spans_latte.is_empty());

        let spans_light = cache
            .get_or_tokenize_line(0, &doc, AppTheme::Light)
            .to_vec();
        assert!(!spans_light.is_empty());

        // Dracula string is yellow (241, 250, 140), Latte string is green (64, 160, 43)
        let str_dracula = spans_dracula
            .iter()
            .find(|s| &code[s.start_byte..s.end_byte] == "hello")
            .unwrap();
        let str_latte = spans_latte
            .iter()
            .find(|s| &code[s.start_byte..s.end_byte] == "hello")
            .unwrap();
        let str_light = spans_light
            .iter()
            .find(|s| &code[s.start_byte..s.end_byte] == "hello")
            .unwrap();

        assert_ne!(
            str_dracula.color, str_latte.color,
            "Theme switch must invalidate cache and change token colors"
        );
        assert_ne!(
            str_latte.color, str_light.color,
            "Latte and Light themes should have distinct colors"
        );
    }
}
