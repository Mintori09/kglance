use crate::app::KglanceApp;
use crate::app::messages::Message;
use iced::Task;
use iced::widget::operation;

pub fn handle_sidebar_toggled(app: &mut KglanceApp) -> Task<Message> {
    app.state.epub.sidebar_visible = !app.state.epub.sidebar_visible;

    let idx = app.state.epub.active_chapter;
    if let Some(active_ch) = app.state.epub.chapters.get(idx) {
        let content_width = app.state.epub_content_width();
        let old_y = app.state.epub.markdown_state.scroll_y;
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
    if idx < app.state.epub.chapters.len() {
        if app.state.epub.chapters[idx].blocks.is_empty() {
            let path_str = app.state.file_name.clone();
            let chapter_file_href = app.state.epub.chapters[idx].file_href.clone();
            let chapter_anchor = app.state.epub.chapters[idx].anchor.clone();

            if !path_str.is_empty() {
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
        ensure_chapter_loaded(app, idx);
        app.record_read_position();
        app.state.epub.markdown_state.scroll_y = 0.0;
        app.state.epub.markdown_state.smooth_scroll.is_animating = false;

        return operation::snap_to(
            "content_scroll",
            operation::RelativeOffset { x: 0.0, y: 0.0 },
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
}
