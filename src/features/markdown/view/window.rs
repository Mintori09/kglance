use std::ops::Range;

const CHUNK_SIZE: usize = 6;
const OVERSCAN_CHUNKS: usize = 4;
const MIN_VIEWPORT: f32 = 600.0;

fn overscan_px(viewport_height: f32) -> f32 {
    (viewport_height.max(MIN_VIEWPORT) * 2.0).clamp(1600.0, 3200.0)
}

/// Computes a fresh range of blocks to render for a virtualized document.
///
/// `offsets` holds the estimated top Y of every block.
pub fn visible_range(
    offsets: &[f32],
    scroll_y: f32,
    viewport_height: f32,
    block_count: usize,
) -> Range<usize> {
    let overscan = overscan_px(viewport_height);
    let view_top = (scroll_y - overscan).max(0.0);
    let view_bottom = scroll_y + viewport_height + overscan;

    let raw_first = offsets.partition_point(|&y| y < view_top).saturating_sub(1);
    let raw_last = offsets
        .partition_point(|&y| y <= view_bottom)
        .min(block_count);

    let first = raw_first.saturating_sub(OVERSCAN_CHUNKS * CHUNK_SIZE) / CHUNK_SIZE * CHUNK_SIZE;
    let last = (raw_last + OVERSCAN_CHUNKS * CHUNK_SIZE)
        .min(block_count)
        .div_ceil(CHUNK_SIZE)
        * CHUNK_SIZE;
    first..last.min(block_count)
}

/// Returns `true` when `window` still covers the viewport plus a safety margin.
fn covers_viewport(
    window: &Range<usize>,
    offsets: &[f32],
    scroll_y: f32,
    viewport_height: f32,
) -> bool {
    if window.start >= window.end || window.end > offsets.len() {
        return false;
    }
    let margin = viewport_height.max(MIN_VIEWPORT);
    let top_ok = window.start == 0 || offsets[window.start] <= scroll_y - margin;
    let bottom_ok =
        window.end == offsets.len() || offsets[window.end] >= scroll_y + viewport_height + margin;
    top_ok && bottom_ok
}

/// Keeps `window` unchanged while it still covers the viewport (hysteresis),
/// and recomputes it only when the viewport nears its edge. This prevents the
/// window, and with it the spacer heights, from flipping back and forth around
/// a boundary while scrolling.
pub fn sync_window(
    window: &mut Range<usize>,
    offsets: &[f32],
    scroll_y: f32,
    viewport_height: f32,
    virtual_threshold: usize,
) {
    if offsets.len() <= virtual_threshold {
        if window.start < window.end {
            *window = 0..0;
        }
        return;
    }
    if !covers_viewport(window, offsets, scroll_y, viewport_height) {
        *window = visible_range(offsets, scroll_y, viewport_height, offsets.len());
    }
}

/// Window used by views: the stored one when valid, otherwise a fresh one.
pub fn render_range(
    window: &Range<usize>,
    offsets: &[f32],
    scroll_y: f32,
    viewport_height: f32,
) -> Range<usize> {
    if window.start >= window.end || window.end > offsets.len() {
        visible_range(offsets, scroll_y, viewport_height, offsets.len())
    } else {
        window.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offsets(n: usize) -> Vec<f32> {
        (0..n).map(|i| i as f32 * 100.0).collect()
    }

    #[test]
    fn window_does_not_flip_when_scroll_jitters() {
        let o = offsets(1000);
        let mut window = 0..0;
        sync_window(&mut window, &o, 20_000.0, 800.0, 60);
        let first = window.clone();
        for y in [20_100.0, 19_900.0, 20_200.0, 19_800.0, 20_000.0] {
            sync_window(&mut window, &o, y, 800.0, 60);
            assert_eq!(window, first);
        }
    }

    #[test]
    fn window_moves_and_covers_viewport_after_long_scroll() {
        let o = offsets(1000);
        let mut window = 0..0;
        sync_window(&mut window, &o, 20_000.0, 800.0, 60);
        sync_window(&mut window, &o, 40_000.0, 800.0, 60);
        assert!(window.start * 100 <= 40_000 && window.end * 100 >= 40_800);
    }

    #[test]
    fn small_documents_are_not_windowed() {
        let o = offsets(10);
        let mut window = 2..5;
        sync_window(&mut window, &o, 0.0, 800.0, 60);
        assert_eq!(window, 0..0);
    }
}
