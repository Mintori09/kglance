/// Performance and memory limit policy for large source code files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PerformancePolicy {
    /// Maximum memory allocated for syntax token cache (MB).
    pub max_syntax_cache_mb: usize,
    /// Maximum memory allocated for display layout measurement cache (MB).
    pub max_measure_cache_mb: usize,
    /// Maximum number of visible rows in a viewport.
    pub max_visible_rows: usize,
    /// Maximum number of concurrent background worker jobs.
    pub max_background_jobs: usize,
    /// Large file threshold to apply strict virtualization (Bytes).
    pub large_file_threshold_bytes: usize,
}

impl PerformancePolicy {
    /// Creates default configuration optimized for KDE Plasma 6.
    pub const fn default_policy() -> Self {
        Self {
            max_syntax_cache_mb: 32,
            max_measure_cache_mb: 16,
            max_visible_rows: 200,
            max_background_jobs: 3,
            large_file_threshold_bytes: 10 * 1024 * 1024, // 10 MB
        }
    }

    /// Checks whether the file exceeds the large file threshold.
    #[inline]
    pub const fn is_large_file(&self, size_bytes: usize) -> bool {
        size_bytes >= self.large_file_threshold_bytes
    }

    /// Returns the recommended checkpoint interval based on total line count.
    #[inline]
    pub const fn recommended_checkpoint_interval(&self, total_lines: usize) -> usize {
        if total_lines > 100_000 {
            512
        } else if total_lines > 20_000 {
            256
        } else {
            128
        }
    }
}

impl Default for PerformancePolicy {
    fn default() -> Self {
        Self::default_policy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_policy_defaults() {
        let policy = PerformancePolicy::default();
        assert_eq!(policy.max_syntax_cache_mb, 32);
        assert!(policy.is_large_file(15 * 1024 * 1024));
        assert!(!policy.is_large_file(2 * 1024 * 1024));
        assert_eq!(policy.recommended_checkpoint_interval(5_000), 128);
        assert_eq!(policy.recommended_checkpoint_interval(50_000), 256);
        assert_eq!(policy.recommended_checkpoint_interval(200_000), 512);
    }
}
