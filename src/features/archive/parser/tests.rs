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
    assert!(parser.is_supported(Path::new("a.xz")));
    assert!(parser.is_supported(Path::new("a.txz")));
    assert!(parser.is_supported(Path::new("a.zst")));
    assert!(parser.is_supported(Path::new("a.tzst")));
    assert!(parser.is_supported(Path::new("a.tar.zst")));
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

#[test]
fn extract_entry_rejects_path_traversal() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let zip_path = tmp_dir.path().join("traversal.zip");
    {
        use ::zip::write::SimpleFileOptions;

        let file = File::create(&zip_path).unwrap();
        let mut zip = ::zip::ZipWriter::new(file);
        zip.start_file("normal.txt", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"safe").unwrap();
        zip.finish().unwrap();
    }

    let traversal_paths = [
        "../evil.txt",
        "../../etc/passwd",
        "/etc/passwd",
        "a/../../b.txt",
        "./../../secret",
    ];

    for path in traversal_paths {
        let result = extract_entry(&zip_path, path);
        assert!(
            matches!(result, Err(ParseError::ParseFailed(_))),
            "Expected traversal rejection for path: {path}"
        );
    }
}

#[test]
fn extract_entry_succeeds_for_valid_entry() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let zip_path = tmp_dir.path().join("valid.zip");
    {
        use ::zip::write::SimpleFileOptions;

        let file = File::create(&zip_path).unwrap();
        let mut zip = ::zip::ZipWriter::new(file);
        zip.start_file("docs/readme.txt", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"readme content").unwrap();
        zip.finish().unwrap();
    }

    let extracted = extract_entry(&zip_path, "docs/readme.txt").expect("should extract valid file");
    assert!(extracted.path.exists());
    let content = std::fs::read_to_string(&extracted.path).unwrap();
    assert_eq!(content, "readme content");
}

#[test]
fn parses_and_extracts_tar_gz_archive() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let tar_gz_path = tmp_dir.path().join("test.tar.gz");
    {
        let file = File::create(&tar_gz_path).unwrap();
        let enc = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut builder = ::tar::Builder::new(enc);

        let mut file_header = ::tar::Header::new_gnu();
        file_header.set_path("hello.txt").unwrap();
        file_header.set_size(12);
        file_header.set_entry_type(::tar::EntryType::Regular);
        file_header.set_cksum();
        builder
            .append(&file_header, b"hello tar gz" as &[u8])
            .unwrap();

        let enc = builder.into_inner().unwrap();
        enc.finish().unwrap();
    }

    let parser = ArchiveParser;
    let result = parser.parse(&tar_gz_path).unwrap();
    match result {
        ParsedContent::Archive {
            entries,
            total_files,
        } => {
            assert_eq!(total_files, 1);
            assert_eq!(entries[0].path, "hello.txt");
            assert_eq!(entries[0].size, 12);
        }
        _ => panic!("expected Archive variant"),
    }

    let extracted = extract_entry(&tar_gz_path, "hello.txt").expect("should extract tar.gz");
    assert!(extracted.path.exists());
    let content = std::fs::read_to_string(&extracted.path).unwrap();
    assert_eq!(content, "hello tar gz");
}

#[test]
fn parses_and_extracts_tar_xz_archive() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let tar_xz_path = tmp_dir.path().join("test.tar.xz");
    {
        let file = File::create(&tar_xz_path).unwrap();
        let mut xz_writer =
            lzma_rust2::XzWriter::new(file, lzma_rust2::XzOptions::default()).unwrap();
        {
            let mut builder = ::tar::Builder::new(&mut xz_writer);
            let mut file_header = ::tar::Header::new_gnu();
            file_header.set_path("nested/world.txt").unwrap();
            file_header.set_size(12);
            file_header.set_entry_type(::tar::EntryType::Regular);
            file_header.set_cksum();
            builder
                .append(&file_header, b"hello tar xz" as &[u8])
                .unwrap();
            builder.finish().unwrap();
        }
        xz_writer.finish().unwrap();
    }

    let parser = ArchiveParser;
    let result = parser.parse(&tar_xz_path).unwrap();
    match result {
        ParsedContent::Archive {
            entries,
            total_files,
        } => {
            assert_eq!(total_files, 1);
            assert_eq!(entries[0].path, "nested/world.txt");
            assert_eq!(entries[0].size, 12);
        }
        _ => panic!("expected Archive variant"),
    }

    let extracted = extract_entry(&tar_xz_path, "nested/world.txt").expect("should extract tar.xz");
    assert!(extracted.path.exists());
    let content = std::fs::read_to_string(&extracted.path).unwrap();
    assert_eq!(content, "hello tar xz");
}

#[test]
fn parses_and_extracts_tar_zst_archive() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let tar_zst_path = tmp_dir.path().join("test.tar.zst");
    {
        let file = File::create(&tar_zst_path).unwrap();
        let mut zst_encoder = zstd::stream::write::Encoder::new(file, 0).unwrap();
        {
            let mut builder = ::tar::Builder::new(&mut zst_encoder);
            let mut file_header = ::tar::Header::new_gnu();
            file_header.set_path("nested/zstd.txt").unwrap();
            file_header.set_size(13);
            file_header.set_entry_type(::tar::EntryType::Regular);
            file_header.set_cksum();
            builder
                .append(&file_header, b"hello tar zst" as &[u8])
                .unwrap();
            builder.finish().unwrap();
        }
        zst_encoder.finish().unwrap();
    }

    let parser = ArchiveParser;
    let result = parser.parse(&tar_zst_path).unwrap();
    match result {
        ParsedContent::Archive {
            entries,
            total_files,
        } => {
            assert_eq!(total_files, 1);
            assert_eq!(entries[0].path, "nested/zstd.txt");
            assert_eq!(entries[0].size, 13);
        }
        _ => panic!("expected Archive variant"),
    }

    let extracted =
        extract_entry(&tar_zst_path, "nested/zstd.txt").expect("should extract tar.zst");
    assert!(extracted.path.exists());
    let content = std::fs::read_to_string(&extracted.path).unwrap();
    assert_eq!(content, "hello tar zst");
}
