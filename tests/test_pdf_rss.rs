use kglance::core::preview::compute_pdf_page_offsets;
use kglance::core::types::PageCache;
use kglance::features::pdf::geometry::visible_page_range;
use kglance::features::pdf::parser::{empty_mupdf_store, render_pdf_pages_batch};
use std::path::Path;

#[test]
fn test_reference_pdf_memory_footprint() {
    let pdf_path = Path::new("/home/mintori/Desktop/Mimi_Kara_Oboeru_N3-Goi.pdf");
    if !pdf_path.exists() {
        println!("Reference PDF not found, skipping benchmark.");
        return;
    }

    let doc = mupdf::Document::open(pdf_path).expect("Open document");
    let page_count = doc.page_count().expect("Page count") as usize;
    println!("Reference PDF Page Count: {}", page_count);
    assert!(page_count >= 100);

    let dims =
        kglance::features::pdf::dimensions::extract_page_dimensions(&doc).expect("Page dims");
    let (offsets, ends, total_h) = compute_pdf_page_offsets(&dims, 800.0, 4.0);

    let mut cache = PageCache::new(page_count);

    // Simulate scrolling through 10 distinct viewport positions
    for step in 0..10 {
        let scroll_y = (step as f32 / 10.0) * (total_h - 800.0).max(0.0);
        let visible = visible_page_range(&offsets, &ends, scroll_y, 800.0).unwrap_or(0..=0);
        let anchor = *visible.start();

        let pages_to_fetch: Vec<usize> = visible.collect();
        let rendered = render_pdf_pages_batch(pdf_path, &pages_to_fetch);

        for (idx, res) in rendered {
            if let Ok(p) = res {
                let handle = iced::widget::image::Handle::from_bytes(p.data);
                cache.insert(
                    idx,
                    kglance::core::types::PageCacheEntry {
                        width: p.width,
                        height: p.height,
                        handle,
                    },
                    anchor,
                );
            }
        }
        empty_mupdf_store();

        assert!(cache.count() <= PageCache::MAX_COUNT);
        assert!(cache.accounted_decoded_bytes() <= PageCache::MAX_BYTES);
    }

    println!(
        "Cache after 10 scroll steps: count={}, accounted_bytes={:.2} MiB",
        cache.count(),
        cache.accounted_decoded_bytes() as f64 / (1024.0 * 1024.0)
    );
}
