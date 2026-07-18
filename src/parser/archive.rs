use std::path::Path;

use crate::parser::{ArchiveEntry, ParseError, ParsedContent, PreviewParser};

pub struct ArchiveParser;

impl ArchiveParser {
    fn list_zip(&self, path: &Path) -> Result<Vec<ArchiveEntry>, ParseError> {
        let file = std::fs::File::open(path)
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        let mut entries = Vec::with_capacity(archive.len());
        for i in 0..archive.len() {
            let entry = archive.by_index(i)
                .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
            entries.push(ArchiveEntry {
                path: entry.name().to_string(),
                size: entry.size(),
                is_dir: entry.is_dir(),
            });
        }
        Ok(entries)
    }

    #[cfg(feature = "7z")]
    fn list_7z(&self, path: &Path) -> Result<Vec<ArchiveEntry>, ParseError> {
        use sevenz_rust::{Password, SevenZReader};

        let reader = SevenZReader::open(path, Password::empty())
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        let entries: Vec<ArchiveEntry> = reader.archive().files.iter().map(|entry| {
            ArchiveEntry {
                path: entry.name.clone(),
                size: entry.size,
                is_dir: entry.is_directory,
            }
        }).collect();

        Ok(entries)
    }

    #[cfg(not(feature = "7z"))]
    fn list_7z(&self, _path: &Path) -> Result<Vec<ArchiveEntry>, ParseError> {
        Err(ParseError::ParseFailed("7z support not enabled".into()))
    }

    fn list_tar(&self, path: &Path) -> Result<Vec<ArchiveEntry>, ParseError> {
        let file = std::fs::File::open(path)
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        let mut archive = tar::Archive::new(file);

        let mut entries = Vec::new();
        for entry in archive.entries()
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?
        {
            let entry = entry
                .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
            let path_name = entry.path()
                .map_err(|e| ParseError::ParseFailed(e.to_string()))?
                .to_string_lossy()
                .to_string();
            entries.push(ArchiveEntry {
                path: path_name,
                size: entry.size(),
                is_dir: entry.header().entry_type().is_dir(),
            });
        }
        Ok(entries)
    }
}

impl PreviewParser for ArchiveParser {
    fn supported_extensions(&self) -> &[&str] {
        &["zip", "tar", "gz", "tgz", "xz", "txz", "7z"]
    }

    fn is_supported(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| self.supported_extensions().contains(&e.to_lowercase().as_str()))
            .unwrap_or(false)
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let entries = match ext {
            "zip" => self.list_zip(path)?,
            "7z" => self.list_7z(path)?,
            _ if ext == "tar" || ext == "gz" || ext == "tgz"
                  || ext == "xz" || ext == "txz" => self.list_tar(path)?,
            _ => return Err(ParseError::UnsupportedFormat),
        };

        let total_files = entries.len();
        Ok(ParsedContent::Archive { entries, total_files })
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
            zip.start_file("test.txt", SimpleFileOptions::default()).unwrap();
            zip.write_all(b"hello there").unwrap();
            zip.finish().unwrap();
        }
        let parser = ArchiveParser;
        let result = parser.parse(&zip_path).unwrap();
        match result {
            ParsedContent::Archive { entries, total_files } => {
                assert_eq!(total_files, 2);
                assert!(entries.iter().any(|e| e.path == "adir/"));
                assert!(entries.iter().any(|e| e.path == "test.txt"));
            }
            _ => panic!("expected Archive variant"),
        }
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
        let result = parser.parse(&tar_path).unwrap();
        match result {
            ParsedContent::Archive { entries, total_files } => {
                assert_eq!(total_files, 2);
                assert!(entries.iter().any(|e| e.path == "adir/"));
                assert!(entries.iter().any(|e| e.path == "test.txt"));
            }
            _ => panic!("expected Archive variant"),
        }
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
            writer.push_archive_entry(entry, Some(b"hello" as &[u8])).unwrap();
            writer.finish().unwrap();
        }
        let parser = ArchiveParser;
        let result = parser.parse(&sz_path).unwrap();
        match result {
            ParsedContent::Archive { entries, total_files } => {
                assert_eq!(total_files, 1);
                assert_eq!(entries[0].path, "test.txt");
            }
            _ => panic!("expected Archive variant"),
        }
    }
}
