use crate::core::types::KglanceState;
use crate::features::common::parser::traits::ParseError;
use crate::features::pdf::types::PageDimensions;
use crate::parsers::markdown::{Block, estimated_block_height, extract_toc};
use std::collections::HashMap;
use std::path::Path;

/// Compute cumulative Y offsets for each block in a single linear pass.
/// Returns `(offsets, total_height)` where `offsets[i]` is the Y start of block `i`.
fn compute_block_y_offsets(
    blocks: &[Block],
    font_size: f32,
    image_sizes: &HashMap<usize, (u32, u32)>,
) -> (Vec<f32>, f32) {
    let mut offsets = Vec::with_capacity(blocks.len());
    let mut y: f32 = 15.0;
    for (i, block) in blocks.iter().enumerate() {
        offsets.push(y);
        y += estimated_block_height(block, font_size, i, image_sizes);
    }
    (offsets, y)
}

/// Compute cumulative Y offsets for PDF pages given their dimensions and display width.
/// Returns `(offsets, ends, total_height)`.
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
        blocks: Vec<crate::parsers::markdown::Block>,
        raw_text: String,
    },
    Pdf {
        page_count: usize,
        current_page: usize,
        data: Vec<u8>,
        width: u32,
        height: u32,
        outline: Vec<crate::parsers::pdf::PdfTocEntry>,
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
        outline: Vec<crate::parsers::pdf::PdfTocEntry>,
        page_dimensions: Vec<PageDimensions>,
    },
    Media {
        url: String,
        metadata: String,
        thumbnail_or_waveform: Vec<u8>,
        width: u32,
        height: u32,
    },
    Folder {
        rows: Vec<crate::core::types::FolderRowState>,
        total_size: u64,
    },
    Spreadsheet {
        sheets: Vec<crate::core::types::SheetInfo>,
        active_sheet: usize,
    },
    Json {
        nodes: Vec<crate::parsers::json::JsonNode>,
        content: String,
        pretty: String,
        has_parse_error: bool,
    },
    Epub {
        title: String,
        author: String,
        chapters: Vec<crate::core::types::EpubChapterInfo>,
        active_chapter: usize,
        images: std::collections::HashMap<String, Vec<u8>>,
    },
    Font {
        name: String,
        metadata: String,
        sample: Vec<u8>,
        sample_width: u32,
        sample_height: u32,
    },
    Error(String),
}

pub trait FilePreviewer {
    fn parse(&self, path: &Path) -> Result<PreviewData, ParseError>;
}

impl PreviewData {
    pub fn populate_state(&self, state: &mut KglanceState) {
        if !state.file_name.is_empty() {
            let path = Path::new(&state.file_name);
            if let Ok(meta) = std::fs::metadata(path) {
                state.file_size_text = crate::parsers::human_size(meta.len());
                if let Ok(modified) = meta.modified() {
                    state.file_modified_text = crate::parsers::human_time(modified);
                }
            }
        }

        match self {
            PreviewData::Text {
                content,
                line_numbers,
                language,
            } => {
                let words = content.split_whitespace().count();
                let chars = content.chars().count();
                let mins = (words as f32 / 200.0).ceil() as usize;

                state.text.content = iced::widget::text_editor::Content::with_text(content);
                let path_ext = if !state.file_name.is_empty() {
                    Path::new(&state.file_name)
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or(language)
                } else {
                    language
                };
                state.text.extension = path_ext.to_string();
                state.text.line_numbers.clone_from(line_numbers);
                state.text.word_count = words;
                state.text.char_count = chars;
                state.text.reading_time_mins = mins;
                state.text.symbols = crate::features::text::extract_symbols(content, path_ext);
                state.file_type_text = language.clone();
            }
            PreviewData::Image {
                data,
                width,
                height,
                format_info,
                exif_content,
            } => {
                state.image = crate::core::ImageState {
                    handle: Some(iced::widget::image::Handle::from_bytes(data.clone())),
                    image_bytes: data.clone(),
                    width: *width,
                    height: *height,
                    exif_content: exif_content.clone().unwrap_or_default(),
                    format_info: format_info.clone(),
                    load_state: crate::features::image::ImageLoadState::Ready,
                    ..Default::default()
                };
                state.file_type_text.clone_from(format_info);
            }
            PreviewData::Pdf {
                page_count,
                outline,
                page_dimensions,
                ..
            } => {
                let old_sidebar_visible = state.pdf.sidebar_visible;
                let old_sidebar_mode = state.pdf.sidebar_mode;
                let old_sidebar_width = state.pdf.sidebar_width;
                state.pdf = crate::core::PdfState::default();
                state.pdf.page_count = *page_count;
                state.pdf.pages = crate::core::types::PageCache::new(*page_count);
                state.pdf.thumbnails = crate::core::types::ThumbnailCache::new(*page_count);
                state.pdf.sidebar_visible = old_sidebar_visible;
                state.pdf.sidebar_mode = old_sidebar_mode;
                state.pdf.sidebar_width = if old_sidebar_width > 0.0 {
                    old_sidebar_width
                } else {
                    220.0
                };
                state.pdf.outline = outline.clone();
                state.pdf.page_dimensions = page_dimensions.clone();
                let display_width = (state.font_size / 14.0) * 800.0;
                let (offsets, ends, total_h) =
                    compute_pdf_page_offsets(page_dimensions, display_width, 4.0);
                state.pdf.display_width = display_width;
                state.pdf.page_y_offsets = offsets;
                state.pdf.page_ends = ends;
                state.pdf.total_content_height = total_h;
                state.file_type_text = "PDF Document".to_string();
            }
            PreviewData::Typst {
                page_count,
                source,
                error,
                outline,
                page_dimensions,
                ..
            } => {
                let old_sidebar_visible = state.typst.pdf.sidebar_visible;
                let old_sidebar_mode = state.typst.pdf.sidebar_mode;
                let old_sidebar_width = state.typst.pdf.sidebar_width;
                let display_width = (state.font_size / 14.0) * 800.0;
                let (offsets, ends, total_h) =
                    compute_pdf_page_offsets(page_dimensions, display_width, 4.0);
                state.typst = crate::core::TypstState {
                    pdf: crate::core::PdfState {
                        page_count: *page_count,
                        pages: crate::core::types::PageCache::new(*page_count),
                        thumbnails: crate::core::types::ThumbnailCache::new(*page_count),
                        sidebar_visible: old_sidebar_visible,
                        sidebar_mode: old_sidebar_mode,
                        sidebar_width: if old_sidebar_width > 0.0 {
                            old_sidebar_width
                        } else {
                            220.0
                        },
                        outline: outline.clone(),
                        page_dimensions: page_dimensions.clone(),
                        display_width,
                        page_y_offsets: offsets,
                        page_ends: ends,
                        total_content_height: total_h,
                        ..Default::default()
                    },
                    source_content: iced::widget::text_editor::Content::with_text(source),
                    show_source: error.is_some(),
                    error: error.clone(),
                };
                state.file_type_text = "Typst Document".to_string();
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
                let fs = state.font_size;
                let full_text: String = blocks
                    .iter()
                    .map(|b| match b {
                        Block::Heading { content, .. } | Block::Paragraph(content) => {
                            crate::parsers::markdown::flatten_inlines(content)
                        }
                        Block::CodeBlock { code, .. } => code.clone(),
                        Block::Math(code) => code.clone(),
                        _ => String::new(),
                    })
                    .collect::<Vec<_>>()
                    .join(" ");

                let words = full_text.split_whitespace().count();
                let chars = full_text.chars().count();
                let mins = (words as f32 / 200.0).ceil() as usize;

                let old_scroll_y = state.markdown.scroll_y;
                let old_toc_visible = state.markdown.toc_visible;
                let old_collapsed = std::mem::take(&mut state.markdown.collapsed_headings);
                let old_mermaid = std::mem::take(&mut state.markdown.cached_mermaid_handles);
                let old_image_h = std::mem::take(&mut state.markdown.cached_image_handles);
                let old_image_s = std::mem::take(&mut state.markdown.cached_image_sizes);

                let old_sidebar_w = state.markdown.sidebar_width;

                // Compute cumulative Y offsets for virtual rendering (one pass over blocks).
                let (block_y_offsets, total_content_height) =
                    compute_block_y_offsets(blocks, fs, &old_image_s);

                state.markdown = crate::core::types::MarkdownState {
                    toc: extract_toc(blocks, fs, &old_image_s),
                    toc_visible: old_toc_visible,
                    sidebar_width: if old_sidebar_w > 0.0 {
                        old_sidebar_w
                    } else {
                        220.0
                    },
                    sidebar_resizing: false,
                    sidebar_drag_start_x: None,
                    sidebar_drag_start_width: 220.0,
                    collapsed_headings: old_collapsed,
                    scroll_y: old_scroll_y,
                    cached_mermaid_handles: old_mermaid,
                    cached_image_handles: old_image_h,
                    cached_image_sizes: old_image_s,
                    word_count: words,
                    char_count: chars,
                    reading_time_mins: mins,
                    block_y_offsets,
                    total_content_height,
                    viewport_height: 800.0,
                    search_visible: false,
                    search_query: String::new(),
                    search_match_count: 0,
                    search_match_index: 0,
                    search_match_blocks: Vec::new(),
                    search_info: String::new(),
                    selected_text: None,
                    selection_range: None,
                    is_dragging_selection: false,
                    auto_scroll_delta: None,
                    drag_last_y: 0.0,
                };
                for (i, block) in blocks.iter().enumerate() {
                    if let Block::Mermaid {
                        rendered: Some(png),
                        ..
                    } = block
                    {
                        let handle = match image::load_from_memory(png) {
                            Ok(img) => {
                                let rgba = img.to_rgba8();
                                let (w, h) = rgba.dimensions();
                                iced::widget::image::Handle::from_rgba(w, h, rgba.into_raw())
                            }
                            Err(_) => iced::widget::image::Handle::from_bytes(png.clone()),
                        };
                        state.markdown.cached_mermaid_handles.insert(i, handle);
                    }
                }
                state.file_type_text = "Markdown Document".to_string();
            }
            PreviewData::Spreadsheet {
                sheets,
                active_sheet,
            } => {
                state.spreadsheet.sheets = sheets.clone();
                state.spreadsheet.active_sheet = *active_sheet;
                state.file_type_text = "Spreadsheet".to_string();
            }
            PreviewData::Epub {
                title,
                author,
                chapters,
                active_chapter,
                images,
            } => {
                let old_sidebar = state.epub.sidebar_visible;
                let old_scroll = state.epub.scroll_y;
                let old_collapsed = std::mem::take(&mut state.epub.collapsed_chapters);
                let mut markdown_state = crate::core::MarkdownState::default();
                let mut block_global_idx = 0;
                for ch in chapters {
                    for block in &ch.blocks {
                        if let crate::parsers::markdown::Block::Image { path: img_path, .. } = block
                        {
                            let filename = std::path::Path::new(img_path)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or(img_path);
                            if let Some(bytes) =
                                images.get(img_path).or_else(|| images.get(filename))
                            {
                                let handle = iced::widget::image::Handle::from_bytes(bytes.clone());
                                markdown_state
                                    .cached_image_handles
                                    .insert(block_global_idx, handle);
                                if let Ok(img) = image::load_from_memory(bytes) {
                                    markdown_state
                                        .cached_image_sizes
                                        .insert(block_global_idx, (img.width(), img.height()));
                                }
                            }
                        }
                        block_global_idx += 1;
                    }
                }
                let old_sidebar_width = state.epub.sidebar_width;
                state.epub = crate::core::types::EpubState {
                    title: title.clone(),
                    author: author.clone(),
                    chapters: chapters.clone(),
                    active_chapter: *active_chapter,
                    sidebar_visible: old_sidebar,
                    sidebar_width: if old_sidebar_width > 0.0 {
                        old_sidebar_width
                    } else {
                        240.0
                    },
                    sidebar_resizing: false,
                    sidebar_drag_start_x: None,
                    sidebar_drag_start_width: 240.0,
                    scroll_y: old_scroll,
                    collapsed_chapters: old_collapsed,
                    markdown_state,
                };
                state.file_type_text = format!("EPUB E-Book ({} chapters)", chapters.len());
            }
            PreviewData::Json {
                nodes,
                content: _,
                pretty,
                has_parse_error,
            } => {
                let old_scroll = state.json.scroll_y;
                let old_tree_mode = state.json.tree_mode;

                let mut expanded = std::collections::HashSet::new();
                for (i, node) in nodes.iter().enumerate() {
                    if node.depth == 0 {
                        expanded.insert(i);
                    }
                }

                let minified = serde_json::from_str::<serde_json::Value>(pretty)
                    .ok()
                    .and_then(|v| serde_json::to_string(&v).ok())
                    .unwrap_or_else(|| pretty.clone());

                state.json = crate::core::types::JsonState {
                    nodes: nodes.clone(),
                    expanded,
                    raw_content: pretty.clone(),
                    pretty_content: pretty.clone(),
                    tree_mode: old_tree_mode,
                    scroll_y: old_scroll,
                    has_parse_error: *has_parse_error,
                    raw_editor: iced::widget::text_editor::Content::with_text(pretty),
                    search_visible: false,
                    search_query: String::new(),
                    minified_content: minified,
                    raw_pretty: true,
                    active_node: None,
                    editing_node: None,
                    edit_value: String::new(),
                    schema_visible: false,
                    schema_info: String::new(),
                };
                state.file_type_text = "JSON Document".to_string();
            }
            PreviewData::Font {
                name,
                metadata,
                sample,
                sample_width,
                sample_height,
            } => {
                state.image = crate::core::ImageState {
                    handle: Some(iced::widget::image::Handle::from_rgba(
                        *sample_width,
                        *sample_height,
                        sample.clone(),
                    )),
                    image_bytes: sample.clone(),
                    width: *sample_width,
                    height: *sample_height,
                    format_info: format!("Font — {name}"),
                    exif_content: metadata.clone(),
                    load_state: crate::features::image::ImageLoadState::Ready,
                    ..Default::default()
                };
                state.file_type_text = "Font".to_string();
            }
            PreviewData::Media { metadata, .. } => {
                state.media = crate::core::MediaState::default();
                state.media.metadata = metadata.clone();
                state.file_type_text = if metadata.contains("Video") {
                    "Video File"
                } else {
                    "Audio File"
                }
                .to_string();
            }
            PreviewData::Error(err) => {
                state.file_type_text = format!("Error: {}", err);
            }
        }
    }

    pub fn initial_window_size(&self) -> iced::Size {
        match self {
            PreviewData::Image { width, height, .. } => {
                crate::features::image::view::helpers::calculate_window_size(*width, *height)
            }
            PreviewData::Media { .. } => iced::Size::new(850.0, 550.0),
            _ => iced::Size::new(1024.0, 768.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_populate_state_metadata() {
        let temp_dir = std::env::temp_dir().join("kglance-meta-test");
        let _ = std::fs::create_dir_all(&temp_dir);
        let test_file = temp_dir.join("sample.txt");
        let test_content = b"Hello, metadata test!";
        std::fs::write(&test_file, test_content).unwrap();

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

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_initial_window_size() {
        let img_preview = PreviewData::Image {
            data: vec![],
            width: 800,
            height: 600,
            format_info: "PNG".into(),
            exif_content: None,
        };
        let size = img_preview.initial_window_size();
        assert!(size.width > 0.0 && size.height > 0.0);

        let media_preview = PreviewData::Media {
            url: "test.mp4".into(),
            metadata: "".into(),
            thumbnail_or_waveform: vec![],
            width: 0,
            height: 0,
        };
        assert_eq!(
            media_preview.initial_window_size(),
            iced::Size::new(850.0, 550.0)
        );

        let text_preview = PreviewData::Text {
            content: "".into(),
            line_numbers: "".into(),
            language: "".into(),
        };
        assert_eq!(
            text_preview.initial_window_size(),
            iced::Size::new(1024.0, 768.0)
        );
    }

    #[test]
    fn test_font_preview_populate_state() {
        let temp_dir = std::env::temp_dir().join("kglance-font-populate-test");
        let _ = std::fs::create_dir_all(&temp_dir);
        let test_file = temp_dir.join("font.ttf");
        std::fs::write(&test_file, b"dummy").unwrap();

        let mut state = KglanceState {
            file_name: test_file.to_string_lossy().to_string(),
            ..Default::default()
        };

        let sample = vec![128u8; 60 * 30 * 4];
        let preview_data = PreviewData::Font {
            name: "TestFont".to_string(),
            metadata: "Name: TestFont".to_string(),
            sample: sample.clone(),
            sample_width: 60,
            sample_height: 30,
        };

        preview_data.populate_state(&mut state);

        assert_eq!(state.file_type_text, "Font");
        assert!(state.image.format_info.contains("TestFont"));
        assert_eq!(state.image.exif_content, "Name: TestFont");
        assert!(state.image.handle.is_some());
        assert_eq!(state.image.width, 60);
        assert_eq!(state.image.height, 30);
        assert_eq!(state.image.image_bytes, sample);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_font_preview_initial_window_size() {
        let preview = PreviewData::Font {
            name: "Test".into(),
            metadata: "meta".into(),
            sample: vec![0u8; 100 * 50 * 4],
            sample_width: 100,
            sample_height: 50,
        };
        let size = preview.initial_window_size();
        assert!(size.width > 0.0 && size.height > 0.0);
    }
}
