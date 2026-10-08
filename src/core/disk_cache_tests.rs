use super::*;
use tempfile::tempdir;

#[test]
fn test_compute_cache_key_deterministic() {
    let dir = tempdir().unwrap();
    let sample_file = dir.path().join("sample.docx");
    fs::write(&sample_file, b"sample content docx").unwrap();

    let key1 = compute_cache_key("office", &sample_file);
    let key2 = compute_cache_key("office", &sample_file);

    assert!(key1.is_some());
    assert_eq!(key1, key2);

    let key_other_cat = compute_cache_key("media", &sample_file);
    assert_ne!(key1, key_other_cat);
}

#[test]
fn test_compute_cache_key_invalidates_on_change() {
    let dir = tempdir().unwrap();
    let sample_file = dir.path().join("doc.docx");
    fs::write(&sample_file, b"version 1").unwrap();

    let key1 = compute_cache_key("office", &sample_file).unwrap();

    // Modify file content and size
    std::thread::sleep(std::time::Duration::from_millis(15));
    fs::write(&sample_file, b"version 2 updated content").unwrap();

    let key2 = compute_cache_key("office", &sample_file).unwrap();
    assert_ne!(key1, key2);
}

#[test]
fn test_atomic_create_and_get_cached_file() {
    let temp_cache_dir = tempdir().unwrap();
    // Safety for tests: point cache directory to temp_cache_dir
    unsafe {
        std::env::set_var("KGLANCE_CACHE_DIR", temp_cache_dir.path().to_str().unwrap());
    }

    let source_dir = tempdir().unwrap();
    let source_file = source_dir.path().join("presentation.pptx");
    fs::write(&source_file, b"presentation binary data").unwrap();

    // Initial check: cache miss
    assert!(get_cached_path("office", &source_file, "pdf").is_none());

    // Create cached file
    let cached = create_cached_file("office", &source_file, "pdf", |temp_path| {
        fs::write(temp_path, b"%PDF-1.4 sample converted pdf")
    })
    .unwrap();

    assert!(cached.exists());
    assert_eq!(cached.extension().and_then(|e| e.to_str()), Some("pdf"));

    // Second check: cache hit
    let hit = get_cached_path("office", &source_file, "pdf");
    assert_eq!(hit, Some(cached.clone()));

    let content = fs::read(cached).unwrap();
    assert_eq!(content, b"%PDF-1.4 sample converted pdf");
}

#[test]
fn test_put_bytes_and_get_cached_path() {
    let temp_cache_dir = tempdir().unwrap();
    unsafe {
        std::env::set_var("KGLANCE_CACHE_DIR", temp_cache_dir.path().to_str().unwrap());
    }

    let source_dir = tempdir().unwrap();
    let source_video = source_dir.path().join("video.mp4");
    fs::write(&source_video, b"fake video bytes").unwrap();

    let thumb_data = b"\xFF\xD8\xFF\xE0 fake jpeg thumbnail";
    let stored = put_bytes("media", &source_video, "jpg", thumb_data).unwrap();

    assert!(stored.exists());
    let hit = get_cached_path("media", &source_video, "jpg");
    assert_eq!(hit, Some(stored));
}

#[test]
fn test_atomic_write_cleans_up_on_failure() {
    let temp_cache_dir = tempdir().unwrap();
    unsafe {
        std::env::set_var("KGLANCE_CACHE_DIR", temp_cache_dir.path().to_str().unwrap());
    }

    let source_dir = tempdir().unwrap();
    let source_file = source_dir.path().join("corrupt.docx");
    fs::write(&source_file, b"docx data").unwrap();

    let res = create_cached_file("office", &source_file, "pdf", |temp_path| {
        fs::write(temp_path, b"partial data").unwrap();
        Err(io::Error::other("conversion error"))
    });

    assert!(res.is_err());
    assert!(get_cached_path("office", &source_file, "pdf").is_none());

    // Verify temp directory contains no leftover .tmp files
    let cat_dir = category_dir("office");
    if cat_dir.exists() {
        let entries: Vec<_> = fs::read_dir(&cat_dir).unwrap().flatten().collect();
        assert!(entries.is_empty());
    }
}

#[test]
fn test_prune_to_budget_removes_oldest() {
    let temp_cache_dir = tempdir().unwrap();
    unsafe {
        std::env::set_var("KGLANCE_CACHE_DIR", temp_cache_dir.path().to_str().unwrap());
    }

    let source_dir = tempdir().unwrap();
    let src1 = source_dir.path().join("file1.docx");
    let src2 = source_dir.path().join("file2.docx");
    let src3 = source_dir.path().join("file3.docx");

    fs::write(&src1, b"1").unwrap();
    fs::write(&src2, b"2").unwrap();
    fs::write(&src3, b"3").unwrap();

    let path1 = put_bytes("office", &src1, "pdf", &[0u8; 1000]).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(20));
    let path2 = put_bytes("office", &src2, "pdf", &[0u8; 1000]).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(20));
    let path3 = put_bytes("office", &src3, "pdf", &[0u8; 1000]).unwrap();

    assert!(path1.exists());
    assert!(path2.exists());
    assert!(path3.exists());

    // Total size ~ 3000 bytes. Prune to budget of 1500 bytes.
    // Target is 85% of 1500 = 1275 bytes.
    // path1 (oldest) and path2 should be removed, path3 kept.
    let remaining = prune_to_budget(1500).unwrap();
    assert!(remaining <= 1500);

    assert!(!path1.exists());
    assert!(path3.exists());
}
