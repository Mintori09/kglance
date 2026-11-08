use crate::core::utils::human_time;
use crate::features::common::parser::traits::{ParseError, PreviewParser};
use crate::features::common::parser::types::ParsedContent;
use crate::features::folder::types::DirEntry;
use std::path::Path;

pub struct FolderParser;

impl PreviewParser for FolderParser {
    fn supported_extensions(&self) -> &[&str] {
        &[]
    }

    fn is_supported(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let mut entries = Vec::new();
        let dir = std::fs::read_dir(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        for entry in dir {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let metadata = match entry.metadata() {
                Ok(meta) => meta,
                Err(_) => match entry.path().symlink_metadata() {
                    Ok(symlink_meta) => symlink_meta,
                    Err(_) => continue,
                },
            };

            let modified_time = metadata
                .modified()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let raw_modified = modified_time
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            let entry_path = entry.path().to_string_lossy().to_string();

            entries.push(DirEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                path: entry_path,
                is_dir: metadata.is_dir(),
                size: metadata.len(),
                modified: human_time(modified_time),
                raw_modified,
            });
        }

        Ok(ParsedContent::Folder { entries })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_folder_parser_broken_symlink_does_not_crash() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let valid_file = temp_dir.path().join("valid.txt");
        std::fs::write(&valid_file, b"hello").expect("write valid file");

        let broken_symlink = temp_dir.path().join("broken_link.txt");
        let nonexistent_target = temp_dir.path().join("nonexistent_file.txt");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&nonexistent_target, &broken_symlink)
            .expect("create broken symlink");

        let parser = FolderParser;
        let res = parser
            .parse(temp_dir.path())
            .expect("parse should succeed despite broken symlink");

        match res {
            ParsedContent::Folder { entries } => {
                assert!(entries.iter().any(|e| e.name == "valid.txt"));
                #[cfg(unix)]
                {
                    let broken_entry = entries.iter().find(|e| e.name == "broken_link.txt");
                    assert!(
                        broken_entry.is_some(),
                        "broken symlink entry should be preserved"
                    );
                    assert!(broken_entry.unwrap().path.contains("broken_link.txt"));
                }
            }
            _ => panic!("Expected ParsedContent::Folder"),
        }
    }

    #[test]
    fn test_folder_parser_retains_absolute_path() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let file_path = temp_dir.path().join("file.txt");
        std::fs::write(&file_path, b"content").expect("write file");

        let parser = FolderParser;
        let res = parser.parse(temp_dir.path()).expect("parse succeeds");

        match res {
            ParsedContent::Folder { entries } => {
                let entry = entries.iter().find(|e| e.name == "file.txt").unwrap();
                let p = Path::new(&entry.path);
                assert!(p.is_absolute());
                assert_eq!(p, file_path);
            }
            _ => panic!("Expected ParsedContent::Folder"),
        }
    }
}
