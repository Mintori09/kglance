use std::collections::HashMap;

use super::*;
use crate::features::lightbox::types::LightboxItem;
use crate::features::markdown::{Block, Inline, ListItem, TableBlock, TableCell};

#[test]
fn test_collect_markdown_lightbox_items_filters_unloaded_images() {
    let mut cached_handles = HashMap::new();
    let img_handle = iced::widget::image::Handle::from_bytes(vec![137, 80, 78, 71]);
    cached_handles.insert("loaded.png".to_string(), img_handle);

    let blocks = vec![
        Block::Image {
            alt: "Loaded Alt".into(),
            path: "loaded.png".into(),
            link_url: Some("https://example.com".into()),
            width: None,
            height: None,
        },
        Block::Image {
            alt: "Unloaded Alt".into(),
            path: "unloaded.png".into(),
            link_url: None,
            width: None,
            height: None,
        },
    ];

    let items = collect_markdown_lightbox_items(&blocks, &cached_handles);

    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        LightboxItem {
            id: "loaded.png".into(),
            title: Some("Loaded Alt".into()),
            subtitle: "loaded.png".into(),
            link_url: Some("https://example.com".into()),
            in_memory_bytes: None,
        }
    );
}

#[test]
fn test_collect_markdown_lightbox_items_includes_rendered_mermaid() {
    let cached_handles = HashMap::new();

    let mermaid_png = vec![137, 80, 78, 71, 13, 10, 26, 10];
    let blocks = vec![
        Block::Mermaid {
            lines: vec!["graph TD; A-->B;".into()],
            rendered: Some(mermaid_png.clone()),
        },
        Block::Mermaid {
            lines: vec!["graph TD; C-->D;".into()],
            rendered: None, // not yet rendered
        },
    ];

    let items = collect_markdown_lightbox_items(&blocks, &cached_handles);

    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0],
        LightboxItem {
            id: "mermaid:0".into(),
            title: None,
            subtitle: "Mermaid Diagram #1".into(),
            link_url: None,
            in_memory_bytes: Some(mermaid_png),
        }
    );
}

#[test]
fn test_collect_markdown_lightbox_items_in_nested_blocks_preserves_order() {
    let mut cached_handles = HashMap::new();
    let h = iced::widget::image::Handle::from_bytes(vec![1, 2, 3]);
    cached_handles.insert("img1.png".into(), h.clone());
    cached_handles.insert("img2.png".into(), h.clone());
    cached_handles.insert("table_img.png".into(), h);

    let mermaid_png = vec![4, 5, 6];
    let blocks = vec![
        Block::Image {
            alt: "First".into(),
            path: "img1.png".into(),
            link_url: None,
            width: None,
            height: None,
        },
        Block::List {
            ordered: false,
            start_number: 1,
            items: vec![ListItem {
                is_task: None,
                content: vec![],
                sub_blocks: vec![Block::Mermaid {
                    lines: vec!["flowchart LR".into()],
                    rendered: Some(mermaid_png.clone()),
                }],
            }],
        },
        Block::Table(TableBlock {
            headers: vec![TableCell {
                content: vec![Inline::Image {
                    alt: "In Header".into(),
                    url: "table_img.png".into(),
                    width: None,
                    height: None,
                }],
            }],
            rows: vec![],
            column_weights: vec![1],
            alignments: vec![],
        }),
        Block::Quote(vec![Block::Image {
            alt: "Last".into(),
            path: "img2.png".into(),
            link_url: None,
            width: None,
            height: None,
        }]),
    ];

    let items = collect_markdown_lightbox_items(&blocks, &cached_handles);

    assert_eq!(items.len(), 4);
    assert_eq!(items[0].id, "img1.png");
    assert_eq!(items[1].id, "mermaid:1.0.0");
    assert_eq!(items[2].id, "table_img.png");
    assert_eq!(items[3].id, "img2.png");
}

#[test]
fn test_handle_open_markdown_image_lightbox() {
    let png = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    let b64_data = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
    let blocks = vec![
        Block::Image {
            alt: "First".into(),
            path: b64_data.into(),
            link_url: None,
            width: None,
            height: None,
        },
        Block::Mermaid {
            lines: vec!["graph TD; A-->B;".into()],
            rendered: Some(png),
        },
    ];

    let mut app = crate::app::test_util::test_app(Some(crate::core::PreviewData::Markdown {
        blocks,
        raw_text: "# Title".into(),
    }));

    let h = iced::widget::image::Handle::from_bytes(vec![1, 2, 3]);
    app.state
        .markdown
        .cached_image_handles
        .insert(b64_data.into(), h);

    assert!(app.state.lightbox.is_none());

    // Open first image
    let _ = handle_open_markdown_image_lightbox(&mut app, b64_data);
    assert!(app.state.lightbox.is_some());
    let lb = app.state.lightbox.as_ref().unwrap();
    assert_eq!(lb.current_index, 0);
    assert_eq!(lb.items.len(), 2);
    assert_eq!(lb.items[0].subtitle, "Embedded image");

    // Open mermaid diagram directly
    let _ = handle_open_markdown_image_lightbox(&mut app, "mermaid:1");
    let lb = app.state.lightbox.as_ref().unwrap();
    assert_eq!(lb.current_index, 1);
    assert_eq!(lb.items[1].subtitle, "Mermaid Diagram #1");
}
