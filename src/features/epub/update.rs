use crate::app::KglanceApp;
use crate::app::messages::Message;
use iced::Task;
use iced::widget::operation;

pub fn handle_sidebar_toggled(app: &mut KglanceApp) -> Task<Message> {
    app.state.epub.sidebar_visible = !app.state.epub.sidebar_visible;

    let content_width = app.state.epub_content_width();
    let old_y = app.state.epub.markdown_state.scroll_y;

    if app.state.epub.reading_mode == crate::core::config::EpubReadingMode::Continuous {
        crate::features::markdown::recompute_markdown_layout(
            &mut app.state.epub.markdown_state,
            &app.state.epub.continuous_blocks,
            app.state.font_size,
            content_width,
        );
        app.state.epub.markdown_state.scroll_y = old_y;
        return operation::scroll_to(
            "content_scroll",
            operation::AbsoluteOffset { x: 0.0, y: old_y },
        );
    }

    let idx = app.state.epub.active_chapter;
    if let Some(active_ch) = app.state.epub.chapters.get(idx) {
        crate::features::markdown::recompute_markdown_layout(
            &mut app.state.epub.markdown_state,
            &active_ch.blocks,
            app.state.font_size,
            content_width,
        );
        app.state.epub.markdown_state.scroll_y = old_y;
        return operation::scroll_to(
            "content_scroll",
            operation::AbsoluteOffset { x: 0.0, y: old_y },
        );
    }

    let y = app.state.epub.markdown_state.scroll_y;
    operation::scroll_to("content_scroll", operation::AbsoluteOffset { x: 0.0, y })
}

pub fn ensure_chapter_loaded(app: &mut KglanceApp, idx: usize) {
    if app.state.epub.reading_mode == crate::core::config::EpubReadingMode::Continuous {
        return;
    }
    if idx < app.state.epub.chapters.len() {
        if app.state.epub.chapters[idx].blocks.is_empty() {
            let path_str = app.state.file_name.clone();
            let chapter_file_href = app.state.epub.chapters[idx].file_href.clone();
            let chapter_anchor = app.state.epub.chapters[idx].anchor.clone();

            let existing_blocks = app
                .state
                .epub
                .chapters
                .iter()
                .find(|ch| ch.file_href == chapter_file_href && !ch.blocks.is_empty())
                .map(|ch| ch.blocks.clone());

            if let Some(blocks) = existing_blocks {
                if let Some(crate::core::PreviewData::Epub {
                    chapters: preview_chapters,
                    ..
                }) = &mut app.current_content
                    && let Some(preview_ch) = preview_chapters.get_mut(idx)
                {
                    preview_ch.blocks = blocks.clone();
                }
                app.state.epub.chapters[idx].blocks = blocks;
            } else if !path_str.is_empty() {
                let path = std::path::Path::new(&path_str);
                if let Ok((new_blocks, new_images)) =
                    crate::features::epub::parser::load_chapter_content_and_images_from_epub(
                        path,
                        &chapter_file_href,
                        chapter_anchor.as_deref(),
                    )
                {
                    if let Some(crate::core::PreviewData::Epub {
                        images,
                        chapters: preview_chapters,
                        ..
                    }) = &mut app.current_content
                    {
                        for (k, v) in new_images {
                            images.insert(k, v);
                        }
                        if let Some(preview_ch) = preview_chapters.get_mut(idx) {
                            preview_ch.blocks = new_blocks.clone();
                        }
                    }
                    app.state.epub.chapters[idx].blocks = new_blocks;
                }
            }
        }

        app.state.epub.active_chapter = idx;

        if let Some(crate::core::PreviewData::Epub { images, .. }) = &app.current_content {
            crate::features::epub::state::ensure_chapter_images(
                &mut app.state.epub.markdown_state,
                &app.state.epub.chapters,
                idx,
                images,
            );
        }

        if let Some(active_ch) = app.state.epub.chapters.get(idx) {
            let content_width = app.state.epub_content_width();
            crate::features::markdown::recompute_markdown_layout(
                &mut app.state.epub.markdown_state,
                &active_ch.blocks,
                app.state.font_size,
                content_width,
            );
        }
    }
}

pub fn handle_chapter_clicked(app: &mut KglanceApp, idx: usize) -> Task<Message> {
    if idx < app.state.epub.chapters.len() {
        if app.state.epub.reading_mode == crate::core::config::EpubReadingMode::Continuous {
            app.state.epub.active_chapter = idx;
            app.record_read_position();

            let chapter = &app.state.epub.chapters[idx];
            let start_block_idx = app
                .state
                .epub
                .chapter_block_offsets
                .get(idx)
                .copied()
                .unwrap_or(0);
            let local_block_offset = crate::features::epub::parser::find_block_index(
                &chapter.blocks,
                chapter.anchor.as_deref(),
                Some(&chapter.title),
            )
            .unwrap_or(0);
            let target_block_idx = start_block_idx + local_block_offset;
            let target_y = app
                .state
                .epub
                .markdown_state
                .block_y_offsets
                .get(target_block_idx)
                .copied()
                .unwrap_or(0.0);

            let max_y = crate::core::scroll::max_scroll_y(
                app.state.epub.markdown_state.total_content_height,
                app.state.epub.markdown_state.viewport_height,
            );
            let clamped_y = if max_y > 0.0 {
                target_y.clamp(0.0, max_y)
            } else {
                target_y
            };

            app.state.epub.markdown_state.scroll_y = clamped_y;
            app.state
                .epub
                .markdown_state
                .scroll_controller
                .stop(clamped_y);
            app.state.epub.markdown_state.smooth_scroll.stop(clamped_y);

            return operation::scroll_to(
                "content_scroll",
                operation::AbsoluteOffset {
                    x: 0.0,
                    y: clamped_y,
                },
            );
        }

        ensure_chapter_loaded(app, idx);
        app.record_read_position();

        let chapter = &app.state.epub.chapters[idx];
        let target_y = if let Some(block_idx) = crate::features::epub::parser::find_block_index(
            &chapter.blocks,
            chapter.anchor.as_deref(),
            Some(&chapter.title),
        ) {
            app.state
                .epub
                .markdown_state
                .block_y_offsets
                .get(block_idx)
                .copied()
                .unwrap_or(0.0)
        } else {
            0.0
        };

        let max_y = crate::core::scroll::max_scroll_y(
            app.state.epub.markdown_state.total_content_height,
            app.state.epub.markdown_state.viewport_height,
        );
        let clamped_y = if max_y > 0.0 {
            target_y.clamp(0.0, max_y)
        } else {
            target_y
        };

        app.state.epub.markdown_state.scroll_y = clamped_y;
        app.state
            .epub
            .markdown_state
            .scroll_controller
            .stop(clamped_y);
        app.state.epub.markdown_state.smooth_scroll.stop(clamped_y);

        return operation::scroll_to(
            "content_scroll",
            operation::AbsoluteOffset {
                x: 0.0,
                y: clamped_y,
            },
        );
    }
    Task::none()
}

pub fn handle_chapter_toggle_collapse(app: &mut KglanceApp, idx: usize) -> Task<Message> {
    if app.state.epub.collapsed_chapters.contains(&idx) {
        app.state.epub.collapsed_chapters.remove(&idx);
    } else {
        app.state.epub.collapsed_chapters.insert(idx);
    }
    Task::none()
}

#[cfg(test)]
mod tests {
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
            137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1,
            8, 6, 0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 10, 73, 68, 65, 84, 120, 156, 99, 0, 1, 0, 0,
            5, 0, 1, 13, 10, 45, 180, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
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
}
