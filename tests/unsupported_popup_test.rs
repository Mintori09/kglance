use kglance::core::preview::PreviewData;
use kglance::core::types::KglanceState;
use kglance::features::common::parser::types::{ParsedContent, UnsupportedFileInfo};
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_unsupported_file_parsing_and_popup_metadata() {
    let mut file = NamedTempFile::with_suffix(".pkt").expect("failed to create temp file");
    // Write random binary bytes (invalid utf-8)
    file.write_all(&[0x00, 0xff, 0xfe, 0xba, 0xbe, 0xde, 0xad])
        .expect("write binary");
    file.flush().expect("flush");

    let registry = kglance::engine::build_registry();
    let parsed = registry
        .parse(file.path())
        .expect("parse unsupported binary file");

    match parsed {
        ParsedContent::Unsupported(info) => {
            assert!(info.file_name.ends_with(".pkt"));
            assert_eq!(info.file_size, 7);
            assert!(info.is_binary);
            assert!(info.file_type.contains("PKT") || info.file_type.contains("Octet-stream"));
        }
        other => panic!("Expected ParsedContent::Unsupported, got {other:?}"),
    }
}

#[test]
fn test_unsupported_preview_data_window_size_and_state() {
    let info = UnsupportedFileInfo {
        file_name: "test_binary.pkt".to_string(),
        file_size: 1024,
        file_type: "Packet Capture / Octet-stream".to_string(),
        modified: Some(std::time::SystemTime::UNIX_EPOCH),
        location: "/tmp".to_string(),
        is_binary: true,
    };

    let preview = PreviewData::Unsupported(info);
    let mut state = KglanceState::default();

    // Verify window size is 450x380 px
    let size = preview.initial_window_size(&state);
    assert_eq!(size.width, 450.0);
    assert_eq!(size.height, 380.0);

    // Verify populate_state sets file_type_text
    preview.populate_state(&mut state);
    assert_eq!(state.file_type_text, "Packet Capture / Octet-stream");
}

#[test]
fn test_unsupported_all_themes_contrast_and_render() {
    use kglance::ui::theme::AppTheme;
    use kglance::ui::views::unsupported::view_unsupported;

    let info = UnsupportedFileInfo {
        file_name: "Lab 1 - Coffee.pkt".to_string(),
        file_size: 235116,
        file_type: "Packet Tracer Packet Capture".to_string(),
        modified: Some(std::time::SystemTime::UNIX_EPOCH),
        location: "/home/mintori/Desktop/Study-University/Network/Lab 1 - Coffee.pkt".to_string(),
        is_binary: true,
    };

    for theme in AppTheme::ALL {
        let palette = theme.palette();
        assert_ne!(palette.roles.accent_text, iced::Color::TRANSPARENT);

        // Under pastel/bright accents, ensure accent_text provides readable contrast
        match theme {
            AppTheme::GruvboxDark
            | AppTheme::CatppuccinMocha
            | AppTheme::Nord
            | AppTheme::TokyoNight
            | AppTheme::Dracula
            | AppTheme::ProperDark => {
                // Should use dark contrast text instead of pure white on pastel accents
                assert_ne!(palette.roles.accent_text, iced::Color::WHITE);
            }
            AppTheme::Dark | AppTheme::Light | AppTheme::CatppuccinLatte => {
                assert_eq!(palette.roles.accent_text, iced::Color::WHITE);
            }
        }

        let _element = view_unsupported(&info, theme, None, None);
    }
}
