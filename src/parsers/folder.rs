use crate::parsers::{DirEntry, ParseError, ParsedContent, PreviewParser};
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
            let entry = entry.map_err(|e| ParseError::ParseFailed(e.to_string()))?;
            let metadata = entry
                .metadata()
                .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

            entries.push(DirEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                is_dir: metadata.is_dir(),
                size: metadata.len(),
                modified: chrono::DateTime::<chrono::Local>::from(
                    metadata
                        .modified()
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                )
                .format("%Y-%m-%d %H:%M")
                .to_string(),
            });
        }

        entries.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(ParsedContent::Folder { entries })
    }
}
