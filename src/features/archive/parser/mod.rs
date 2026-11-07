mod seven_zip;
mod tar;
mod zip;

#[cfg(test)]
mod tests;

use std::path::{Component, Path, PathBuf};

use crate::features::common::parser::{
    traits::{ParseError, PreviewParser},
    types::ParsedContent,
};

pub const MAX_EXTRACT_BYTES: u64 = 100 * 1024 * 1024; // 100 MB

pub struct ArchiveParser;

pub struct ExtractedFile {
    #[allow(dead_code)]
    dir: tempfile::TempDir,
    pub path: PathBuf,
}

impl PreviewParser for ArchiveParser {
    fn supported_extensions(&self) -> &[&str] {
        &["zip", "tar", "gz", "tgz", "xz", "txz", "7z"]
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        let entries = match extension {
            "zip" => zip::list_zip_entries(path)?,
            "7z" => seven_zip::list_7z_entries(path)?,
            "tar" | "gz" | "tgz" | "xz" | "txz" => tar::list_tar_entries(path)?,
            _ => return Err(ParseError::UnsupportedFormat),
        };

        let total_files = entries.len();
        Ok(ParsedContent::Archive {
            entries,
            total_files,
        })
    }
}

pub fn extract_entry(archive_path: &Path, entry_path: &str) -> Result<ExtractedFile, ParseError> {
    let sanitized_path = Path::new(entry_path);
    if sanitized_path.is_absolute()
        || sanitized_path.components().any(|c| {
            matches!(
                c,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(ParseError::ParseFailed(
            "invalid or insecure archive entry path".into(),
        ));
    }

    let extension = archive_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    let temp_dir = tempfile::tempdir().map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let output_path = temp_dir.path().join(sanitized_path);

    if !output_path.starts_with(temp_dir.path()) {
        return Err(ParseError::ParseFailed(
            "invalid or insecure archive entry path".into(),
        ));
    }

    if let Some(parent_dir) = output_path.parent() {
        std::fs::create_dir_all(parent_dir).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    }

    match extension {
        "zip" => zip::extract_from_zip(archive_path, entry_path, &output_path)?,
        "7z" => seven_zip::extract_from_7z(archive_path, entry_path, &output_path)?,
        _ => tar::extract_from_tar(archive_path, entry_path, &output_path)?,
    }

    Ok(ExtractedFile {
        dir: temp_dir,
        path: output_path,
    })
}
