use crate::features::text::document::CodeDocument;

/// Line wrapping display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WrapMode {
    #[default]
    None,
    Word,
}

/// Represents a visual sub-row within a wrapped logical line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisualSubSlice {
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_col: usize,
    pub end_col: usize,
}

pub const CONTENT_PADDING_LEFT: f32 = 16.0;
pub const CONTENT_PADDING_RIGHT: f32 = 24.0;

/// Maps coordinates from Logical Line to Visual Row and manages 2D bounding boxes.
#[derive(Debug, Clone)]
pub struct DisplayMap {
    pub wrap_mode: WrapMode,
    pub font_size: f32,
    pub line_height: f32,
    pub char_width: f32,
    pub viewport_width: f32,
    pub gutter_width: f32,
    pub total_lines: usize,
    /// Prefix sum of visual rows for each logical line: visual_row_offsets[i] = starting visual row of line i.
    pub visual_row_offsets: Vec<usize>,
    pub total_visual_rows: usize,
}

impl DisplayMap {
    /// Initializes a new DisplayMap.
    pub fn new(
        doc: &CodeDocument,
        font_size: f32,
        wrap_mode: WrapMode,
        viewport_width: f32,
    ) -> Self {
        let line_height = font_size * 1.50;
        let char_width = font_size * 0.60;
        let total_lines = doc.total_lines();
        let gutter_width = Self::calculate_gutter_width(doc.max_digits(), font_size);
        let viewport_w = viewport_width.max(100.0);

        let (visual_row_offsets, total_visual_rows) =
            Self::build_visual_offsets(doc, wrap_mode, viewport_w, gutter_width, char_width);

        Self {
            wrap_mode,
            font_size,
            line_height,
            char_width,
            viewport_width: viewport_w,
            gutter_width,
            total_lines,
            visual_row_offsets,
            total_visual_rows,
        }
    }

    /// Calculates gutter width based on max digit count and font size.
    pub fn calculate_gutter_width(max_digits: usize, font_size: f32) -> f32 {
        let char_width = font_size * 0.60;
        ((max_digits.max(1) as f32) * char_width + 28.0).max(44.0)
    }

    /// Builds visual row offsets for all lines in the document.
    pub fn build_visual_offsets(
        doc: &CodeDocument,
        wrap_mode: WrapMode,
        viewport_width: f32,
        gutter_width: f32,
        char_width: f32,
    ) -> (Vec<usize>, usize) {
        let total_lines = doc.total_lines();

        if total_lines == 0 {
            return (vec![0], 0);
        }

        if wrap_mode == WrapMode::None {
            let mut offsets = Vec::with_capacity(total_lines);
            for i in 0..total_lines {
                offsets.push(i);
            }
            return (offsets, total_lines);
        }

        let max_cols =
            ((viewport_width - gutter_width - CONTENT_PADDING_LEFT - CONTENT_PADDING_RIGHT)
                .max(100.0)
                / char_width.max(1.0))
            .floor()
            .max(10.0) as usize;

        let mut offsets = Vec::with_capacity(total_lines);
        let mut running_rows = 0;

        for i in 0..total_lines {
            offsets.push(running_rows);
            let line = doc.get_line(i);
            let rows = count_visual_rows(line, max_cols);
            running_rows += rows;
        }

        (offsets, running_rows)
    }

    /// Updates viewport size, font metrics, and rebuilds offsets if needed.
    pub fn update_geometry(
        &mut self,
        doc: &CodeDocument,
        viewport_width: f32,
        font_size: f32,
        wrap_mode: WrapMode,
    ) {
        let viewport_w = viewport_width.max(100.0);
        let changed = (self.viewport_width - viewport_w).abs() > 1.0
            || (self.font_size - font_size).abs() > 0.1
            || self.wrap_mode != wrap_mode
            || self.total_lines != doc.total_lines();

        self.viewport_width = viewport_w;
        self.font_size = font_size;
        self.line_height = font_size * 1.50;
        self.char_width = font_size * 0.60;
        self.gutter_width = Self::calculate_gutter_width(doc.max_digits(), font_size);
        self.total_lines = doc.total_lines();
        self.wrap_mode = wrap_mode;

        if changed {
            let (offsets, total_rows) = Self::build_visual_offsets(
                doc,
                wrap_mode,
                self.viewport_width,
                self.gutter_width,
                self.char_width,
            );
            self.visual_row_offsets = offsets;
            self.total_visual_rows = total_rows;
        }
    }

    /// Max characters per visual row based on current viewport.
    #[inline]
    pub fn max_cols_per_row(&self) -> usize {
        ((self.viewport_width - self.gutter_width - CONTENT_PADDING_LEFT - CONTENT_PADDING_RIGHT)
            .max(100.0)
            / self.char_width.max(1.0))
        .floor()
        .max(10.0) as usize
    }

    /// Converts a visual row index (0..total_visual_rows) to `(logical_line_idx, sub_row_idx)`.
    pub fn visual_row_to_line(&self, visual_row: usize) -> (usize, usize) {
        if self.visual_row_offsets.is_empty() {
            return (0, 0);
        }
        let vrow = visual_row.min(self.total_visual_rows.saturating_sub(1));
        let idx = self
            .visual_row_offsets
            .partition_point(|&offset| offset <= vrow);
        let line_idx = idx
            .saturating_sub(1)
            .min(self.total_lines.saturating_sub(1));
        let sub_row = vrow.saturating_sub(self.visual_row_offsets[line_idx]);
        (line_idx, sub_row)
    }

    /// Converts logical line index to its starting visual row.
    #[inline]
    pub fn line_to_visual_row(&self, line_idx: usize) -> usize {
        if line_idx < self.visual_row_offsets.len() {
            self.visual_row_offsets[line_idx]
        } else {
            self.total_visual_rows
        }
    }

    /// Total content height for scrollbar calculations.
    #[inline]
    pub fn total_content_height(&self) -> f32 {
        (self.total_visual_rows as f32 * self.line_height) + 40.0
    }

    /// Calculates starting Y coordinate of a logical line.
    #[inline]
    pub fn get_line_y(&self, line_idx: usize) -> f32 {
        self.line_to_visual_row(line_idx) as f32 * self.line_height
    }

    /// Computes visible visual row range (start_vrow, end_vrow).
    pub fn compute_visible_range(
        &self,
        scroll_y: f32,
        viewport_height: f32,
        overscan_rows: usize,
    ) -> (usize, usize) {
        if self.total_visual_rows == 0 || self.line_height <= 0.0 {
            return (0, 0);
        }
        let clamped_scroll_y = scroll_y.max(0.0);
        let first_visible = (clamped_scroll_y / self.line_height).floor() as usize;
        let start_row = first_visible.saturating_sub(overscan_rows);
        let visible_count = (viewport_height / self.line_height).ceil() as usize;
        let end_row =
            (first_visible + visible_count + overscan_rows + 1).min(self.total_visual_rows);
        (start_row, end_row.max(start_row + 1))
    }

    /// Converts relative pixel Y coordinate to (logical_line_index, sub_row_index).
    #[inline]
    pub fn y_to_line_and_sub_row(&self, y: f32) -> (usize, usize) {
        if self.line_height <= 0.0 || self.total_visual_rows == 0 {
            return (0, 0);
        }
        let vrow = (y.max(0.0) / self.line_height).floor() as usize;
        self.visual_row_to_line(vrow)
    }

    /// Converts relative pixel Y coordinate to logical line index.
    #[inline]
    pub fn y_to_line_index(&self, y: f32) -> usize {
        self.y_to_line_and_sub_row(y).0
    }
}

impl Default for DisplayMap {
    fn default() -> Self {
        Self {
            wrap_mode: WrapMode::None,
            font_size: 14.0,
            line_height: 14.0 * 1.50,
            char_width: 14.0 * 0.60,
            viewport_width: 800.0,
            gutter_width: 44.0,
            total_lines: 1,
            visual_row_offsets: vec![0],
            total_visual_rows: 1,
        }
    }
}

/// Counts how many visual rows a line occupies given max characters per row.
pub fn count_visual_rows(line: &str, max_cols: usize) -> usize {
    let char_count = line.chars().count();
    if char_count <= max_cols || max_cols == 0 {
        1
    } else {
        wrap_line_to_slices(line, max_cols).len().max(1)
    }
}

/// Breaks a single line of text into visual sub-slices adhering to word boundaries.
pub fn wrap_line_to_slices(line: &str, max_cols: usize) -> Vec<VisualSubSlice> {
    let char_count = line.chars().count();
    if line.is_empty() || max_cols == 0 || char_count <= max_cols {
        return vec![VisualSubSlice {
            start_byte: 0,
            end_byte: line.len(),
            start_col: 0,
            end_col: char_count,
        }];
    }

    let mut slices = Vec::new();
    let mut cur_slice_start_byte = 0;
    let mut cur_slice_start_col = 0;
    let mut cur_slice_len = 0;
    let mut cur_byte = 0;
    let mut cur_col = 0;

    let mut chars = line.char_indices().peekable();
    while let Some(&(start_b, ch)) = chars.peek() {
        let is_space = ch.is_whitespace();
        let mut token_char_count = 0;
        let mut token_end_byte = start_b;

        while let Some(&(b, c)) = chars.peek() {
            if c.is_whitespace() == is_space {
                token_char_count += 1;
                token_end_byte = b + c.len_utf8();
                chars.next();
            } else {
                break;
            }
        }

        let token_byte_len = token_end_byte - start_b;

        if is_space && !slices.is_empty() && cur_slice_len == 0 {
            cur_byte += token_byte_len;
            cur_col += token_char_count;
            cur_slice_start_byte = cur_byte;
            cur_slice_start_col = cur_col;
            continue;
        }

        if cur_slice_len + token_char_count <= max_cols {
            cur_slice_len += token_char_count;
            cur_byte += token_byte_len;
            cur_col += token_char_count;
        } else {
            if cur_slice_len > 0 {
                slices.push(VisualSubSlice {
                    start_byte: cur_slice_start_byte,
                    end_byte: cur_byte,
                    start_col: cur_slice_start_col,
                    end_col: cur_col,
                });
                cur_slice_start_byte = cur_byte;
                cur_slice_start_col = cur_col;
                cur_slice_len = 0;
            }

            if token_char_count > max_cols {
                let token_str = &line[start_b..token_end_byte];
                let mut token_chars = token_str.char_indices().peekable();
                let mut sub_start_b = 0;
                let mut sub_start_c = 0;
                let mut sub_c = 0;

                while let Some(&(b, c)) = token_chars.peek() {
                    sub_c += 1;
                    let next_b = b + c.len_utf8();
                    token_chars.next();

                    if sub_c - sub_start_c >= max_cols || token_chars.peek().is_none() {
                        let chunk_b = next_b - sub_start_b;
                        let chunk_c = sub_c - sub_start_c;

                        if token_chars.peek().is_none() && chunk_c < max_cols {
                            cur_slice_start_byte = cur_byte;
                            cur_slice_start_col = cur_col;
                            cur_byte += chunk_b;
                            cur_col += chunk_c;
                            cur_slice_len = chunk_c;
                        } else {
                            slices.push(VisualSubSlice {
                                start_byte: cur_byte,
                                end_byte: cur_byte + chunk_b,
                                start_col: cur_col,
                                end_col: cur_col + chunk_c,
                            });
                            cur_byte += chunk_b;
                            cur_col += chunk_c;
                            cur_slice_start_byte = cur_byte;
                            cur_slice_start_col = cur_col;
                            cur_slice_len = 0;
                        }
                        sub_start_b = next_b;
                        sub_start_c = sub_c;
                    }
                }
            } else {
                cur_slice_len = token_char_count;
                cur_byte += token_byte_len;
                cur_col += token_char_count;
            }
        }
    }

    if cur_byte > cur_slice_start_byte || slices.is_empty() {
        slices.push(VisualSubSlice {
            start_byte: cur_slice_start_byte,
            end_byte: cur_byte,
            start_col: cur_slice_start_col,
            end_col: cur_col,
        });
    }

    slices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_map_visible_range() {
        let doc = CodeDocument::new("1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n");
        let map = DisplayMap::new(&doc, 20.0, WrapMode::None, 800.0);

        let (start, end) = map.compute_visible_range(0.0, 60.0, 2);
        assert_eq!(start, 0);
        assert!(end >= 5);
    }

    #[test]
    fn test_y_to_line_index() {
        let doc = CodeDocument::new("a\nb\nc\nd\ne\n");
        let map = DisplayMap::new(&doc, 20.0, WrapMode::None, 800.0);
        assert_eq!(map.y_to_line_index(0.0), 0);
        assert_eq!(map.y_to_line_index(35.0), 1);
        assert_eq!(map.y_to_line_index(1000.0), 4);
    }

    #[test]
    fn test_wrapped_slices_and_offsets() {
        let text = "Hello world this is a test of word wrapping for very long lines in documents.\nSecond line after wrapped line.";
        let doc = CodeDocument::new(text);
        // Set small width so line 0 wraps into multiple visual rows
        let map = DisplayMap::new(&doc, 14.0, WrapMode::Word, 250.0);

        assert!(map.total_visual_rows > 2);
        let (line_0, sub_0) = map.visual_row_to_line(0);
        assert_eq!(line_0, 0);
        assert_eq!(sub_0, 0);

        let (line_0_sub1, sub_1) = map.visual_row_to_line(1);
        assert_eq!(line_0_sub1, 0);
        assert_eq!(sub_1, 1);

        // Line 1 must start at or after line 0's total visual rows
        let line_1_start = map.line_to_visual_row(1);
        assert!(line_1_start >= 2);
        let (line_1, sub_line_1) = map.visual_row_to_line(line_1_start);
        assert_eq!(line_1, 1);
        assert_eq!(sub_line_1, 0);
    }

    #[test]
    fn test_wrap_line_to_slices_strips_leading_space() {
        let line =
            "Hệ thống quản lý này đáp ứng được các tính năng cơ bản của một hệ thống quản lý";
        let slices = wrap_line_to_slices(line, 30);
        assert!(slices.len() > 1);
        // Continuation slice should not start with space
        let second_slice_str = &line[slices[1].start_byte..slices[1].end_byte];
        assert!(!second_slice_str.starts_with(' '));
    }
}
