use crate::features::pdf::types::PageDimensions;
use std::ops::RangeInclusive;

#[derive(Debug, Clone, PartialEq)]
pub struct VirtualizedLayout {
    pub first_render: usize,
    pub last_render: usize,
    pub top_spacer_height: f32,
    pub bottom_spacer_height: f32,
}

/// Compute content-space Y offsets starting at 0.0 without container padding.
/// Guarantees that `offsets` and `ends` are finite and strictly monotonic increasing.
/// Returns `(offsets, ends, heights, total_content_height)`.
pub fn compute_pdf_page_offsets(
    dims: &[PageDimensions],
    display_width: f32,
    spacing: f32,
) -> (Vec<f32>, Vec<f32>, Vec<f32>, f32) {
    if dims.is_empty() {
        return (Vec::new(), Vec::new(), Vec::new(), 0.0);
    }
    let mut offsets = Vec::with_capacity(dims.len());
    let mut ends = Vec::with_capacity(dims.len());
    let mut heights = Vec::with_capacity(dims.len());
    let mut y = 0.0;

    for (i, dim) in dims.iter().enumerate() {
        offsets.push(y);
        let page_h = dim.display_height(display_width).max(1.0);
        heights.push(page_h);
        y += page_h;
        ends.push(y);
        if i + 1 < dims.len() {
            y += spacing.max(0.0);
        }
    }
    let total_h = y;
    (offsets, ends, heights, total_h)
}

/// Pure O(log N) binary search for strictly visible page range:
/// Page `i` is visible if `ends[i] > vp_top` AND `offsets[i] < vp_bottom`.
/// Precondition: `offsets` and `ends` are strictly monotonic increasing.
pub fn visible_page_range(
    offsets: &[f32],
    ends: &[f32],
    scroll_y: f32,
    viewport_height: f32,
) -> Option<RangeInclusive<usize>> {
    if offsets.is_empty() || ends.is_empty() || offsets.len() != ends.len() {
        return None;
    }

    let vp_top = scroll_y;
    let vp_bottom = scroll_y + viewport_height;

    // Binary search: first page where ends[i] > vp_top
    let first = ends.partition_point(|&bottom| bottom <= vp_top);
    if first >= offsets.len() {
        return None;
    }

    // Binary search: first page where offsets[i] >= vp_bottom
    let last_candidate = offsets.partition_point(|&top| top < vp_bottom);
    if last_candidate == 0 {
        return None;
    }
    let last = last_candidate - 1;

    if first > last {
        return None;
    }

    Some(first..=last)
}

/// Expands a strictly visible range by `buffer_pages` on both sides.
pub fn buffered_page_range(
    visible: Option<RangeInclusive<usize>>,
    total_pages: usize,
    buffer_pages: usize,
) -> Option<RangeInclusive<usize>> {
    let range = visible?;
    if total_pages == 0 {
        return None;
    }
    let start = range.start().saturating_sub(buffer_pages);
    let end = (range.end() + buffer_pages).min(total_pages.saturating_sub(1));
    Some(start..=end)
}

/// Computes exact top and bottom spacer heights for a rendered page range.
pub fn calculate_virtualized_layout(
    offsets: &[f32],
    total_content_height: f32,
    spacing: f32,
    render_range: RangeInclusive<usize>,
) -> VirtualizedLayout {
    let total_pages = offsets.len();
    if total_pages == 0 {
        return VirtualizedLayout {
            first_render: 0,
            last_render: 0,
            top_spacer_height: 0.0,
            bottom_spacer_height: 0.0,
        };
    }

    let first = *render_range.start();
    let last = (*render_range.end()).min(total_pages.saturating_sub(1));

    let top_spacer_height = if first > 0 {
        (offsets[first] - spacing).max(0.0)
    } else {
        0.0
    };

    let bottom_spacer_height = if last + 1 < total_pages {
        (total_content_height - offsets[last + 1]).max(0.0)
    } else {
        0.0
    };

    VirtualizedLayout {
        first_render: first,
        last_render: last,
        top_spacer_height,
        bottom_spacer_height,
    }
}

pub const THUMBNAIL_SPACING: f32 = 10.0;

pub fn compute_thumbnail_offsets(
    dims: &[PageDimensions],
    thumb_width: f32,
    spacing: f32,
) -> (Vec<f32>, Vec<f32>, Vec<f32>, f32) {
    if dims.is_empty() {
        return (Vec::new(), Vec::new(), Vec::new(), 0.0);
    }
    let mut offsets = Vec::with_capacity(dims.len());
    let mut ends = Vec::with_capacity(dims.len());
    let mut heights = Vec::with_capacity(dims.len());
    let mut y = 0.0;

    for (i, dim) in dims.iter().enumerate() {
        offsets.push(y);
        let ar = dim.aspect_ratio();
        let item_h = (thumb_width / ar).max(20.0);
        heights.push(item_h);
        y += item_h;
        ends.push(y);
        if i + 1 < dims.len() {
            y += spacing.max(0.0);
        }
    }
    let total_h = y;
    (offsets, ends, heights, total_h)
}

pub fn recalculate_pdf_thumbnail_offsets(pdf_state: &mut crate::core::PdfState) {
    if pdf_state.page_dimensions.is_empty() {
        pdf_state.thumbnail_y_offsets.clear();
        pdf_state.thumbnail_ends.clear();
        pdf_state.total_thumbnail_height = 0.0;
        return;
    }
    let thumb_width = (pdf_state.sidebar_width - 24.0).clamp(100.0, 360.0);
    let (offsets, ends, _, total_h) =
        compute_thumbnail_offsets(&pdf_state.page_dimensions, thumb_width, THUMBNAIL_SPACING);
    pdf_state.thumbnail_y_offsets = offsets;
    pdf_state.thumbnail_ends = ends;
    pdf_state.total_thumbnail_height = total_h;
}

/// Finds the primary visible thumbnail page index for a given scroll Y and viewport height.
pub fn find_visible_thumbnail_page(offsets: &[f32], scroll_y: f32, viewport_height: f32) -> usize {
    if offsets.is_empty() {
        return 0;
    }
    let target_y = scroll_y + viewport_height * 0.3;
    match offsets.binary_search_by(|&probe| {
        if probe <= target_y {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    }) {
        Ok(idx) => idx.min(offsets.len() - 1),
        Err(idx) => idx.saturating_sub(1).min(offsets.len() - 1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::pdf::types::PageDimensions;

    #[test]
    fn test_compute_thumbnail_offsets() {
        let dims = vec![
            PageDimensions {
                width_pts: 100.0,
                height_pts: 200.0,
            },
            PageDimensions {
                width_pts: 200.0,
                height_pts: 200.0,
            },
        ];
        let (offsets, ends, heights, total_h) = compute_thumbnail_offsets(&dims, 100.0, 10.0);
        assert_eq!(offsets.len(), 2);
        assert_eq!(offsets[0], 0.0);
        assert_eq!(heights[0], 200.0);
        assert_eq!(ends[0], 200.0);
        assert_eq!(offsets[1], 210.0);
        assert_eq!(heights[1], 100.0);
        assert_eq!(ends[1], 310.0);
        assert_eq!(total_h, 310.0);
    }

    #[test]
    fn test_recalculate_pdf_thumbnail_offsets() {
        let mut pdf_state = crate::core::PdfState {
            sidebar_width: 224.0, // thumb_width = 200.0
            page_dimensions: vec![
                PageDimensions {
                    width_pts: 100.0,
                    height_pts: 100.0,
                },
                PageDimensions {
                    width_pts: 100.0,
                    height_pts: 100.0,
                },
            ],
            ..Default::default()
        };

        recalculate_pdf_thumbnail_offsets(&mut pdf_state);
        assert_eq!(pdf_state.thumbnail_y_offsets.len(), 2);
        assert_eq!(pdf_state.thumbnail_y_offsets[0], 0.0);
        assert_eq!(pdf_state.thumbnail_y_offsets[1], 210.0); // 200 + 10
        assert_eq!(pdf_state.total_thumbnail_height, 410.0);
    }

    #[test]
    fn test_find_visible_thumbnail_page() {
        let offsets = vec![0.0, 200.0, 400.0, 600.0];
        assert_eq!(find_visible_thumbnail_page(&offsets, 0.0, 300.0), 0);
        assert_eq!(find_visible_thumbnail_page(&offsets, 150.0, 300.0), 1);
        assert_eq!(find_visible_thumbnail_page(&offsets, 550.0, 300.0), 3);
    }
}
