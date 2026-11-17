use crate::features::markdown::inlines_metrics;
use crate::parsers::markdown::{Inline, TableBlock};

#[derive(Debug, Clone, PartialEq)]
pub struct TableColumnPlan {
    pub col_widths: Vec<f32>,
    pub total_width: f32,
    pub overflows: bool,
}

fn char_visual_units(c: char) -> usize {
    if ('\u{2E80}'..='\u{9FFF}').contains(&c)
        || ('\u{3040}'..='\u{30FF}').contains(&c)
        || ('\u{AC00}'..='\u{D7AF}').contains(&c)
        || ('\u{FF01}'..='\u{FF60}').contains(&c)
    {
        2
    } else {
        1
    }
}

fn count_str_unbreakable_runs(s: &str, current_run: &mut usize, max_run: &mut usize) {
    for c in s.chars() {
        if c.is_whitespace() {
            *max_run = (*max_run).max(*current_run);
            *current_run = 0;
        } else {
            *current_run += char_visual_units(c);
        }
    }
}

fn count_inlines_unbreakable_runs(
    inlines: &[Inline],
    current_run: &mut usize,
    max_run: &mut usize,
) {
    for inline in inlines {
        match inline {
            Inline::Text(t) => count_str_unbreakable_runs(t, current_run, max_run),
            Inline::Bold(c)
            | Inline::Italic(c)
            | Inline::Strikethrough(c)
            | Inline::Underline(c)
            | Inline::Styled { content: c, .. } => {
                count_inlines_unbreakable_runs(c, current_run, max_run);
            }
            Inline::Code(t) => {
                *current_run += 2;
                count_str_unbreakable_runs(t, current_run, max_run);
            }
            Inline::Link { text, .. } => {
                count_inlines_unbreakable_runs(text, current_run, max_run);
            }
            Inline::Image { alt, .. } => count_str_unbreakable_runs(alt, current_run, max_run),
            Inline::InlineMath(latex) | Inline::DisplayMath(latex) => {
                count_str_unbreakable_runs(latex, current_run, max_run);
            }
            Inline::FootnoteReference(label) => {
                *current_run += 3 + label.len();
            }
            Inline::SoftBreak => {
                *max_run = (*max_run).max(*current_run);
                *current_run = 0;
            }
            Inline::Ruby { text, .. } => {
                count_str_unbreakable_runs(text, current_run, max_run);
            }
        }
    }
}

/// Returns the maximum contiguous non-whitespace visual units in `inlines`.
fn inlines_max_unbreakable_units(inlines: &[Inline]) -> usize {
    let mut current_run = 0usize;
    let mut max_run = 0usize;
    count_inlines_unbreakable_runs(inlines, &mut current_run, &mut max_run);
    max_run.max(current_run)
}

/// Resolves column widths based on cell content, font size, and available content width.
///
/// Ensures every column has a strictly bounded width (`Length::Fixed`) so that
/// nested text elements (`SelectableText`) wrap multi-line text reliably rather than
/// stretching infinitely, while enabling horizontal scrolling whenever unbreakable
/// cell tokens cannot fit inside `content_width`.
pub fn resolve_column_plan(
    table: &TableBlock,
    font_size: f32,
    content_width: f32,
) -> TableColumnPlan {
    let num_cols = table
        .headers
        .len()
        .max(table.rows.first().map_or(0, |r| r.len()));

    if num_cols == 0 {
        return TableColumnPlan {
            col_widths: Vec::new(),
            total_width: 0.0,
            overflows: false,
        };
    }

    let style = &crate::features::markdown::view::components::style::STYLE;
    let header_size = crate::ui::theme::scale_size(style.table.header_font_size, font_size);
    let cell_size = crate::ui::theme::scale_size(style.table.cell_font_size, font_size);
    let header_char_w = header_size * 0.55;
    let cell_char_w = cell_size * 0.52;
    let pad_h = (style.table.cell_padding[1] * 2) as f32; // 24px padding per cell
    let min_col_w = crate::ui::theme::scale_size(64.0, font_size).max(64.0);
    let max_col_w = (font_size * 22.0).max(300.0);
    let prose_cap = (font_size * 16.0).clamp(180.0, 260.0);

    // Track desired width and unbreakable minimum width in pixels per column,
    // using header_char_w for headers and cell_char_w for body rows.
    let mut col_desired_text_px = vec![0.0_f32; num_cols];
    let mut col_unbreakable_text_px = vec![0.0_f32; num_cols];

    for (i, h) in table.headers.iter().enumerate().take(num_cols) {
        let total_px = inlines_metrics(&h.content).0 as f32 * header_char_w;
        let unbreakable_px = inlines_max_unbreakable_units(&h.content) as f32 * header_char_w;
        col_desired_text_px[i] = col_desired_text_px[i].max(total_px);
        col_unbreakable_text_px[i] = col_unbreakable_text_px[i].max(unbreakable_px);
    }
    for row in &table.rows {
        for (i, cell) in row.iter().enumerate().take(num_cols) {
            let total_px = inlines_metrics(&cell.content).0 as f32 * cell_char_w;
            let unbreakable_px = inlines_max_unbreakable_units(&cell.content) as f32 * cell_char_w;
            col_desired_text_px[i] = col_desired_text_px[i].max(total_px);
            col_unbreakable_text_px[i] = col_unbreakable_text_px[i].max(unbreakable_px);
        }
    }

    let extra_margin = 6.0;
    // Desired ideal width if everything was laid out without wrapping
    let desired_widths: Vec<f32> = col_desired_text_px
        .iter()
        .map(|&px| (px + pad_h + extra_margin).max(min_col_w))
        .collect();

    let col_min_widths: Vec<f32> = col_unbreakable_text_px
        .iter()
        .map(|&px| (px + pad_h + extra_margin).clamp(min_col_w, max_col_w))
        .collect();

    let sum_min_widths: f32 = col_min_widths.iter().sum();

    // Check if the table fits within available content width without crushing unbreakable tokens
    if content_width > 0.0 && sum_min_widths <= content_width {
        let sum_desired: f32 = desired_widths.iter().sum();

        if sum_desired <= content_width {
            // Content fits completely without even needing to wrap much!
            // Proportionally expand columns to fill the full width neatly.
            let extra = content_width - sum_desired;
            let col_widths: Vec<f32> = desired_widths
                .iter()
                .map(|&dw| dw + (dw / sum_desired) * extra)
                .collect();

            TableColumnPlan {
                col_widths,
                total_width: content_width,
                overflows: false,
            }
        } else {
            // Desired width exceeds content width, but unbreakable min widths fit!
            // Allocate each column its min width plus a portion of remaining space proportional to its excess demand.
            let available_extra = (content_width - sum_min_widths).max(0.0);
            let excess_demands: Vec<f32> = desired_widths
                .iter()
                .zip(col_min_widths.iter())
                .map(|(&dw, &min_w)| (dw - min_w).max(0.0))
                .collect();
            let sum_excess: f32 = excess_demands.iter().sum();

            let col_widths: Vec<f32> = if sum_excess > 0.0 {
                col_min_widths
                    .iter()
                    .zip(excess_demands.iter())
                    .map(|(&min_w, &excess)| min_w + (excess / sum_excess) * available_extra)
                    .collect()
            } else {
                col_min_widths
                    .iter()
                    .map(|&min_w| min_w + available_extra / num_cols as f32)
                    .collect()
            };

            TableColumnPlan {
                col_widths,
                total_width: content_width,
                overflows: false,
            }
        }
    } else {
        // Table has too many columns, unbreakable tokens exceed available width, or window is narrow: enable horizontal scroll!
        // Cap wrappable prose columns at prose_cap so they wrap cleanly instead of stretching on one line,
        // while still allowing genuinely long unbreakable tokens up to max_col_w.
        let col_widths: Vec<f32> = desired_widths
            .iter()
            .zip(col_min_widths.iter())
            .map(|(&dw, &min_w)| {
                let cap = prose_cap.max(min_w).min(max_col_w);
                dw.clamp(min_w, cap)
            })
            .collect();
        let total_width = col_widths.iter().sum();

        TableColumnPlan {
            col_widths,
            total_width,
            overflows: content_width > 0.0 && total_width > content_width,
        }
    }
}

#[cfg(test)]
#[path = "column_sizing_tests.rs"]
mod tests;
