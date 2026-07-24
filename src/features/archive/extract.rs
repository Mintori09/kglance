use std::path::Path;

use crate::parsers::ParseError;

pub struct ExtractedFile {
    #[allow(dead_code)]
    dir: tempfile::TempDir,
    pub path: std::path::PathBuf,
}

pub fn extract_entry(archive_path: &Path, entry_path: &str) -> Result<ExtractedFile, ParseError> {
    let ext = archive_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let dir = tempfile::tempdir().map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let out_path = dir.path().join(entry_path);

    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    }

    match ext {
        "zip" => extract_from_zip(archive_path, entry_path, &out_path)?,
        "7z" => extract_from_7z(archive_path, entry_path, &out_path)?,
        _ => extract_from_tar(archive_path, entry_path, &out_path)?,
    }

    Ok(ExtractedFile {
        dir,
        path: out_path,
    })
}

fn extract_from_zip(
    archive_path: &Path,
    entry_path: &str,
    out_path: &Path,
) -> Result<(), ParseError> {
    let file =
        std::fs::File::open(archive_path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let mut entry = archive
        .by_name(entry_path)
        .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let mut out =
        std::fs::File::create(out_path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    std::io::copy(&mut entry, &mut out).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    Ok(())
}

fn extract_from_tar(
    archive_path: &Path,
    entry_path: &str,
    out_path: &Path,
) -> Result<(), ParseError> {
    let file =
        std::fs::File::open(archive_path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let mut archive = tar::Archive::new(file);
    let entries = archive
        .entries()
        .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    for result in entries {
        let mut entry = result.map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        let name = entry
            .path()
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?
            .to_string_lossy()
            .to_string();
        if name == entry_path {
            let mut out = std::fs::File::create(out_path)
                .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
            std::io::copy(&mut entry, &mut out)
                .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
            return Ok(());
        }
    }
    Err(ParseError::ParseFailed("entry not found in tar".into()))
}

#[cfg(feature = "7z")]
fn extract_from_7z(
    archive_path: &Path,
    entry_path: &str,
    out_path: &Path,
) -> Result<(), ParseError> {
    use sevenz_rust::{Password, SevenZReader};
    use std::io::Read;

    let file =
        std::fs::File::open(archive_path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let len = file
        .metadata()
        .map_err(|e| ParseError::ParseFailed(e.to_string()))?
        .len();
    let mut reader = SevenZReader::new(file, len, Password::empty())
        .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let entry_path_owned = entry_path.to_string();
    let out_path_owned = out_path.to_owned();
    let mut found = false;
    reader
        .for_each_entries(|entry, entry_reader: &mut dyn Read| {
            if entry.name() == entry_path_owned {
                if let Some(parent) = out_path_owned.parent() {
                    std::fs::create_dir_all(parent).ok();
                }
                let mut out =
                    std::fs::File::create(&out_path_owned).map_err(sevenz_rust::Error::io)?;
                if entry.size() > 0 {
                    std::io::copy(entry_reader, &mut out).map_err(sevenz_rust::Error::io)?;
                }
                found = true;
            }
            Ok(true)
        })
        .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    if found {
        Ok(())
    } else {
        Err(ParseError::ParseFailed("entry not found in 7z".into()))
    }
}

#[cfg(not(feature = "7z"))]
fn extract_from_7z(
    _archive_path: &Path,
    _entry_path: &str,
    _out_path: &Path,
) -> Result<(), ParseError> {
    Err(ParseError::ParseFailed("7z extraction not enabled".into()))
}
