use super::selection::{PdfPageText, PdfSelection};

#[derive(Debug, Clone, PartialEq)]
pub struct SelectedFragment {
    pub text: String,
    pub rect: [f32; 4], // [min_x, min_y, max_x, max_y]
}

impl SelectedFragment {
    #[inline]
    pub fn height(&self) -> f32 {
        (self.rect[3] - self.rect[1]).abs()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VisualRow {
    pub cells: Vec<SelectedFragment>,
}

pub fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

pub fn extract_selected_content(
    page_texts: &[Option<PdfPageText>],
    selection: &PdfSelection,
) -> (Option<String>, Option<String>) {
    let (start, end) = selection.normalized();
    let mut all_page_rows: Vec<Vec<VisualRow>> = Vec::new();

    for p in start.page..=end.page {
        let Some(page_text) = page_texts.get(p).and_then(|t| t.as_ref()) else {
            continue;
        };

        let start_line = if p == start.page { start.line } else { 0 };
        let end_line = if p == end.page {
            end.line
        } else {
            page_text.lines.len().saturating_sub(1)
        };

        let mut fragments = Vec::new();

        for line_idx in start_line..=end_line {
            if line_idx >= page_text.lines.len() {
                break;
            }
            let line = &page_text.lines[line_idx];
            let char_start = if p == start.page && line_idx == start.line {
                start.char_idx.min(line.chars.len())
            } else {
                0
            };
            let char_end = if p == end.page && line_idx == end.line {
                end.char_idx.min(line.chars.len())
            } else {
                line.chars.len()
            };

            if char_start >= char_end {
                continue;
            }

            let mut frag_text = String::with_capacity(char_end - char_start);
            let mut min_x = f32::MAX;
            let mut max_x = f32::MIN;
            let mut min_y = f32::MAX;
            let mut max_y = f32::MIN;

            for ch in &line.chars[char_start..char_end] {
                frag_text.push(ch.ch);
                let [cx0, cy0, cx1, cy1] = ch.rect;
                min_x = min_x.min(cx0.min(cx1));
                max_x = max_x.max(cx0.max(cx1));
                min_y = min_y.min(cy0.min(cy1));
                max_y = max_y.max(cy0.max(cy1));
            }

            if min_x > max_x {
                let [lx0, _, lx1, _] = line.rect;
                min_x = lx0.min(lx1);
                max_x = lx0.max(lx1);
            }
            if min_y > max_y {
                let [_, ly0, _, ly1] = line.rect;
                min_y = ly0.min(ly1);
                max_y = ly0.max(ly1);
            }

            fragments.push(SelectedFragment {
                text: frag_text,
                rect: [min_x, min_y, max_x, max_y],
            });
        }

        if !fragments.is_empty() {
            let page_rows = cluster_fragments_into_rows(fragments);
            all_page_rows.push(page_rows);
        }
    }

    if all_page_rows.is_empty() {
        return (None, None);
    }

    let plain = build_plain_text(&all_page_rows);
    let html = build_html_text(&all_page_rows);

    let plain_opt = if plain.trim().is_empty() {
        None
    } else {
        Some(plain)
    };
    let html_opt = if html.is_empty() { None } else { Some(html) };

    (plain_opt, html_opt)
}

fn cluster_fragments_into_rows(mut fragments: Vec<SelectedFragment>) -> Vec<VisualRow> {
    // Sort primarily by top Y, tie-breaking by left X
    fragments.sort_by(|a, b| {
        let diff_y = a.rect[1] - b.rect[1];
        if diff_y.abs() > 4.0 {
            a.rect[1]
                .partial_cmp(&b.rect[1])
                .unwrap_or(std::cmp::Ordering::Equal)
        } else {
            a.rect[0]
                .partial_cmp(&b.rect[0])
                .unwrap_or(std::cmp::Ordering::Equal)
        }
    });

    let mut rows: Vec<VisualRow> = Vec::new();

    for frag in fragments {
        let mut target_row_idx = None;

        // Try to place in an existing row with matching vertical band
        for (idx, row) in rows.iter().enumerate() {
            let row_y0 = row.cells.iter().map(|c| c.rect[1]).fold(f32::MAX, f32::min);
            let row_y1 = row.cells.iter().map(|c| c.rect[3]).fold(f32::MIN, f32::max);
            let row_h = (row_y1 - row_y0).max(8.0);
            let frag_h = frag.height().max(8.0);
            let min_h = row_h.min(frag_h);

            let overlap_y = (row_y1.min(frag.rect[3]) - row_y0.max(frag.rect[1])).max(0.0);
            let y_diff = (frag.rect[1] - row_y0).abs();
            let is_aligned = overlap_y >= 0.4 * min_h || y_diff <= 4.0;

            let overlaps_x = row
                .cells
                .iter()
                .any(|c| c.rect[0].max(frag.rect[0]) < c.rect[2].min(frag.rect[2]) - 2.0);

            if is_aligned && !overlaps_x {
                target_row_idx = Some(idx);
                break;
            }
        }

        if let Some(idx) = target_row_idx {
            let row = &mut rows[idx];
            row.cells.push(frag);
            row.cells.sort_by(|a, b| {
                a.rect[0]
                    .partial_cmp(&b.rect[0])
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        } else {
            rows.push(VisualRow { cells: vec![frag] });
        }
    }

    // Sort rows by their top Y coordinate
    rows.sort_by(|a, b| {
        let a_y = a.cells.iter().map(|c| c.rect[1]).fold(f32::MAX, f32::min);
        let b_y = b.cells.iter().map(|c| c.rect[1]).fold(f32::MAX, f32::min);
        a_y.partial_cmp(&b_y).unwrap_or(std::cmp::Ordering::Equal)
    });

    rows
}

fn build_plain_text(all_pages: &[Vec<VisualRow>]) -> String {
    let mut out = String::new();

    for (p_idx, page_rows) in all_pages.iter().enumerate() {
        if p_idx > 0 && !out.is_empty() {
            out.push('\n');
        }
        for row in page_rows {
            if !out.is_empty() && !out.ends_with('\n') {
                out.push('\n');
            }
            let row_text = row
                .cells
                .iter()
                .map(|c| c.text.as_str())
                .collect::<Vec<_>>()
                .join("\t");
            out.push_str(&row_text);
        }
    }

    out
}

fn build_html_text(all_pages: &[Vec<VisualRow>]) -> String {
    let mut body = String::new();

    for (p_idx, page_rows) in all_pages.iter().enumerate() {
        if p_idx > 0 {
            body.push_str(
                "<hr style=\"border: none; border-top: 1px dashed #ccc; margin: 12px 0;\">",
            );
        }

        let mut row_idx = 0;
        while row_idx < page_rows.len() {
            let row = &page_rows[row_idx];

            if row.cells.len() > 1 {
                // Collect contiguous table rows
                let mut table_rows = Vec::new();
                while row_idx < page_rows.len() && page_rows[row_idx].cells.len() > 1 {
                    table_rows.push(&page_rows[row_idx]);
                    row_idx += 1;
                }

                body.push_str("<table style=\"border-collapse: collapse; margin: 6px 0;\"><tbody>");
                for t_row in table_rows {
                    body.push_str("<tr>");
                    for (c_idx, cell) in t_row.cells.iter().enumerate() {
                        let padding = if c_idx + 1 < t_row.cells.len() {
                            "padding: 2px 14px 2px 0; vertical-align: top;"
                        } else {
                            "padding: 2px 0; vertical-align: top;"
                        };
                        let escaped = escape_html(&cell.text);
                        body.push_str(&format!("<td style=\"{padding}\">{escaped}</td>"));
                    }
                    body.push_str("</tr>");
                }
                body.push_str("</tbody></table>");
            } else if let Some(cell) = row.cells.first() {
                let text = cell.text.trim();
                let escaped = escape_html(&cell.text);
                let is_heading = cell.height() >= 16.0
                    || (text.len() >= 5
                        && text
                            .chars()
                            .filter(|c| c.is_alphabetic())
                            .all(|c| c.is_uppercase()));

                if is_heading {
                    body.push_str(&format!(
                        "<p style=\"font-size: 1.15em; font-weight: bold; margin: 6px 0;\">{escaped}</p>"
                    ));
                } else {
                    body.push_str(&format!("<p style=\"margin: 3px 0;\">{escaped}</p>"));
                }
                row_idx += 1;
            } else {
                row_idx += 1;
            }
        }
    }

    if body.is_empty() {
        return String::new();
    }

    format!(
        "<meta http-equiv=\"content-type\" content=\"text/html; charset=utf-8\"><div style=\"font-family: system-ui, -apple-system, sans-serif; font-size: 11pt; line-height: 1.4; color: #111;\">{body}</div>"
    )
}
