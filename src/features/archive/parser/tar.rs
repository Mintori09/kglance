use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use super::MAX_EXTRACT_BYTES;
use crate::{
    core::utils::format_timestamp,
    features::{archive::types::ArchiveEntry, common::parser::traits::ParseError},
};

fn open_tar_reader(archive_path: &Path) -> Result<Box<dyn Read>, ParseError> {
    let mut file = File::open(archive_path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let mut magic = [0u8; 6];
    let bytes_read = file.read(&mut magic).unwrap_or(0);
    file.seek(SeekFrom::Start(0))
        .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

    let reader: Box<dyn Read> = if bytes_read >= 2 && magic[0..2] == [0x1F, 0x8B] {
        Box::new(flate2::read::MultiGzDecoder::new(BufReader::new(file)))
    } else if bytes_read >= 6 && magic[0..6] == [0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00] {
        Box::new(lzma_rust2::XzReader::new(BufReader::new(file), true))
    } else if bytes_read >= 4 && magic[0..4] == [0x28, 0xB5, 0x2F, 0xFD] {
        Box::new(
            zstd::stream::read::Decoder::new(file)
                .map_err(|e| ParseError::ParseFailed(e.to_string()))?,
        )
    } else {
        Box::new(BufReader::new(file))
    };

    Ok(reader)
}

pub fn list_tar_entries(archive_path: &Path) -> Result<Vec<ArchiveEntry>, ParseError> {
    let reader = open_tar_reader(archive_path)?;
    let mut archive = tar::Archive::new(reader);
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
    let reader = open_tar_reader(archive_path)?;
    let mut archive = tar::Archive::new(reader);

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
            let mut limited_reader = (&mut entry).take(MAX_EXTRACT_BYTES + 1);
            let copied_bytes = std::io::copy(&mut limited_reader, &mut output_file)
                .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

            if copied_bytes > MAX_EXTRACT_BYTES {
                let _ = std::fs::remove_file(output_path);
                return Err(ParseError::TooLarge);
            }
            return Ok(());
        }
    }

    Err(ParseError::ParseFailed("entry not found in tar".into()))
}
