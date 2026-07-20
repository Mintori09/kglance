use std::path::Path;
use std::process::Command;

use chrono::NaiveDate;

use crate::parsers::{ParseError, ParsedContent, PreviewParser, SheetData};

pub struct OfficeParser;

impl PreviewParser for OfficeParser {
    fn supported_extensions(&self) -> &[&str] {
        &["docx", "xlsx", "pptx", "odt", "ods", "odp"]
    }

    fn is_supported(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| {
                self.supported_extensions()
                    .contains(&e.to_lowercase().as_str())
            })
            .unwrap_or(false)
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let path_str = path.to_string_lossy().to_string();

        match ext.as_str() {
            "docx" => {
                if let Ok(content) = try_docx_direct(&path_str) {
                    return Ok(ParsedContent::Office {
                        content,
                        format: "DOCX".into(),
                        page_count: 1,
                    });
                }
            }
            "xlsx" => {
                if let Ok(spreadsheet) = try_xlsx_direct(&path_str) {
                    return Ok(spreadsheet);
                }
            }
            _ => {}
        }

        fallback_lo(&path_str, &ext)
    }
}

fn try_docx_direct(path: &str) -> Result<String, ParseError> {
    use std::io::Read;

    let file = std::fs::File::open(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let mut document = archive
        .by_name("word/document.xml")
        .map_err(|_| ParseError::ParseFailed("no document.xml".into()))?;
    let mut xml = String::new();
    document
        .read_to_string(&mut xml)
        .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

    let content = extract_docx_text(&xml);
    if content.trim().is_empty() {
        Err(ParseError::ParseFailed("empty document".into()))
    } else {
        Ok(content)
    }
}

fn extract_docx_text(xml: &str) -> String {
    let mut result = String::new();
    let mut in_para = false;
    let mut in_text = false;
    let bytes = xml.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'<' {
            let tag_end = xml[i..].find('>').map(|p| i + p + 1).unwrap_or(len);
            let tag = &xml[i..tag_end];
            if tag.starts_with("<w:p") {
                in_para = true;
                if !result.is_empty() && !result.ends_with('\n') {
                    result.push('\n');
                }
            } else if tag.starts_with("</w:p") {
                in_para = false;
                in_text = false;
            } else if tag.starts_with("<w:t") && !tag.starts_with("</") {
                in_text = true;
            } else if tag.starts_with("</w:t") {
                in_text = false;
            } else if tag.starts_with("<w:br") && in_para {
                result.push('\n');
            }
            i = tag_end;
        } else if in_text {
            let text_end = xml[i..].find('<').map(|p| i + p).unwrap_or(len);
            result.push_str(&xml[i..text_end]);
            i = text_end;
        } else {
            i += 1;
        }
    }

    result
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
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

fn try_xlsx_direct(path: &str) -> Result<ParsedContent, ParseError> {
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
        Ok(ParsedContent::Spreadsheet { sheets })
    }
}

fn fallback_lo(path: &str, _ext: &str) -> Result<ParsedContent, ParseError> {
    let out_dir = tempfile::tempdir().map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let out_path = out_dir.path().join("page.png");

    let status = Command::new("soffice")
        .args([
            "--headless",
            "--convert-to",
            "png",
            "--outdir",
            out_dir.path().to_string_lossy().as_ref(),
            path,
        ])
        .status();

    match status {
        Ok(s) if s.success() => {
            let content = format!("Converted via LibreOffice\nOutput: {}\n", out_path.display());
            Ok(ParsedContent::Office {
                content,
                format: "Document (LO)".into(),
                page_count: 1,
            })
        }
        _ => Err(ParseError::ParseFailed(
            "LibreOffice not available or conversion failed. Install libreoffice for office document preview.".into(),
        )),
    }
}
