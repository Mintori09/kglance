use crate::core::types::KglanceState;
use crate::features::pdf::PdfTocEntry;
use crate::features::pdf::types::PageDimensions;

pub fn populate_state(
    state: &mut KglanceState,
    page_count: usize,
    source: &str,
    error: Option<String>,
    outline: &[PdfTocEntry],
    page_dimensions: &[PageDimensions],
) {
    let old_sidebar_visible = state.typst.pdf.sidebar_visible;
    let old_sidebar_mode = state.typst.pdf.sidebar_mode;
    let old_sidebar_width = state.typst.pdf.sidebar_width;

    let win_w = if state.current_window_size.width > 0.0 {
        state.current_window_size.width
    } else {
        960.0
    };

    let effective_sidebar_w = if old_sidebar_width > 0.0 {
        old_sidebar_width
    } else {
        220.0
    };

    let sidebar_w = if old_sidebar_visible {
        effective_sidebar_w + 1.0
    } else {
        0.0
    };

    let desired_w: f32 = 800.0;
    let max_w = (win_w - sidebar_w - 40.0).clamp(300.0, 2400.0);
    let display_width = desired_w.min(max_w);

    let (offsets, ends, _, total_h) = crate::features::pdf::geometry::compute_pdf_page_offsets(
        page_dimensions,
        display_width,
        4.0,
    );

    let mut typst_pdf_state = crate::core::PdfState {
        page_count,
        pages: crate::core::types::PageCache::new(page_count),
        thumbnails: crate::core::types::ThumbnailCache::new(page_count),
        sidebar_visible: old_sidebar_visible,
        sidebar_mode: old_sidebar_mode,
        sidebar_width: effective_sidebar_w,
        outline: outline.to_vec(),
        page_dimensions: page_dimensions.to_vec(),
        display_width,
        desired_width: desired_w,
        page_y_offsets: offsets,
        page_ends: ends,
        total_content_height: total_h,
        ..Default::default()
    };

    crate::features::pdf::geometry::recalculate_pdf_thumbnail_offsets(&mut typst_pdf_state);

    state.typst = crate::core::TypstState {
        pdf: typst_pdf_state,
        source_content: iced::widget::text_editor::Content::with_text(source),
        show_source: error.is_some(),
        error,
    };
    state.file_type_text = "Typst Document".to_string();
}
