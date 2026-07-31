use crate::core::types::GridThumbnail;
use std::path::{Path, PathBuf};

pub fn get_freedesktop_thumbnail_path(file_path: &str) -> Option<PathBuf> {
    let canonical_path = Path::new(file_path).canonicalize().ok()?;
    let uri = format!("file://{}", canonical_path.display());
    let hex_hash = format!("{:x}.png", md5::compute(uri.as_bytes()));

    let home_dir = std::env::var("HOME").ok()?;
    let user_home = PathBuf::from(home_dir);

    let large_thumbnail = user_home.join(".cache/thumbnails/large").join(&hex_hash);
    if large_thumbnail.exists() {
        return Some(large_thumbnail);
    }

    let normal_thumbnail = user_home.join(".cache/thumbnails/normal").join(&hex_hash);
    if normal_thumbnail.exists() {
        return Some(normal_thumbnail);
    }

    None
}

pub fn filter_thumbnails<'a>(
    thumbnails: &'a [GridThumbnail],
    search_query: &str,
) -> Vec<(usize, &'a GridThumbnail)> {
    if search_query.is_empty() {
        thumbnails.iter().enumerate().collect()
    } else {
        let query_lowercase = search_query.to_lowercase();
        thumbnails
            .iter()
            .enumerate()
            .filter(|(_, item)| item.name.to_lowercase().contains(&query_lowercase))
            .collect()
    }
}
