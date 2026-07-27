use super::*;
use std::fs::File;
use std::io::Write;

#[test]
fn parses_zip_archive() {
    use ::zip::write::SimpleFileOptions;

    let tmp_dir = tempfile::tempdir().unwrap();
    let zip_path = tmp_dir.path().join("test.zip");
    {
        let file = File::create(&zip_path).unwrap();
        let mut zip = ::zip::ZipWriter::new(file);
        let options = SimpleFileOptions::default();
        zip.add_directory("adir/", options).unwrap();
        zip.start_file("test.txt", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"hello there").unwrap();
        zip.finish().unwrap();
    }

    let parser = ArchiveParser;
    let result = parser.parse(&zip_path).unwrap();
    match result {
        ParsedContent::Archive {
            entries,
            total_files,
        } => {
            assert_eq!(total_files, 2);
            assert!(entries.iter().any(|entry| entry.path == "adir/"));
            assert!(entries.iter().any(|entry| entry.path == "test.txt"));
        }
        _ => panic!("expected Archive variant"),
    }
}

#[test]
fn parses_tar_archive() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let tar_path = tmp_dir.path().join("test.tar");
    {
        let file = File::create(&tar_path).unwrap();
        let mut builder = ::tar::Builder::new(file);

        let mut dir_header = ::tar::Header::new_gnu();
        dir_header.set_path("adir/").unwrap();
        dir_header.set_entry_type(::tar::EntryType::Directory);
        dir_header.set_size(0);
        dir_header.set_cksum();
        builder.append(&dir_header, &[] as &[u8]).unwrap();

        let mut file_header = ::tar::Header::new_gnu();
        file_header.set_path("test.txt").unwrap();
        file_header.set_size(5);
        file_header.set_entry_type(::tar::EntryType::Regular);
        file_header.set_cksum();
        builder.append(&file_header, b"hello" as &[u8]).unwrap();

        builder.finish().unwrap();
    }

    let parser = ArchiveParser;
    let result = parser.parse(&tar_path).unwrap();
    match result {
        ParsedContent::Archive {
            entries,
            total_files,
        } => {
            assert_eq!(total_files, 2);
            assert!(entries.iter().any(|entry| entry.path == "adir/"));
            assert!(entries.iter().any(|entry| entry.path == "test.txt"));
        }
        _ => panic!("expected Archive variant"),
    }
}

#[test]
fn supports_archive_extensions() {
    let parser = ArchiveParser;
    assert!(parser.is_supported(Path::new("a.zip")));
    assert!(parser.is_supported(Path::new("a.tar")));
    assert!(parser.is_supported(Path::new("a.7z")));
    assert!(parser.is_supported(Path::new("a.gz")));
    assert!(parser.is_supported(Path::new("a.tgz")));
    assert!(!parser.is_supported(Path::new("a.pdf")));
    assert!(!parser.is_supported(Path::new("a.txt")));
}

#[test]
fn rejects_unsupported_extension() {
    let parser = ArchiveParser;
    let tmp_dir = tempfile::tempdir().unwrap();
    let path = tmp_dir.path().join("test.xyz");
    std::fs::write(&path, b"data").unwrap();
    let result = parser.parse(&path);
    assert!(matches!(result, Err(ParseError::UnsupportedFormat)));
}

#[test]
fn rejects_invalid_zip() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let path = tmp_dir.path().join("test.zip");
    std::fs::write(&path, b"not a zip file").unwrap();
    let parser = ArchiveParser;
    let result = parser.parse(&path);
    assert!(result.is_err());
}

#[cfg(feature = "7z")]
#[test]
fn parses_7z_archive() {
    use sevenz_rust::{SevenZArchiveEntry, SevenZWriter};

    let tmp_dir = tempfile::tempdir().unwrap();
    let sz_path = tmp_dir.path().join("test.7z");
    {
        let mut writer = SevenZWriter::create(&sz_path).unwrap();
        let mut entry = SevenZArchiveEntry::default();
        entry.name = "test.txt".into();
        entry.has_stream = true;
        writer
            .push_archive_entry(entry, Some(b"hello" as &[u8]))
            .unwrap();
        writer.finish().unwrap();
    }

    let parser = ArchiveParser;
    let result = parser.parse(&sz_path).unwrap();
    match result {
        ParsedContent::Archive {
            entries,
            total_files,
        } => {
            assert_eq!(total_files, 1);
            assert_eq!(entries[0].path, "test.txt");
        }
        _ => panic!("expected Archive variant"),
    }
}
