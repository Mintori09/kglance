use std::path::Path;
use std::process::Command;

use crate::parser::{ParseError, ParsedContent, PreviewParser};

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
                if let Ok(content) = try_xlsx_direct(&path_str) {
                    return Ok(ParsedContent::Office {
                        content,
                        format: "XLSX".into(),
                        page_count: 1,
                    });
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

fn try_xlsx_direct(path: &str) -> Result<String, ParseError> {
    use calamine::{open_workbook, Reader, Xlsx};

    let mut workbook: Xlsx<_> = match open_workbook(path) {
        Ok(w) => w,
        Err(e) => return Err(ParseError::ParseFailed(e.to_string())),
    };

    let mut result = String::new();
    let sheet_names = workbook.sheet_names().to_vec();

    for (i, name) in sheet_names.iter().enumerate() {
        if i > 0 {
            result.push('\n');
        }
        result.push_str(&format!("=== {name} ===\n"));
        if let Ok(range) = workbook.worksheet_range(name) {
            for row in range.rows() {
                let line: Vec<String> = row
                    .iter()
                    .map(|cell| match cell {
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
                        calamine::Data::DateTime(_) => cell.to_string(),
                        calamine::Data::Error(e) => format!("#{e}"),
                        _ => String::new(),
                    })
                    .collect();
                result.push_str(&line.join("\t"));
                result.push('\n');
            }
        }
    }

    if result.trim().is_empty() {
        Err(ParseError::ParseFailed("empty spreadsheet".into()))
    } else {
        Ok(result)
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
