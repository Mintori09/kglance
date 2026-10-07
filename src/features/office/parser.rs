use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::NaiveDate;

use crate::features::common::parser::traits::{ParseError, PreviewParser};
use crate::features::common::parser::types::ParsedContent;
use crate::features::office::types::SheetData;

pub struct OfficeParser;

impl PreviewParser for OfficeParser {
    fn supported_extensions(&self) -> &[&str] {
        &[
            "docx", "doc", "xlsx", "xls", "pptx", "ppt", "odt", "ods", "odp",
        ]
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let path_str = path.to_string_lossy().to_string();

        match ext.as_str() {
            "xlsx" | "xls" | "ods" | "xlsb" => {
                if let Ok(spreadsheet) = try_spreadsheet_direct(&path_str) {
                    return Ok(spreadsheet);
                }
            }
            _ => {}
        }

        compile_office_to_pdf(path)
    }
}

fn convert_via_libreoffice(input_path: &Path, output_pdf_path: &Path) -> Result<(), ParseError> {
    let temp_dir = tempfile::tempdir().map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let temp_profile = tempfile::tempdir().map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let profile_arg = format!(
        "-env:UserInstallation=file://{}",
        temp_profile.path().to_string_lossy()
    );

    let status = Command::new("soffice")
        .args([
            &profile_arg,
            "--headless",
            "--convert-to",
            "pdf",
            "--outdir",
            temp_dir.path().to_string_lossy().as_ref(),
            input_path.to_string_lossy().as_ref(),
        ])
        .status()
        .map_err(|e| ParseError::ParseFailed(format!("Failed to execute soffice: {e}")))?;

    if !status.success() {
        return Err(ParseError::ParseFailed(
            "LibreOffice not available or conversion failed. Install libreoffice for office document preview.".into(),
        ));
    }

    let stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("document");
    let generated_pdf = temp_dir.path().join(format!("{stem}.pdf"));

    if generated_pdf.exists() {
        std::fs::copy(&generated_pdf, output_pdf_path)
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        return Ok(());
    }

    if let Ok(entries) = std::fs::read_dir(temp_dir.path()) {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|e| e.to_str()) == Some("pdf") {
                std::fs::copy(entry.path(), output_pdf_path)
                    .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
                return Ok(());
            }
        }
    }

    Err(ParseError::ParseFailed(
        "LibreOffice conversion produced no PDF file".into(),
    ))
}

pub fn get_or_compile_office_to_pdf(path: &Path) -> Result<PathBuf, ParseError> {
    let metadata = std::fs::metadata(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let mtime = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let size = metadata.len();
    let hash_input = format!("{}:{mtime}:{size}", path.to_string_lossy());
    let hash = format!("{:x}", md5::compute(hash_input.as_bytes()));
    let cache_dir = std::env::temp_dir().join("kglance_office");
    let _ = std::fs::create_dir_all(&cache_dir);
    let cache_pdf = cache_dir.join(format!("{hash}.pdf"));

    if cache_pdf.exists()
        && std::fs::metadata(&cache_pdf)
            .map(|m| m.len() > 0)
            .unwrap_or(false)
    {
        return Ok(cache_pdf);
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext == "docx" && docxide_pdf::convert_docx_to_pdf(path, &cache_pdf).is_ok() {
        return Ok(cache_pdf);
    }

    convert_via_libreoffice(path, &cache_pdf)?;
    Ok(cache_pdf)
}

pub fn get_or_compile_docx_to_pdf(docx_path: &Path) -> Result<PathBuf, ParseError> {
    get_or_compile_office_to_pdf(docx_path)
}

pub fn compile_office_to_pdf(path: &Path) -> Result<ParsedContent, ParseError> {
    let pdf_path = get_or_compile_office_to_pdf(path)?;
    let doc = mupdf::Document::open(pdf_path.as_path())
        .map_err(|e| ParseError::ParseFailed(format!("Failed to open generated PDF: {e}")))?;

    let page_count = doc
        .page_count()
        .map_err(|e| ParseError::ParseFailed(e.to_string()))? as u32;

    let outline = crate::features::pdf::parser::extract_pdf_toc(&doc);
    let page_dimensions = crate::features::pdf::dimensions::extract_page_dimensions(&doc)
        .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

    let first_page = if page_count > 0 {
        let raw_page = crate::features::pdf::parser::render_pdf_page(&pdf_path, 0)?;
        let compressed = crate::features::pdf::compress::compress_rgba_to_png(
            &raw_page.data,
            raw_page.width,
            raw_page.height,
        )
        .unwrap_or(raw_page.data);
        crate::features::pdf::types::PageData {
            width: raw_page.width,
            height: raw_page.height,
            data: compressed,
        }
    } else {
        crate::features::pdf::types::PageData {
            width: 0,
            height: 0,
            data: Vec::new(),
        }
    };
    let first_page_text = if page_count > 0 {
        crate::features::pdf::parser::extract_page_text_from_doc(&doc, 0)
    } else {
        None
    };

    crate::features::pdf::parser::empty_mupdf_store();

    Ok(ParsedContent::Pdf {
        page_count,
        first_page,
        outline,
        page_dimensions,
        first_page_text,
    })
}

pub fn compile_docx_to_pdf(path: &Path) -> Result<ParsedContent, ParseError> {
    compile_office_to_pdf(path)
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

fn try_spreadsheet_direct(path: &str) -> Result<ParsedContent, ParseError> {
    use calamine::{Reader, open_workbook_auto};

    let mut workbook =
        open_workbook_auto(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_office_parser_supported_extensions() {
        let parser = OfficeParser;
        assert!(parser.supported_extensions().contains(&"docx"));
        assert!(parser.supported_extensions().contains(&"xlsx"));
        assert!(parser.supported_extensions().contains(&"pptx"));
        assert!(parser.supported_extensions().contains(&"odp"));
    }

    #[test]
    fn test_docx_invalid_path_fails() {
        let parser = OfficeParser;
        let result = parser.parse(Path::new("/nonexistent/invalid_file.docx"));
        assert!(result.is_err());
    }
}
