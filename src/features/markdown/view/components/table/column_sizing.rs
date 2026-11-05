use crate::features::markdown::inlines_metrics;
use crate::parsers::markdown::TableBlock;

#[derive(Debug, Clone, PartialEq)]
pub struct TableColumnPlan {
    pub col_widths: Vec<f32>,
    pub total_width: f32,
    pub overflows: bool,
}

/// Resolves column widths based on cell content, font size, and available content width.
///
/// Ensures every column has a strictly bounded width (`Length::Fixed`) so that
/// nested text elements (`SelectableText`) wrap multi-line text reliably rather than
/// stretching infinitely.
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

    let char_w = font_size * 0.55;
    let pad_h = (crate::features::markdown::view::components::style::STYLE
        .table
        .cell_padding[1]
        * 2) as f32; // 24px padding per cell
    let min_col_w =
        (crate::ui::theme::scale_size(crate::ui::theme::tokens::tables::ROW_HEIGHT, font_size)
            * 2.0)
            .max(80.0);

    // Track maximum visual units across rows for each column
    let mut col_max_units = vec![0usize; num_cols];
    for (i, h) in table.headers.iter().enumerate().take(num_cols) {
        col_max_units[i] = col_max_units[i].max(inlines_metrics(&h.content).0);
    }
    for row in &table.rows {
        for (i, cell) in row.iter().enumerate().take(num_cols) {
            col_max_units[i] = col_max_units[i].max(inlines_metrics(&cell.content).0);
        }
    }

    // Desired ideal width if everything was laid out without wrapping
    let desired_widths: Vec<f32> = col_max_units
        .iter()
        .map(|&units| (units as f32 * char_w + pad_h + 16.0).max(min_col_w))
        .collect();

    let sum_min_widths: f32 = num_cols as f32 * min_col_w;

    // Check if the table fits within available content width
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
            // Desired width exceeds content width, but min widths fit!
            // Allocate each column its min_col_w plus a portion of remaining space proportional to its excess demand.
            let available_extra = (content_width - sum_min_widths).max(0.0);
            let excess_demands: Vec<f32> = desired_widths
                .iter()
                .map(|&dw| (dw - min_col_w).max(0.0))
                .collect();
            let sum_excess: f32 = excess_demands.iter().sum();

            let col_widths: Vec<f32> = if sum_excess > 0.0 {
                excess_demands
                    .iter()
                    .map(|&excess| min_col_w + (excess / sum_excess) * available_extra)
                    .collect()
            } else {
                vec![content_width / num_cols as f32; num_cols]
            };

            TableColumnPlan {
                col_widths,
                total_width: content_width,
                overflows: false,
            }
        }
    } else {
        // Table has too many columns or window is too narrow: enable horizontal scroll!
        let max_col_w = (font_size * 28.0).max(380.0);
        let col_widths: Vec<f32> = desired_widths
            .iter()
            .map(|&dw| dw.clamp(min_col_w, max_col_w))
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
