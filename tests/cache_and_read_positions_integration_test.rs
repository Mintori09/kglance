use kglance::core::cache::CachedContent;
use kglance::core::preview::PreviewData;
use kglance::core::read_positions::{ReadPosition, ReadPositions};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;

#[test]
fn test_cache_lru_lifecycle_integration() {
    let mut cache: LruCache<String, CachedContent> = LruCache::new(NonZeroUsize::new(3).unwrap());

    let file_1 = "/path/to/file1.txt".to_string();
    let file_2 = "/path/to/file2.txt".to_string();
    let file_3 = "/path/to/file3.txt".to_string();
    let file_4 = "/path/to/file4.txt".to_string();

    let dummy_preview_1 = Arc::new(PreviewData::Text {
        content: "File 1 content".to_string(),
        line_numbers: "1".to_string(),
        language: "Plain Text".to_string(),
    });
    let dummy_preview_2 = Arc::new(PreviewData::Text {
        content: "File 2 content".to_string(),
        line_numbers: "1".to_string(),
        language: "Plain Text".to_string(),
    });
    let dummy_preview_3 = Arc::new(PreviewData::Text {
        content: "File 3 content".to_string(),
        line_numbers: "1".to_string(),
        language: "Plain Text".to_string(),
    });
    let dummy_preview_4 = Arc::new(PreviewData::Text {
        content: "File 4 content".to_string(),
        line_numbers: "1".to_string(),
        language: "Plain Text".to_string(),
    });

    cache.put(file_1.clone(), CachedContent::from_preview(dummy_preview_1));
    cache.put(file_2.clone(), CachedContent::from_preview(dummy_preview_2));
    cache.put(file_3.clone(), CachedContent::from_preview(dummy_preview_3));

    assert_eq!(cache.len(), 3);
    assert!(cache.contains(&file_1));

    // Access file_1 so file_2 becomes the least recently used
    let _ = cache.get(&file_1);

    // Insert file_4 (capacity is 3, so file_2 should be evicted)
    cache.put(file_4.clone(), CachedContent::from_preview(dummy_preview_4));

    assert_eq!(cache.len(), 3);
    assert!(
        cache.contains(&file_1),
        "file_1 should remain because it was accessed"
    );
    assert!(cache.contains(&file_3), "file_3 should remain");
    assert!(cache.contains(&file_4), "file_4 should be inserted");
    assert!(
        !cache.contains(&file_2),
        "file_2 should have been evicted by LRU policy"
    );
}

#[test]
fn test_read_positions_tracking_and_retrieval() {
    let mut positions = ReadPositions::default();

    let path_book = "/home/user/books/rust_in_action.epub".to_string();
    let path_doc = "/home/user/docs/architecture.md".to_string();

    positions.insert(
        path_book.clone(),
        ReadPosition {
            scroll_y: 1250.5,
            chapter: 3,
        },
    );

    positions.insert(
        path_doc.clone(),
        ReadPosition {
            scroll_y: 420.0,
            chapter: 0,
        },
    );

    let retrieved_book = positions.get(&path_book);
    assert!(retrieved_book.is_some());
    assert_eq!(retrieved_book.unwrap().chapter, 3);
    assert_eq!(retrieved_book.unwrap().scroll_y, 1250.5);

    let retrieved_doc = positions.get(&path_doc);
    assert!(retrieved_doc.is_some());
    assert_eq!(retrieved_doc.unwrap().chapter, 0);
    assert_eq!(retrieved_doc.unwrap().scroll_y, 420.0);

    let nonexistent = positions.get("/nonexistent/file.pdf");
    assert!(nonexistent.is_none());
}
