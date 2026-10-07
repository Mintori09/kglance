use iced::Rectangle;

#[derive(Debug, Clone, PartialEq)]
pub struct PdfChar {
    pub ch: char,
    pub rect: [f32; 4], // [x0, y0, x1, y1] in PDF points
}

#[derive(Debug, Clone, PartialEq)]
pub struct PdfLine {
    pub text: String,
    pub rect: [f32; 4], // [x0, y0, x1, y1] in PDF points
    pub chars: Vec<PdfChar>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PdfPageText {
    pub page_index: usize,
    pub lines: Vec<PdfLine>,
}

impl PdfPageText {
    pub fn new(page_index: usize, lines: Vec<PdfLine>) -> Self {
        Self { page_index, lines }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn hit_test(&self, pt_x: f32, pt_y: f32) -> Option<(usize, usize)> {
        if self.lines.is_empty() {
            return None;
        }

        // 1. Direct hit test: check if (pt_x, pt_y) is inside any line's bounding box
        let mut direct_hit: Option<usize> = None;
        let mut min_direct_area = f32::MAX;

        for (idx, line) in self.lines.iter().enumerate() {
            let [lx0, ly0, lx1, ly1] = line.rect;
            let left = lx0.min(lx1);
            let right = lx0.max(lx1);
            let top = ly0.min(ly1);
            let bottom = ly0.max(ly1);

            if pt_x >= left && pt_x <= right && pt_y >= top && pt_y <= bottom {
                let area = (right - left) * (bottom - top);
                if area < min_direct_area {
                    min_direct_area = area;
                    direct_hit = Some(idx);
                }
            }
        }

        let best_line_idx = if let Some(idx) = direct_hit {
            idx
        } else {
            // 2. Proximity hit test: find closest line by weighted 2D distance
            let mut best_idx = 0;
            let mut min_dist_sq = f32::MAX;

            for (idx, line) in self.lines.iter().enumerate() {
                let [lx0, ly0, lx1, ly1] = line.rect;
                let left = lx0.min(lx1);
                let right = lx0.max(lx1);
                let top = ly0.min(ly1);
                let bottom = ly0.max(ly1);

                let dx = if pt_x < left {
                    left - pt_x
                } else if pt_x > right {
                    pt_x - right
                } else {
                    0.0
                };

                let dy = if pt_y < top {
                    top - pt_y
                } else if pt_y > bottom {
                    pt_y - bottom
                } else {
                    0.0
                };

                let dist_sq = dx * dx + (dy * 3.0) * (dy * 3.0);
                if dist_sq < min_dist_sq {
                    min_dist_sq = dist_sq;
                    best_idx = idx;
                }
            }
            best_idx
        };

        let line = &self.lines[best_line_idx];
        if line.chars.is_empty() {
            return Some((best_line_idx, 0));
        }

        let mut char_idx = line.chars.len();
        for (c_idx, ch) in line.chars.iter().enumerate() {
            let [cx0, _, cx1, _] = ch.rect;
            let left = cx0.min(cx1);
            let right = cx0.max(cx1);
            let mid_x = (left + right) * 0.5;

            if pt_x < mid_x {
                char_idx = c_idx;
                break;
            }
        }

        Some((best_line_idx, char_idx))
    }

    pub fn is_point_over_text(&self, pt_x: f32, pt_y: f32) -> bool {
        for line in &self.lines {
            let [lx0, ly0, lx1, ly1] = line.rect;
            let left = lx0.min(lx1) - 4.0;
            let right = lx0.max(lx1) + 4.0;
            let top = ly0.min(ly1) - 2.0;
            let bottom = ly0.max(ly1) + 2.0;

            if pt_x >= left && pt_x <= right && pt_y >= top && pt_y <= bottom {
                return true;
            }
        }
        false
    }

    pub fn find_word_range(&self, line_idx: usize, char_idx: usize) -> (usize, usize) {
        let Some(line) = self.lines.get(line_idx) else {
            return (0, 0);
        };
        if line.chars.is_empty() {
            return (0, 0);
        }
        let len = line.chars.len();
        let idx = char_idx.min(len.saturating_sub(1));

        let is_word_char = |c: char| !c.is_whitespace() && !c.is_ascii_punctuation();

        if !is_word_char(line.chars[idx].ch) {
            return (idx, (idx + 1).min(len));
        }

        let mut start = idx;
        while start > 0 && is_word_char(line.chars[start - 1].ch) {
            start -= 1;
        }

        let mut end = idx;
        while end < len && is_word_char(line.chars[end].ch) {
            end += 1;
        }

        (start, end)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct PdfPosition {
    pub page: usize,
    pub line: usize,
    pub char_idx: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PdfSelection {
    pub start: PdfPosition,
    pub end: PdfPosition,
}

impl PdfSelection {
    pub fn new(start: PdfPosition, end: PdfPosition) -> Self {
        Self { start, end }
    }

    pub fn normalized(&self) -> (PdfPosition, PdfPosition) {
        if self.start <= self.end {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }

    #[inline]
    pub fn is_collapsed(&self) -> bool {
        self.start == self.end
    }

    pub fn char_range_on_line(
        &self,
        page: usize,
        line: usize,
        line_char_count: usize,
    ) -> Option<(usize, usize)> {
        let (start, end) = self.normalized();
        if page < start.page || page > end.page {
            return None;
        }
        let is_start_page = page == start.page;
        let is_end_page = page == end.page;

        if is_start_page && is_end_page {
            if line < start.line || line > end.line {
                return None;
            }
            let c_start = if line == start.line {
                start.char_idx.min(line_char_count)
            } else {
                0
            };
            let c_end = if line == end.line {
                end.char_idx.min(line_char_count)
            } else {
                line_char_count
            };
            if c_start < c_end {
                Some((c_start, c_end))
            } else {
                None
            }
        } else if is_start_page {
            if line < start.line {
                return None;
            }
            let c_start = if line == start.line {
                start.char_idx.min(line_char_count)
            } else {
                0
            };
            let c_end = line_char_count;
            if c_start < c_end {
                Some((c_start, c_end))
            } else {
                None
            }
        } else if is_end_page {
            if line > end.line {
                return None;
            }
            let c_start = 0;
            let c_end = if line == end.line {
                end.char_idx.min(line_char_count)
            } else {
                line_char_count
            };
            if c_start < c_end {
                Some((c_start, c_end))
            } else {
                None
            }
        } else if line_char_count > 0 {
            Some((0, line_char_count))
        } else {
            None
        }
    }
}

pub fn extract_selected_content(
    page_texts: &[Option<PdfPageText>],
    selection: &PdfSelection,
) -> (Option<String>, Option<String>) {
    crate::features::pdf::rich_text::extract_selected_content(page_texts, selection)
}

pub fn extract_selected_text(
    page_texts: &[Option<PdfPageText>],
    selection: &PdfSelection,
) -> Option<String> {
    extract_selected_content(page_texts, selection).0
}

pub fn compute_selection_rects(
    page_text: &PdfPageText,
    page_index: usize,
    selection: &PdfSelection,
    scale: f32,
    offset_x: f32,
    offset_y: f32,
) -> Vec<Rectangle> {
    let mut rects = Vec::new();

    for (line_idx, line) in page_text.lines.iter().enumerate() {
        let Some((char_start, char_end)) =
            selection.char_range_on_line(page_index, line_idx, line.chars.len())
        else {
            continue;
        };

        if char_start >= char_end {
            continue;
        }

        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        for ch in &line.chars[char_start..char_end] {
            let [cx0, cy0, cx1, cy1] = ch.rect;
            let left = cx0.min(cx1);
            let right = cx0.max(cx1);
            let top = cy0.min(cy1);
            let bottom = cy0.max(cy1);

            if left < min_x {
                min_x = left;
            }
            if right > max_x {
                max_x = right;
            }
            if top < min_y {
                min_y = top;
            }
            if bottom > max_y {
                max_y = bottom;
            }
        }

        if min_x > max_x {
            let [lx0, _, lx1, _] = line.rect;
            min_x = lx0.min(lx1);
            max_x = lx0.max(lx1);
        }
        if min_y >= max_y - 0.1 {
            let [_, ly0, _, ly1] = line.rect;
            min_y = ly0.min(ly1);
            max_y = ly0.max(ly1);
        }

        let screen_x = offset_x + min_x * scale;
        let screen_y = offset_y + min_y * scale;
        let screen_w = ((max_x - min_x) * scale).max(2.0);
        let screen_h = ((max_y - min_y) * scale).max(2.0);

        rects.push(Rectangle {
            x: screen_x,
            y: screen_y,
            width: screen_w,
            height: screen_h,
        });
    }

    rects
}

pub fn handle_selection_drag_start(pdf_state: &mut crate::core::PdfState, pos: PdfPosition) {
    pdf_state.is_selecting = true;
    pdf_state.selection_drag_start = Some(pos);
    pdf_state.selection = Some(PdfSelection::new(pos, pos));
    pdf_state.selected_text = None;
    pdf_state.selected_html = None;
}

pub fn handle_selection_drag_update(pdf_state: &mut crate::core::PdfState, pos: PdfPosition) {
    if let Some(start) = pdf_state.selection_drag_start {
        let sel = PdfSelection::new(start, pos);
        let (plain, html) = extract_selected_content(&pdf_state.page_texts, &sel);
        pdf_state.selected_text = plain;
        pdf_state.selected_html = html;
        pdf_state.selection = Some(sel);
    }
}

pub fn handle_selection_drag_end(pdf_state: &mut crate::core::PdfState) {
    if !pdf_state.is_selecting {
        return;
    }
    pdf_state.is_selecting = false;
    if let Some(selection) = pdf_state.selection {
        if selection.is_collapsed() {
            pdf_state.selection = None;
            pdf_state.selected_text = None;
            pdf_state.selected_html = None;
        } else {
            let (plain, html) = extract_selected_content(&pdf_state.page_texts, &selection);
            pdf_state.selected_text = plain;
            pdf_state.selected_html = html;
        }
    }
}

pub fn handle_selection_clear(pdf_state: &mut crate::core::PdfState) {
    pdf_state.clear_selection();
}

pub fn handle_select_all(pdf_state: &mut crate::core::PdfState) {
    if pdf_state.page_count == 0 || pdf_state.page_texts.is_empty() {
        return;
    }
    let start = PdfPosition {
        page: 0,
        line: 0,
        char_idx: 0,
    };
    let mut last_pos = start;
    for (p_idx, pt_opt) in pdf_state.page_texts.iter().enumerate().rev() {
        if let Some(pt) = pt_opt
            && !pt.lines.is_empty()
        {
            let last_line_idx = pt.lines.len().saturating_sub(1);
            let last_char_idx = pt.lines[last_line_idx].chars.len();
            last_pos = PdfPosition {
                page: p_idx,
                line: last_line_idx,
                char_idx: last_char_idx,
            };
            break;
        }
    }
    if last_pos > start {
        let sel = PdfSelection::new(start, last_pos);
        let (plain, html) = extract_selected_content(&pdf_state.page_texts, &sel);
        pdf_state.selected_text = plain;
        pdf_state.selected_html = html;
        pdf_state.selection = Some(sel);
    }
}

#[cfg(test)]
#[path = "selection_tests.rs"]
mod tests;
