/// Text cursor position represented as (line, col).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct TextPosition {
    pub line: usize,
    pub col: usize,
}

impl TextPosition {
    #[inline]
    pub const fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

/// Highlighted selection range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SelectionRange {
    pub start: TextPosition,
    pub end: TextPosition,
}

impl SelectionRange {
    #[inline]
    pub const fn new(start: TextPosition, end: TextPosition) -> Self {
        Self { start, end }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Normalizes order (start is always <= end).
    pub fn normalized(&self) -> (TextPosition, TextPosition) {
        if self.start <= self.end {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }

    /// Checks whether a line falls within the selection range.
    pub fn contains_line(&self, line: usize) -> bool {
        if self.is_empty() {
            return false;
        }
        let (start, end) = self.normalized();
        line >= start.line && line <= end.line
    }

    /// Returns the column range selected in line `line`.
    pub fn line_col_range(&self, line: usize, line_char_count: usize) -> Option<(usize, usize)> {
        if !self.contains_line(line) {
            return None;
        }
        let (start, end) = self.normalized();

        let start_col = if line == start.line { start.col } else { 0 };
        let end_col = if line == end.line {
            end.col.min(line_char_count)
        } else {
            line_char_count
        };

        if start_col <= end_col {
            Some((start_col, end_col))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_normalization() {
        let sel = SelectionRange::new(TextPosition::new(5, 10), TextPosition::new(2, 4));
        let (start, end) = sel.normalized();
        assert_eq!(start, TextPosition::new(2, 4));
        assert_eq!(end, TextPosition::new(5, 10));
    }

    #[test]
    fn test_selection_line_col_range() {
        let sel = SelectionRange::new(TextPosition::new(1, 4), TextPosition::new(3, 8));
        assert_eq!(sel.line_col_range(1, 20), Some((4, 20)));
        assert_eq!(sel.line_col_range(2, 20), Some((0, 20)));
        assert_eq!(sel.line_col_range(3, 20), Some((0, 8)));
        assert_eq!(sel.line_col_range(4, 20), None);
    }
}
