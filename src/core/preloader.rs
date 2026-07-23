use std::path::Path;

pub const MAX_PRELOAD_FILE_SIZE: u64 = 20 * 1024 * 1024; // 20 MB safety threshold

pub fn calculate_preload_window(current_index: usize, playlist_len: usize) -> Vec<usize> {
    if playlist_len <= 1 {
        return vec![];
    }
    let mut indices = Vec::new();
    // Prev 1
    let prev = if current_index == 0 {
        playlist_len - 1
    } else {
        current_index - 1
    };
    indices.push(prev);

    // Next 3
    for offset in 1..=3 {
        let next = (current_index + offset) % playlist_len;
        if next != current_index && !indices.contains(&next) {
            indices.push(next);
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
