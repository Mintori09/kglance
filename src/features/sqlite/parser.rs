use std::fs::File;
use std::io::Read;
use std::path::Path;

use rusqlite::types::ValueRef;
use rusqlite::{Connection, OpenFlags};

use crate::features::common::parser::traits::{ParseError, PreviewParser};
use crate::features::common::parser::types::ParsedContent;
use crate::features::sheet::types::SheetData;

pub const SQLITE_MAGIC_HEADER: &[u8; 16] = b"SQLite format 3\0";
const MAX_TABLES_COUNT: usize = 30;
const MAX_PREVIEW_ROWS: usize = 100;
const MAX_TEXT_CELL_CHARS: usize = 300;

pub struct SqliteParser;

impl PreviewParser for SqliteParser {
    fn supported_extensions(&self) -> &[&str] {
        &["sqlite", "sqlite3", "db", "db3", "s3db", "sl3"]
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let meta = path.metadata().map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => ParseError::FileNotFound,
            std::io::ErrorKind::PermissionDenied => ParseError::PermissionDenied,
            _ => ParseError::ParseFailed(format!("Failed to read file metadata: {e}")),
        })?;

        if meta.len() == 0 {
            return Ok(ParsedContent::Spreadsheet {
                sheets: vec![SheetData {
                    name: "Database Info".to_string(),
                    headers: vec!["Status".to_string()],
                    rows: vec![vec!["Database file is empty (0 bytes)".to_string()]],
                }],
            });
        }

        verify_sqlite_magic_header(path)?;

        let path_str = path.to_string_lossy();
        let uri = format!("file:{path_str}?mode=ro&immutable=1");
        let conn = Connection::open_with_flags(
            &uri,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
        )
        .or_else(|_| {
            Connection::open_with_flags(
                path,
                OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
            )
        })
        .map_err(|e| ParseError::ParseFailed(format!("Failed to open SQLite database: {e}")))?;

        let table_names = query_user_tables_and_views(&conn)?;
        let mut sheets = Vec::with_capacity(table_names.len());

        for table_name in table_names.iter().take(MAX_TABLES_COUNT) {
            let sheet = extract_table_sheet(&conn, table_name);
            sheets.push(sheet);
        }

        if sheets.is_empty() {
            sheets.push(SheetData {
                name: "Database Info".to_string(),
                headers: vec!["Status".to_string()],
                rows: vec![vec!["Database contains no tables or views".to_string()]],
            });
        }

        Ok(ParsedContent::Spreadsheet { sheets })
    }
}

pub fn verify_sqlite_magic_header(path: &Path) -> Result<(), ParseError> {
    let mut file = File::open(path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => ParseError::FileNotFound,
        std::io::ErrorKind::PermissionDenied => ParseError::PermissionDenied,
        _ => ParseError::ParseFailed(format!("Failed to open file for header verification: {e}")),
    })?;

    let mut header_buf = [0u8; 16];
    let bytes_read = file
        .read(&mut header_buf)
        .map_err(|e| ParseError::ParseFailed(format!("Failed to read file header: {e}")))?;

    if bytes_read < 16 || &header_buf != SQLITE_MAGIC_HEADER {
        return Err(ParseError::UnsupportedFormat);
    }

    Ok(())
}

fn query_user_tables_and_views(conn: &Connection) -> Result<Vec<String>, ParseError> {
    let mut stmt = conn
        .prepare(
            "SELECT name FROM sqlite_master \
             WHERE type IN ('table', 'view') AND name NOT LIKE 'sqlite_%' \
             ORDER BY name ASC;",
        )
        .map_err(|e| ParseError::ParseFailed(format!("Failed to query schema: {e}")))?;

    let table_iter = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| ParseError::ParseFailed(format!("Failed to read schema rows: {e}")))?;

    let tables: Vec<String> = table_iter.flatten().collect();

    Ok(tables)
}

fn query_table_columns(conn: &Connection, table_name: &str) -> Vec<String> {
    let escaped_name = table_name.replace('"', "\"\"");
    let pragma_sql = format!("PRAGMA table_info(\"{escaped_name}\");");

    if let Ok(mut stmt) = conn.prepare(&pragma_sql)
        && let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(1))
    {
        let cols: Vec<String> = rows.filter_map(Result::ok).collect();
        if !cols.is_empty() {
            return cols;
        }
    }

    Vec::new()
}

fn extract_table_sheet(conn: &Connection, table_name: &str) -> SheetData {
    let escaped_name = table_name.replace('"', "\"\"");
    let mut headers = query_table_columns(conn, table_name);

    let query_sql = format!("SELECT * FROM \"{escaped_name}\" LIMIT {MAX_PREVIEW_ROWS};");
    let mut stmt = match conn.prepare(&query_sql) {
        Ok(s) => s,
        Err(e) => {
            return SheetData {
                name: table_name.to_string(),
                headers: vec!["Error".to_string()],
                rows: vec![vec![format!("Failed to prepare query: {e}")]],
            };
        }
    };

    if headers.is_empty() {
        headers = stmt.column_names().into_iter().map(String::from).collect();
    }
    if headers.is_empty() {
        headers.push("Column 1".to_string());
    }

    let col_count = stmt.column_count();
    let mut rows = Vec::new();

    if let Ok(mut query_rows) = stmt.query([]) {
        while let Ok(Some(row)) = query_rows.next() {
            let mut row_cells = Vec::with_capacity(col_count);
            for col_idx in 0..col_count {
                let cell_str = match row.get_ref(col_idx) {
                    Ok(ValueRef::Null) => "[NULL]".to_string(),
                    Ok(ValueRef::Integer(i)) => format!("{i}"),
                    Ok(ValueRef::Real(f)) => format_sqlite_real(f),
                    Ok(ValueRef::Text(bytes)) => format_sqlite_text(bytes),
                    Ok(ValueRef::Blob(bytes)) => {
                        let blob_len = bytes.len();
                        format!("[BLOB {blob_len} bytes]")
                    }
                    Err(e) => format!("[ERR: {e}]"),
                };
                row_cells.push(cell_str);
            }
            rows.push(row_cells);
        }
    }

    SheetData {
        name: table_name.to_string(),
        headers,
        rows,
    }
}

fn format_sqlite_real(f: f64) -> String {
    if f.is_nan() {
        "NaN".to_string()
    } else if f.is_infinite() {
        if f.is_sign_positive() {
            "Infinity".to_string()
        } else {
            "-Infinity".to_string()
        }
    } else if f == f.trunc() && f >= (i64::MIN as f64) && f <= (i64::MAX as f64) {
        let int_val = f as i64;
        format!("{int_val}")
    } else {
        format!("{f}")
    }
}

fn format_sqlite_text(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    if text.chars().count() > MAX_TEXT_CELL_CHARS {
        let truncated: String = text.chars().take(MAX_TEXT_CELL_CHARS - 3).collect();
        format!("{truncated}...")
    } else {
        text.into_owned()
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
