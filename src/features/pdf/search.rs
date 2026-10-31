use crate::features::pdf::selection::PdfPageText;
use iced::Rectangle;

pub const SEARCH_INPUT_ID: &str = "pdf_search_input";

/// Bounding rectangle for a match in PDF points [x0, y0, x1, y1]
#[derive(Debug, Clone, PartialEq)]
pub struct PdfSearchMatch {
    pub page_index: usize,
    pub line_index: usize,
    pub char_start: usize,
    pub char_end: usize,
    pub rect: [f32; 4],
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PdfSearchState {
    pub is_visible: bool,
    pub query: String,
    pub info: String,
    pub matches: Vec<PdfSearchMatch>,
    pub active_match_index: usize,
}

impl PdfSearchState {
    pub fn open(&mut self) {
        self.is_visible = true;
    }

    pub fn close(&mut self) {
        self.is_visible = false;
        self.query.clear();
        self.info.clear();
        self.matches.clear();
        self.active_match_index = 0;
    }

    pub fn total_matches(&self) -> usize {
        self.matches.len()
    }

    pub fn current_match(&self) -> Option<&PdfSearchMatch> {
        if self.matches.is_empty() {
            None
        } else {
            self.matches.get(self.active_match_index)
        }
    }

    pub fn next_match(&mut self) -> Option<&PdfSearchMatch> {
        if self.matches.is_empty() {
            return None;
        }
        self.active_match_index = (self.active_match_index + 1) % self.matches.len();
        self.update_info();
        self.matches.get(self.active_match_index)
    }

    pub fn prev_match(&mut self) -> Option<&PdfSearchMatch> {
        if self.matches.is_empty() {
            return None;
        }
        if self.active_match_index == 0 {
            self.active_match_index = self.matches.len() - 1;
        } else {
            self.active_match_index -= 1;
        }
        self.update_info();
        self.matches.get(self.active_match_index)
    }

    pub fn update_info(&mut self) {
        self.info = if self.query.trim().is_empty() {
            String::new()
        } else if self.matches.is_empty() {
            "No matches".to_string()
        } else {
            format!("{}/{}", self.active_match_index + 1, self.matches.len())
        };
    }
}

/// Matches `query` (case-insensitive) against all lines in `page_text`.
pub fn find_matches_in_page(page_text: &PdfPageText, query: &str) -> Vec<PdfSearchMatch> {
    let query_trimmed = query.trim();
    if query_trimmed.is_empty() {
        return Vec::new();
    }

    let query_lower = query_trimmed.to_lowercase();
    let mut results = Vec::new();

    for (line_idx, line) in page_text.lines.iter().enumerate() {
        if line.text.is_empty() || line.chars.is_empty() {
            continue;
        }

        let line_text_lower = line.text.to_lowercase();
        let mut start_byte = 0;

        while let Some(found_byte) = line_text_lower[start_byte..].find(&query_lower) {
            let actual_match_byte_start = start_byte + found_byte;
            let actual_match_byte_end = actual_match_byte_start + query_lower.len();

            // Map byte offsets back to character indices in line.text
            let mut char_start_idx = 0;
            let mut char_end_idx = 0;

            for (c_idx, (b_idx, ch)) in line.text.char_indices().enumerate() {
                if b_idx <= actual_match_byte_start {
                    char_start_idx = c_idx;
                }
                let current_byte = b_idx + ch.len_utf8();
                if current_byte <= actual_match_byte_end {
                    char_end_idx = c_idx + 1;
                }
            }

            if char_start_idx < char_end_idx && char_start_idx < line.chars.len() {
                let bounded_end = char_end_idx.min(line.chars.len());
                let slice = &line.chars[char_start_idx..bounded_end];

                if !slice.is_empty() {
                    let mut min_x = f32::MAX;
                    let mut max_x = f32::MIN;
                    let mut min_y = f32::MAX;
                    let mut max_y = f32::MIN;

                    for ch in slice {
                        let [cx0, cy0, cx1, cy1] = ch.rect;
                        min_x = min_x.min(cx0.min(cx1));
                        max_x = max_x.max(cx0.max(cx1));
                        min_y = min_y.min(cy0.min(cy1));
                        max_y = max_y.max(cy0.max(cy1));
                    }

                    results.push(PdfSearchMatch {
                        page_index: page_text.page_index,
                        line_index: line_idx,
                        char_start: char_start_idx,
                        char_end: bounded_end,
                        rect: [min_x, min_y, max_x, max_y],
                    });
                }
            }

            // Advance start_byte forward
            start_byte =
                actual_match_byte_start + query_lower.chars().next().map_or(1, |c| c.len_utf8());
            if start_byte >= line_text_lower.len() {
                break;
            }
        }
    }

    results
}

/// Converts PDF point rect into screen Rectangle with given scale and offsets
pub fn match_to_screen_rect(
    match_rect: &[f32; 4],
    scale: f32,
    offset_x: f32,
    offset_y: f32,
) -> Rectangle {
    let [x0, y0, x1, y1] = *match_rect;
    let min_x = x0.min(x1);
    let max_x = x0.max(x1);
    let min_y = y0.min(y1);
    let max_y = y0.max(y1);

    let screen_x = offset_x + min_x * scale;
    let screen_y = offset_y + min_y * scale;
    let screen_w = ((max_x - min_x) * scale).max(2.0);
    let screen_h = ((max_y - min_y) * scale).max(2.0);

    Rectangle {
        x: screen_x,
        y: screen_y,
        width: screen_w,
        height: screen_h,
    }
}

/// Searches the PDF text lazily around visible/current viewport.
/// If text for nearby pages is not yet extracted, extracts it on-demand from the document path.
pub fn update_search_results(
    pdf_state: &mut crate::core::PdfState,
    doc_path: Option<&std::path::Path>,
    query: &str,
) {
    let query_trimmed = query.trim();
    if query_trimmed.is_empty() || pdf_state.page_count == 0 {
        pdf_state.search.matches.clear();
        pdf_state.search.active_match_index = 0;
        return;
    }

    let current_page = pdf_state
        .visible_page
        .load(std::sync::atomic::Ordering::Relaxed)
        .min(pdf_state.page_count.saturating_sub(1));

    // Lazy window around visible page
    let radius = 10;
    let start_page = current_page.saturating_sub(radius);
    let end_page = (current_page + radius).min(pdf_state.page_count.saturating_sub(1));

    // If doc_path is available, extract missing text for the window
    if let Some(path) = doc_path {
        let missing_pages: Vec<usize> = (start_page..=end_page)
            .filter(|&p| p < pdf_state.page_texts.len() && pdf_state.page_texts[p].is_none())
            .collect();

        if !missing_pages.is_empty() {
            let extracted =
                crate::features::pdf::parser::extract_pdf_pages_text_batch(path, &missing_pages);
            for (p, text_opt) in extracted {
                if p < pdf_state.page_texts.len() {
                    pdf_state.page_texts[p] = text_opt;
                }
            }
        }
    }

    // Match across all currently loaded page texts
    let mut all_matches = Vec::new();
    for page_text in pdf_state.page_texts.iter().flatten() {
        let page_matches = find_matches_in_page(page_text, query_trimmed);
        all_matches.extend(page_matches);
    }

    // Sort matches by page_index, then line_index, then char_start
    all_matches.sort_by(|a, b| {
        a.page_index
            .cmp(&b.page_index)
            .then_with(|| a.line_index.cmp(&b.line_index))
            .then_with(|| a.char_start.cmp(&b.char_start))
    });

    // Try to find the closest match starting from current visible page
    let first_visible_or_after = all_matches
        .iter()
        .position(|m| m.page_index >= current_page)
        .unwrap_or(0);

    pdf_state.search.matches = all_matches;
    pdf_state.search.active_match_index = first_visible_or_after;
    pdf_state.search.update_info();
}

pub fn scroll_to_match(
    pdf_state: &mut crate::core::PdfState,
    match_item: &PdfSearchMatch,
) -> iced::Task<crate::app::Message> {
    let page = match_item.page_index;
    let page_y = pdf_state.page_y_offsets.get(page).copied().unwrap_or(0.0);

    let display_w = if pdf_state.display_width > 0.0 {
        pdf_state.display_width
    } else {
        pdf_state.desired_width
    };

    let pts_w = pdf_state
        .page_dimensions
        .get(page)
        .map(|d| d.width_pts)
        .unwrap_or(595.0)
        .max(1.0);

    let scale = display_w / pts_w;
    let match_y_on_page = match_item.rect[1].min(match_item.rect[3]) * scale;

    let target_y = (page_y + match_y_on_page - 150.0).max(0.0);
    let max_y = crate::core::scroll::max_scroll_y(
        pdf_state.total_content_height,
        pdf_state.viewport_height,
    );
    let clamped_y = target_y.clamp(0.0, max_y);

    pdf_state.scroll_controller.stop(pdf_state.scroll_y);
    pdf_state
        .smooth_scroll
        .start_navigation(pdf_state.scroll_y, clamped_y, max_y);

    pdf_state.current_page = page;
    pdf_state
        .visible_page
        .store(page, std::sync::atomic::Ordering::Relaxed);

    iced::widget::operation::scroll_to(
        "content_scroll",
        iced::widget::operation::AbsoluteOffset {
            x: 0.0,
            y: clamped_y,
        },
    )
}

pub fn handle_search_open(app: &mut crate::app::KglanceApp) -> iced::Task<crate::app::Message> {
    let pdf_state = crate::features::pdf::update::active_pdf_state_mut(app);
    pdf_state.search.open();
    iced::widget::operation::focus(SEARCH_INPUT_ID)
}

pub fn handle_search_close(app: &mut crate::app::KglanceApp) -> iced::Task<crate::app::Message> {
    let pdf_state = crate::features::pdf::update::active_pdf_state_mut(app);
    pdf_state.search.close();
    iced::Task::none()
}

pub fn handle_search_query_changed(
    app: &mut crate::app::KglanceApp,
    query: String,
) -> iced::Task<crate::app::Message> {
    let doc_path = if app.state.file_name.is_empty() {
        None
    } else {
        Some(std::path::PathBuf::from(&app.state.file_name))
    };
    let pdf_state = crate::features::pdf::update::active_pdf_state_mut(app);
    pdf_state.search.query = query.clone();

    update_search_results(pdf_state, doc_path.as_deref(), &query);

    if let Some(first_match) = pdf_state.search.current_match().cloned() {
        scroll_to_match(pdf_state, &first_match)
    } else {
        iced::Task::none()
    }
}

pub fn handle_search_next(app: &mut crate::app::KglanceApp) -> iced::Task<crate::app::Message> {
    let pdf_state = crate::features::pdf::update::active_pdf_state_mut(app);
    if let Some(next_match) = pdf_state.search.next_match().cloned() {
        scroll_to_match(pdf_state, &next_match)
    } else {
        iced::Task::none()
    }
}

pub fn handle_search_prev(app: &mut crate::app::KglanceApp) -> iced::Task<crate::app::Message> {
    let pdf_state = crate::features::pdf::update::active_pdf_state_mut(app);
    if let Some(prev_match) = pdf_state.search.prev_match().cloned() {
        scroll_to_match(pdf_state, &prev_match)
    } else {
        iced::Task::none()
    }
}

#[cfg(test)]
#[path = "search_tests.rs"]
mod tests;
