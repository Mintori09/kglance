use crate::features::pdf::types::PageDimensions;
use mupdf::{Document, Error};

/// Extract dimensions (in PDF points) for every page without rendering.
/// This is a metadata-only pass — no pixmaps are created.
pub fn extract_page_dimensions(doc: &Document) -> Result<Vec<PageDimensions>, Error> {
    let count = doc.page_count()?;
    let mut dims = Vec::with_capacity(count as usize);
    for i in 0..count {
        let page = doc.load_page(i)?;
        let bounds = page.bounds()?;
        let w = (bounds.x1 - bounds.x0).abs();
        let h = (bounds.y1 - bounds.y0).abs();
        dims.push(PageDimensions {
            width_pts: if w > 0.0 { w } else { 595.0 },
            height_pts: if h > 0.0 { h } else { 842.0 },
        });
    }
    crate::features::pdf::parser::empty_mupdf_store();
    Ok(dims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_dimensions_aspect_ratio() {
        let landscape = PageDimensions {
            width_pts: 842.0,
            height_pts: 595.0,
        };
        assert!((landscape.aspect_ratio() - 1.415).abs() < 0.01);

        let portrait = PageDimensions {
            width_pts: 595.0,
            height_pts: 842.0,
        };
        assert!((portrait.aspect_ratio() - 0.707).abs() < 0.01);
    }

    #[test]
    fn display_height_matches_aspect() {
        let dims = PageDimensions {
            width_pts: 612.0,
            height_pts: 792.0,
        };
        let display_w = 800.0;
        let display_h = dims.display_height(display_w);
        let expected = 800.0 / (612.0 / 792.0);
        assert!((display_h - expected).abs() < 0.1);
    }

    #[test]
    fn zero_height_page_returns_unit_ratio() {
        let dims = PageDimensions {
            width_pts: 100.0,
            height_pts: 0.0,
        };
        assert_eq!(dims.aspect_ratio(), 1.0);
    }
}
