use super::types::{CellRange, SheetInfo};

/// Escapes a cell's string value for TSV/CSV according to RFC 4180 rules.
/// If the text contains tabs, newlines, carriage returns, or quotes, it is enclosed in double quotes
/// and any internal quotes are doubled (" -> "").
pub fn escape_tsv_cell(cell: &str) -> String {
    if cell.contains('\t') || cell.contains('\n') || cell.contains('\r') || cell.contains('"') {
        let escaped = cell.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        cell.to_string()
    }
}

/// Builds an RFC-4180 compliant TSV string for the given selected cell range.
pub fn build_tsv_text(
    sheet: &SheetInfo,
    display_indices: &[usize],
    range: CellRange,
) -> Option<String> {
    let (min_r, max_r) = range.row_range();
    let (min_c, max_c) = range.col_range();

    let mut lines = Vec::new();
    for r in min_r..=max_r {
        if let Some(row_data) = display_indices
            .get(r)
            .and_then(|&orig_idx| sheet.rows.get(orig_idx))
        {
            let mut row_cells = Vec::new();
            for c in min_c..=max_c {
                let cell_text = row_data.get(c).map(String::as_str).unwrap_or("");
                row_cells.push(escape_tsv_cell(cell_text));
            }
            lines.push(row_cells.join("\t"));
        }
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

/// Escapes text for safe inclusion within HTML tags.
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\n', "<br>")
}

/// Builds an HTML `<table>` representation of the selected cell range for clipboard pasting into
/// rich-text targets (e.g. Google Docs, LibreOffice Calc, Excel, Notion, Slack).
pub fn build_html_table(
    sheet: &SheetInfo,
    display_indices: &[usize],
    range: CellRange,
) -> Option<String> {
    let (min_r, max_r) = range.row_range();
    let (min_c, max_c) = range.col_range();

    let mut rows_html = Vec::new();
    for r in min_r..=max_r {
        if let Some(row_data) = display_indices
            .get(r)
            .and_then(|&orig_idx| sheet.rows.get(orig_idx))
        {
            let mut cells_html = Vec::new();
            for c in min_c..=max_c {
                let cell_text = row_data.get(c).map(String::as_str).unwrap_or("");
                cells_html.push(format!("<td>{}</td>", escape_html(cell_text)));
            }
            rows_html.push(format!("<tr>{}</tr>", cells_html.join("")));
        }
    }

    if rows_html.is_empty() {
        None
    } else {
        Some(format!(
            "<table border=\"1\" style=\"border-collapse:collapse;\"><tbody>{}</tbody></table>",
            rows_html.join("")
        ))
    }
}
