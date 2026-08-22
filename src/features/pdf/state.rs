use crate::core::PdfState;
use crate::core::types::{KglanceState, PageCache, ThumbnailCache};
use crate::features::pdf::PdfTocEntry;
use crate::features::pdf::geometry::{compute_pdf_page_offsets, recalculate_pdf_thumbnail_offsets};
use crate::features::pdf::types::PageDimensions;

pub fn populate_state(
    state: &mut KglanceState,
    page_count: usize,
    outline: Vec<PdfTocEntry>,
    page_dimensions: Vec<PageDimensions>,
) {
    let old_sidebar_visible = state.pdf.sidebar_visible;
    let old_sidebar_mode = state.pdf.sidebar_mode;
    let old_sidebar_width = state.pdf.sidebar_width;

    state.pdf = PdfState::default();
    state.pdf.page_count = page_count;
    state.pdf.pages = PageCache::new(page_count);
    state.pdf.thumbnails = ThumbnailCache::new(page_count);
    state.pdf.sidebar_visible = old_sidebar_visible;
    state.pdf.sidebar_mode = old_sidebar_mode;
    state.pdf.sidebar_width = if old_sidebar_width > 0.0 {
        old_sidebar_width
    } else {
        220.0
    };
    state.pdf.outline = outline;
    state.pdf.page_dimensions = page_dimensions.clone();

    let win_w = if state.current_window_size.width > 0.0 {
        state.current_window_size.width
    } else {
        960.0
    };

    let sidebar_w = if state.pdf.sidebar_visible {
        state.pdf.sidebar_width + 1.0
    } else {
        0.0
    };

    let desired_w: f32 = 800.0;
    let max_w = (win_w - sidebar_w - 40.0).clamp(300.0, 2400.0);
    let display_width = desired_w.min(max_w);

    let (offsets, ends, _, total_h) =
        compute_pdf_page_offsets(&page_dimensions, display_width, 4.0);

    state.pdf.desired_width = desired_w;
    state.pdf.display_width = display_width;
    state.pdf.page_y_offsets = offsets;
    state.pdf.page_ends = ends;
    state.pdf.total_content_height = total_h;

    recalculate_pdf_thumbnail_offsets(&mut state.pdf);
    state.file_type_text = "PDF Document".to_string();
}
