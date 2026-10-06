use crate::core::types::{EpubChapterInfo, FolderRowState, KglanceState, SheetInfo};
use crate::features::common::parser::traits::ParseError;
use crate::features::json::JsonNode;
use crate::features::pdf::PdfTocEntry;
use crate::features::pdf::types::PageDimensions;
use crate::parsers::markdown::Block;
use std::collections::HashMap;
use std::path::Path;

#[doc(hidden)]
pub use crate::features::markdown::compute_block_y_offsets;

pub fn compute_pdf_page_offsets(
    dims: &[PageDimensions],
    display_width: f32,
    spacing: f32,
) -> (Vec<f32>, Vec<f32>, f32) {
    let (offsets, ends, _, total_h) =
        crate::features::pdf::geometry::compute_pdf_page_offsets(dims, display_width, spacing);
    (offsets, ends, total_h)
}

#[derive(Debug, Clone)]
pub enum PreviewData {
    Image {
        data: Vec<u8>,
        width: u32,
        height: u32,
        format_info: String,
        exif_content: Option<String>,
    },
    Text {
        content: String,
        line_numbers: String,
        language: String,
    },
    Markdown {
        blocks: Vec<Block>,
        raw_text: String,
    },
    Pdf {
        page_count: usize,
        current_page: usize,
        data: Vec<u8>,
        width: u32,
        height: u32,
        outline: Vec<PdfTocEntry>,
        page_dimensions: Vec<PageDimensions>,
    },
    Typst {
        page_count: usize,
        current_page: usize,
        data: Vec<u8>,
        width: u32,
        height: u32,
        source: String,
        error: Option<String>,
        outline: Vec<PdfTocEntry>,
        page_dimensions: Vec<PageDimensions>,
    },
    Audio {
        path: String,
        title: String,
        artist: String,
        album: String,
        duration_secs: u64,
        metadata: String,
        cover_art: Option<Vec<u8>>,
    },
    Media {
        url: String,
        metadata: String,
        thumbnail_or_waveform: Vec<u8>,
        width: u32,
        height: u32,
    },
    Folder {
        rows: Vec<FolderRowState>,
        total_size: u64,
    },
    Spreadsheet {
        sheets: Vec<SheetInfo>,
        active_sheet: usize,
    },
    Json {
        nodes: Vec<JsonNode>,
        content: String,
        pretty: String,
        has_parse_error: bool,
    },
    Epub {
        title: String,
        author: String,
        chapters: Vec<EpubChapterInfo>,
        active_chapter: usize,
        images: HashMap<String, Vec<u8>>,
    },
    Font {
        name: String,
        family: String,
        post_script_name: Option<String>,
        weight: u16,
        is_italic: bool,
        metadata: String,
        sample: Vec<u8>,
        sample_width: u32,
        sample_height: u32,
        data: Vec<u8>,
    },
    Error(String),
}

pub trait FilePreviewer {
    fn parse(&self, path: &Path) -> Result<PreviewData, ParseError>;
}

pub fn is_slow_to_parse(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        // Office documents that build PDF or parse complex spreadsheets
        "docx" | "doc" | "xlsx" | "xls" | "pptx" | "ppt" | "odt" | "ods" | "odp" | "xlsb" => true,
        // Typst that compiles to PDF
        "typ" => true,
        // PDF documents
        "pdf" => true,
        // Archives
        "7z" | "tar" | "zip" | "gz" | "bz2" | "xz" | "tgz" | "tbz2" | "txz" | "kra" => true,
        // Audio & Video
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "webm" | "mp3" | "wav" | "flac" | "ogg" | "aac"
        | "m4a" | "opus" => true,
        _ => {
            if let Ok(meta) = path.metadata() {
                meta.len() > 512 * 1024
            } else {
                false
            }
        }
    }
}

fn update_file_metadata(state: &mut KglanceState) {
    if !state.file_name.is_empty() {
        let path = Path::new(&state.file_name);
        if let Ok(meta) = std::fs::metadata(path) {
            state.file_size_text = crate::parsers::human_size(meta.len());
            if let Ok(modified) = meta.modified() {
                state.file_modified_text = crate::parsers::human_time(modified);
            }
        }
    }
}

impl PreviewData {
    pub fn populate_state(&self, state: &mut KglanceState) {
        update_file_metadata(state);

        match self {
            PreviewData::Text {
                content,
                line_numbers,
                language,
            } => {
                crate::features::text::populate_state(
                    state,
                    content.clone(),
                    line_numbers.clone(),
                    language,
                );
            }
            PreviewData::Image {
                data,
                width,
                height,
                format_info,
                exif_content,
                ..
            } => {
                crate::features::image::populate_image_state(
                    state,
                    data,
                    *width,
                    *height,
                    format_info,
                    exif_content.as_deref(),
                );
            }
            PreviewData::Pdf {
                page_count,
                outline,
                page_dimensions,
                ..
            } => {
                crate::features::pdf::populate_state(
                    state,
                    *page_count,
                    outline.clone(),
                    page_dimensions.clone(),
                );
            }
            PreviewData::Typst {
                page_count,
                source,
                error,
                outline,
                page_dimensions,
                ..
            } => {
                crate::features::typst::populate_state(
                    state,
                    *page_count,
                    source,
                    error.clone(),
                    outline,
                    page_dimensions,
                );
            }
            PreviewData::Folder { rows, total_size } => {
                state.folder.rows = rows.clone();
                state.folder.total_size = *total_size;
                state.folder.folder_path = state.file_name.clone();
                state.folder.selected_index = None;
                state.file_type_text = "Folder / Archive".to_string();
                state.file_size_text.clear();
            }
            PreviewData::Markdown { blocks, .. } => {
                crate::features::markdown::populate_state(state, blocks);
            }
            PreviewData::Spreadsheet {
                sheets,
                active_sheet,
            } => {
                crate::features::csv::populate_state(state, sheets, *active_sheet);
            }
            PreviewData::Epub {
                title,
                author,
                chapters,
                active_chapter,
                images,
            } => {
                crate::features::epub::populate_state(
                    state,
                    title,
                    author,
                    chapters,
                    *active_chapter,
                    images,
                );
            }
            PreviewData::Json {
                nodes,
                pretty,
                has_parse_error,
                ..
            } => {
                crate::features::json::populate_state(state, nodes, pretty, *has_parse_error);
            }
            PreviewData::Font {
                name,
                metadata,
                sample,
                sample_width,
                sample_height,
                ..
            } => {
                crate::features::font::populate_state(
                    state,
                    name,
                    metadata,
                    sample,
                    *sample_width,
                    *sample_height,
                );
            }
            PreviewData::Audio {
                title,
                artist,
                album,
                duration_secs,
                metadata,
                cover_art,
                ..
            } => {
                let total_secs = *duration_secs;
                let hours = total_secs / 3600;
                let mins = (total_secs % 3600) / 60;
                let rem_secs = total_secs % 60;
                let time_str = if total_secs > 0 {
                    if hours > 0 {
                        format!("0:00:00 / {hours}:{mins:02}:{rem_secs:02}")
                    } else {
                        format!("0:00 / {mins}:{rem_secs:02}")
                    }
                } else {
                    String::new()
                };
                let cover_handle = cover_art
                    .as_ref()
                    .map(|bytes| iced::widget::image::Handle::from_bytes(bytes.clone()));
                let prev_volume = state.audio.volume;
                let prev_muted = state.audio.muted;
                let prev_vol_before = state.audio.volume_before_mute;
                state.audio = crate::features::audio::AudioState {
                    title: title.clone(),
                    artist: artist.clone(),
                    album: album.clone(),
                    duration_secs: *duration_secs as f64,
                    metadata: metadata.clone(),
                    cover_art: cover_handle,
                    time: time_str,
                    volume: if prev_volume > 0.0 || prev_muted {
                        prev_volume
                    } else {
                        1.0
                    },
                    muted: prev_muted,
                    volume_before_mute: if prev_vol_before > 0.0 {
                        prev_vol_before
                    } else {
                        1.0
                    },
                    ..Default::default()
                };
                state.file_type_text = "Audio File".to_string();
            }
            PreviewData::Media { metadata, .. } => {
                state.media = crate::core::MediaState::default();
                state.media.metadata = metadata.clone();
                state.file_type_text = "Video File".to_string();
            }
            PreviewData::Error(err) => {
                state.file_type_text = format!("Error: {err}");
            }
        }

        state.apply_scroll_controllers();
    }

    pub fn initial_window_size(&self, state: &KglanceState) -> iced::Size {
        match self {
            PreviewData::Image { width, height, .. } => {
                let max_w = if state.window_width > 0.0 {
                    state.window_width
                } else {
                    state.window_default_size.width
                };
                let max_h = if state.window_height > 0.0 {
                    state.window_height
                } else {
                    state.window_default_size.height
                };
                crate::features::image::view::calculate_window_size(max_w, max_h, *width, *height)
            }
            PreviewData::Audio { .. } => iced::Size::new(
                680.0f32.min(state.window_default_size.width),
                280.0f32.min(state.window_default_size.height),
            ),
            PreviewData::Media { .. } => iced::Size::new(
                850.0f32.min(state.window_default_size.width),
                550.0f32.min(state.window_default_size.height),
            ),
            _ => state.window_default_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_populate_state_metadata() {
        let temp_dir = tempfile::TempDir::new().expect("failed to create temp dir");
        let test_file = temp_dir.path().join("sample.txt");
        let test_content = b"Hello, metadata test!";
        std::fs::write(&test_file, test_content).expect("failed to write test file");

        let mut state = KglanceState {
            file_name: test_file.to_string_lossy().to_string(),
            ..Default::default()
        };

        let preview_data = PreviewData::Text {
            content: "Hello, metadata test!".to_string(),
            line_numbers: "1".to_string(),
            language: "Plain Text".to_string(),
        };

        preview_data.populate_state(&mut state);

        assert_eq!(state.file_type_text, "Plain Text");
        assert!(
            !state.file_size_text.is_empty(),
            "file_size_text should be populated"
        );
        assert!(
            state.file_size_text.contains("B"),
            "file_size_text should display bytes"
        );
    }

    #[test]
    fn test_font_preview_populate_state() {
        let temp_dir = tempfile::TempDir::new().expect("failed to create temp dir");
        let test_file = temp_dir.path().join("font.ttf");
        std::fs::write(&test_file, b"dummy").expect("failed to write test file");

        let mut state = KglanceState {
            file_name: test_file.to_string_lossy().to_string(),
            ..Default::default()
        };

        let sample = vec![128u8; 60 * 30 * 4];
        let preview_data = PreviewData::Font {
            name: "TestFont".to_string(),
            family: "TestFont".to_string(),
            post_script_name: Some("TestFont-Regular".to_string()),
            weight: 400,
            is_italic: false,
            metadata: "Name: TestFont".to_string(),
            sample: sample.clone(),
            sample_width: 60,
            sample_height: 30,
            data: vec![0u8; 100],
        };

        preview_data.populate_state(&mut state);

        assert_eq!(state.file_type_text, "Font");
        assert!(state.image.format_info.contains("TestFont"));
        assert_eq!(state.image.exif_content, "Name: TestFont");
        assert!(state.image.handle.is_some());
        assert_eq!(state.image.image_bytes, sample);
    }

    #[test]
    fn test_populate_state_preserves_disabled_smooth_scroll() {
        let mut state = KglanceState::default();
        let scroll_cfg = crate::core::config::ScrollConfigOptions {
            smooth_scroll_enabled: false,
            friction: 4.2,
            spring_stiffness: 180.0,
        };
        state.apply_scroll_config(&scroll_cfg);

        // Populate Text
        let text_preview = PreviewData::Text {
            content: "hello world".to_string(),
            line_numbers: "1".to_string(),
            language: "rs".to_string(),
        };
        text_preview.populate_state(&mut state);
        assert!(!state.text.scroll_controller.smooth_enabled);
        assert!(!state.text.smooth_scroll.smooth_enabled);

        // Populate Markdown
        let md_preview = PreviewData::Markdown {
            blocks: vec![crate::parsers::markdown::Block::Paragraph(vec![
                crate::parsers::markdown::Inline::Text("test".to_string()),
            ])],
            raw_text: "test".to_string(),
        };
        md_preview.populate_state(&mut state);
        assert!(!state.markdown.scroll_controller.smooth_enabled);
        assert!(!state.markdown.smooth_scroll.smooth_enabled);

        // Populate JSON
        let json_preview = PreviewData::Json {
            nodes: Vec::new(),
            content: "{}".to_string(),
            pretty: "{}".to_string(),
            has_parse_error: false,
        };
        json_preview.populate_state(&mut state);
        assert!(!state.json.scroll_controller.smooth_enabled);
        assert!(!state.json.smooth_scroll.smooth_enabled);
        assert!(!state.json.raw_text.scroll_controller.smooth_enabled);
        assert!(!state.json.raw_text.smooth_scroll.smooth_enabled);
    }

    #[test]
    fn test_audio_preview_populate_state() {
        let mut state = KglanceState::default();
        let preview = PreviewData::Audio {
            path: "/path/to/song.flac".to_string(),
            title: "Midnight City".to_string(),
            artist: "M83".to_string(),
            album: "Hurry Up, We're Dreaming".to_string(),
            duration_secs: 243,
            metadata: "FLAC • 44.1 kHz • 24-bit".to_string(),
            cover_art: None,
        };
        preview.populate_state(&mut state);
        assert_eq!(state.file_type_text, "Audio File");
        assert_eq!(state.audio.title, "Midnight City");
        assert_eq!(state.audio.artist, "M83");
        assert_eq!(state.audio.album, "Hurry Up, We're Dreaming");
        assert_eq!(state.audio.duration_secs, 243.0);
        assert_eq!(state.audio.time, "0:00 / 4:03");
        assert_eq!(state.audio.metadata, "FLAC • 44.1 kHz • 24-bit");
    }

    #[test]
    fn test_is_slow_to_parse() {
        assert!(is_slow_to_parse(Path::new("doc.pdf")));
        assert!(is_slow_to_parse(Path::new("doc.typ")));
        assert!(is_slow_to_parse(Path::new("doc.docx")));
        assert!(is_slow_to_parse(Path::new("doc.xlsx")));
        assert!(is_slow_to_parse(Path::new("archive.7z")));
        assert!(is_slow_to_parse(Path::new("archive.zip")));
        assert!(!is_slow_to_parse(Path::new("small.txt")));
        assert!(!is_slow_to_parse(Path::new("readme.md")));
    }
}
