use std::path::Path;

use mupdf::{Colorspace, Document, Error, Matrix};

use crate::parsers::{PageData, ParseError, ParsedContent, PreviewParser};

const RENDER_DPI: f32 = 150.0;

fn render_page(doc: &Document, page_index: i32) -> Result<PageData, Error> {
    let scale = RENDER_DPI / 72.0;
    let page = doc.load_page(page_index)?;
    let pixmap = page.to_pixmap(
        &Matrix::new_scale(scale, scale),
        &Colorspace::device_rgb(),
        false,
        true,
    )?;
    let w = pixmap.width();
    let h = pixmap.height();
    let rgb = pixmap.samples();
    let mut data = Vec::with_capacity((w * h * 4) as usize);
    for chunk in rgb.chunks(3) {
        data.push(chunk[0]);
        data.push(chunk[1]);
        data.push(chunk[2]);
        data.push(255);
    }
    Ok(PageData {
        width: w,
        height: h,
        data,
    })
}

pub fn render_pdf_page(path: &Path, page_index: u32) -> Result<PageData, ParseError> {
    let doc = Document::open(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    render_page(&doc, page_index as i32).map_err(|e| ParseError::ParseFailed(e.to_string()))
}

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
        let doc = Document::open(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        let page_count = doc
            .page_count()
            .map_err(|e| ParseError::ParseFailed(e.to_string()))? as u32;
        let first_page = if page_count > 0 {
            render_page(&doc, 0).map_err(|e| ParseError::ParseFailed(e.to_string()))?
        } else {
            PageData {
                width: 0,
                height: 0,
                data: Vec::new(),
            }
        };
        Ok(ParsedContent::Pdf {
            page_count,
            first_page,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pdf_metadata() {
        let pdf_bytes = create_simple_pdf();
        let dir = std::env::temp_dir().join("kglance-pdf-test");
        let _ = std::fs::create_dir_all(&dir);
        let file_path = dir.join("test.pdf");
        assert!(std::fs::write(&file_path, &pdf_bytes).is_ok());

        let parser = PdfParser;
        let result = parser.parse(&file_path);
        let _ = std::fs::remove_dir_all(&dir);

        match result {
            Ok(ParsedContent::Pdf {
                page_count,
                first_page,
            }) => {
                assert_eq!(page_count, 1);
                assert!(first_page.width > 0);
                assert!(first_page.height > 0);
                assert!(!first_page.data.is_empty());
            }
            Ok(_) => panic!("expected Pdf variant"),
            Err(e) => panic!("parse failed: {e}"),
        }
    }

    #[test]
    fn render_pdf_page_works() {
        let pdf_bytes = create_simple_pdf();
        let dir = std::env::temp_dir().join("kglance-pdf-render-test");
        let _ = std::fs::create_dir_all(&dir);
        let file_path = dir.join("test.pdf");
        assert!(std::fs::write(&file_path, &pdf_bytes).is_ok());

        let result = render_pdf_page(&file_path, 0);
        let _ = std::fs::remove_dir_all(&dir);

        let page = result.expect("render_pdf_page should succeed");
        assert!(page.width > 0);
        assert!(page.height > 0);
        assert!(!page.data.is_empty());
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
        let mut tmp = tempfile::Builder::new().suffix(".pdf").tempfile().unwrap();
        use std::io::Write;
        write!(tmp, "not a pdf file").unwrap();
        let parser = PdfParser;
        let result = parser.parse(tmp.path());
        assert!(result.is_err());
    }

    fn create_simple_pdf() -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"%PDF-1.4\n");
        buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
        buf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
        buf.extend_from_slice(
            b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792]
            /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>\nendobj\n",
        );
        buf.extend_from_slice(
            b"4 0 obj\n<< /Length 44 >>\nstream\nBT /F1 24 Tf 100 700 Td (Hello PDF!) Tj ET\nendstream\nendobj\n",
        );
        buf.extend_from_slice(
            b"5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n",
        );
        buf.extend_from_slice(b"xref\n0 6\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \n0000000266 00000 n \n0000000364 00000 n \ntrailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n435\n%%EOF\n");
        for b in &mut buf {
            if *b == b'\t' {
                *b = b' ';
            }
        }
        buf
    }
}
