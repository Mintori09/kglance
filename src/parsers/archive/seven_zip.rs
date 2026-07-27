use std::path::Path;

use crate::parsers::{ArchiveEntry, ParseError};

#[cfg(feature = "7z")]
const SEVEN_ZIP_TIME_SCALE: u64 = 10_000_000;
#[cfg(feature = "7z")]
const FILETIME_TO_UNIX_EPOCH_OFFSET_SECONDS: u64 = 11_644_473_600;

#[cfg(feature = "7z")]
pub fn list_7z_entries(archive_path: &Path) -> Result<Vec<ArchiveEntry>, ParseError> {
    use crate::parsers::format_timestamp;
    use sevenz_rust::{Password, SevenZReader};

    let reader = SevenZReader::open(archive_path, Password::empty())
        .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let archive_path_str = archive_path.to_string_lossy().to_string();

    let entries = reader
        .archive()
        .files
        .iter()
        .map(|entry| {
            let modified = if entry.has_creation_date {
                let filetime_ticks = entry.creation_date.to_raw();
                let unix_seconds =
                    filetime_ticks / SEVEN_ZIP_TIME_SCALE - FILETIME_TO_UNIX_EPOCH_OFFSET_SECONDS;
                format_timestamp(unix_seconds)
            } else {
                String::new()
            };

            ArchiveEntry {
                path: entry.name.clone(),
                size: entry.size,
                is_dir: entry.is_directory,
                modified,
                archive_path: archive_path_str.clone(),
            }
        })
        .collect();

    Ok(entries)
}

#[cfg(not(feature = "7z"))]
pub fn list_7z_entries(_archive_path: &Path) -> Result<Vec<ArchiveEntry>, ParseError> {
    Err(ParseError::ParseFailed("7z support not enabled".into()))
}

#[cfg(feature = "7z")]
pub fn extract_from_7z(
    archive_path: &Path,
    entry_path: &str,
    output_path: &Path,
) -> Result<(), ParseError> {
    use sevenz_rust::{Password, SevenZReader};
    use std::fs::File;
    use std::io::Read;

    let file = File::open(archive_path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let file_len = file
        .metadata()
        .map_err(|e| ParseError::ParseFailed(e.to_string()))?
        .len();

    let mut reader = SevenZReader::new(file, file_len, Password::empty())
        .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

    let target_entry_path = entry_path.to_string();
    let target_output_path = output_path.to_owned();
    let mut is_entry_found = false;

    reader
        .for_each_entries(|entry, entry_reader: &mut dyn Read| {
            if entry.name() == target_entry_path {
                if let Some(parent_dir) = target_output_path.parent() {
                    let _ = std::fs::create_dir_all(parent_dir);
                }

                let mut output_file =
                    File::create(&target_output_path).map_err(sevenz_rust::Error::io)?;

                if entry.size() > 0 {
                    std::io::copy(entry_reader, &mut output_file)
                        .map_err(sevenz_rust::Error::io)?;
                }
                is_entry_found = true;
            }
            Ok(true)
        })
        .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

    if is_entry_found {
        Ok(())
    } else {
        Err(ParseError::ParseFailed("entry not found in 7z".into()))
    }
}

#[cfg(not(feature = "7z"))]
pub fn extract_from_7z(
    _archive_path: &Path,
    _entry_path: &str,
    _output_path: &Path,
) -> Result<(), ParseError> {
    Err(ParseError::ParseFailed("7z extraction not enabled".into()))
}
