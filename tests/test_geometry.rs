use kglance::features::pdf::geometry::{
    buffered_page_range, calculate_virtualized_layout, compute_pdf_page_offsets,
    visible_page_range,
};
use kglance::features::pdf::types::PageDimensions;

fn visible_page_range_reference(
    offsets: &[f32],
    ends: &[f32],
    scroll_y: f32,
    viewport_height: f32,
) -> Option<std::ops::RangeInclusive<usize>> {
    let vp_top = scroll_y;
    let vp_bottom = scroll_y + viewport_height;

    let mut first = None;
    let mut last = None;

    for (i, (&top, &bottom)) in offsets.iter().zip(ends.iter()).enumerate() {
        if bottom > vp_top && top < vp_bottom {
            if first.is_none() {
                first = Some(i);
            }
            last = Some(i);
        }
    }

    match (first, last) {
        (Some(f), Some(l)) => Some(f..=l),
        _ => None,
    }
}

#[test]
fn test_visible_page_range_differential_against_reference_10k() {
    let mut rng_state: u64 = 12345;
    let mut pseudo_rand = || {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (rng_state >> 33) as f32 / 2147483648.0
    };

    for _ in 0..10_000 {
        let page_count = (pseudo_rand() * 150.0) as usize + 1;
        let mut dims = Vec::with_capacity(page_count);
        for _ in 0..page_count {
            dims.push(PageDimensions {
                width_pts: 600.0,
                height_pts: 400.0 + pseudo_rand() * 800.0,
            });
        }
        let spacing = 4.0;
        let display_w = 800.0;
        let (offsets, ends, _, total_h) = compute_pdf_page_offsets(&dims, display_w, spacing);

        let scroll_y = (pseudo_rand() * (total_h + 500.0)) - 200.0;
        let viewport_h = 200.0 + pseudo_rand() * 1200.0;

        let opt_range = visible_page_range(&offsets, &ends, scroll_y, viewport_h);
        let ref_range = visible_page_range_reference(&offsets, &ends, scroll_y, viewport_h);

        assert_eq!(
            opt_range, ref_range,
            "Mismatch at scroll_y={scroll_y}, vp_h={viewport_h}, page_count={page_count}"
        );
    }
}

#[test]
fn test_child_sequence_height_parity_randomized_10k() {
    let mut rng_state: u64 = 67890;
    let mut pseudo_rand = || {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (rng_state >> 33) as f32 / 2147483648.0
    };

    for _ in 0..10_000 {
        let page_count = (pseudo_rand() * 100.0) as usize + 1;
        let mut dims = Vec::with_capacity(page_count);
        for _ in 0..page_count {
            dims.push(PageDimensions {
                width_pts: 600.0,
                height_pts: 300.0 + pseudo_rand() * 700.0,
            });
        }
        let spacing = 4.0;
        let display_w = 800.0;
        let (offsets, _, heights, total_h) = compute_pdf_page_offsets(&dims, display_w, spacing);

        let start = (pseudo_rand() * page_count as f32) as usize;
        let end = (start + (pseudo_rand() * 10.0) as usize).min(page_count - 1);
        let render_range = start..=end;

        let layout = calculate_virtualized_layout(&offsets, total_h, spacing, render_range.clone());

        let mut child_heights: Vec<f32> = Vec::new();
        if layout.top_spacer_height > 0.0 {
            child_heights.push(layout.top_spacer_height);
        }
        for i in *render_range.start()..=*render_range.end() {
            child_heights.push(heights[i]);
        }
        if layout.bottom_spacer_height > 0.0 {
            child_heights.push(layout.bottom_spacer_height);
        }

        let child_count = child_heights.len();
        let total_spacing = if child_count > 0 {
            (child_count - 1) as f32 * spacing
        } else {
            0.0
        };
        let reconstructed_h: f32 = child_heights.iter().sum::<f32>() + total_spacing;

        assert!(
            (reconstructed_h - total_h).abs() < 0.01,
            "Child sequence parity failed: reconstructed={reconstructed_h}, total={total_h}"
        );
    }
}

#[test]
fn test_buffered_page_range_expansion() {
    let visible = Some(5..=10);
    let total_pages = 20;

    let buffered = buffered_page_range(visible, total_pages, 2);
    assert_eq!(buffered, Some(3..=12));

    // Clamp at start
    let near_start = buffered_page_range(Some(1..=4), total_pages, 2);
    assert_eq!(near_start, Some(0..=6));

    // Clamp at end
    let near_end = buffered_page_range(Some(18..=19), total_pages, 2);
    assert_eq!(near_end, Some(16..=19));

    // None visible
    assert_eq!(buffered_page_range(None, total_pages, 2), None);
}
