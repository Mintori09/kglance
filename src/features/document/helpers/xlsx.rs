use chrono::NaiveDate;

use crate::features::document::content::spreadsheet_content::SpreadsheetContent;
use crate::features::document::types::SheetData;
use crate::parsers::ParseError;

pub(crate) fn try_xlsx_direct(path: &str) -> Result<SpreadsheetContent, ParseError> {
    use calamine::{Reader, Xlsx, open_workbook};

    let mut workbook: Xlsx<_> = match open_workbook(path) {
        Ok(w) => w,
        Err(e) => return Err(ParseError::ParseFailed(e.to_string())),
    };

    let sheet_names = workbook.sheet_names().to_vec();
    let mut sheets = Vec::new();

    for name in &sheet_names {
        if let Ok(range) = workbook.worksheet_range(name) {
            let mut rows_iter = range.rows();
            let headers: Vec<String> = rows_iter
                .next()
                .map(|row| row.iter().map(cell_to_string).collect())
                .unwrap_or_default();

            let rows: Vec<Vec<String>> = rows_iter
                .map(|row| row.iter().map(cell_to_string).collect())
                .collect();

            sheets.push(SheetData {
                name: name.clone(),
                headers,
                rows,
            });
        }
    }

    if sheets.is_empty() {
        Err(ParseError::ParseFailed("empty spreadsheet".into()))
    } else {
        Ok(SpreadsheetContent { sheets })
    }
}

fn excel_serial_to_date(serial: f64) -> String {
    let days = serial as i64;
    let frac = serial - days as f64;

    if days == 60 {
        return "1900-02-29".to_string();
    }

    let epoch = match NaiveDate::from_ymd_opt(1899, 12, 30) {
        Some(d) => d,
        None => return format!("{serial}"),
    };

    let date = epoch + chrono::Duration::days(days + 1);
    if frac > 0.0 {
        let total_secs = (frac * 86400.0).round() as u32;
        let h = total_secs / 3600;
        let m = (total_secs % 3600) / 60;
        let s = total_secs % 60;
        date.format("%Y-%m-%d").to_string() + &format!(" {h:02}:{m:02}:{s:02}")
    } else {
        date.format("%Y-%m-%d").to_string()
    }
}

fn cell_to_string(cell: &calamine::Data) -> String {
    match cell {
        calamine::Data::String(s) => s.clone(),
        calamine::Data::Float(fv) => {
            if *fv == fv.trunc() {
                format!("{}", *fv as i64)
            } else {
                format!("{fv}")
            }
        }
        calamine::Data::Int(i) => i.to_string(),
        calamine::Data::Bool(b) => b.to_string(),
        calamine::Data::Empty => String::new(),
        calamine::Data::DateTime(dt) => excel_serial_to_date(dt.as_f64()),
        calamine::Data::Error(e) => format!("#{e}"),
        _ => String::new(),
    }
}
