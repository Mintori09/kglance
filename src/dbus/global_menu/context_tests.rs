use super::*;
use crate::core::config::{AppConfig, EpubReadingMode, RubyDisplayMode};
use crate::core::preview::PreviewData;
use crate::ui::theme::AppTheme;

#[test]
fn test_root_menu_includes_selection_menu_and_remember_size() {
    let mut config = AppConfig::default();
    config.selection_menu.enabled = true;
    config.ui.remember_window_size = true;
    let theme = AppTheme::Dark;

    let root = crate::dbus::global_menu::build_root_menu(&config, &theme);
    let view_menu = &root.children[2];
    assert_eq!(view_menu.label, "View");

    let labels: Vec<&str> = view_menu
        .children
        .iter()
        .map(|c| c.label.as_str())
        .collect();
    assert!(labels.contains(&"Selection Context Menu"));
    assert!(labels.contains(&"Remember Window Size"));

    let sel_item = view_menu
        .children
        .iter()
        .find(|c| c.id == crate::dbus::global_menu::ID_VIEW_SELECTION_MENU)
        .expect("selection menu item must exist");
    assert_eq!(sel_item.toggle_state, ToggleState::On);

    let rem_item = view_menu
        .children
        .iter()
        .find(|c| c.id == crate::dbus::global_menu::ID_VIEW_REMEMBER_SIZE)
        .expect("remember size item must exist");
    assert_eq!(rem_item.toggle_state, ToggleState::On);
}

#[test]
fn test_epub_context_menu_builder() {
    let content = PreviewData::Epub {
        title: "Test Book".to_string(),
        author: "Author".to_string(),
        chapters: vec![],
        active_chapter: 0,
        images: std::collections::HashMap::new(),
    };

    let options = ContextMenuOptions {
        epub_override_book_font: true,
        epub_reading_mode: EpubReadingMode::Continuous,
        ruby_display_mode: RubyDisplayMode::Stacked,
        ..Default::default()
    };

    let (label, items) = build_context_menu_items(Some(&content), &options);
    assert_eq!(label, "EPUB");

    let override_item = items
        .iter()
        .find(|i| i.id == ID_EPUB_OVERRIDE_FONT)
        .expect("override font item should exist");
    assert_eq!(override_item.toggle_state, ToggleState::On);
    assert_eq!(
        override_item.action_id.as_deref(),
        Some("epub.toggle_override_font")
    );

    let reading_mode_submenu = items
        .iter()
        .find(|i| i.id == ID_EPUB_READING_MODE_SUBMENU)
        .expect("reading mode submenu should exist");
    assert_eq!(reading_mode_submenu.children.len(), 2);
    assert_eq!(
        reading_mode_submenu.children[0].toggle_state,
        ToggleState::On
    );
    assert_eq!(
        reading_mode_submenu.children[1].toggle_state,
        ToggleState::Off
    );

    let ruby_submenu = items
        .iter()
        .find(|i| i.id == ID_EPUB_RUBY_SUBMENU)
        .expect("ruby submenu should exist");
    assert_eq!(ruby_submenu.children.len(), 4);
    assert_eq!(ruby_submenu.children[0].toggle_state, ToggleState::On);

    assert!(items.iter().any(|i| i.id == ID_ZOOM_IN));
    assert!(items.iter().any(|i| i.id == ID_ZOOM_OUT));
    assert!(items.iter().any(|i| i.id == ID_ZOOM_RESET));
}

#[test]
fn test_json_context_menu_builder() {
    let content = PreviewData::Json {
        nodes: vec![],
        content: "{}".to_string(),
        pretty: "{}".to_string(),
        has_parse_error: false,
    };

    let options = ContextMenuOptions {
        json_tree_mode: true,
        ..Default::default()
    };

    let (label, items) = build_context_menu_items(Some(&content), &options);
    assert_eq!(label, "JSON");

    let tree_mode_item = items
        .iter()
        .find(|i| i.id == ID_JSON_TOGGLE_MODE)
        .expect("tree mode item must exist");
    assert_eq!(tree_mode_item.toggle_state, ToggleState::On);

    assert!(items.iter().any(|i| i.id == ID_JSON_TOGGLE_FORMAT));
    assert!(items.iter().any(|i| i.id == ID_JSON_EXPAND_ALL));
    assert!(items.iter().any(|i| i.id == ID_JSON_COLLAPSE_ALL));
}

#[test]
fn test_spreadsheet_context_menu_builder() {
    let content = PreviewData::Spreadsheet {
        sheets: vec![],
        active_sheet: 0,
    };

    let options = ContextMenuOptions {
        sheet_count: 3,
        active_sheet: 0,
        ..Default::default()
    };

    let (label, items) = build_context_menu_items(Some(&content), &options);
    assert_eq!(label, "Spreadsheet");

    let prev_sheet = items.iter().find(|i| i.id == ID_SHEET_PREV_SHEET).unwrap();
    let next_sheet = items.iter().find(|i| i.id == ID_SHEET_NEXT_SHEET).unwrap();
    assert!(!prev_sheet.enabled); // At sheet 0, prev is disabled
    assert!(next_sheet.enabled); // At sheet 0 of 3, next is enabled

    assert!(items.iter().any(|i| i.id == ID_SHEET_SELECT_ALL));
    assert!(items.iter().any(|i| i.id == ID_SHEET_COPY_SELECTION));
}

#[test]
fn test_image_context_menu_builder() {
    let content = PreviewData::Image {
        data: vec![],
        width: 100,
        height: 100,
        format_info: "PNG".to_string(),
        exif_content: None,
    };

    let options = ContextMenuOptions {
        image_show_info: true,
        ..Default::default()
    };

    let (label, items) = build_context_menu_items(Some(&content), &options);
    assert_eq!(label, "Image");

    assert!(items.iter().any(|i| i.id == ID_IMAGE_FLIP_H));
    assert!(items.iter().any(|i| i.id == ID_IMAGE_FLIP_V));

    let info_item = items.iter().find(|i| i.id == ID_IMAGE_TOGGLE_INFO).unwrap();
    assert_eq!(info_item.toggle_state, ToggleState::On);
}

#[test]
fn test_typst_context_menu_builder() {
    let content = PreviewData::Typst {
        page_count: 1,
        current_page: 0,
        data: vec![],
        width: 100,
        height: 100,
        source: "test".to_string(),
        error: None,
        outline: vec![],
        page_dimensions: vec![],
        first_page_text: None,
    };

    let options = ContextMenuOptions {
        typst_show_source: true,
        ..Default::default()
    };

    let (label, items) = build_context_menu_items(Some(&content), &options);
    assert_eq!(label, "Typst");

    let source_item = items
        .iter()
        .find(|i| i.id == ID_TYPST_TOGGLE_SOURCE)
        .unwrap();
    assert_eq!(source_item.toggle_state, ToggleState::On);
}

#[test]
fn test_media_context_menu_builder() {
    let audio_content = PreviewData::Audio {
        path: "test.mp3".to_string(),
        title: "Song".to_string(),
        artist: "Artist".to_string(),
        album: "Album".to_string(),
        duration_secs: 120,
        metadata: "Meta".to_string(),
        cover_art: None,
    };

    let audio_options = ContextMenuOptions {
        audio_muted: true,
        media_playing: true,
        ..Default::default()
    };

    let (label, items) = build_context_menu_items(Some(&audio_content), &audio_options);
    assert_eq!(label, "Audio");
    let play_item = items.iter().find(|i| i.id == ID_MEDIA_PLAY_PAUSE).unwrap();
    assert_eq!(play_item.label, "Pause");
    let mute_item = items.iter().find(|i| i.id == ID_MEDIA_TOGGLE_MUTE).unwrap();
    assert_eq!(mute_item.toggle_state, ToggleState::On);

    let video_content = PreviewData::Media {
        url: "test.mp4".to_string(),
        metadata: "Video".to_string(),
        thumbnail_or_waveform: vec![],
        width: 1920,
        height: 1080,
    };

    let video_options = ContextMenuOptions {
        media_subtitles_enabled: true,
        ..Default::default()
    };
    let (v_label, v_items) = build_context_menu_items(Some(&video_content), &video_options);
    assert_eq!(v_label, "Video");
    let sub_item = v_items
        .iter()
        .find(|i| i.id == ID_MEDIA_TOGGLE_SUBTITLES)
        .unwrap();
    assert_eq!(sub_item.toggle_state, ToggleState::On);
}
