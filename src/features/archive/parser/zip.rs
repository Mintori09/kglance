use std::fs::File;
use std::path::Path;

use crate::features::{archive::types::ArchiveEntry, common::parser::traits::ParseError};

pub fn list_zip_entries(archive_path: &Path) -> Result<Vec<ArchiveEntry>, ParseError> {
    let file = File::open(archive_path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let archive_path_str = archive_path.to_string_lossy().to_string();

    let mut entries = Vec::with_capacity(archive.len());
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        let modified = entry
            .last_modified()
            .map(|date_time| {
                format!(
                    "{:04}-{:02}-{:02} {:02}:{:02}",
                    date_time.year(),
                    date_time.month(),
                    date_time.day(),
                    date_time.hour(),
                    date_time.minute()
                )
            })
            .unwrap_or_default();

        entries.push(ArchiveEntry {
            path: entry.name().to_string(),
            size: entry.size(),
            is_dir: entry.is_dir(),
            modified,
            archive_path: archive_path_str.clone(),
        });
    }
    Ok(entries)
}

pub fn extract_from_zip(
    archive_path: &Path,
    entry_path: &str,
    output_path: &Path,
) -> Result<(), ParseError> {
    let file = File::open(archive_path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| ParseError::ParseFailed(e.to_string()))?;

    let mut entry = archive
        .by_name(entry_path)
        .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let mut output_file =
        File::create(output_path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;

    std::io::copy(&mut entry, &mut output_file)
        .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    Ok(())
}
