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

/// Computes the TextPosition for a point relative to the content bounds.
pub fn hit_test_position(
    doc: &crate::features::text::CodeDocument,
    display_map: Option<&crate::features::text::DisplayMap>,
    wrap: bool,
    font_size: f32,
    rel_pos: iced::Point,
) -> TextPosition {
    let line_h = display_map
        .map(|d| d.line_height)
        .unwrap_or(font_size * 1.35);
    let gutter_w = if let Some(d) = display_map {
        d.gutter_width
    } else {
        let digits = doc.max_digits();
        let char_w = font_size * 0.60;
        ((digits as f32) * char_w + 28.0).max(44.0)
    };
    let char_w = display_map
        .map(|d| d.char_width)
        .unwrap_or(font_size * 0.60);

    let vrow = (rel_pos.y / line_h).floor() as usize;
    let (line_idx, sub_row_idx) = if let Some(d) = display_map {
        d.visual_row_to_line(vrow)
    } else {
        (vrow.min(doc.total_lines().saturating_sub(1)), 0)
    };

    let x_in_code = (rel_pos.x - gutter_w).max(0.0);
    let line_text = doc.get_line(line_idx);
    let max_cols = display_map
        .map(|d| d.max_cols_per_row())
        .unwrap_or(usize::MAX);

    let sub_slices = if wrap {
        crate::features::text::display_map::wrap_line_to_slices(line_text, max_cols)
    } else {
        vec![crate::features::text::display_map::VisualSubSlice {
            start_byte: 0,
            end_byte: line_text.len(),
            start_col: 0,
            end_col: line_text.chars().count(),
        }]
    };

    let sub_slice = sub_slices.get(sub_row_idx).copied().unwrap_or(
        crate::features::text::display_map::VisualSubSlice {
            start_byte: 0,
            end_byte: line_text.len(),
            start_col: 0,
            end_col: line_text.chars().count(),
        },
    );

    let col_in_sub = (x_in_code / char_w).round() as usize;
    let col_idx = (sub_slice.start_col + col_in_sub).min(sub_slice.end_col);

    TextPosition::new(line_idx, col_idx)
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

    #[test]
    fn test_hit_test_position() {
        let doc = crate::features::text::CodeDocument::new("hello world\nsecond line\nthird line");
        let pos = hit_test_position(&doc, None, false, 14.0, iced::Point::new(60.0, 5.0));
        assert_eq!(pos.line, 0);

        let pos2 = hit_test_position(&doc, None, false, 14.0, iced::Point::new(60.0, 25.0));
        assert_eq!(pos2.line, 1);
    }
}
