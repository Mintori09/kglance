use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub const CACHE_MAGIC: &[u8; 7] = b"KGLPDF\x01";
pub const PNG_SIGNATURE: &[u8; 8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedPage {
    pub png_bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub struct PdfDiskCache {
    pub dir: PathBuf,
}

impl PdfDiskCache {
    pub fn new(session_id: usize) -> io::Result<Self> {
        let dir =
            std::env::temp_dir().join(format!("kglance-pdf-{}-{}", std::process::id(), session_id));
        fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }

    pub fn page_path(&self, page_index: usize) -> PathBuf {
        self.dir.join(format!("page_{}.png", page_index))
    }

    pub fn save_page_with_meta(
        &self,
        page_index: usize,
        data: &[u8],
        width: u32,
        height: u32,
    ) -> io::Result<()> {
        let unique_ctr = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let tmp_path = self.dir.join(format!(
            ".tmp_{}_{}_{}_{}",
            page_index,
            std::process::id(),
            nanos,
            unique_ctr
        ));
        let final_path = self.page_path(page_index);

        let mut buffer = Vec::with_capacity(15 + data.len());
        buffer.extend_from_slice(CACHE_MAGIC);
        buffer.extend_from_slice(&width.to_le_bytes());
        buffer.extend_from_slice(&height.to_le_bytes());
        buffer.extend_from_slice(data);

        fs::write(&tmp_path, buffer)?;
        fs::rename(&tmp_path, final_path)
    }

    pub fn load_page_with_meta(&self, page_index: usize) -> io::Result<CachedPage> {
        let raw = fs::read(self.page_path(page_index))?;
        if raw.len() < 23 || &raw[0..7] != CACHE_MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid cache magic header",
            ));
        }
        let width_bytes: [u8; 4] = raw[7..11]
            .try_into()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Corrupt width header"))?;
        let height_bytes: [u8; 4] = raw[11..15]
            .try_into()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Corrupt height header"))?;

        let width = u32::from_le_bytes(width_bytes);
        let height = u32::from_le_bytes(height_bytes);
        if width == 0 || height == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid page dimensions",
            ));
        }

        let png_bytes = raw[15..].to_vec();
        if png_bytes.len() < 8 || &png_bytes[0..8] != PNG_SIGNATURE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid PNG payload signature",
            ));
        }

        Ok(CachedPage {
            png_bytes,
            width,
            height,
        })
    }

    pub fn save_page(&self, page_index: usize, compressed_data: &[u8]) -> io::Result<()> {
        self.save_page_with_meta(page_index, compressed_data, 0, 0)
    }

    pub fn load_page(&self, page_index: usize) -> io::Result<Vec<u8>> {
        match self.load_page_with_meta(page_index) {
            Ok(cached) => Ok(cached.png_bytes),
            Err(_) => fs::read(self.page_path(page_index)),
        }
    }

    pub fn has_page(&self, page_index: usize) -> bool {
        self.page_path(page_index).exists()
    }
}

/// Checks if a process with `pid` is currently alive.
#[cfg(unix)]
fn is_process_alive(pid: i32) -> bool {
    if pid <= 0 {
        return false;
    }
    // kill(pid, 0) checks if process exists without sending a signal.
    // If it returns 0, process exists. If ESRCH, process does not exist.
    // If EPERM, process exists but belongs to another user.
    let ret = unsafe { libc::kill(pid, 0) };
    if ret == 0 {
        true
    } else {
        std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
    }
}

#[cfg(not(unix))]
fn is_process_alive(_pid: i32) -> bool {
    true
}

/// Scans the system temporary directory for orphaned PDF session cache directories
/// (`kglance-pdf-{pid}-{timestamp}`) left behind by previous crashes or unclean exits.
/// If the owner process is no longer alive, the directory is safely removed.
pub fn clean_stale_pdf_caches() {
    let temp_dir = std::env::temp_dir();
    let entries = match fs::read_dir(&temp_dir) {
        Ok(entries) => entries,
        Err(e) => {
            crate::log_debug!("[PDF_CACHE] Failed to read temp directory: {e}");
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let file_name = match path.file_name().and_then(|f| f.to_str()) {
            Some(name) => name,
            None => continue,
        };

        if let Some(rest) = file_name.strip_prefix("kglance-pdf-") {
            let mut parts = rest.split('-');
            let pid_str = match parts.next() {
                Some(p) => p,
                None => continue,
            };

            let pid: i32 = match pid_str.parse() {
                Ok(p) => p,
                Err(_) => continue,
            };

            if !is_process_alive(pid) {
                crate::log_info!(
                    "[PDF_CACHE] Cleaning stale cache dir for dead pid {pid}: {}",
                    path.display()
                );
                let _ = fs::remove_dir_all(&path);
            }
        }
    }
}

impl Drop for PdfDiskCache {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_cache_save_load_with_meta() {
        let cache = PdfDiskCache::new(99999).unwrap();
        assert!(!cache.has_page(0));

        let mut sample_png = Vec::new();
        sample_png.extend_from_slice(PNG_SIGNATURE);
        sample_png.extend_from_slice(b"fake_png_data_payload");

        cache
            .save_page_with_meta(0, &sample_png, 800, 1100)
            .unwrap();
        assert!(cache.has_page(0));

        let loaded = cache.load_page_with_meta(0).unwrap();
        assert_eq!(loaded.width, 800);
        assert_eq!(loaded.height, 1100);
        assert_eq!(loaded.png_bytes, sample_png);
    }

    #[test]
    fn test_disk_cache_rejects_corrupted_magic_or_signature() {
        let cache = PdfDiskCache::new(99998).unwrap();
        // Missing magic
        let bad_magic_file = cache.page_path(1);
        fs::write(&bad_magic_file, b"NOT_MAGIC_HEADER_DATA").unwrap();
        assert!(cache.load_page_with_meta(1).is_err());

        // Invalid PNG signature
        let mut bad_sig_data = Vec::new();
        bad_sig_data.extend_from_slice(b"NOT_PNG_BYTES_HERE");
        cache
            .save_page_with_meta(2, &bad_sig_data, 100, 100)
            .unwrap();
        assert!(cache.load_page_with_meta(2).is_err());
    }

    #[test]
    fn test_disk_cache_cleanup_on_drop() {
        let path = {
            let cache = PdfDiskCache::new(88888).unwrap();
            let mut sample_png = Vec::new();
            sample_png.extend_from_slice(PNG_SIGNATURE);
            sample_png.extend_from_slice(b"page_1");
            cache.save_page_with_meta(1, &sample_png, 10, 10).unwrap();
            cache.dir.clone()
        };
        assert!(!path.exists());
    }

    #[test]
    fn test_clean_stale_pdf_caches_removes_dead_pid_and_preserves_alive() {
        let temp_dir = std::env::temp_dir();
        // A pid unlikely to exist (999999)
        let dead_pid = 999_999;
        let dead_dir = temp_dir.join(format!("kglance-pdf-{dead_pid}-123456"));
        fs::create_dir_all(&dead_dir).unwrap();
        fs::write(dead_dir.join("dummy.png"), b"test").unwrap();

        // Current process is alive
        let alive_pid = std::process::id();
        let alive_dir = temp_dir.join(format!("kglance-pdf-{alive_pid}-654321"));
        fs::create_dir_all(&alive_dir).unwrap();
        fs::write(alive_dir.join("dummy.png"), b"test").unwrap();

        // Unrelated folder in temp
        let unrelated_dir = temp_dir.join("kglance-other-test-folder");
        fs::create_dir_all(&unrelated_dir).unwrap();

        clean_stale_pdf_caches();

        assert!(!dead_dir.exists(), "Stale dead pid cache must be removed");
        assert!(alive_dir.exists(), "Active pid cache must be preserved");
        assert!(
            unrelated_dir.exists(),
            "Unrelated temp dir must be preserved"
        );

        // Cleanup test fixtures
        let _ = fs::remove_dir_all(&alive_dir);
        let _ = fs::remove_dir_all(&unrelated_dir);
    }
}
