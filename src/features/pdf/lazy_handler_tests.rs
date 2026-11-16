use super::*;
use iced::futures::StreamExt;

#[test]
fn test_window_range_filtering() {
    let range = WindowRange::new(10, 100, 2); // 8..=12
    assert!(range.contains(8));
    assert!(range.contains(12));
    assert!(!range.contains(7));
    assert!(!range.contains(13));
}

#[tokio::test]
async fn test_process_page_loading_renders_subsequent_pages_on_scroll() {
    let pdf_path = "/home/mintori/Downloads/Nghệ Thuật Tư Duy Ngược Dòng - Humphrey Bancroft Neill & Thùy Dương (dịch).pdf";
    if !std::path::Path::new(pdf_path).exists() {
        return;
    }

    let (tx, mut rx) = iced::futures::channel::mpsc::channel(16);
    let visible_page = Arc::new(AtomicUsize::new(0));
    let generation_id = Arc::new(AtomicUsize::new(1));

    let make_msg =
        |idx: usize, _data: PageData, _text: Option<crate::features::pdf::PdfPageText>| {
            Message::Pdf(crate::app::messages::PdfMsg::PageReady(
                idx,
                Vec::new(),
                100,
                100,
                None,
            ))
        };
    let done_msg = Message::Pdf(crate::app::messages::PdfMsg::PagesLoaded(Vec::new()));

    let v_clone = visible_page.clone();
    let g_clone = generation_id.clone();
    let handle = tokio::spawn(async move {
        process_page_loading(
            tx,
            pdf_path.to_string(),
            297,
            v_clone,
            g_clone,
            None,
            make_msg,
            done_msg,
        )
        .await;
    });

    let mut initial_pages = Vec::new();

    // Read the initial window pages (0..=3)
    let timeout = tokio::time::Duration::from_millis(1500);
    while let Ok(Some(msg)) = tokio::time::timeout(timeout, rx.next()).await {
        if let Message::Pdf(crate::app::messages::PdfMsg::PageReady(idx, ..)) = msg {
            initial_pages.push(idx);
            if initial_pages.len() >= 3 {
                break;
            }
        }
    }

    assert!(initial_pages.contains(&1) || initial_pages.contains(&2) || initial_pages.contains(&3));
    assert!(!initial_pages.contains(&10));

    // Now simulate scrolling to page 11 (index 10)
    visible_page.store(10, Ordering::Relaxed);

    // The background task should still be alive and render pages in window 7..=13!
    let mut scrolled_pages = Vec::new();
    while let Ok(Some(msg)) = tokio::time::timeout(timeout, rx.next()).await {
        if let Message::Pdf(crate::app::messages::PdfMsg::PageReady(idx, ..)) = msg {
            scrolled_pages.push(idx);
            if scrolled_pages.contains(&10) {
                break;
            }
        }
    }

    assert!(
        scrolled_pages.contains(&10),
        "Expected page 10 to be rendered after scrolling, got: {:?}",
        scrolled_pages
    );

    // Cancel generation to let task terminate cleanly
    generation_id.store(2, Ordering::Relaxed);
    let _ = handle.await;
}

#[tokio::test]
async fn test_process_page_loading_exits_on_generation_id_change() {
    let pdf_path = "/home/mintori/Downloads/Nghệ Thuật Tư Duy Ngược Dòng - Humphrey Bancroft Neill & Thùy Dương (dịch).pdf";
    if !std::path::Path::new(pdf_path).exists() {
        return;
    }

    let (tx, mut rx) = iced::futures::channel::mpsc::channel(16);
    let visible_page = Arc::new(AtomicUsize::new(0));
    let generation_id = Arc::new(AtomicUsize::new(1));

    let make_msg =
        |idx: usize, _data: PageData, _text: Option<crate::features::pdf::PdfPageText>| {
            Message::Pdf(crate::app::messages::PdfMsg::PageReady(
                idx,
                Vec::new(),
                100,
                100,
                None,
            ))
        };
    let done_msg = Message::Pdf(crate::app::messages::PdfMsg::PagesLoaded(Vec::new()));

    let v_clone = visible_page.clone();
    let g_clone = generation_id.clone();
    let handle = tokio::spawn(async move {
        process_page_loading(
            tx,
            pdf_path.to_string(),
            297,
            v_clone,
            g_clone,
            None,
            make_msg,
            done_msg,
        )
        .await;
    });

    // Wait for at least one page to be received so expected_gen is established
    let first = tokio::time::timeout(tokio::time::Duration::from_millis(2000), rx.next()).await;
    assert!(
        matches!(first, Ok(Some(_))),
        "Expected at least one page ready message"
    );

    // Invalidate generation
    generation_id.store(999, Ordering::Relaxed);

    // Task must finish promptly
    let res = tokio::time::timeout(tokio::time::Duration::from_millis(2000), handle).await;
    assert!(
        res.is_ok(),
        "Task must terminate when generation_id changes"
    );

    // Channel should close
    while rx.next().await.is_some() {}
}
