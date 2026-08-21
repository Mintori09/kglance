use kglance::core::types::{InsertResult, PageCache, PageCacheEntry, ThumbnailCache};

fn fake_entry(width: u32, height: u32) -> PageCacheEntry {
    PageCacheEntry {
        width,
        height,
        handle: iced::widget::image::Handle::from_rgba(1, 1, vec![0; 4]),
    }
}

#[test]
fn test_page_cache_enforces_count_limit_and_tracks_bytes() {
    let mut cache = PageCache::new(50);

    for i in 0..20 {
        let entry = fake_entry(10, 10);
        let res = cache.insert(i, entry, 25);
        assert_eq!(res, InsertResult::Inserted);
        assert!(cache.count() <= PageCache::MAX_COUNT);
        assert_eq!(
            cache.accounted_decoded_bytes(),
            cache.compute_actual_decoded_bytes()
        );
    }
}

#[test]
fn test_page_cache_enforces_byte_budget() {
    let mut cache = PageCache::new(50);

    // 4 entries of 2000x2000 (16 MiB decoded each) = 64 MiB > 48 MiB budget
    for i in 0..4 {
        let entry = fake_entry(2000, 2000);
        let res = cache.insert(i, entry, 3);
        assert_eq!(res, InsertResult::Inserted);
    }

    assert!(cache.accounted_decoded_bytes() <= PageCache::MAX_BYTES);
    assert_eq!(
        cache.accounted_decoded_bytes(),
        cache.compute_actual_decoded_bytes()
    );
}

#[test]
fn test_page_cache_rejects_oversized_single_page() {
    let mut cache = PageCache::new(50);
    // 4000x4000 = 64 MiB > 48 MiB MAX_BYTES
    let oversized = fake_entry(4000, 4000);
    let res = cache.insert(0, oversized, 0);
    assert_eq!(res, InsertResult::RejectedOversized);
    assert_eq!(cache.count(), 0);
    assert_eq!(cache.accounted_decoded_bytes(), 0);
}

#[test]
fn test_page_cache_eviction_tie_breaking() {
    let mut cache = PageCache::new(50);

    // Populate 8 items around anchor 10
    for &idx in &[9, 11, 8, 12, 7, 13, 6, 14] {
        cache.insert(idx, fake_entry(10, 10), 10);
    }
    assert_eq!(cache.count(), 8);

    // Inserting 10 (dist 0) forces eviction of furthest (dist 4: 6 vs 14).
    // On tie, higher index (14) is evicted first, preserving smaller index (6).
    cache.insert(10, fake_entry(10, 10), 10);
    assert_eq!(cache.count(), 8);
    assert!(cache.get(10).is_some());
    assert!(cache.get(6).is_some()); // distance 4, smaller index -> preserved
    assert!(cache.get(14).is_none()); // distance 4, larger index -> evicted
}

#[test]
fn test_thumbnail_cache_enforces_count_limit() {
    let mut cache = ThumbnailCache::new(50);

    for i in 0..35 {
        let entry = fake_entry(50, 50);
        let res = cache.insert(i, entry, 20);
        assert_eq!(res, InsertResult::Inserted);
        assert!(cache.count() <= ThumbnailCache::MAX_COUNT);
    }
}

#[test]
fn test_page_cache_randomized_stress_1000_operations() {
    let mut rng_state: u64 = 54321;
    let mut pseudo_rand = || {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (rng_state >> 33) as f32 / 2147483648.0
    };

    let mut cache = PageCache::new(180);

    for _ in 0..1000 {
        let page_idx = (pseudo_rand() * 180.0) as usize;
        let anchor = (pseudo_rand() * 180.0) as usize;
        let side = 100 + (pseudo_rand() * 1200.0) as u32;
        let entry = fake_entry(side, side);

        cache.insert(page_idx, entry, anchor);

        assert!(cache.count() <= PageCache::MAX_COUNT);
        assert!(cache.accounted_decoded_bytes() <= PageCache::MAX_BYTES);
        assert_eq!(
            cache.accounted_decoded_bytes(),
            cache.compute_actual_decoded_bytes()
        );
    }
}

#[test]
fn test_disk_cache_atomic_write_and_metadata_validation() {
    let disk_cache = kglance::features::pdf::PdfDiskCache::new(12345).unwrap();
    let mut sample_png = Vec::new();
    sample_png.extend_from_slice(kglance::features::pdf::cache::PNG_SIGNATURE);
    sample_png.extend_from_slice(b"valid_payload_stream");

    disk_cache
        .save_page_with_meta(42, &sample_png, 1024, 768)
        .unwrap();
    assert!(disk_cache.has_page(42));

    let loaded = disk_cache.load_page_with_meta(42).unwrap();
    assert_eq!(loaded.width, 1024);
    assert_eq!(loaded.height, 768);
    assert_eq!(loaded.png_bytes, sample_png);
}
