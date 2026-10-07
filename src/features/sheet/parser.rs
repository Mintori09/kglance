use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use crate::features::common::parser::traits::{ParseError, PreviewParser};
use crate::features::common::parser::types::ParsedContent;
use crate::features::sheet::types::{ColumnMeta, ColumnType, SheetData};

const SAMPLE_SIZE_BYTES: usize = 8192;
const MAX_PREVIEW_ROWS: usize = 100_000;
pub const MIN_COLUMN_WIDTH: f32 = 100.0;
pub const MAX_COLUMN_WIDTH: f32 = 350.0;

pub struct CsvParser;

impl PreviewParser for CsvParser {
    fn supported_extensions(&self) -> &[&str] {
        &["csv", "tsv", "tab"]
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let file = File::open(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        let mut reader = BufReader::new(file);

        let mut sample_buf = vec![0u8; SAMPLE_SIZE_BYTES];
        let bytes_read = reader
            .read(&mut sample_buf)
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        sample_buf.truncate(bytes_read);

        let ext_hint = path.extension().and_then(|s| s.to_str());
        let delimiter = sniff_delimiter(&sample_buf, ext_hint);

        // Re-open for full streaming parse
        let file = File::open(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        let mut csv_reader = csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .has_headers(false)
            .flexible(true)
            .trim(csv::Trim::None)
            .from_reader(BufReader::new(file));

        let mut raw_records = csv_reader.records();

        // 1. First record -> headers
        let headers: Vec<String> = match raw_records.next() {
            Some(Ok(record)) => record.iter().map(|s| s.trim().to_string()).collect(),
            Some(Err(e)) => return Err(ParseError::ParseFailed(e.to_string())),
            None => Vec::new(),
        };

        // 2. Remaining records -> rows
        let mut rows: Vec<Vec<String>> = Vec::new();
        for result in raw_records.take(MAX_PREVIEW_ROWS) {
            match result {
                Ok(record) => {
                    let row: Vec<String> = record.iter().map(|s| s.trim().to_string()).collect();
                    rows.push(row);
                }
                Err(_) => {
                    // Tolerant parsing: skip corrupted lines rather than aborting preview
                    continue;
                }
            }
        }

        let sheet_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Sheet1")
            .to_string();

        Ok(ParsedContent::Spreadsheet {
            sheets: vec![SheetData {
                name: sheet_name,
                headers,
                rows,
            }],
        })
    }
}

/// Sniffs the most likely delimiter from a raw byte sample.
pub fn sniff_delimiter(sample: &[u8], extension_hint: Option<&str>) -> u8 {
    if sample.is_empty() {
        return b',';
    }

    let candidates = *b",;\t|";
    let hint_delim = match extension_hint {
        Some("tsv" | "tab") => Some(b'\t'),
        Some("csv") => Some(b','),
        _ => None,
    };

    let mut best_delim = hint_delim.unwrap_or(b',');
    let mut best_score = -1.0f32;

    for delim in candidates {
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(delim)
            .has_headers(false)
            .flexible(true)
            .from_reader(sample);

        let mut row_counts: Vec<usize> = Vec::new();
        for record in rdr.records().take(40).flatten() {
            row_counts.push(record.len());
        }

        if row_counts.is_empty() {
            continue;
        }

        let num_rows = row_counts.len();
        let max_fields = *row_counts.iter().max().unwrap_or(&0);
        if max_fields <= 1 {
            continue;
        }

        // Count frequency of field counts
        let mut field_freq = rustc_hash::FxHashMap::default();
        for &cnt in &row_counts {
            *field_freq.entry(cnt).or_insert(0usize) += 1;
        }

        let (mode_count, mode_freq) = field_freq
            .into_iter()
            .max_by_key(|&(_, freq)| freq)
            .unwrap_or((0, 0));

        if mode_count <= 1 {
            continue;
        }

        let consistency = mode_freq as f32 / num_rows as f32;
        let mut score = consistency * 100.0 + (mode_count as f32).min(20.0) * 8.0;

        if Some(delim) == hint_delim {
            score += 15.0;
        }

        if score > best_score {
            best_score = score;
            best_delim = delim;
        }
    }

    best_delim
}

/// Infers column types and computes proportional pixel widths from headers and sample rows.
pub fn infer_column_types_and_widths(headers: &[String], rows: &[Vec<String>]) -> Vec<ColumnMeta> {
    let col_count = headers.len().max(rows.first().map_or(0, |r| r.len()));
    let sample_rows = &rows[..rows.len().min(300)];

    let mut columns = Vec::with_capacity(col_count);

    for col_idx in 0..col_count {
        let name = headers
            .get(col_idx)
            .cloned()
            .unwrap_or_else(|| format!("Column {}", col_idx + 1));

        let mut is_integer = true;
        let mut is_float = true;
        let mut is_date = true;
        let mut has_non_empty = false;
        let header_char_len = name.chars().count();
        let mut max_data_char_len = 0;

        for row in sample_rows {
            if let Some(cell) = row.get(col_idx) {
                let trimmed = cell.trim();
                let char_len = trimmed.chars().count();
                if char_len > max_data_char_len {
                    max_data_char_len = char_len;
                }

                if !trimmed.is_empty() {
                    has_non_empty = true;
                    if is_integer && trimmed.parse::<i64>().is_err() {
                        is_integer = false;
                    }
                    if is_float && trimmed.parse::<f64>().is_err() {
                        is_float = false;
                    }
                    if is_date && try_parse_date_or_datetime(trimmed).is_none() {
                        is_date = false;
                    }
                }
            }
        }

        let col_type = if !has_non_empty {
            ColumnType::Empty
        } else if is_integer {
            ColumnType::Integer
        } else if is_float {
            ColumnType::Float
        } else if is_date {
            ColumnType::Date
        } else {
            ColumnType::Text
        };

        // Header contributes up to 25 chars, data contributes up to 35 chars
        let effective_char_len = header_char_len.min(25).max(max_data_char_len.min(35));
        let estimated_width =
            ((effective_char_len as f32 * 8.5) + 36.0).clamp(MIN_COLUMN_WIDTH, MAX_COLUMN_WIDTH);

        columns.push(ColumnMeta {
            name,
            col_type,
            width: estimated_width,
        });
    }

    columns
}

/// Parses a date or datetime string into a normalized sortable timestamp (seconds from epoch approximation).
/// Supports ISO (YYYY-MM-DD), DMY (DD/MM/YYYY or DD-MM-YYYY), and optional time components (HH:MM:SS).
pub fn try_parse_date_or_datetime(s: &str) -> Option<i64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    // Split date and optional time component
    let mut parts = s.split(|c: char| c.is_whitespace() || c == 'T');
    let date_part = parts.next()?;
    let time_part = parts.next();

    let (year, month, day) = parse_date_components(date_part)?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) || !(1000..=9999).contains(&year) {
        return None;
    }

    let (hour, min, sec) = if let Some(tp) = time_part {
        parse_time_components(tp)?
    } else {
        (0, 0, 0)
    };

    if hour > 23 || min > 59 || sec > 59 {
        return None;
    }

    // Days since epoch approximation (enough for exact ordering)
    let y = year as i64;
    let m = month as i64;
    let d = day as i64;

    // Approximate cumulative days
    let days = y * 365 + (y / 4) - (y / 100) + (y / 400) + (m * 30) + d;
    let seconds = days * 86400 + (hour as i64 * 3600) + (min as i64 * 60) + (sec as i64);
    Some(seconds)
}

fn parse_date_components(date_str: &str) -> Option<(u32, u32, u32)> {
    if date_str.contains('-') {
        let mut segs = date_str.split('-');
        let first = segs.next()?;
        let second = segs.next()?;
        let third = segs.next()?;
        if segs.next().is_some() {
            return None;
        }

        // YYYY-MM-DD
        if first.len() == 4 {
            let y = first.parse().ok()?;
            let m = second.parse().ok()?;
            let d = third.parse().ok()?;
            Some((y, m, d))
        } else if third.len() == 4 {
            // DD-MM-YYYY
            let d = first.parse().ok()?;
            let m = second.parse().ok()?;
            let y = third.parse().ok()?;
            Some((y, m, d))
        } else {
            None
        }
    } else if date_str.contains('/') {
        let mut segs = date_str.split('/');
        let first = segs.next()?;
        let second = segs.next()?;
        let third = segs.next()?;
        if segs.next().is_some() {
            return None;
        }

        if third.len() == 4 {
            // DD/MM/YYYY
            let d = first.parse().ok()?;
            let m = second.parse().ok()?;
            let y = third.parse().ok()?;
            Some((y, m, d))
        } else if first.len() == 4 {
            // YYYY/MM/DD
            let y = first.parse().ok()?;
            let m = second.parse().ok()?;
            let d = third.parse().ok()?;
            Some((y, m, d))
        } else {
            None
        }
    } else {
        None
    }
}

fn parse_time_components(time_str: &str) -> Option<(u32, u32, u32)> {
    let mut parts = time_str.split(':');
    let h = parts.next()?.parse().ok()?;
    let m = parts.next()?.parse().ok()?;
    let s = if let Some(sec_str) = parts.next() {
        // Strip any millisecond/subsecond fraction
        let sec_cleaned = sec_str.split('.').next().unwrap_or(sec_str);
        sec_cleaned.parse().ok()?
    } else {
        0
    };
    Some((h, m, s))
}
