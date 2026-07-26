use std::path::Path;

use crate::parsers::{ParseError, ParsedContent, PreviewParser, SheetData};

pub struct CsvParser;

impl PreviewParser for CsvParser {
    fn supported_extensions(&self) -> &[&str] {
        &["csv"]
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
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
        Ok(ParsedContent::Spreadsheet {
            sheets: vec![SheetData {
                name: path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Sheet1")
                    .to_string(),
                headers,
                rows,
            }],
        })
    }
}
