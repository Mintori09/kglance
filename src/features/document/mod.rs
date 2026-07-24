pub mod types;

pub mod content;
pub mod helpers;
pub mod parsers;
pub mod view;

pub use content::folder_content;
pub use content::spreadsheet_content;
pub use helpers::docx;
pub use helpers::xlsx;
pub use parsers::csv;
pub use parsers::epub;
pub use parsers::folder;
pub use parsers::office;
pub use parsers::text;
pub use view::spreadsheet_view::view_spreadsheet;
pub use view::table_view::view_table;

use std::path::Path;

use crate::app::Message;
use crate::core::preview::PreviewContent;
use crate::core::utils::human_time;
use crate::parsers::{ParseError, PreviewParser};

use content::folder_content::FolderContent;
use content::spreadsheet_content::SpreadsheetContent;
use types::{DirEntry, SheetData};

impl PreviewParser<Message> for parsers::csv::CsvParser {
    fn supported_extensions(&self) -> &[&str] {
        &["csv"]
    }

    fn is_supported(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e == "csv")
            .unwrap_or(false)
    }

    fn parse(&self, path: &Path) -> Result<Box<dyn PreviewContent<Message>>, ParseError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        let mut lines = content.lines();
        let headers: Vec<String> = lines
            .next()
            .map(|line| line.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();
        let rows: Vec<Vec<String>> = lines
            .map(|line| line.split(',').map(|s| s.trim().to_string()).collect())
            .collect();
        Ok(Box::new(SpreadsheetContent {
            sheets: vec![SheetData {
                name: path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Sheet1")
                    .to_string(),
                headers,
                rows,
            }],
        }))
    }
}

impl PreviewParser<Message> for parsers::epub::EpubParser {
    fn supported_extensions(&self) -> &[&str] {
        &["epub"]
    }

    fn is_supported(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase() == "epub")
            .unwrap_or(false)
    }

    fn parse(&self, path: &Path) -> Result<Box<dyn PreviewContent<Message>>, ParseError> {
        parsers::epub::parse_epub(path)
    }
}

impl PreviewParser<Message> for parsers::folder::FolderParser {
    fn supported_extensions(&self) -> &[&str] {
        &[]
    }

    fn is_supported(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn parse(&self, path: &Path) -> Result<Box<dyn PreviewContent<Message>>, ParseError> {
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
        Ok(Box::new(FolderContent { entries }))
    }
}

impl PreviewParser<Message> for parsers::office::OfficeParser {
    fn supported_extensions(&self) -> &[&str] {
        parsers::office::supported_office_extensions()
    }

    fn is_supported(&self, path: &Path) -> bool {
        path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
            parsers::office::supported_office_extensions().contains(&e.to_lowercase().as_str())
        })
    }

    fn parse(&self, path: &Path) -> Result<Box<dyn PreviewContent<Message>>, ParseError> {
        parsers::office::parse_office(path)
    }
}

impl PreviewParser<Message> for parsers::text::TextParser {
    fn supported_extensions(&self) -> &[&str] {
        parsers::text::supported_text_extensions()
    }

    fn is_supported(&self, path: &Path) -> bool {
        parsers::text::is_supported_text(path)
    }

    fn parse(&self, path: &Path) -> Result<Box<dyn PreviewContent<Message>>, ParseError> {
        parsers::text::parse_text(self, path)
    }
}
