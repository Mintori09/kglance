use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::UNIX_EPOCH;

static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Return root cache directory:
/// $KGLANCE_CACHE_DIR if set and non-empty
/// $XDG_CACHE_HOME/kglance (via `dirs::cache_dir()`)
/// $HOME/.cache/kglance
/// /tmp/kglance_cache
pub fn cache_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("KGLANCE_CACHE_DIR")
        && !dir.trim().is_empty()
    {
        return PathBuf::from(dir);
    }

    if let Some(mut dir) = dirs::cache_dir() {
        dir.push("kglance");
        return dir;
    }

    dirs::home_dir()
        .map(|h| h.join(".cache").join("kglance"))
        .unwrap_or_else(|| std::env::temp_dir().join("kglance_cache"))
}

pub fn compute_cache_key(category: &str, source_path: &Path) -> Option<String> {
    let metadata = fs::metadata(source_path).ok()?;
    let mtime = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let size = metadata.len();
    let canonical = source_path
        .canonicalize()
        .unwrap_or_else(|_| source_path.to_path_buf());
    let path_str = canonical.to_string_lossy();

    let input = format!("{category}:{path_str}:{mtime}:{size}");
    Some(format!("{:x}", md5::compute(input.as_bytes())))
}

pub fn category_dir(category: &str) -> PathBuf {
    cache_dir().join(category)
}

pub fn cached_file_path(category: &str, source_path: &Path, ext: &str) -> Option<PathBuf> {
    let key = compute_cache_key(category, source_path)?;
    let file_name = if ext.is_empty() {
        key
    } else {
        format!("{key}.{ext}")
    };
    Some(category_dir(category).join(file_name))
}

/// Check if a cached artifact exists and is non-empty.
pub fn get_cached_path(category: &str, source_path: &Path, ext: &str) -> Option<PathBuf> {
    let target = cached_file_path(category, source_path, ext)?;
    if target.exists() && fs::metadata(&target).map(|m| m.len() > 0).unwrap_or(false) {
        Some(target)
    } else {
        None
    }
}

pub fn create_cached_file<F>(
    category: &str,
    source_path: &Path,
    ext: &str,
    writer: F,
) -> io::Result<PathBuf>
where
    F: FnOnce(&Path) -> io::Result<()>,
{
    let target_path = cached_file_path(category, source_path, ext).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Could not compute cache key for source file",
        )
    })?;

    if target_path.exists()
        && fs::metadata(&target_path)
            .map(|m| m.len() > 0)
            .unwrap_or(false)
    {
        return Ok(target_path);
    }

    let cat_dir = category_dir(category);
    fs::create_dir_all(&cat_dir)?;

    let unique_ctr = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let tmp_file_name = if ext.is_empty() {
        format!(".tmp_{}_{}_{}", std::process::id(), nanos, unique_ctr)
    } else {
        format!(".tmp_{}_{}_{}.{ext}", std::process::id(), nanos, unique_ctr)
    };
    let tmp_path = cat_dir.join(tmp_file_name);

    let write_result = writer(&tmp_path);
    if let Err(err) = write_result {
        let _ = fs::remove_file(&tmp_path);
        return Err(err);
    }

    if !tmp_path.exists()
        || !fs::metadata(&tmp_path)
            .map(|m| m.len() > 0)
            .unwrap_or(false)
    {
        let _ = fs::remove_file(&tmp_path);
        return Err(io::Error::other(
            "Cache generator produced an empty or missing output file",
        ));
    }

    // Atomic rename
    fs::rename(&tmp_path, &target_path)?;
    Ok(target_path)
}

pub fn put_bytes(
    category: &str,
    source_path: &Path,
    ext: &str,
    data: &[u8],
) -> io::Result<PathBuf> {
    create_cached_file(category, source_path, ext, |tmp_path| {
        fs::write(tmp_path, data)
    })
}

#[derive(Debug)]
struct DiskCacheEntry {
    path: PathBuf,
    size_bytes: u64,
    modified: std::time::SystemTime,
}

pub fn prune_to_budget(max_bytes: u64) -> io::Result<u64> {
    let base = cache_dir();
    if !base.exists() {
        return Ok(0);
    }

    let mut entries = Vec::new();
    let mut total_bytes: u64 = 0;

    collect_cache_entries(&base, &mut entries, &mut total_bytes)?;

    if total_bytes <= max_bytes {
        return Ok(total_bytes);
    }

    // Sort by modified time ascending (oldest first)
    entries.sort_by_key(|e| e.modified);

    // Target 85% of max_bytes to avoid constant pruning on every write
    let target_bytes = (max_bytes as f64 * 0.85) as u64;

    for entry in entries {
        if total_bytes <= target_bytes {
            break;
        }
        if let Ok(()) = fs::remove_file(&entry.path) {
            total_bytes = total_bytes.saturating_sub(entry.size_bytes);
        }
    }

    Ok(total_bytes)
}

fn collect_cache_entries(
    dir: &Path,
    entries: &mut Vec<DiskCacheEntry>,
    total_bytes: &mut u64,
) -> io::Result<()> {
    if let Ok(read_dir) = fs::read_dir(dir) {
        for item in read_dir.flatten() {
            let path = item.path();
            if path.is_dir() {
                let _ = collect_cache_entries(&path, entries, total_bytes);
            } else if let Ok(meta) = item.metadata() {
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default();
                if file_name.starts_with(".tmp_") {
                    let _ = fs::remove_file(&path);
                    continue;
                }

                let size = meta.len();
                let modified = meta.modified().unwrap_or(UNIX_EPOCH);
                *total_bytes = total_bytes.saturating_add(size);
                entries.push(DiskCacheEntry {
                    path,
                    size_bytes: size,
                    modified,
                });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "disk_cache_tests.rs"]
mod tests;
