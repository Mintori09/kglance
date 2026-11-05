use super::types::{CellRange, SheetInfo};

#[derive(Debug, Clone, PartialEq)]
pub struct SelectionStats {
    pub cell_count: usize,
    pub non_empty_count: usize,
    pub numeric_count: usize,
    pub sum: Option<f64>,
    pub avg: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl SelectionStats {
    /// Formats the stats into a clean, concise string for the status bar / info display.
    pub fn format_summary(&self) -> Option<String> {
        if self.cell_count <= 1 {
            return None;
        }

        if self.numeric_count > 0
            && let (Some(sum), Some(avg)) = (self.sum, self.avg)
        {
            let sum_str = format_compact_num(sum);
            let avg_str = format_compact_num(avg);
            Some(format!(
                "Selected: {} cells · Sum: {sum_str} · Avg: {avg_str}",
                self.cell_count
            ))
        } else {
            Some(format!("Selected: {} cells", self.cell_count))
        }
    }
}

fn format_compact_num(n: f64) -> String {
    if n.fract().abs() < 1e-4 {
        format!("{:.0}", n)
    } else {
        let s = format!("{:.2}", n);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// Calculates selection statistics from the currently active sheet data and row display indices.
pub fn calculate_selection_stats(
    sheet: &SheetInfo,
    display_indices: &[usize],
    range: CellRange,
) -> SelectionStats {
    let (min_r, max_r) = range.row_range();
    let (min_c, max_c) = range.col_range();

    let cell_count = (max_r - min_r + 1) * (max_c - min_c + 1);
    let mut non_empty_count = 0;
    let mut numeric_count = 0;
    let mut sum_val = 0.0;
    let mut min_val = f64::INFINITY;
    let mut max_val = f64::NEG_INFINITY;

    for r in min_r..=max_r {
        if let Some(row_data) = display_indices
            .get(r)
            .and_then(|&orig_idx| sheet.rows.get(orig_idx))
        {
            for c in min_c..=max_c {
                if let Some(val_str) = row_data.get(c) {
                    let trimmed = val_str.trim();
                    if !trimmed.is_empty() {
                        non_empty_count += 1;
                        // Clean numeric formatting like commas in "$1,234.56"
                        let clean_num: String = trimmed
                            .chars()
                            .filter(|&ch| ch.is_ascii_digit() || ch == '.' || ch == '-')
                            .collect();
                        if let Ok(num) = clean_num.parse::<f64>() {
                            numeric_count += 1;
                            sum_val += num;
                            if num < min_val {
                                min_val = num;
                            }
                            if num > max_val {
                                max_val = num;
                            }
                        }
                    }
                }
            }
        }
    }

    let (sum, avg, min, max) = if numeric_count > 0 {
        (
            Some(sum_val),
            Some(sum_val / numeric_count as f64),
            Some(min_val),
            Some(max_val),
        )
    } else {
        (None, None, None, None)
    };

    SelectionStats {
        cell_count,
        non_empty_count,
        numeric_count,
        sum,
        avg,
        min,
        max,
    }
}
