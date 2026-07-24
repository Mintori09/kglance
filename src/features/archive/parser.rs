use std::path::Path;

use crate::core::preview::PreviewContent;
use crate::features::archive::content::ArchiveContent;
use crate::features::archive::types::ArchiveEntry;
use crate::parsers::{ParseError, PreviewParser};

fn format_timestamp(secs: u64) -> String {
    let dur = std::time::Duration::from_secs(secs);
    let sys_time = std::time::UNIX_EPOCH + dur;
    let dt: chrono::DateTime<chrono::Local> = sys_time.into();
    dt.format("%Y-%m-%d %H:%M").to_string()
}

pub struct ArchiveParser;

impl ArchiveParser {
    fn list_zip(&self, archive_path: &Path) -> Result<Vec<ArchiveEntry>, ParseError> {
        let file = std::fs::File::open(archive_path)
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        let ap = archive_path.to_string_lossy().to_string();

        let mut entries = Vec::with_capacity(archive.len());
        for i in 0..archive.len() {
            let entry = archive
                .by_index(i)
                .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
            let modified = entry
                .last_modified()
                .map(|d| {
                    format!(
                        "{:04}-{:02}-{:02} {:02}:{:02}",
                        d.year(),
                        d.month(),
                        d.day(),
                        d.hour(),
                        d.minute()
                    )
                })
                .unwrap_or_default();
            entries.push(ArchiveEntry {
                path: entry.name().to_string(),
                size: entry.size(),
                is_dir: entry.is_dir(),
                modified,
                archive_path: ap.clone(),
            });
        }
        Ok(entries)
    }

    #[cfg(feature = "7z")]
    fn list_7z(&self, archive_path: &Path) -> Result<Vec<ArchiveEntry>, ParseError> {
        use sevenz_rust::{Password, SevenZReader};

        let reader = SevenZReader::open(archive_path, Password::empty())
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        let ap = archive_path.to_string_lossy().to_string();

        let entries: Vec<ArchiveEntry> = reader
            .archive()
            .files
            .iter()
            .map(|entry| {
                let modified = if entry.has_creation_date {
                    let raw = entry.creation_date.to_raw();
                    let secs = raw / 10_000_000 - 11644473600;
                    format_timestamp(secs)
                } else {
                    String::new()
                };
                ArchiveEntry {
                    path: entry.name.clone(),
                    size: entry.size,
                    is_dir: entry.is_directory,
                    modified,
                    archive_path: ap.clone(),
                }
            })
            .collect();

        Ok(entries)
    }

    #[cfg(not(feature = "7z"))]
    fn list_7z(&self, _archive_path: &Path) -> Result<Vec<ArchiveEntry>, ParseError> {
        Err(ParseError::ParseFailed("7z support not enabled".into()))
    }

    fn list_tar(&self, archive_path: &Path) -> Result<Vec<ArchiveEntry>, ParseError> {
        let file = std::fs::File::open(archive_path)
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        let mut archive = tar::Archive::new(file);
        let ap = archive_path.to_string_lossy().to_string();

        let mut entries = Vec::new();
        let tar_entries = archive
            .entries()
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        for result in tar_entries {
            let entry = result.map_err(|e| ParseError::ParseFailed(e.to_string()))?;
            let path_name = entry
                .path()
                .map_err(|e| ParseError::ParseFailed(e.to_string()))?
                .to_string_lossy()
                .to_string();
            let modified = {
                let ts = entry.header().mtime().unwrap_or(0);
                format_timestamp(ts)
            };
            entries.push(ArchiveEntry {
                path: path_name,
                size: entry.size(),
                is_dir: entry.header().entry_type().is_dir(),
                modified,
                archive_path: ap.clone(),
            });
        }
        Ok(entries)
    }
}

impl PreviewParser<crate::app::Message> for ArchiveParser {
    fn supported_extensions(&self) -> &[&str] {
        &["zip", "tar", "gz", "tgz", "xz", "txz", "7z"]
    }

    fn is_supported(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| {
                self.supported_extensions()
                    .contains(&e.to_lowercase().as_str())
            })
            .unwrap_or(false)
    }

    fn parse(
        &self,
        path: &Path,
    ) -> Result<Box<dyn PreviewContent<crate::app::Message>>, ParseError> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let entries = match ext {
            "zip" => self.list_zip(path)?,
            "7z" => self.list_7z(path)?,
            _ if ext == "tar" || ext == "gz" || ext == "tgz" || ext == "xz" || ext == "txz" => {
                self.list_tar(path)?
            }
            _ => return Err(ParseError::UnsupportedFormat),
        };

        let total_files = entries.len();
        Ok(Box::new(ArchiveContent {
            entries,
            total_files,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    #[test]
    fn parses_zip_archive() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let zip_path = tmp_dir.path().join("test.zip");
        {
            let f = std::fs::File::create(&zip_path).unwrap();
            let mut zip = zip::ZipWriter::new(f);
            let opts = SimpleFileOptions::default();
            zip.add_directory("adir/", opts).unwrap();
            zip.start_file("test.txt", SimpleFileOptions::default())
                .unwrap();
            zip.write_all(b"hello there").unwrap();
            zip.finish().unwrap();
        }
        let parser = ArchiveParser;
        let result = parser.parse(&zip_path);
        assert!(result.is_ok());
    }

    #[test]
    fn parses_tar_archive() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let tar_path = tmp_dir.path().join("test.tar");
        {
            let f = std::fs::File::create(&tar_path).unwrap();
            let mut builder = tar::Builder::new(f);

            let mut header = tar::Header::new_gnu();
            header.set_path("adir/").unwrap();
            header.set_entry_type(tar::EntryType::Directory);
            header.set_size(0);
            header.set_cksum();
            builder.append(&header, &[] as &[u8]).unwrap();

            let mut header = tar::Header::new_gnu();
            header.set_path("test.txt").unwrap();
            header.set_size(5);
            header.set_entry_type(tar::EntryType::Regular);
            header.set_cksum();
            builder.append(&header, b"hello" as &[u8]).unwrap();

            builder.finish().unwrap();
        }
        let parser = ArchiveParser;
        let result = parser.parse(&tar_path);
        assert!(result.is_ok());
    }

    #[test]
    fn supports_archive_extensions() {
        let parser = ArchiveParser;
        assert!(parser.is_supported(Path::new("a.zip")));
        assert!(parser.is_supported(Path::new("a.tar")));
        assert!(parser.is_supported(Path::new("a.7z")));
        assert!(parser.is_supported(Path::new("a.gz")));
        assert!(parser.is_supported(Path::new("a.tgz")));
        assert!(!parser.is_supported(Path::new("a.pdf")));
        assert!(!parser.is_supported(Path::new("a.txt")));
    }

    #[test]
    fn rejects_unsupported_extension() {
        let parser = ArchiveParser;
        let tmp_dir = tempfile::tempdir().unwrap();
        let path = tmp_dir.path().join("test.xyz");
        std::fs::write(&path, b"data").unwrap();
        let result = parser.parse(&path);
        assert!(matches!(result, Err(ParseError::UnsupportedFormat)));
    }

    #[test]
    fn rejects_invalid_zip() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let path = tmp_dir.path().join("test.zip");
        std::fs::write(&path, b"not a zip file").unwrap();
        let parser = ArchiveParser;
        let result = parser.parse(&path);
        assert!(result.is_err());
    }

    #[cfg(feature = "7z")]
    #[test]
    fn parses_7z_archive() {
        use sevenz_rust::{SevenZArchiveEntry, SevenZWriter};

        let tmp_dir = tempfile::tempdir().unwrap();
        let sz_path = tmp_dir.path().join("test.7z");
        {
            let mut writer = SevenZWriter::create(&sz_path).unwrap();
            let mut entry = SevenZArchiveEntry::default();
            entry.name = "test.txt".into();
            entry.has_stream = true;
            writer
                .push_archive_entry(entry, Some(b"hello" as &[u8]))
                .unwrap();
            writer.finish().unwrap();
        }
        let parser = ArchiveParser;
        let result = parser.parse(&sz_path);
        assert!(result.is_ok());
    }
}
