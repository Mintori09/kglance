use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Condvar, Mutex, OnceLock, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);
static PRUNER_SIGNAL: (Mutex<bool>, Condvar) = (Mutex::new(false), Condvar::new());
static GLOBAL_DISK_CACHE: OnceLock<DiskCache> = OnceLock::new();

pub const INDEX_FILE_NAME: &str = "cache_index.json";

/// Metadata record for a single persistent disk cache entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiskCacheEntryMeta {
    pub category: String,
    pub source_path: PathBuf,
    pub cached_path: PathBuf,
    pub size_bytes: u64,
    pub last_accessed_nanos: u128,
}

/// Central database index containing all cached entries and total size accounting.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DiskCacheIndex {
    #[serde(skip)]
    pub cache_root: PathBuf,
    pub entries: HashMap<String, DiskCacheEntryMeta>,
    pub total_bytes: u64,
}

fn is_dir_writable(dir: &Path) -> bool {
    if fs::create_dir_all(dir).is_err() {
        return false;
    }
    let probe = dir.join(format!(
        ".probe_{}_{}",
        std::process::id(),
        current_timestamp_nanos()
    ));
    if fs::write(&probe, b"").is_err() {
        return false;
    }
    let _ = fs::remove_file(&probe);
    true
}

/// Return root cache directory:
/// 1. $KGLANCE_CACHE_DIR if set, non-empty, and writable
/// 2. $XDG_CACHE_HOME/kglance (via `dirs::cache_dir()`) if writable
/// 3. $HOME/.cache/kglance if writable
/// 4. /tmp/kglance_cache or /tmp/kglance_cache_<pid> fallback
pub fn cache_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("KGLANCE_CACHE_DIR")
        && !dir.trim().is_empty()
    {
        let path = PathBuf::from(dir);
        if is_dir_writable(&path) {
            return path;
        }
    }

    if let Some(mut dir) = dirs::cache_dir() {
        dir.push("kglance");
        if is_dir_writable(&dir) {
            return dir;
        }
    }

    if let Some(mut dir) = dirs::home_dir() {
        dir.push(".cache");
        dir.push("kglance");
        if is_dir_writable(&dir) {
            return dir;
        }
    }

    let temp_cache = std::env::temp_dir().join("kglance_cache");
    if is_dir_writable(&temp_cache) {
        return temp_cache;
    }

    let fallback = std::env::temp_dir().join(format!("kglance_cache_{}", std::process::id()));
    let _ = fs::create_dir_all(&fallback);
    fallback
}

fn current_timestamp_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

/// Persistent disk cache manager with index tracking, LRU eviction, and atomic writes.
#[derive(Debug)]
pub struct DiskCache {
    root: PathBuf,
    index: RwLock<DiskCacheIndex>,
}

impl DiskCache {
    /// Create or load a disk cache at the specified root path.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let index = load_or_reconcile_index_from_disk(&root);
        Self {
            root,
            index: RwLock::new(index),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn category_dir(&self, category: &str) -> PathBuf {
        self.root.join(category)
    }

    pub fn index_file_path(&self) -> PathBuf {
        self.root.join(INDEX_FILE_NAME)
    }

    pub fn cached_file_path(
        &self,
        category: &str,
        source_path: &Path,
        ext: &str,
    ) -> Option<PathBuf> {
        let key = compute_cache_key(category, source_path)?;
        let file_name = if ext.is_empty() {
            key
        } else {
            format!("{key}.{ext}")
        };
        Some(self.category_dir(category).join(file_name))
    }

    pub fn get_cached_path(
        &self,
        category: &str,
        source_path: &Path,
        ext: &str,
    ) -> Option<PathBuf> {
        let key = compute_cache_key(category, source_path)?;
        let mut guard = self
            .index
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if let Some(entry) = guard.entries.get_mut(&key)
            && entry.cached_path.exists()
            && fs::metadata(&entry.cached_path)
                .map(|m| m.len() > 0)
                .unwrap_or(false)
        {
            entry.last_accessed_nanos = current_timestamp_nanos();
            return Some(entry.cached_path.clone());
        }

        // Fallback: check filesystem directly
        let file_name = if ext.is_empty() {
            key.clone()
        } else {
            format!("{key}.{ext}")
        };
        let target = self.category_dir(category).join(file_name);
        if target.exists() && fs::metadata(&target).map(|m| m.len() > 0).unwrap_or(false) {
            let size = fs::metadata(&target).map(|m| m.len()).unwrap_or(0);
            guard.entries.insert(
                key,
                DiskCacheEntryMeta {
                    category: category.to_string(),
                    source_path: source_path.to_path_buf(),
                    cached_path: target.clone(),
                    size_bytes: size,
                    last_accessed_nanos: current_timestamp_nanos(),
                },
            );
            guard.total_bytes = guard.total_bytes.saturating_add(size);
            let _ = save_index_to_disk(&guard, &self.root);
            Some(target)
        } else {
            None
        }
    }

    pub fn create_cached_file<F>(
        &self,
        category: &str,
        source_path: &Path,
        ext: &str,
        writer: F,
    ) -> io::Result<PathBuf>
    where
        F: FnOnce(&Path) -> io::Result<()>,
    {
        let key = compute_cache_key(category, source_path).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "Could not compute cache key for source file",
            )
        })?;

        let file_name = if ext.is_empty() {
            key.clone()
        } else {
            format!("{key}.{ext}")
        };
        let cat_dir = self.category_dir(category);
        let target_path = cat_dir.join(file_name);

        if target_path.exists()
            && fs::metadata(&target_path)
                .map(|m| m.len() > 0)
                .unwrap_or(false)
        {
            return Ok(target_path);
        }

        fs::create_dir_all(&cat_dir)?;

        let unique_ctr = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let nanos = current_timestamp_nanos();
        let tmp_file_name = if ext.is_empty() {
            format!(".tmp_{pid}_{nanos}_{unique_ctr}")
        } else {
            format!(".tmp_{pid}_{nanos}_{unique_ctr}.{ext}")
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

        // Register entry in cache index
        let size = fs::metadata(&target_path).map(|m| m.len()).unwrap_or(0);
        {
            let mut guard = self
                .index
                .write()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let old_size = guard
                .entries
                .insert(
                    key,
                    DiskCacheEntryMeta {
                        category: category.to_string(),
                        source_path: source_path.to_path_buf(),
                        cached_path: target_path.clone(),
                        size_bytes: size,
                        last_accessed_nanos: current_timestamp_nanos(),
                    },
                )
                .map(|e| e.size_bytes)
                .unwrap_or(0);

            guard.total_bytes = guard
                .total_bytes
                .saturating_sub(old_size)
                .saturating_add(size);
            let _ = save_index_to_disk(&guard, &self.root);
        }

        notify_cache_changed();
        Ok(target_path)
    }

    pub fn put_bytes(
        &self,
        category: &str,
        source_path: &Path,
        ext: &str,
        data: &[u8],
    ) -> io::Result<PathBuf> {
        self.create_cached_file(category, source_path, ext, |tmp_path| {
            fs::write(tmp_path, data)
        })
    }

    pub fn prune_to_budget(&self, max_bytes: u64) -> io::Result<u64> {
        let mut guard = self
            .index
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let threshold_80 = (max_bytes as f64 * 0.80) as u64;

        if guard.total_bytes <= threshold_80 {
            return Ok(guard.total_bytes);
        }

        // Stage 1: Remove orphaned cache entries whose source file was deleted
        let mut orphaned_keys = Vec::new();
        for (key, entry) in guard.entries.iter() {
            if !entry.source_path.exists() {
                orphaned_keys.push(key.clone());
            }
        }

        for key in orphaned_keys {
            if let Some(entry) = guard.entries.remove(&key) {
                let _ = fs::remove_file(&entry.cached_path);
                guard.total_bytes = guard.total_bytes.saturating_sub(entry.size_bytes);
            }
        }

        if guard.total_bytes <= threshold_80 {
            let _ = save_index_to_disk(&guard, &self.root);
            return Ok(guard.total_bytes);
        }

        // Stage 2: Prune oldest LRU entries to 70% budget target
        let target_70 = (max_bytes as f64 * 0.70) as u64;
        let mut sorted_keys: Vec<(String, u128, u64, PathBuf)> = guard
            .entries
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    v.last_accessed_nanos,
                    v.size_bytes,
                    v.cached_path.clone(),
                )
            })
            .collect();

        // Sort ascending by last_accessed_nanos (oldest first)
        sorted_keys.sort_by_key(|item| item.1);

        for (key, _nanos, size, cached_path) in sorted_keys {
            if guard.total_bytes <= target_70 {
                break;
            }
            let _ = fs::remove_file(&cached_path);
            guard.entries.remove(&key);
            guard.total_bytes = guard.total_bytes.saturating_sub(size);
        }

        let _ = save_index_to_disk(&guard, &self.root);
        Ok(guard.total_bytes)
    }

    pub fn reconcile(&self, clean_unindexed: bool) {
        let mut guard = self
            .index
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        reconcile_index_with_disk(&mut guard, &self.root, clean_unindexed);
        let _ = save_index_to_disk(&guard, &self.root);
    }

    pub fn inspect_index<R>(&self, f: impl FnOnce(&DiskCacheIndex) -> R) -> R {
        let guard = self
            .index
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        f(&guard)
    }

    pub fn inspect_index_mut<R>(&self, f: impl FnOnce(&mut DiskCacheIndex) -> R) -> R {
        let mut guard = self
            .index
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        f(&mut guard)
    }
}

pub fn global_cache() -> &'static DiskCache {
    GLOBAL_DISK_CACHE.get_or_init(|| {
        let root = cache_dir();
        DiskCache::new(root)
    })
}

pub fn category_dir(category: &str) -> PathBuf {
    global_cache().category_dir(category)
}

pub fn index_file_path() -> PathBuf {
    global_cache().index_file_path()
}

pub fn cached_file_path(category: &str, source_path: &Path, ext: &str) -> Option<PathBuf> {
    global_cache().cached_file_path(category, source_path, ext)
}

pub fn get_cached_path(category: &str, source_path: &Path, ext: &str) -> Option<PathBuf> {
    global_cache().get_cached_path(category, source_path, ext)
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
    global_cache().create_cached_file(category, source_path, ext, writer)
}

pub fn put_bytes(
    category: &str,
    source_path: &Path,
    ext: &str,
    data: &[u8],
) -> io::Result<PathBuf> {
    global_cache().put_bytes(category, source_path, ext, data)
}

pub fn prune_to_budget(max_bytes: u64) -> io::Result<u64> {
    global_cache().prune_to_budget(max_bytes)
}

/// Load index from disk or create a fresh one, then perform 2-way reconciliation.
pub fn load_or_reconcile_index_from_disk(root: &Path) -> DiskCacheIndex {
    let index_path = root.join(INDEX_FILE_NAME);
    let index_existed = index_path.exists();
    let mut index: DiskCacheIndex = if index_existed {
        fs::read_to_string(&index_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        DiskCacheIndex::default()
    };
    index.cache_root = root.to_path_buf();

    reconcile_index_with_disk(&mut index, root, index_existed);
    let _ = save_index_to_disk(&index, root);
    index
}

fn save_index_to_disk(index: &DiskCacheIndex, root: &Path) -> io::Result<()> {
    if !root.exists() {
        fs::create_dir_all(root)?;
    }
    let target_path = root.join(INDEX_FILE_NAME);
    let serialized =
        serde_json::to_string(index).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let unique_ctr = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    let nanos = current_timestamp_nanos();
    let tmp_path = root.join(format!(".tmp_index_{pid}_{nanos}_{unique_ctr}"));

    fs::write(&tmp_path, serialized.as_bytes())?;
    fs::rename(&tmp_path, &target_path)?;
    Ok(())
}

pub fn reconcile_index_with_disk(index: &mut DiskCacheIndex, root: &Path, clean_unindexed: bool) {
    if !root.exists() {
        index.entries.clear();
        index.total_bytes = 0;
        return;
    }

    // Step 1: Index -> Disk validation
    let mut keys_to_remove = Vec::new();
    let mut active_cached_paths = HashSet::new();

    for (key, entry) in index.entries.iter_mut() {
        // If cached artifact is missing on disk
        if !entry.cached_path.exists() {
            keys_to_remove.push(key.clone());
            continue;
        }

        // If source file no longer exists (orphaned)
        if !entry.source_path.exists() {
            let _ = fs::remove_file(&entry.cached_path);
            keys_to_remove.push(key.clone());
            continue;
        }

        // Update actual file size from disk
        if let Ok(meta) = fs::metadata(&entry.cached_path) {
            entry.size_bytes = meta.len();
            active_cached_paths.insert(entry.cached_path.clone());
        } else {
            keys_to_remove.push(key.clone());
        }
    }

    for key in keys_to_remove {
        index.entries.remove(&key);
    }

    // Step 2: Disk -> Index cleanup (stale .tmp files and unindexed files)
    clean_unindexed_and_temp_files(root, &active_cached_paths, clean_unindexed);

    // Step 3: Recalculate total bytes
    index.total_bytes = index.entries.values().map(|e| e.size_bytes).sum();
}

fn clean_unindexed_and_temp_files(
    dir: &Path,
    active_paths: &HashSet<PathBuf>,
    clean_unindexed: bool,
) {
    if let Ok(read_dir) = fs::read_dir(dir) {
        for item in read_dir.flatten() {
            let path = item.path();
            if path.is_dir() {
                clean_unindexed_and_temp_files(&path, active_paths, clean_unindexed);
            } else if let Ok(meta) = item.metadata() {
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default();

                if file_name == INDEX_FILE_NAME {
                    continue;
                }

                if file_name.starts_with(".tmp_") {
                    let _ = fs::remove_file(&path);
                    continue;
                }

                // If file is not in the active index, clean it up to prevent silent bloat
                if clean_unindexed && meta.is_file() && !active_paths.contains(&path) {
                    let _ = fs::remove_file(&path);
                }
            }
        }
    }
}

/// Compute a deterministic cache key based on category, source file path, mtime, and size.
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

/// Notify background pruner that new cache artifacts have been written.
pub fn notify_cache_changed() {
    let (lock, cvar) = &PRUNER_SIGNAL;
    if let Ok(mut dirty) = lock.lock() {
        *dirty = true;
        cvar.notify_one();
    }
}

/// Spawns the background disk cache pruner daemon thread.
/// Runs startup reconciliation & prune once, then sleeps waiting for write notifications.
pub fn start_disk_cache_pruner_daemon(max_bytes: u64) {
    std::thread::spawn(move || {
        global_cache().reconcile(true);
        let _ = global_cache().prune_to_budget(max_bytes);

        let (lock, cvar) = &PRUNER_SIGNAL;
        loop {
            let mut dirty = match lock.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };

            while !*dirty {
                dirty = match cvar.wait(dirty) {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                };
            }
            *dirty = false;
            drop(dirty);

            // Debounce: sleep 5 seconds to batch burst writes
            std::thread::sleep(std::time::Duration::from_secs(5));

            let _ = global_cache().prune_to_budget(max_bytes);
        }
    });
}

#[cfg(test)]
#[path = "disk_cache_tests.rs"]
mod tests;
