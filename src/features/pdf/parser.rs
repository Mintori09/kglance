use std::path::Path;

use mupdf::{Colorspace, Document, Error, Matrix};

use crate::features::common::parser::traits::{ParseError, PreviewParser};
use crate::features::common::parser::types::ParsedContent;
use crate::features::pdf::types::PageData;

const TARGET_PAGE_RENDER_WIDTH: f32 = 1400.0;
const TARGET_THUMB_RENDER_WIDTH: f32 = 160.0;

fn render_page(doc: &Document, page_index: i32) -> Result<PageData, Error> {
    let page = doc.load_page(page_index)?;
    let bounds = page.bounds().unwrap_or(mupdf::Rect {
        x0: 0.0,
        y0: 0.0,
        x1: 595.0,
        y1: 842.0,
    });
    let pt_w = (bounds.x1 - bounds.x0).abs().max(1.0);
    let scale = (TARGET_PAGE_RENDER_WIDTH / pt_w).clamp(0.2, 6.0);

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

fn render_page_at_dpi(doc: &Document, page_index: i32, _dpi: f32) -> Result<PageData, Error> {
    let page = doc.load_page(page_index)?;
    let bounds = page.bounds().unwrap_or(mupdf::Rect {
        x0: 0.0,
        y0: 0.0,
        x1: 595.0,
        y1: 842.0,
    });
    let pt_w = (bounds.x1 - bounds.x0).abs().max(1.0);
    let scale = (TARGET_THUMB_RENDER_WIDTH / pt_w).clamp(0.02, 2.0);

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

unsafe extern "C" {
    fn fz_empty_store(ctx: *mut std::ffi::c_void);
}

pub fn empty_mupdf_store() {
    let ctx = mupdf::Context::get();
    let raw_ctx: *mut std::ffi::c_void = unsafe { std::mem::transmute(ctx) };
    unsafe {
        fz_empty_store(raw_ctx);
    }
}

pub fn render_pdf_pages_batch(
    path: &Path,
    pages: &[usize],
) -> Vec<(usize, Result<PageData, ParseError>)> {
    let doc = match Document::open(path) {
        Ok(d) => d,
        Err(e) => {
            return pages
                .iter()
                .map(|&p| (p, Err(ParseError::ParseFailed(e.to_string()))))
                .collect();
        }
    };
    let res: Vec<_> = pages
        .iter()
        .map(|&page_index| {
            let res = render_page(&doc, page_index as i32)
                .map_err(|e| ParseError::ParseFailed(e.to_string()));
            (page_index, res)
        })
        .collect();
    empty_mupdf_store();
    res
}

pub fn render_pdf_pages_batch_at_dpi(
    path: &Path,
    pages: &[usize],
    dpi: f32,
) -> Vec<(usize, Result<PageData, ParseError>)> {
    let doc = match Document::open(path) {
        Ok(d) => d,
        Err(e) => {
            return pages
                .iter()
                .map(|&p| (p, Err(ParseError::ParseFailed(e.to_string()))))
                .collect();
        }
    };
    let res: Vec<_> = pages
        .iter()
        .map(|&page_index| {
            let res = render_page_at_dpi(&doc, page_index as i32, dpi)
                .map_err(|e| ParseError::ParseFailed(e.to_string()));
            (page_index, res)
        })
        .collect();
    empty_mupdf_store();
    res
}

pub fn render_pdf_page_at_dpi(
    path: &Path,
    page_index: u32,
    dpi: f32,
) -> Result<PageData, ParseError> {
    let doc = Document::open(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let res = render_page_at_dpi(&doc, page_index as i32, dpi)
        .map_err(|e| ParseError::ParseFailed(e.to_string()));
    empty_mupdf_store();
    res
}

#[derive(Debug, Clone)]
pub struct PdfTocEntry {
    pub title: String,
    pub page: usize,
    pub level: u8,
}

fn extract_outline_nodes(
    nodes: &[mupdf::Outline],
    doc: &Document,
    level: u8,
    out: &mut Vec<PdfTocEntry>,
) {
    for node in nodes {
        let title = node.title.clone();
        let page = if let Some(ref uri) = node.uri {
            if let Ok(Some(dest)) = doc.resolve_link(uri) {
                dest.loc.page_number as usize
            } else if let Some(stripped) = uri.strip_prefix('#') {
                if let Ok(num) = stripped.parse::<usize>() {
                    num.saturating_sub(1)
                } else if let Some(page_str) = stripped.strip_prefix("page=") {
                    page_str.parse::<usize>().unwrap_or(1).saturating_sub(1)
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            0
        };
        if !title.is_empty() {
            out.push(PdfTocEntry { title, page, level });
        }
        if !node.down.is_empty() {
            extract_outline_nodes(&node.down, doc, level + 1, out);
        }
    }
}

pub fn extract_pdf_toc(doc: &Document) -> Vec<PdfTocEntry> {
    let mut toc = Vec::new();
    if let Ok(outlines) = doc.outlines() {
        extract_outline_nodes(&outlines, doc, 1, &mut toc);
    }
    toc
}

pub struct PdfParser;

impl PreviewParser for PdfParser {
    fn supported_extensions(&self) -> &[&str] {
        &["pdf"]
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let doc = Document::open(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        let page_count = doc
            .page_count()
            .map_err(|e| ParseError::ParseFailed(e.to_string()))? as u32;
        let outline = extract_pdf_toc(&doc);
        let page_dimensions = crate::features::pdf::dimensions::extract_page_dimensions(&doc)
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        let first_page = if page_count > 0 {
            let raw_page =
                render_page(&doc, 0).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
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
        empty_mupdf_store();
        Ok(ParsedContent::Pdf {
            page_count,
            first_page,
            outline,
            page_dimensions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pdf_metadata() {
        let pdf_bytes = create_simple_pdf();
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.pdf");
        std::fs::write(&file_path, &pdf_bytes).unwrap();

        let parser = PdfParser;
        let result = parser.parse(&file_path);

        match result {
            Ok(ParsedContent::Pdf {
                page_count,
                first_page,
                ..
            }) => {
                assert_eq!(page_count, 1);
                assert!(first_page.width > 0);
            }
            _ => panic!("Expected Pdf variant"),
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
