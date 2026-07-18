use std::path::Path;

use crate::parser::{ParseError, ParsedContent, PreviewParser};

pub struct PdfParser;

impl PreviewParser for PdfParser {
    fn supported_extensions(&self) -> &[&str] {
        &["pdf"]
    }

    fn is_supported(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e == "pdf")
            .unwrap_or(false)
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let data =
            std::fs::read(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        let doc = lopdf::Document::load_mem(&data)
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        let page_count = doc.get_pages().len() as u32;

        Ok(ParsedContent::Pdf {
            page_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pdf_metadata() {
        let mut tmp = tempfile::Builder::new()
            .suffix(".pdf")
            .tempfile()
            .unwrap();
        use std::io::Write;
        writeln!(tmp, "%PDF-1.4").unwrap();
        writeln!(tmp, "1 0 obj").unwrap();
        writeln!(tmp, "<< /Type /Catalog /Pages 2 0 R >>").unwrap();
        writeln!(tmp, "endobj").unwrap();
        writeln!(tmp, "2 0 obj").unwrap();
        writeln!(tmp, "<< /Type /Pages /Kids [3 0 R] /Count 1 >>").unwrap();
        writeln!(tmp, "endobj").unwrap();
        writeln!(tmp, "3 0 obj").unwrap();
        writeln!(tmp, "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>").unwrap();
        writeln!(tmp, "endobj").unwrap();
        writeln!(tmp, "xref").unwrap();
        writeln!(tmp, "0 4").unwrap();
        writeln!(tmp, "0000000000 65535 f ").unwrap();
        writeln!(tmp, "0000000009 00000 n ").unwrap();
        writeln!(tmp, "0000000058 00000 n ").unwrap();
        writeln!(tmp, "0000000115 00000 n ").unwrap();
        writeln!(tmp, "trailer").unwrap();
        writeln!(tmp, "<< /Size 4 /Root 1 0 R >>").unwrap();
        writeln!(tmp, "startxref").unwrap();
        // xref starts at byte 186
        writeln!(tmp, "186").unwrap();
        writeln!(tmp, "%%EOF").unwrap();
        let parser = PdfParser;
        let result = parser.parse(tmp.path()).unwrap();
        match result {
            ParsedContent::Pdf {
                page_count,
            } => {
                assert_eq!(page_count, 1);
            }
            _ => panic!("expected Pdf variant"),
        }
    }

    #[test]
    fn supports_pdf_extension() {
        let parser = PdfParser;
        assert!(parser.is_supported(Path::new("doc.pdf")));
        assert!(!parser.is_supported(Path::new("doc.txt")));
        assert!(!parser.is_supported(Path::new("file.png")));
    }

    #[test]
    fn returns_error_for_invalid_pdf() {
        let mut tmp = tempfile::Builder::new()
            .suffix(".pdf")
            .tempfile()
            .unwrap();
        use std::io::Write;
        write!(tmp, "not a pdf file").unwrap();
        let parser = PdfParser;
        let result = parser.parse(tmp.path());
        assert!(result.is_err());
    }
}
