use std::fs::File;
use std::io::{BufReader, Read, Result as IoResult};
use std::ops::Deref;
use std::path::Path;

/// Internal backing storage for a memory-mapped or buffered file.
enum MmapStorage {
    Mapped(memmap2::Mmap),
    Heap(Vec<u8>),
    Empty,
}

pub struct MmapFile {
    storage: MmapStorage,
}

impl MmapFile {
    pub fn open<P: AsRef<Path>>(path: P) -> IoResult<Self> {
        let file = File::open(path.as_ref())?;
        Self::from_file(file)
    }

    pub fn from_file(file: File) -> IoResult<Self> {
        let metadata = file.metadata()?;
        let len = metadata.len();

        if len == 0 {
            return Ok(Self {
                storage: MmapStorage::Empty,
            });
        }

        match unsafe { memmap2::MmapOptions::new().map(&file) } {
            Ok(mmap) => Ok(Self {
                storage: MmapStorage::Mapped(mmap),
            }),
            Err(_) => {
                let mut reader = BufReader::new(file);
                let mut data = Vec::with_capacity(len.min(1024 * 1024) as usize);
                reader.read_to_end(&mut data)?;
                Ok(Self {
                    storage: MmapStorage::Heap(data),
                })
            }
        }
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        match &self.storage {
            MmapStorage::Mapped(mmap) => &mmap[..],
            MmapStorage::Heap(vec) => &vec[..],
            MmapStorage::Empty => &[],
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.as_bytes().len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.as_bytes().is_empty()
    }

    #[inline]
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(self.as_bytes())
    }

    #[inline]
    pub fn to_string_lossy(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(self.as_bytes())
    }

    pub fn count_lines(&self) -> usize {
        let bytes = self.as_bytes();
        if bytes.is_empty() {
            return 0;
        }
        let newlines = memchr::memchr_iter(b'\n', bytes).count();
        if bytes.ends_with(b"\n") {
            newlines
        } else {
            newlines + 1
        }
    }
}

impl Deref for MmapFile {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}

impl AsRef<[u8]> for MmapFile {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_mmap_file_read_and_lines() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "Line 1").unwrap();
        writeln!(temp, "Line 2").unwrap();
        write!(temp, "Line 3").unwrap();
        temp.flush().unwrap();

        let mmap = MmapFile::open(temp.path()).unwrap();
        assert_eq!(mmap.len(), 20);
        assert!(!mmap.is_empty());
        assert_eq!(mmap.as_str().unwrap(), "Line 1\nLine 2\nLine 3");
        assert_eq!(mmap.count_lines(), 3);
    }

    #[test]
    fn test_mmap_empty_file() {
        let temp = NamedTempFile::new().unwrap();
        let mmap = MmapFile::open(temp.path()).unwrap();
        assert_eq!(mmap.len(), 0);
        assert!(mmap.is_empty());
        assert_eq!(mmap.as_str().unwrap(), "");
        assert_eq!(mmap.count_lines(), 0);
    }

    #[test]
    fn test_mmap_vs_fs_read_comparison() {
        use std::time::Instant;

        let mut temp = NamedTempFile::new().unwrap();
        for i in 0..50_000 {
            writeln!(temp, "2026-10-08 12:00:{:02}.{:03} [INFO] Transaction id={i} payload=abcdefghijklmnopqrstuvwxyz", i % 60, i % 1000).unwrap();
        }
        temp.flush().unwrap();
        let path = temp.path();

        // 1. Benchmark std::fs::read_to_string + lines().count()
        let start = Instant::now();
        let mut fs_lines = 0;
        for _ in 0..20 {
            let content = std::fs::read_to_string(path).unwrap();
            fs_lines = content.lines().count();
        }
        let fs_dur = start.elapsed() / 20;

        // 2. Benchmark MmapFile + count_lines() (zero-copy SIMD memchr)
        let start = Instant::now();
        let mut mmap_lines = 0;
        for _ in 0..20 {
            let mmap = MmapFile::open(path).unwrap();
            mmap_lines = mmap.count_lines();
        }
        let mmap_dur = start.elapsed() / 20;

        assert_eq!(fs_lines, mmap_lines);
        println!("\n=== PERFORMANCE COMPARISON (50,000 lines / ~4 MB) ===");
        println!("• std::fs::read_to_string : {fs_dur:?}");
        println!("• Zero-Copy MmapFile       : {mmap_dur:?}");
        let speedup = fs_dur.as_secs_f64() / mmap_dur.as_secs_f64().max(0.000001);
        println!("• Speedup                  : {speedup:.2}x faster");
        println!("======================================================\n");
    }
}
