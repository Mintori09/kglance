use std::fs::File;
use std::path::Path;

use crate::{
    core::utils::format_timestamp,
    features::{archive::types::ArchiveEntry, common::parser::traits::ParseError},
};
pub fn list_tar_entries(archive_path: &Path) -> Result<Vec<ArchiveEntry>, ParseError> {
    let file = File::open(archive_path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let mut archive = tar::Archive::new(file);
    let archive_path_str = archive_path.to_string_lossy().to_string();

    let tar_entries = archive
        .entries()
        .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

    let mut entries = Vec::new();
    for result in tar_entries {
        let entry = result.map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        let entry_path = entry
            .path()
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?
            .to_string_lossy()
            .to_string();

        let modification_timestamp = entry.header().mtime().unwrap_or(0);
        let modified = format_timestamp(modification_timestamp);

        entries.push(ArchiveEntry {
            path: entry_path,
            size: entry.size(),
            is_dir: entry.header().entry_type().is_dir(),
            modified,
            archive_path: archive_path_str.clone(),
        });
    }
    Ok(entries)
}

pub fn extract_from_tar(
    archive_path: &Path,
    entry_path: &str,
    output_path: &Path,
) -> Result<(), ParseError> {
    let file = File::open(archive_path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let mut archive = tar::Archive::new(file);

    let entries = archive
        .entries()
        .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

    for result in entries {
        let mut entry = result.map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        let current_path = entry
            .path()
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?
            .to_string_lossy()
            .to_string();

        if current_path == entry_path {
            let mut output_file =
                File::create(output_path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
            std::io::copy(&mut entry, &mut output_file)
                .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
            return Ok(());
        }
    }

    Err(ParseError::ParseFailed("entry not found in tar".into()))
}
