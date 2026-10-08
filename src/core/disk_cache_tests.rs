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

    std::thread::sleep(std::time::Duration::from_millis(15));
    fs::write(&sample_file, b"version 2 updated content").unwrap();

    let key2 = compute_cache_key("office", &sample_file).unwrap();
    assert_ne!(key1, key2);
}

#[test]
fn test_atomic_create_and_get_cached_file() {
    let temp_cache_dir = tempdir().unwrap();
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

    // Check index file was saved
    let index_file = index_file_path();
    assert!(index_file.exists());

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

    // Verify category dir contains no leftover .tmp files
    let cat_dir = category_dir("office");
    if cat_dir.exists() {
        let entries: Vec<_> = fs::read_dir(&cat_dir).unwrap().flatten().collect();
        assert!(entries.is_empty());
    }
}

#[test]
fn test_startup_reconciliation_cleans_orphaned_and_missing_and_unindexed() {
    let temp_cache_dir = tempdir().unwrap();
    let cache_root = temp_cache_dir.path();
    unsafe {
        std::env::set_var("KGLANCE_CACHE_DIR", cache_root.to_str().unwrap());
    }

    let source_dir = tempdir().unwrap();
    let src_active = source_dir.path().join("active.docx");
    let src_deleted = source_dir.path().join("deleted.docx");
    fs::write(&src_active, b"active").unwrap();
    fs::write(&src_deleted, b"to be deleted").unwrap();

    let cached_active = put_bytes("office", &src_active, "pdf", b"pdf active").unwrap();
    let cached_deleted = put_bytes("office", &src_deleted, "pdf", b"pdf deleted").unwrap();

    // Create an unindexed rogue file on disk
    let rogue_file = category_dir("office").join("unindexed_file.bin");
    fs::write(&rogue_file, b"rogue garbage").unwrap();

    // Create a stale temp file
    let stale_tmp = category_dir("office").join(".tmp_123_456");
    fs::write(&stale_tmp, b"stale tmp").unwrap();

    // Now delete the source file of cached_deleted
    let _ = fs::remove_file(&src_deleted);

    // Run reconciliation
    access_index(|index| {
        reconcile_index_with_disk(index, cache_root, true);
    });

    assert!(cached_active.exists(), "Active cache file must stay");
    assert!(
        !cached_deleted.exists(),
        "Orphaned cache file must be deleted from disk"
    );
    assert!(
        !rogue_file.exists(),
        "Unindexed file must be cleaned from disk"
    );
    assert!(!stale_tmp.exists(), "Stale temp file must be cleaned");

    access_index(|index| {
        assert_eq!(index.entries.len(), 1);
        assert_eq!(index.total_bytes, b"pdf active".len() as u64);
    });
}

#[test]
fn test_prune_does_not_run_under_80_percent_threshold() {
    let temp_cache_dir = tempdir().unwrap();
    unsafe {
        std::env::set_var("KGLANCE_CACHE_DIR", temp_cache_dir.path().to_str().unwrap());
    }

    let source_dir = tempdir().unwrap();
    let src1 = source_dir.path().join("file1.docx");
    fs::write(&src1, b"1").unwrap();

    let path1 = put_bytes("office", &src1, "pdf", &[0u8; 500]).unwrap();
    assert!(path1.exists());

    // Budget = 1000 bytes. 500 bytes is 50% <= 80%.
    let remaining = prune_to_budget(1000).unwrap();
    assert_eq!(remaining, 500);
    assert!(path1.exists(), "File should NOT be pruned under 80%");
}

#[test]
fn test_prune_two_stage_orphaned_then_lru() {
    let temp_cache_dir = tempdir().unwrap();
    unsafe {
        std::env::set_var("KGLANCE_CACHE_DIR", temp_cache_dir.path().to_str().unwrap());
    }

    let source_dir = tempdir().unwrap();
    let src_orphan = source_dir.path().join("orphan.docx");
    let src_old = source_dir.path().join("old.docx");
    let src_new = source_dir.path().join("new.docx");

    fs::write(&src_orphan, b"orphan").unwrap();
    fs::write(&src_old, b"old").unwrap();
    fs::write(&src_new, b"new").unwrap();

    let path_orphan = put_bytes("office", &src_orphan, "pdf", &[0u8; 400]).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(20));
    let path_old = put_bytes("office", &src_old, "pdf", &[0u8; 300]).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(20));
    let path_new = put_bytes("office", &src_new, "pdf", &[0u8; 300]).unwrap();

    // Total size = 1000 bytes. Budget = 1000 bytes. Threshold 80% = 800 bytes.
    // 1000 > 800 -> Triggers prune.
    // First, delete source of orphan:
    let _ = fs::remove_file(&src_orphan);

    // Stage 1 will delete orphan (400 bytes). Remaining = 600 bytes <= 800 bytes (80%).
    // So Stage 2 (LRU) won't even need to touch path_old!
    let remaining = prune_to_budget(1000).unwrap();
    assert_eq!(remaining, 600);

    assert!(!path_orphan.exists(), "Orphaned cache file must be removed");
    assert!(
        path_old.exists(),
        "Old file kept because orphan cleanup dropped under 80%"
    );
    assert!(path_new.exists(), "New file kept");

    // Now test Stage 2 LRU: Add more data to push past 80% without orphans
    let src_extra = source_dir.path().join("extra.docx");
    fs::write(&src_extra, b"extra").unwrap();
    let path_extra = put_bytes("office", &src_extra, "pdf", &[0u8; 400]).unwrap();

    // Total = 600 + 400 = 1000 bytes > 800 (80%).
    // Budget = 1000, target 70% = 700 bytes.
    // path_old (300 bytes) should be deleted, bringing total to 700.
    let remaining2 = prune_to_budget(1000).unwrap();
    assert!(remaining2 <= 700);

    assert!(!path_old.exists(), "Oldest file removed in Stage 2 LRU");
    assert!(path_new.exists());
    assert!(path_extra.exists());
}
