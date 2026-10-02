use crate::features::text::document::CodeDocument;
use crate::features::text::highlight::{HighlightedSpan, SyntaxCache};
use crate::ui::theme::AppTheme;

/// Priority of the background syntax indexing task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IndexPriority {
    Prefetch = 0,
    Overscan = 1,
    Visible = 2,
}

/// Request to parse syntax for a line range in the document.
#[derive(Debug, Clone)]
pub struct IndexRequest {
    pub start_line: usize,
    pub end_line: usize,
    pub priority: IndexPriority,
    pub version: usize,
}

/// Syntax parsing result returned to the UI.
#[derive(Debug, Clone)]
pub struct IndexResult {
    pub start_line: usize,
    pub spans: Vec<Vec<HighlightedSpan>>,
    pub version: usize,
}

/// Background syntax parsing coordinator with task cancellation during fast scrolling.
#[derive(Debug, Clone, Default)]
pub struct BackgroundIndexer {
    current_version: usize,
}

impl BackgroundIndexer {
    pub fn new() -> Self {
        Self { current_version: 0 }
    }

    /// Increments version when user jumps/scrolls or opens a new file to discard stale results.
    pub fn next_version(&mut self) -> usize {
        self.current_version = self.current_version.wrapping_add(1);
        self.current_version
    }

    /// Returns the current version.
    #[inline]
    pub fn current_version(&self) -> usize {
        self.current_version
    }

    /// Processes tokenization for a line range (runs in `iced::Task` / worker thread).
    pub fn process_range(
        doc: &CodeDocument,
        syntax_cache: &mut SyntaxCache,
        start_line: usize,
        end_line: usize,
        theme: AppTheme,
        version: usize,
    ) -> IndexResult {
        let spans = syntax_cache.get_or_tokenize_range(start_line, end_line, doc, theme);
        IndexResult {
            start_line,
            spans,
            version,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_background_indexer_versioning() {
        let mut indexer = BackgroundIndexer::new();
        let v1 = indexer.next_version();
        let v2 = indexer.next_version();
        assert_eq!(v2, v1 + 1);
    }
}
