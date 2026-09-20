use std::path::Path;

pub const MAX_PRELOAD_FILE_SIZE: u64 = 30 * 1024 * 1024; // 30 MB safety threshold
pub const MIN_PRELOAD_LOOKAHEAD: usize = 4;
pub const MID_PRELOAD_LOOKAHEAD: usize = 6;
pub const MAX_PRELOAD_LOOKAHEAD: usize = 10;

pub fn calculate_preload_window(
    current_index: usize,
    playlist_len: usize,
    lookahead: usize,
) -> Vec<usize> {
    if playlist_len <= 1 || lookahead == 0 {
        return vec![];
    }
    let current_index = current_index % playlist_len;
    let mut indices = Vec::with_capacity(lookahead * 2);

    for offset in 1..=lookahead {
        // Next
        let next = (current_index + offset) % playlist_len;
        if next != current_index && !indices.contains(&next) {
            indices.push(next);
        }

        // Prev
        let prev = (current_index + playlist_len - (offset % playlist_len)) % playlist_len;
        if prev != current_index && !indices.contains(&prev) {
            indices.push(prev);
        }

        if indices.len() >= playlist_len - 1 {
            break;
        }
    }

    indices
}

pub fn should_preload_file(path_str: &str) -> bool {
    if let Ok(meta) = std::fs::metadata(Path::new(path_str)) {
        meta.len() <= MAX_PRELOAD_FILE_SIZE
    } else {
        false
    }
}

#[inline]
pub fn calculate_dynamic_lookahead(current_bytes: usize, max_bytes: usize) -> usize {
    if max_bytes == 0 {
        return MIN_PRELOAD_LOOKAHEAD;
    }
    let ratio = (current_bytes as f64) / (max_bytes as f64);
    if ratio < 0.5 {
        MAX_PRELOAD_LOOKAHEAD
    } else if ratio < 0.8 {
        MID_PRELOAD_LOOKAHEAD
    } else {
        MIN_PRELOAD_LOOKAHEAD
    }
}

#[inline]
pub fn is_rapid_navigating(last_navigated_at: Option<std::time::Instant>) -> bool {
    last_navigated_at
        .map(|t| t.elapsed() < std::time::Duration::from_millis(250))
        .unwrap_or(false)
}
