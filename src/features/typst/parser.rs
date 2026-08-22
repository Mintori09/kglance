use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;

use crate::features::common::parser::traits::{ParseError, PreviewParser};
use crate::features::common::parser::types::ParsedContent;
use crate::features::pdf::parser::{PdfTocEntry, extract_pdf_toc, render_pdf_page};
use crate::features::pdf::types::PageData;

pub type TypstCompiledOutput = (
    NamedTempFile,
    u32,
    PageData,
    Vec<PdfTocEntry>,
    Vec<crate::features::pdf::types::PageDimensions>,
);

pub fn compile_typst_to_pdf(path: &Path) -> Result<TypstCompiledOutput, ParseError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let temp_pdf = tempfile::Builder::new()
        .suffix(".pdf")
        .tempfile()
        .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

    let output = Command::new("typst")
        .arg("compile")
        .arg("--format")
        .arg("pdf")
        .arg("--root")
        .arg(parent)
        .arg(path)
        .arg(temp_pdf.path())
        .output()
        .map_err(|e| ParseError::ParseFailed(format!("Failed to execute `typst`: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let err_msg = if !stderr.trim().is_empty() {
            stderr.to_string()
        } else if !stdout.trim().is_empty() {
            stdout.to_string()
        } else {
            format!(
                "typst compilation failed with status {:?}",
                output.status.code()
            )
        };
        return Err(ParseError::ParseFailed(err_msg));
    }

    let doc = mupdf::Document::open(temp_pdf.path())
        .map_err(|e| ParseError::ParseFailed(format!("Failed to open compiled PDF: {e}")))?;
    let page_count = doc
        .page_count()
        .map_err(|e| ParseError::ParseFailed(e.to_string()))? as u32;

    let outline = extract_pdf_toc(&doc);
    let page_dimensions = crate::features::pdf::dimensions::extract_page_dimensions(&doc)
        .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

    let first_page = if page_count > 0 {
        let raw_page = render_pdf_page(temp_pdf.path(), 0)?;
        let compressed = crate::features::pdf::compress::compress_rgba_to_png(
            &raw_page.data,
            raw_page.width,
            raw_page.height,
        )
        .unwrap_or(raw_page.data);
        PageData {
            width: raw_page.width,
            height: raw_page.height,
            data: compressed,
        }
    } else {
        PageData {
            width: 0,
            height: 0,
            data: Vec::new(),
        }
    };

    Ok((temp_pdf, page_count, first_page, outline, page_dimensions))
}

pub struct TypstParser;

impl PreviewParser for TypstParser {
    fn supported_extensions(&self) -> &[&str] {
        &["typ"]
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let source =
            std::fs::read_to_string(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        match compile_typst_to_pdf(path) {
            Ok((_temp_pdf, page_count, first_page, outline, page_dimensions)) => {
                Ok(ParsedContent::Typst {
                    source,
                    page_count,
                    first_page,
                    error: None,
                    outline,
                    page_dimensions,
                })
            }
            Err(err) => Ok(ParsedContent::Typst {
                source,
                page_count: 0,
                first_page: PageData {
                    width: 0,
                    height: 0,
                    data: Vec::new(),
                },
                error: Some(err.to_string()),
                outline: Vec::new(),
                page_dimensions: Vec::new(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_typ_extension() {
        let parser = TypstParser;
        assert!(parser.is_supported(Path::new("doc.typ")));
        assert!(!parser.is_supported(Path::new("doc.txt")));
        assert!(!parser.is_supported(Path::new("file.pdf")));
    }

    #[test]
    fn parses_simple_document() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.typ");
        std::fs::write(
            &file_path,
            "#set page(width: 200pt, height: 100pt)\nHello, Typst!",
        )
        .unwrap();

        let parser = TypstParser;
        let result = parser.parse(&file_path);

        match &result {
            Ok(ParsedContent::Typst {
                source,
                page_count,
                first_page,
                error,
                ..
            }) => {
                assert!(source.contains("Hello, Typst!"));
                assert_eq!(*page_count, 1);
                assert!(first_page.width > 0);
                assert!(first_page.height > 0);
                assert!(!first_page.data.is_empty());
                assert!(error.is_none());
            }
            other => panic!("expected Typst variant, got {other:?}"),
        }
    }

    #[test]
    fn compile_error_is_surfaced() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("broken.typ");
        std::fs::write(&file_path, "#set page(width: missing_var)\n").unwrap();

        let parser = TypstParser;
        let result = parser.parse(&file_path);

        match result {
            Ok(ParsedContent::Typst {
                page_count,
                error: Some(msg),
                ..
            }) => {
                assert_eq!(page_count, 0);
                assert!(!msg.is_empty());
            }
            other => panic!("expected Typst variant with error, got {other:?}"),
        }
    }
}
