use std::path::Path;
use std::process::Command;

use crate::core::preview::PreviewContent;
use crate::features::text::content::TextContent;
use crate::parsers::ParseError;

use super::super::helpers::{docx, xlsx};

pub struct OfficeParser;

fn fallback_lo(
    path: &str,
    _ext: &str,
) -> Result<Box<dyn PreviewContent<crate::app::Message>>, ParseError> {
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
            let line_count = content.lines().count();
            Ok(Box::new(TextContent {
                content,
                language: "Document (LO)".into(),
                line_count,
                highlighted_html: None,
            }))
        }
        _ => Err(ParseError::ParseFailed(
            "LibreOffice not available or conversion failed. Install libreoffice for office document preview."
                .into(),
        )),
    }
}

pub(crate) fn parse_office(
    path: &Path,
) -> Result<Box<dyn PreviewContent<crate::app::Message>>, ParseError> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let path_str = path.to_string_lossy().to_string();

    match ext.as_str() {
        "docx" => {
            if let Ok(content) = docx::try_docx_direct(&path_str) {
                let line_count = content.lines().count();
                return Ok(Box::new(TextContent {
                    content,
                    language: "DOCX".into(),
                    line_count,
                    highlighted_html: None,
                }));
            }
        }
        "xlsx" => {
            if let Ok(spreadsheet) = xlsx::try_xlsx_direct(&path_str) {
                return Ok(Box::new(spreadsheet));
            }
        }
        _ => {}
    }

    fallback_lo(&path_str, &ext)
}

pub(crate) fn supported_office_extensions() -> &'static [&'static str] {
    &["docx", "xlsx", "pptx", "odt", "ods", "odp"]
}
