use super::*;
use crate::app::test_util::{epub_content, test_app};

#[test]
fn test_chapter_clicked_resets_smooth_scroll_completely() {
    let mut app = test_app(Some(epub_content(&["Ch 1", "Ch 2"])));
    app.state.epub.markdown_state.smooth_scroll.target_y = 400.0;
    app.state.epub.markdown_state.smooth_scroll.is_animating = true;
    app.state.epub.markdown_state.scroll_y = 200.0;

    let _ = handle_chapter_clicked(&mut app, 1);
    assert_eq!(app.state.epub.markdown_state.scroll_y, 0.0);
    assert_eq!(app.state.epub.markdown_state.smooth_scroll.target_y, 0.0);
    assert!(!app.state.epub.markdown_state.smooth_scroll.is_animating);
}

#[test]
fn test_handle_chapter_clicked_switches_active_chapter() {
    let mut app = test_app(Some(epub_content(&[
        "Chapter 1 content",
        "Chapter 2 content",
    ])));
    assert_eq!(app.state.epub.active_chapter, 0);

    let _ = handle_chapter_clicked(&mut app, 1);
    assert_eq!(app.state.epub.active_chapter, 1);
}

#[test]
fn test_handle_chapter_toggle_collapse() {
    let mut app = test_app(Some(epub_content(&["Ch 1"])));
    assert!(!app.state.epub.collapsed_chapters.contains(&0));

    let _ = handle_chapter_toggle_collapse(&mut app, 0);
    assert!(app.state.epub.collapsed_chapters.contains(&0));

    let _ = handle_chapter_toggle_collapse(&mut app, 0);
    assert!(!app.state.epub.collapsed_chapters.contains(&0));
}

#[test]
fn test_handle_chapter_clicked_already_loaded() {
    let mut app = test_app(Some(epub_content(&["Ch 1", "Ch 2"])));
    assert_eq!(app.state.epub.chapters[1].blocks.len(), 1);

    let _ = handle_chapter_clicked(&mut app, 1);
    assert_eq!(app.state.epub.active_chapter, 1);
    assert_eq!(app.state.epub.chapters[1].blocks.len(), 1);
}

#[test]
fn test_handle_chapter_clicked_loads_empty_chapter_from_disk() {
    use std::fs::File;
    use std::io::Write;

    let temp_dir = std::env::temp_dir();
    let test_epub_path = temp_dir.join("test_kglance_update_ondemand.epub");

    let file = File::create(&test_epub_path).unwrap();
    let mut zip = ::zip::ZipWriter::new(file);
    let options = ::zip::write::SimpleFileOptions::default();

    zip.start_file("META-INF/container.xml", options).unwrap();
    zip.write_all(r#"<?xml version="1.0"?><container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#.as_bytes()).unwrap();

    zip.start_file("OEBPS/content.opf", options).unwrap();
    zip.write_all(r#"<?xml version="1.0"?><package><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>Test</dc:title></metadata><manifest><item id="item1" href="ch1.xhtml" media-type="application/xhtml+xml"/><item id="item2" href="ch2.xhtml" media-type="application/xhtml+xml"/><item id="img2" href="images/ch2.png" media-type="image/png"/></manifest><spine><itemref idref="item1"/><itemref idref="item2"/></spine></package>"#.as_bytes()).unwrap();

    zip.start_file("OEBPS/ch1.xhtml", options).unwrap();
    zip.write_all(b"<html><body><h1>Chapter 1</h1><p>Text 1</p></body></html>")
        .unwrap();

    zip.start_file("OEBPS/ch2.xhtml", options).unwrap();
    zip.write_all(b"<html><body><h1 id=\"sec2\">Chapter 2</h1><p><img src=\"images/ch2.png\" alt=\"Img2\"/></p></body></html>").unwrap();

    // 1x1 transparent PNG
    let sample_png = vec![
        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6,
        0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 10, 73, 68, 65, 84, 120, 156, 99, 0, 1, 0, 0, 5, 0, 1,
        13, 10, 45, 180, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
    ];
    zip.start_file("OEBPS/images/ch2.png", options).unwrap();
    zip.write_all(&sample_png).unwrap();

    zip.finish().unwrap();

    let path_str = test_epub_path.to_string_lossy().to_string();

    let chapters = vec![
        crate::core::types::EpubChapterInfo {
            title: "Chapter 1".to_string(),
            level: 1,
            anchor: None,
            file_href: "ch1.xhtml".to_string(),
            blocks: vec![crate::parsers::markdown::Block::Paragraph(vec![
                crate::parsers::markdown::Inline::Text("Text 1".to_string()),
            ])],
        },
        crate::core::types::EpubChapterInfo {
            title: "Chapter 2".to_string(),
            level: 1,
            anchor: Some("sec2".to_string()),
            file_href: "ch2.xhtml".to_string(),
            blocks: Vec::new(),
        },
    ];

    let mut app = test_app(Some(crate::core::PreviewData::Epub {
        title: "Test".to_string(),
        author: "Author".to_string(),
        chapters: chapters.clone(),
        active_chapter: 0,
        images: std::collections::HashMap::new(),
    }));
    app.state.file_name = path_str.clone();
    app.state.epub.chapters = chapters;

    assert_eq!(app.state.epub.active_chapter, 0);
    assert!(app.state.epub.chapters[1].blocks.is_empty());

    // Click chapter 1 (index 1) which has empty blocks
    let _ = handle_chapter_clicked(&mut app, 1);

    assert_eq!(app.state.epub.active_chapter, 1);
    assert!(!app.state.epub.chapters[1].blocks.is_empty());

    // Verify images were loaded into PreviewData and cached handles populated in markdown_state
    if let Some(crate::core::PreviewData::Epub {
        images, chapters, ..
    }) = &app.current_content
    {
        assert!(images.contains_key("images/ch2.png") || images.contains_key("ch2.png"));
        assert!(!chapters[1].blocks.is_empty());
    } else {
        panic!("Expected PreviewData::Epub");
    }

    // Verify chapter image handles are prepared
    assert!(
        !app.state
            .epub
            .markdown_state
            .cached_image_handles
            .is_empty()
    );

    let _ = std::fs::remove_file(test_epub_path);
}

#[test]
fn test_restore_read_position_loads_lazy_chapter_and_restores_scroll() {
    use std::fs::File;
    use std::io::Write;

    let temp_dir = std::env::temp_dir();
    let test_epub_path = temp_dir.join("test_kglance_restore_read_pos.epub");

    let file = File::create(&test_epub_path).unwrap();
    let mut zip = ::zip::ZipWriter::new(file);
    let options = ::zip::write::SimpleFileOptions::default();

    zip.start_file("META-INF/container.xml", options).unwrap();
    zip.write_all(r#"<?xml version="1.0"?><container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#.as_bytes()).unwrap();

    zip.start_file("OEBPS/content.opf", options).unwrap();
    zip.write_all(r#"<?xml version="1.0"?><package><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>Test</dc:title></metadata><manifest><item id="item1" href="ch1.xhtml" media-type="application/xhtml+xml"/><item id="item2" href="ch2.xhtml" media-type="application/xhtml+xml"/></manifest><spine><itemref idref="item1"/><itemref idref="item2"/></spine></package>"#.as_bytes()).unwrap();

    zip.start_file("OEBPS/ch1.xhtml", options).unwrap();
    zip.write_all(b"<html><body><h1>Chapter 1</h1><p>Text 1</p></body></html>")
        .unwrap();

    zip.start_file("OEBPS/ch2.xhtml", options).unwrap();
    zip.write_all(b"<html><body><h1>Chapter 2</h1><p>Restored Text 2</p></body></html>")
        .unwrap();

    zip.finish().unwrap();

    let path_str = test_epub_path.to_string_lossy().to_string();

    let chapters = vec![
        crate::core::types::EpubChapterInfo {
            title: "Chapter 1".to_string(),
            level: 1,
            anchor: None,
            file_href: "ch1.xhtml".to_string(),
            blocks: vec![crate::parsers::markdown::Block::Paragraph(vec![
                crate::parsers::markdown::Inline::Text("Text 1".to_string()),
            ])],
        },
        crate::core::types::EpubChapterInfo {
            title: "Chapter 2".to_string(),
            level: 1,
            anchor: None,
            file_href: "ch2.xhtml".to_string(),
            blocks: Vec::new(),
        },
    ];

    let mut app = test_app(Some(crate::core::PreviewData::Epub {
        title: "Test".to_string(),
        author: "Author".to_string(),
        chapters: chapters.clone(),
        active_chapter: 0,
        images: std::collections::HashMap::new(),
    }));
    app.state.file_name = path_str.clone();
    app.state.epub.chapters = chapters;

    // Record a saved read position for chapter 1 with scroll_y = 120.0
    app.state.read_positions.insert(
        path_str.clone(),
        crate::core::ReadPosition {
            scroll_y: 120.0,
            chapter: 1,
            reading_mode: None,
        },
    );

    assert_eq!(app.state.epub.active_chapter, 0);
    assert!(app.state.epub.chapters[1].blocks.is_empty());

    // Restore read position for the epub file
    let _ = app.restore_read_position_for(&path_str);

    // Verify chapter 1 was loaded, active_chapter switched to 1, and scroll_y was restored
    assert_eq!(app.state.epub.active_chapter, 1);
    assert!(!app.state.epub.chapters[1].blocks.is_empty());
    assert_eq!(app.state.epub.markdown_state.scroll_y, 120.0);
    assert!(!app.state.epub.markdown_state.block_y_offsets.is_empty());

    let _ = std::fs::remove_file(test_epub_path);
}

#[test]
fn test_toc_multiple_hierarchy_anchor_navigation() {
    use std::fs::File;
    use std::io::Write;

    let temp_dir = std::env::temp_dir();
    let test_epub_path = temp_dir.join("test_kglance_toc_hierarchy.epub");

    let file = File::create(&test_epub_path).unwrap();
    let mut zip = ::zip::ZipWriter::new(file);
    let options = ::zip::write::SimpleFileOptions::default();

    zip.start_file("META-INF/container.xml", options).unwrap();
    zip.write_all(r#"<?xml version="1.0"?><container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#.as_bytes()).unwrap();

    zip.start_file("OEBPS/content.opf", options).unwrap();
    zip.write_all(r#"<?xml version="1.0"?><package><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>Test</dc:title></metadata><manifest><item id="item1" href="ch1.xhtml" media-type="application/xhtml+xml"/></manifest><spine><itemref idref="item1"/></spine></package>"#.as_bytes()).unwrap();

    zip.start_file("OEBPS/ch1.xhtml", options).unwrap();
    let xhtml = r#"<html><body>
            <h1>Chapter 1: Principles</h1>
            <p>Intro paragraph 1 with lots of words to give vertical height to the document.</p>
            <p>Intro paragraph 2 with lots of words to give vertical height to the document.</p>
            <p>Intro paragraph 3 with lots of words to give vertical height to the document.</p>
            <h2 id="sec1">1.1 Background</h2>
            <p>Section 1.1 content paragraph 1.</p>
            <p>Section 1.1 content paragraph 2.</p>
            <h2 id="sec2">1.2 Analysis</h2>
            <p>Section 1.2 content paragraph 1.</p>
        </body></html>"#;
    zip.write_all(xhtml.as_bytes()).unwrap();
    zip.finish().unwrap();

    let path_str = test_epub_path.to_string_lossy().to_string();

    let chapters = vec![
        crate::core::types::EpubChapterInfo {
            title: "Chapter 1: Principles".to_string(),
            level: 1,
            anchor: None,
            file_href: "ch1.xhtml".to_string(),
            blocks: Vec::new(),
        },
        crate::core::types::EpubChapterInfo {
            title: "1.1 Background".to_string(),
            level: 2,
            anchor: Some("sec1".to_string()),
            file_href: "ch1.xhtml".to_string(),
            blocks: Vec::new(),
        },
        crate::core::types::EpubChapterInfo {
            title: "1.2 Analysis".to_string(),
            level: 2,
            anchor: Some("sec2".to_string()),
            file_href: "ch1.xhtml".to_string(),
            blocks: Vec::new(),
        },
    ];

    let mut app = test_app(Some(crate::core::PreviewData::Epub {
        title: "Test".to_string(),
        author: "Author".to_string(),
        chapters: chapters.clone(),
        active_chapter: 0,
        images: std::collections::HashMap::new(),
    }));
    app.state.file_name = path_str;
    app.state.epub.chapters = chapters;
    app.state.window_height = 200.0;

    // 1. Initial load for Chapter 1 (index 0)
    let _ = handle_chapter_clicked(&mut app, 0);
    assert_eq!(app.state.epub.active_chapter, 0);
    assert_eq!(app.state.epub.markdown_state.scroll_y, 0.0);
    let total_blocks = app.state.epub.chapters[0].blocks.len();
    assert!(total_blocks >= 7);

    // 2. Click Section 1.1 (index 1) in hierarchy
    let _ = handle_chapter_clicked(&mut app, 1);
    assert_eq!(app.state.epub.active_chapter, 1);
    let sec1_scroll_y = app.state.epub.markdown_state.scroll_y;
    assert!(
        sec1_scroll_y > 0.0,
        "Clicking Section 1.1 must scroll down to anchor 'sec1', got {sec1_scroll_y}"
    );
    // All blocks of the chapter must still be preserved (not truncated!)
    assert_eq!(app.state.epub.chapters[1].blocks.len(), total_blocks);

    // 3. Click Section 1.2 (index 2) in hierarchy
    let _ = handle_chapter_clicked(&mut app, 2);
    assert_eq!(app.state.epub.active_chapter, 2);
    let sec2_scroll_y = app.state.epub.markdown_state.scroll_y;
    assert!(
        sec2_scroll_y > sec1_scroll_y,
        "Section 1.2 scroll Y ({sec2_scroll_y}) must be further down than Section 1.1 ({sec1_scroll_y})"
    );
    assert_eq!(app.state.epub.chapters[2].blocks.len(), total_blocks);

    // 4. Click back to Chapter 1 root (index 0)
    let _ = handle_chapter_clicked(&mut app, 0);
    assert_eq!(app.state.epub.active_chapter, 0);
    assert_eq!(app.state.epub.markdown_state.scroll_y, 0.0);

    let _ = std::fs::remove_file(test_epub_path);
}

#[test]
fn test_continuous_mode_navigation_and_scroll() {
    let chapters = vec![
        crate::core::types::EpubChapterInfo {
            title: "Chapter 1".to_string(),
            level: 1,
            anchor: None,
            file_href: "ch1.xhtml".to_string(),
            blocks: vec![crate::parsers::markdown::Block::Paragraph(vec![
                crate::parsers::markdown::Inline::Text("Chapter 1 text content".to_string()),
            ])],
        },
        crate::core::types::EpubChapterInfo {
            title: "Chapter 2".to_string(),
            level: 1,
            anchor: None,
            file_href: "ch2.xhtml".to_string(),
            blocks: vec![crate::parsers::markdown::Block::Paragraph(vec![
                crate::parsers::markdown::Inline::Text("Chapter 2 text content".to_string()),
            ])],
        },
    ];

    let mut app = test_app(Some(crate::core::PreviewData::Epub {
        title: "Test Continuous".to_string(),
        author: "Author".to_string(),
        chapters: chapters.clone(),
        active_chapter: 0,
        images: std::collections::HashMap::new(),
    }));
    app.state.epub_reading_mode = crate::core::config::EpubReadingMode::Continuous;
    app.state.epub.reading_mode = crate::core::config::EpubReadingMode::Continuous;
    crate::features::epub::state::populate_state(
        &mut app.state,
        "Test Continuous",
        "Author",
        &chapters,
        0,
        &std::collections::HashMap::new(),
    );

    assert_eq!(
        app.state.epub.reading_mode,
        crate::core::config::EpubReadingMode::Continuous
    );
    assert_eq!(app.state.epub.chapter_block_offsets.len(), 2);
    assert!(!app.state.epub.continuous_blocks.is_empty());

    // Click chapter 2 (index 1)
    let _ = handle_chapter_clicked(&mut app, 1);
    assert_eq!(app.state.epub.active_chapter, 1);
    let ch2_offset = app.state.epub.chapter_block_offsets[1];
    let expected_y = app.state.epub.markdown_state.block_y_offsets[ch2_offset];
    assert_eq!(app.state.epub.markdown_state.scroll_y, expected_y);

    // Click back to chapter 1 (index 0)
    let _ = handle_chapter_clicked(&mut app, 0);
    assert_eq!(app.state.epub.active_chapter, 0);
    assert_eq!(app.state.epub.markdown_state.scroll_y, 0.0);
}

#[test]
fn test_restore_read_position_in_continuous_mode() {
    let chapters = vec![
        crate::core::types::EpubChapterInfo {
            title: "Chapter 1".to_string(),
            level: 1,
            anchor: None,
            file_href: "ch1.xhtml".to_string(),
            blocks: vec![crate::parsers::markdown::Block::Paragraph(vec![
                crate::parsers::markdown::Inline::Text("Chapter 1 text content".to_string()),
            ])],
        },
        crate::core::types::EpubChapterInfo {
            title: "Chapter 2".to_string(),
            level: 1,
            anchor: None,
            file_href: "ch2.xhtml".to_string(),
            blocks: vec![crate::parsers::markdown::Block::Paragraph(vec![
                crate::parsers::markdown::Inline::Text("Chapter 2 text content".to_string()),
            ])],
        },
    ];

    let path_str = "/fake/book.epub";
    let mut app = test_app(Some(crate::core::PreviewData::Epub {
        title: "Test Continuous".to_string(),
        author: "Author".to_string(),
        chapters: chapters.clone(),
        active_chapter: 0,
        images: std::collections::HashMap::new(),
    }));
    app.state.file_name = path_str.to_string();
    app.state.epub_reading_mode = crate::core::config::EpubReadingMode::Continuous;
    crate::features::epub::state::populate_state(
        &mut app.state,
        "Test Continuous",
        "Author",
        &chapters,
        0,
        &std::collections::HashMap::new(),
    );

    let ch2_offset = app.state.epub.chapter_block_offsets[1];
    let target_y = app.state.epub.markdown_state.block_y_offsets[ch2_offset];

    // Save position at chapter 2's target_y
    app.state.read_positions.insert(
        path_str.to_string(),
        crate::core::ReadPosition {
            scroll_y: target_y,
            chapter: 1,
            reading_mode: Some(crate::core::config::EpubReadingMode::Continuous),
        },
    );

    // Reset scroll_y to 0
    app.state.epub.markdown_state.scroll_y = 0.0;
    app.state.epub.active_chapter = 0;

    // Restore read position
    let _ = app.restore_read_position_for(path_str);

    assert_eq!(app.state.epub.markdown_state.scroll_y, target_y);
    assert_eq!(app.state.epub.active_chapter, 1);
}

#[test]
fn test_continuous_mode_ctrl_scroll_zoom_preserves_continuous_blocks() {
    let chapters = vec![
        crate::core::types::EpubChapterInfo {
            title: "Chapter 1".to_string(),
            level: 1,
            anchor: None,
            file_href: "ch1.xhtml".to_string(),
            blocks: vec![crate::parsers::markdown::Block::Paragraph(vec![
                crate::parsers::markdown::Inline::Text("Chapter 1 text".to_string()),
            ])],
        },
        crate::core::types::EpubChapterInfo {
            title: "Chapter 2".to_string(),
            level: 1,
            anchor: None,
            file_href: "ch2.xhtml".to_string(),
            blocks: vec![crate::parsers::markdown::Block::Paragraph(vec![
                crate::parsers::markdown::Inline::Text("Chapter 2 text".to_string()),
            ])],
        },
    ];

    let mut app = test_app(Some(crate::core::PreviewData::Epub {
        title: "Test Continuous".to_string(),
        author: "Author".to_string(),
        chapters: chapters.clone(),
        active_chapter: 0,
        images: std::collections::HashMap::new(),
    }));
    app.state.epub_reading_mode = crate::core::config::EpubReadingMode::Continuous;
    crate::features::epub::state::populate_state(
        &mut app.state,
        "Test Continuous",
        "Author",
        &chapters,
        0,
        &std::collections::HashMap::new(),
    );

    let initial_blocks_count = app.state.epub.continuous_blocks.len();
    let total_offsets_count = app.state.epub.markdown_state.block_y_offsets.len();
    assert_eq!(initial_blocks_count, total_offsets_count);

    // Zoom in with Ctrl + Scroll
    app.ctrl_held = true;
    let _ = app.handle_scroll_delta(0.0, 1.0);

    // Verify block_y_offsets still corresponds to all continuous blocks, not just 1 chapter
    assert_eq!(
        app.state.epub.markdown_state.block_y_offsets.len(),
        initial_blocks_count
    );
    assert_eq!(app.state.epub.continuous_blocks.len(), initial_blocks_count);
}

#[test]
fn test_continuous_mode_window_resize_recomputes_continuous_layout() {
    let chapters = vec![
        crate::core::types::EpubChapterInfo {
            title: "Chapter 1".to_string(),
            level: 1,
            anchor: None,
            file_href: "ch1.xhtml".to_string(),
            blocks: vec![crate::parsers::markdown::Block::Paragraph(vec![
                crate::parsers::markdown::Inline::Text("Chapter 1 text".to_string()),
            ])],
        },
        crate::core::types::EpubChapterInfo {
            title: "Chapter 2".to_string(),
            level: 1,
            anchor: None,
            file_href: "ch2.xhtml".to_string(),
            blocks: vec![crate::parsers::markdown::Block::Paragraph(vec![
                crate::parsers::markdown::Inline::Text("Chapter 2 text".to_string()),
            ])],
        },
    ];

    let mut app = test_app(Some(crate::core::PreviewData::Epub {
        title: "Test Continuous".to_string(),
        author: "Author".to_string(),
        chapters: chapters.clone(),
        active_chapter: 0,
        images: std::collections::HashMap::new(),
    }));
    app.state.epub_reading_mode = crate::core::config::EpubReadingMode::Continuous;
    crate::features::epub::state::populate_state(
        &mut app.state,
        "Test Continuous",
        "Author",
        &chapters,
        0,
        &std::collections::HashMap::new(),
    );

    let initial_blocks_count = app.state.epub.continuous_blocks.len();

    // Resize window
    let _ = crate::app::update::misc::update_current_window_size(&mut app, 1200.0, 800.0);

    // Verify continuous block offsets maintained
    assert_eq!(
        app.state.epub.markdown_state.block_y_offsets.len(),
        initial_blocks_count
    );
}

#[test]
fn test_single_chapter_mode_ctrl_scroll_zoom_anchoring_on_lazy_chapter() {
    let ch1_blocks = vec![
        crate::parsers::markdown::Block::Heading {
            level: 1,
            content: vec![crate::parsers::markdown::Inline::Text(
                "Chapter 1".to_string(),
            )],
        },
        crate::parsers::markdown::Block::Paragraph(vec![crate::parsers::markdown::Inline::Text(
            "Paragraph 1".to_string(),
        )]),
    ];
    let ch2_blocks = vec![
        crate::parsers::markdown::Block::Heading {
            level: 1,
            content: vec![crate::parsers::markdown::Inline::Text(
                "Chapter 2".to_string(),
            )],
        },
        crate::parsers::markdown::Block::Paragraph(vec![crate::parsers::markdown::Inline::Text(
            "Paragraph A".to_string(),
        )]),
        crate::parsers::markdown::Block::Paragraph(vec![crate::parsers::markdown::Inline::Text(
            "Paragraph B".to_string(),
        )]),
    ];

    let chapters = vec![
        crate::core::types::EpubChapterInfo {
            title: "Chapter 1".to_string(),
            level: 1,
            anchor: None,
            file_href: "ch1.xhtml".to_string(),
            blocks: ch1_blocks,
        },
        crate::core::types::EpubChapterInfo {
            title: "Chapter 2".to_string(),
            level: 1,
            anchor: None,
            file_href: "ch2.xhtml".to_string(),
            blocks: Vec::new(), // Lazy loaded
        },
    ];

    let mut app = test_app(Some(crate::core::PreviewData::Epub {
        title: "Book".to_string(),
        author: "Author".to_string(),
        chapters: chapters.clone(),
        active_chapter: 0,
        images: std::collections::HashMap::new(),
    }));
    app.state.epub_reading_mode = crate::core::config::EpubReadingMode::SingleChapter;
    crate::features::epub::state::populate_state(
        &mut app.state,
        "Book",
        "Author",
        &chapters,
        0,
        &std::collections::HashMap::new(),
    );

    // Manually populate chapter 2 blocks as if loaded on demand
    app.state.epub.chapters[1].blocks = ch2_blocks;
    let _ = handle_chapter_clicked(&mut app, 1);

    assert_eq!(app.state.epub.active_chapter, 1);
    assert_eq!(app.state.epub.markdown_state.block_y_offsets.len(), 3);
    let ch2_p2_initial_y = app.state.epub.markdown_state.block_y_offsets[2];
    app.state.epub.markdown_state.scroll_y = ch2_p2_initial_y;

    // Zoom in with Ctrl + Scroll
    app.ctrl_held = true;
    let _ = app.handle_scroll_delta(0.0, 1.0);

    // Font size increased and layout recomputed for chapter 2
    assert_eq!(app.state.font_size, 15.0);
    assert_eq!(app.state.epub.markdown_state.block_y_offsets.len(), 3);
    let ch2_p2_new_y = app.state.epub.markdown_state.block_y_offsets[2];
    assert_eq!(app.state.epub.markdown_state.scroll_y, ch2_p2_new_y);
    assert!(ch2_p2_new_y > ch2_p2_initial_y);
}

#[test]
fn test_reading_mode_changed_preserves_position_single_to_continuous() {
    let ch1_blocks = vec![
        crate::parsers::markdown::Block::Paragraph(vec![crate::parsers::markdown::Inline::Text(
            "Chapter 1 Paragraph 1".to_string(),
        )]),
        crate::parsers::markdown::Block::Paragraph(vec![crate::parsers::markdown::Inline::Text(
            "Chapter 1 Paragraph 2".to_string(),
        )]),
    ];
    let ch2_blocks = vec![
        crate::parsers::markdown::Block::Paragraph(vec![crate::parsers::markdown::Inline::Text(
            "Chapter 2 Paragraph 1".to_string(),
        )]),
        crate::parsers::markdown::Block::Paragraph(vec![crate::parsers::markdown::Inline::Text(
            "Chapter 2 Paragraph 2".to_string(),
        )]),
    ];
    let chapters = vec![
        crate::core::types::EpubChapterInfo {
            title: "Chapter 1".to_string(),
            level: 1,
            anchor: None,
            file_href: "ch1.xhtml".to_string(),
            blocks: ch1_blocks,
        },
        crate::core::types::EpubChapterInfo {
            title: "Chapter 2".to_string(),
            level: 1,
            anchor: None,
            file_href: "ch2.xhtml".to_string(),
            blocks: ch2_blocks,
        },
    ];

    let mut app = test_app(Some(crate::core::PreviewData::Epub {
        title: "Test Book".to_string(),
        author: "Author".to_string(),
        chapters: chapters.clone(),
        active_chapter: 0,
        images: std::collections::HashMap::new(),
    }));
    app.state.epub_reading_mode = crate::core::config::EpubReadingMode::SingleChapter;
    crate::features::epub::state::populate_state(
        &mut app.state,
        "Test Book",
        "Author",
        &chapters,
        0,
        &std::collections::HashMap::new(),
    );

    // Switch to chapter 2 and scroll down to second block
    let _ = handle_chapter_clicked(&mut app, 1);
    assert_eq!(app.state.epub.active_chapter, 1);
    let block1_y = app.state.epub.markdown_state.block_y_offsets[1];
    let initial_scroll_y = block1_y + 15.0;
    app.state.epub.markdown_state.scroll_y = initial_scroll_y;

    // Switch to Continuous mode
    let _ = handle_reading_mode_changed(&mut app, crate::core::config::EpubReadingMode::Continuous);

    assert_eq!(
        app.state.epub.reading_mode,
        crate::core::config::EpubReadingMode::Continuous
    );
    assert_eq!(app.state.epub.active_chapter, 1);

    // In continuous mode, scroll_y must NOT be 0.0 or initial_scroll_y,
    // but must match the offset of chapter 2 block 1 in continuous layout + 15.0!
    let ch2_start = app.state.epub.chapter_block_offsets[1];
    let header_count = 2; // HR + Heading
    let cont_target_block = ch2_start + header_count + 1;
    let expected_continuous_y =
        app.state.epub.markdown_state.block_y_offsets[cont_target_block] + 15.0;
    assert_eq!(
        app.state.epub.markdown_state.scroll_y,
        expected_continuous_y
    );
    assert!(app.state.epub.markdown_state.scroll_y > initial_scroll_y);
}

#[test]
fn test_reading_mode_changed_preserves_position_continuous_to_single() {
    let ch1_blocks = vec![
        crate::parsers::markdown::Block::Paragraph(vec![crate::parsers::markdown::Inline::Text(
            "Chapter 1 Paragraph 1".to_string(),
        )]),
        crate::parsers::markdown::Block::Paragraph(vec![crate::parsers::markdown::Inline::Text(
            "Chapter 1 Paragraph 2".to_string(),
        )]),
    ];
    let ch2_blocks = vec![
        crate::parsers::markdown::Block::Paragraph(vec![crate::parsers::markdown::Inline::Text(
            "Chapter 2 Paragraph 1".to_string(),
        )]),
        crate::parsers::markdown::Block::Paragraph(vec![crate::parsers::markdown::Inline::Text(
            "Chapter 2 Paragraph 2".to_string(),
        )]),
    ];
    let chapters = vec![
        crate::core::types::EpubChapterInfo {
            title: "Chapter 1".to_string(),
            level: 1,
            anchor: None,
            file_href: "ch1.xhtml".to_string(),
            blocks: ch1_blocks,
        },
        crate::core::types::EpubChapterInfo {
            title: "Chapter 2".to_string(),
            level: 1,
            anchor: None,
            file_href: "ch2.xhtml".to_string(),
            blocks: ch2_blocks,
        },
    ];

    let mut app = test_app(Some(crate::core::PreviewData::Epub {
        title: "Test Book".to_string(),
        author: "Author".to_string(),
        chapters: chapters.clone(),
        active_chapter: 0,
        images: std::collections::HashMap::new(),
    }));
    app.state.epub_reading_mode = crate::core::config::EpubReadingMode::Continuous;
    crate::features::epub::state::populate_state(
        &mut app.state,
        "Test Book",
        "Author",
        &chapters,
        0,
        &std::collections::HashMap::new(),
    );

    // Position user in continuous mode at chapter 2's second block + 25.0
    let ch2_start = app.state.epub.chapter_block_offsets[1];
    let header_count = 2; // HR + Heading
    let cont_target_block = ch2_start + header_count + 1;
    let cont_scroll_y = app.state.epub.markdown_state.block_y_offsets[cont_target_block] + 25.0;
    app.state.epub.markdown_state.scroll_y = cont_scroll_y;

    // Switch to SingleChapter mode
    let _ = handle_reading_mode_changed(
        &mut app,
        crate::core::config::EpubReadingMode::SingleChapter,
    );

    assert_eq!(
        app.state.epub.reading_mode,
        crate::core::config::EpubReadingMode::SingleChapter
    );
    assert_eq!(app.state.epub.active_chapter, 1);

    // In single chapter mode, scroll_y must correspond to block 1 of chapter 2 + 25.0
    let expected_single_y = app.state.epub.markdown_state.block_y_offsets[1] + 25.0;
    assert_eq!(app.state.epub.markdown_state.scroll_y, expected_single_y);
}

#[test]
fn test_reading_mode_changed_round_trip_preserves_position() {
    let ch1_blocks = vec![crate::parsers::markdown::Block::Paragraph(vec![
        crate::parsers::markdown::Inline::Text("Chapter 1 Paragraph 1".to_string()),
    ])];
    let ch2_blocks = vec![
        crate::parsers::markdown::Block::Paragraph(vec![crate::parsers::markdown::Inline::Text(
            "Chapter 2 Paragraph 1".to_string(),
        )]),
        crate::parsers::markdown::Block::Paragraph(vec![crate::parsers::markdown::Inline::Text(
            "Chapter 2 Paragraph 2".to_string(),
        )]),
    ];
    let chapters = vec![
        crate::core::types::EpubChapterInfo {
            title: "Chapter 1".to_string(),
            level: 1,
            anchor: None,
            file_href: "ch1.xhtml".to_string(),
            blocks: ch1_blocks,
        },
        crate::core::types::EpubChapterInfo {
            title: "Chapter 2".to_string(),
            level: 1,
            anchor: None,
            file_href: "ch2.xhtml".to_string(),
            blocks: ch2_blocks,
        },
    ];

    let mut app = test_app(Some(crate::core::PreviewData::Epub {
        title: "Test Book".to_string(),
        author: "Author".to_string(),
        chapters: chapters.clone(),
        active_chapter: 0,
        images: std::collections::HashMap::new(),
    }));
    app.state.epub_reading_mode = crate::core::config::EpubReadingMode::SingleChapter;
    crate::features::epub::state::populate_state(
        &mut app.state,
        "Test Book",
        "Author",
        &chapters,
        0,
        &std::collections::HashMap::new(),
    );

    let _ = handle_chapter_clicked(&mut app, 1);
    let orig_y = app.state.epub.markdown_state.block_y_offsets[1] + 12.0;
    app.state.epub.markdown_state.scroll_y = orig_y;

    // Single -> Continuous
    let _ = handle_reading_mode_changed(&mut app, crate::core::config::EpubReadingMode::Continuous);
    assert_eq!(app.state.epub.active_chapter, 1);

    // Continuous -> Single
    let _ = handle_reading_mode_changed(
        &mut app,
        crate::core::config::EpubReadingMode::SingleChapter,
    );
    assert_eq!(app.state.epub.active_chapter, 1);
    assert_eq!(app.state.epub.markdown_state.scroll_y, orig_y);
}

#[test]
fn test_reading_mode_changed_with_existing_read_positions() {
    let chapters = vec![
        crate::core::types::EpubChapterInfo {
            title: "Chapter 1".to_string(),
            level: 1,
            anchor: None,
            file_href: "ch1.xhtml".to_string(),
            blocks: vec![crate::parsers::markdown::Block::Paragraph(vec![
                crate::parsers::markdown::Inline::Text("Chapter 1 text content".to_string()),
            ])],
        },
        crate::core::types::EpubChapterInfo {
            title: "Chapter 2".to_string(),
            level: 1,
            anchor: None,
            file_href: "ch2.xhtml".to_string(),
            blocks: vec![crate::parsers::markdown::Block::Paragraph(vec![
                crate::parsers::markdown::Inline::Text("Chapter 2 text content".to_string()),
            ])],
        },
    ];

    let path_str = "/fake/saved_book.epub";
    let mut app = test_app(Some(crate::core::PreviewData::Epub {
        title: "Test Book".to_string(),
        author: "Author".to_string(),
        chapters: chapters.clone(),
        active_chapter: 0,
        images: std::collections::HashMap::new(),
    }));
    app.state.file_name = path_str.to_string();
    app.state.epub_reading_mode = crate::core::config::EpubReadingMode::SingleChapter;

    // Simulate pre-existing read_positions with SingleChapter
    app.state.read_positions.insert(
        path_str.to_string(),
        crate::core::ReadPosition {
            scroll_y: 20.0,
            chapter: 1,
            reading_mode: Some(crate::core::config::EpubReadingMode::SingleChapter),
        },
    );

    crate::features::epub::state::populate_state(
        &mut app.state,
        "Test Book",
        "Author",
        &chapters,
        1,
        &std::collections::HashMap::new(),
    );
    assert_eq!(
        app.state.epub.reading_mode,
        crate::core::config::EpubReadingMode::SingleChapter
    );

    // Now user changes reading mode to Continuous (e.g. from Settings)
    let _ = handle_reading_mode_changed(&mut app, crate::core::config::EpubReadingMode::Continuous);

    assert_eq!(
        app.state.epub.reading_mode,
        crate::core::config::EpubReadingMode::Continuous
    );
    assert_eq!(
        app.state.epub_reading_mode,
        crate::core::config::EpubReadingMode::Continuous
    );
    assert!(!app.state.epub.continuous_blocks.is_empty());
    assert_eq!(
        app.state
            .read_positions
            .get(path_str)
            .and_then(|p| p.reading_mode),
        Some(crate::core::config::EpubReadingMode::Continuous)
    );
}
