/// Viewport anchor to stabilize scroll position during resizing or when lazy measurement updates height.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportAnchor {
    /// Logical line anchored at the top of the viewport.
    pub logical_line: usize,
    /// Relative pixel offset from the top of the logical line to the top of the viewport.
    pub visual_offset_px: f32,
}

impl ViewportAnchor {
    /// Creates a new anchor.
    #[inline]
    pub const fn new(logical_line: usize, visual_offset_px: f32) -> Self {
        Self {
            logical_line,
            visual_offset_px,
        }
    }

    /// Computes the current anchor from `scroll_y` and `line_height`.
    pub fn from_scroll_y(scroll_y: f32, line_height: f32, total_lines: usize) -> Self {
        if line_height <= 0.0 || total_lines == 0 {
            return Self::new(0, 0.0);
        }

        let clamped_scroll_y = scroll_y.max(0.0);
        let raw_line = (clamped_scroll_y / line_height).floor() as usize;
        let logical_line = raw_line.min(total_lines.saturating_sub(1));
        let visual_offset_px = clamped_scroll_y - (logical_line as f32 * line_height);

        Self {
            logical_line,
            visual_offset_px: visual_offset_px.max(0.0),
        }
    }

    /// Restores the `scroll_y` coordinate from the anchor and a new `line_height`.
    #[inline]
    pub fn restore_scroll_y(&self, line_height: f32) -> f32 {
        (self.logical_line as f32 * line_height) + self.visual_offset_px
    }

    /// Restores the `scroll_y` coordinate with delta compensation from lazy measurement.
    #[inline]
    pub fn restore_with_delta(&self, line_height: f32, height_delta: f32) -> f32 {
        ((self.logical_line as f32 * line_height) + self.visual_offset_px + height_delta).max(0.0)
    }
}

impl Default for ViewportAnchor {
    fn default() -> Self {
        Self::new(0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_anchor_roundtrip() {
        let line_height = 20.0;
        let total_lines = 100;
        let original_scroll_y = 65.0; // Line 3 (0-indexed: index 3 at 60px), offset 5px

        let anchor = ViewportAnchor::from_scroll_y(original_scroll_y, line_height, total_lines);
        assert_eq!(anchor.logical_line, 3);
        assert!((anchor.visual_offset_px - 5.0).abs() < 1e-5);

        let restored_scroll_y = anchor.restore_scroll_y(line_height);
        assert!((restored_scroll_y - original_scroll_y).abs() < 1e-5);
    }

    #[test]
    fn test_viewport_anchor_with_delta() {
        let line_height = 20.0;
        let anchor = ViewportAnchor::new(5, 10.0);
        let restored = anchor.restore_with_delta(line_height, 40.0);
        assert_eq!(restored, (5.0 * 20.0) + 10.0 + 40.0);
    }
}
