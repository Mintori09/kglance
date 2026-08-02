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
            let entry = entry.map_err(|e| ParseError::ParseFailed(e.to_string()))?;
            let metadata = entry
                .metadata()
                .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

            let modified_time = metadata
                .modified()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let raw_modified = modified_time
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            entries.push(DirEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                is_dir: metadata.is_dir(),
                size: metadata.len(),
                modified: human_time(modified_time),
                raw_modified,
            });
        }

        entries.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(ParsedContent::Folder { entries })
    }
}
