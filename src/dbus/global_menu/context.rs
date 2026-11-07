use crate::core::PreviewData;
use crate::core::config::{EpubReadingMode, RubyDisplayMode};
use crate::core::types::KglanceState;
use dbus_global_menu::{MenuItem, ToggleState};

// IDs for Context Menu items
pub const ID_IMAGE_ROTATE_CW: i32 = 401;
pub const ID_IMAGE_ROTATE_CCW: i32 = 402;
pub const ID_IMAGE_FLIP_H: i32 = 403;
pub const ID_IMAGE_FLIP_V: i32 = 404;
pub const ID_IMAGE_FIT: i32 = 406;
pub const ID_IMAGE_ACTUAL_SIZE: i32 = 407;
pub const ID_IMAGE_TOGGLE_INFO: i32 = 409;

pub const ID_PDF_PREV_PAGE: i32 = 411;
pub const ID_PDF_NEXT_PAGE: i32 = 412;
pub const ID_PDF_TOGGLE_TOC: i32 = 413;
pub const ID_PDF_GOTO_PAGE: i32 = 414;

pub const ID_EPUB_PREV_CHAPTER: i32 = 421;
pub const ID_EPUB_NEXT_CHAPTER: i32 = 422;
pub const ID_EPUB_TOGGLE_TOC: i32 = 423;
pub const ID_EPUB_OVERRIDE_FONT: i32 = 425;
pub const ID_EPUB_READING_MODE_SUBMENU: i32 = 426;
pub const ID_EPUB_MODE_CONTINUOUS: i32 = 427;
pub const ID_EPUB_MODE_SINGLE: i32 = 428;
pub const ID_EPUB_RUBY_SUBMENU: i32 = 429;
pub const ID_EPUB_RUBY_STACKED: i32 = 430;
pub const ID_EPUB_RUBY_TOOLTIP: i32 = 431;
pub const ID_EPUB_RUBY_PARENTHESES: i32 = 432;
pub const ID_EPUB_RUBY_HIDDEN: i32 = 433;

pub const ID_JSON_TOGGLE_MODE: i32 = 441;
pub const ID_JSON_TOGGLE_FORMAT: i32 = 442;
pub const ID_JSON_EXPAND_ALL: i32 = 443;
pub const ID_JSON_COLLAPSE_ALL: i32 = 444;

pub const ID_MARKDOWN_TOGGLE_TOC: i32 = 461;
pub const ID_MARKDOWN_MERMAID_CLI: i32 = 463;

pub const ID_TEXT_TOGGLE_OUTLINE: i32 = 471;
pub const ID_TEXT_GOTO_LINE: i32 = 472;
pub const ID_TEXT_WORD_WRAP: i32 = 474;

pub const ID_TYPST_TOGGLE_SOURCE: i32 = 481;
pub const ID_TYPST_PREV_PAGE: i32 = 482;
pub const ID_TYPST_NEXT_PAGE: i32 = 483;
pub const ID_TYPST_TOGGLE_TOC: i32 = 484;

pub const ID_MEDIA_PLAY_PAUSE: i32 = 491;
pub const ID_MEDIA_TOGGLE_MUTE: i32 = 492;
pub const ID_MEDIA_CYCLE_SUBTITLES: i32 = 493;
pub const ID_MEDIA_TOGGLE_SUBTITLES: i32 = 494;
pub const ID_MEDIA_SEEK_BACK: i32 = 495;
pub const ID_MEDIA_SEEK_FORWARD: i32 = 496;

pub const ID_SHEET_PREV_SHEET: i32 = 511;
pub const ID_SHEET_NEXT_SHEET: i32 = 512;
pub const ID_SHEET_SELECT_ALL: i32 = 514;
pub const ID_SHEET_COPY_SELECTION: i32 = 515;

pub const ID_ZOOM_IN: i32 = 530;
pub const ID_ZOOM_OUT: i32 = 531;
pub const ID_ZOOM_RESET: i32 = 532;

/// Snapshotted viewer and format settings used to render dynamic context menus.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContextMenuOptions {
    pub epub_override_book_font: bool,
    pub epub_reading_mode: EpubReadingMode,
    pub ruby_display_mode: RubyDisplayMode,

    pub json_tree_mode: bool,

    pub prefer_mermaid_cli: bool,

    pub word_wrap: bool,

    pub typst_show_source: bool,

    pub image_show_info: bool,

    pub media_playing: bool,
    pub media_subtitles_enabled: bool,
    pub audio_muted: bool,

    pub sheet_count: usize,
    pub active_sheet: usize,
}

impl ContextMenuOptions {
    pub fn from_state(state: &KglanceState) -> Self {
        Self {
            epub_override_book_font: state.epub_override_book_font,
            epub_reading_mode: state.epub_reading_mode,
            ruby_display_mode: state.ruby_display_mode,

            json_tree_mode: state.json.tree_mode,

            prefer_mermaid_cli: state.prefer_mermaid_cli,

            word_wrap: state.word_wrap,

            typst_show_source: state.typst.show_source,

            image_show_info: state.image.show_info,

            media_playing: state.media.playing,
            media_subtitles_enabled: state.media.subtitles_enabled,
            audio_muted: state.audio.muted,

            sheet_count: state.spreadsheet.sheets.len(),
            active_sheet: state.spreadsheet.active_sheet,
        }
    }
}

fn zoom_menu_items(sep_id: i32) -> Vec<MenuItem> {
    vec![
        MenuItem::separator(sep_id),
        MenuItem::new(ID_ZOOM_IN, "Zoom In")
            .with_action("zoom.in")
            .with_shortcut_str("Ctrl+plus"),
        MenuItem::new(ID_ZOOM_OUT, "Zoom Out")
            .with_action("zoom.out")
            .with_shortcut_str("Ctrl+minus"),
        MenuItem::new(ID_ZOOM_RESET, "Reset Zoom")
            .with_action("zoom.reset")
            .with_shortcut_str("Ctrl+0"),
    ]
}

fn bool_to_toggle(val: bool) -> ToggleState {
    if val {
        ToggleState::On
    } else {
        ToggleState::Off
    }
}

pub fn build_context_menu_items(
    content: Option<&PreviewData>,
    options: &ContextMenuOptions,
) -> (String, Vec<MenuItem>) {
    let Some(content) = content else {
        return (
            "Actions".to_string(),
            vec![
                MenuItem::new(499, "No specific actions")
                    .with_action("none")
                    .enabled(false),
            ],
        );
    };

    match content {
        PreviewData::Image { .. } => {
            let mut items = vec![
                MenuItem::new(ID_IMAGE_ROTATE_CW, "Rotate Clockwise")
                    .with_action("image.rotate_cw")
                    .with_shortcut_str("R"),
                MenuItem::new(ID_IMAGE_ROTATE_CCW, "Rotate Counter-Clockwise")
                    .with_action("image.rotate_ccw")
                    .with_shortcut_str("Shift+R"),
                MenuItem::new(ID_IMAGE_FLIP_H, "Flip Horizontal").with_action("image.flip_h"),
                MenuItem::new(ID_IMAGE_FLIP_V, "Flip Vertical").with_action("image.flip_v"),
                MenuItem::separator(405),
                MenuItem::new(ID_IMAGE_FIT, "Fit to Window")
                    .with_action("image.fit")
                    .with_shortcut_str("F"),
                MenuItem::new(ID_IMAGE_ACTUAL_SIZE, "Original Size (100%)")
                    .with_action("image.actual_size")
                    .with_shortcut_str("1"),
                MenuItem::separator(408),
                MenuItem::new(ID_IMAGE_TOGGLE_INFO, "Show Info & Metadata")
                    .with_checkbox(bool_to_toggle(options.image_show_info))
                    .with_action("image.toggle_info")
                    .with_shortcut_str("I"),
            ];
            items.extend(zoom_menu_items(410));
            ("Image".to_string(), items)
        }

        PreviewData::Pdf { .. } => {
            let mut items = vec![
                MenuItem::new(ID_PDF_PREV_PAGE, "Previous Page")
                    .with_action("pdf.prev_page")
                    .with_shortcut_str("Page_Up"),
                MenuItem::new(ID_PDF_NEXT_PAGE, "Next Page")
                    .with_action("pdf.next_page")
                    .with_shortcut_str("Page_Down"),
                MenuItem::new(ID_PDF_TOGGLE_TOC, "Toggle Table of Contents")
                    .with_action("pdf.toggle_toc")
                    .with_shortcut_str("T"),
                MenuItem::new(ID_PDF_GOTO_PAGE, "Go to Page...")
                    .with_action("pdf.goto_page")
                    .with_shortcut_str("Ctrl+G"),
            ];
            items.extend(zoom_menu_items(415));
            ("PDF".to_string(), items)
        }

        PreviewData::Epub { .. } => {
            let reading_mode_is_cont = options.epub_reading_mode == EpubReadingMode::Continuous;
            let mut reading_mode_submenu =
                MenuItem::new(ID_EPUB_READING_MODE_SUBMENU, "Reading Mode");
            reading_mode_submenu.children = vec![
                MenuItem::new(ID_EPUB_MODE_CONTINUOUS, "Continuous Scroll")
                    .with_radio(bool_to_toggle(reading_mode_is_cont))
                    .with_action("epub.reading_mode.continuous"),
                MenuItem::new(ID_EPUB_MODE_SINGLE, "Single Chapter (Paged)")
                    .with_radio(bool_to_toggle(!reading_mode_is_cont))
                    .with_action("epub.reading_mode.single"),
            ];

            let ruby_mode = options.ruby_display_mode;
            let mut ruby_submenu = MenuItem::new(ID_EPUB_RUBY_SUBMENU, "Furigana / Ruby Display");
            ruby_submenu.children = vec![
                MenuItem::new(ID_EPUB_RUBY_STACKED, "Stacked (Above text)")
                    .with_radio(bool_to_toggle(ruby_mode == RubyDisplayMode::Stacked))
                    .with_action("epub.ruby_mode.stacked"),
                MenuItem::new(ID_EPUB_RUBY_TOOLTIP, "Tooltip (Hover)")
                    .with_radio(bool_to_toggle(ruby_mode == RubyDisplayMode::Tooltip))
                    .with_action("epub.ruby_mode.tooltip"),
                MenuItem::new(ID_EPUB_RUBY_PARENTHESES, "Parentheses (Text(Ruby))")
                    .with_radio(bool_to_toggle(ruby_mode == RubyDisplayMode::Parentheses))
                    .with_action("epub.ruby_mode.parentheses"),
                MenuItem::new(ID_EPUB_RUBY_HIDDEN, "Hidden")
                    .with_radio(bool_to_toggle(ruby_mode == RubyDisplayMode::Hidden))
                    .with_action("epub.ruby_mode.hidden"),
            ];

            let mut items = vec![
                MenuItem::new(ID_EPUB_PREV_CHAPTER, "Previous Chapter")
                    .with_action("epub.prev_chapter")
                    .with_shortcut_str("Page_Up"),
                MenuItem::new(ID_EPUB_NEXT_CHAPTER, "Next Chapter")
                    .with_action("epub.next_chapter")
                    .with_shortcut_str("Page_Down"),
                MenuItem::new(ID_EPUB_TOGGLE_TOC, "Toggle Table of Contents")
                    .with_action("epub.toggle_toc")
                    .with_shortcut_str("T"),
                MenuItem::separator(424),
                MenuItem::new(ID_EPUB_OVERRIDE_FONT, "Override Book Fonts")
                    .with_checkbox(bool_to_toggle(options.epub_override_book_font))
                    .with_action("epub.toggle_override_font"),
                reading_mode_submenu,
                ruby_submenu,
            ];
            items.extend(zoom_menu_items(434));
            ("EPUB".to_string(), items)
        }

        PreviewData::Json { .. } => {
            let mut items = vec![
                MenuItem::new(ID_JSON_TOGGLE_MODE, "Interactive Tree View")
                    .with_checkbox(bool_to_toggle(options.json_tree_mode))
                    .with_action("json.toggle_mode")
                    .with_shortcut_str("Ctrl+M"),
                MenuItem::new(ID_JSON_TOGGLE_FORMAT, "Toggle Format (Pretty / Compact)")
                    .with_action("json.toggle_format"),
                MenuItem::separator(442),
                MenuItem::new(ID_JSON_EXPAND_ALL, "Expand All Nodes")
                    .with_action("json.expand_all"),
                MenuItem::new(ID_JSON_COLLAPSE_ALL, "Collapse All Nodes")
                    .with_action("json.collapse_all"),
            ];
            items.extend(zoom_menu_items(445));
            ("JSON".to_string(), items)
        }

        PreviewData::Markdown { .. } => {
            let mut items = vec![
                MenuItem::new(ID_MARKDOWN_TOGGLE_TOC, "Toggle Table of Contents")
                    .with_action("markdown.toggle_toc")
                    .with_shortcut_str("T"),
                MenuItem::separator(462),
                MenuItem::new(ID_MARKDOWN_MERMAID_CLI, "Prefer Mermaid Diagram CLI")
                    .with_checkbox(bool_to_toggle(options.prefer_mermaid_cli))
                    .with_action("markdown.toggle_mermaid_cli"),
            ];
            items.extend(zoom_menu_items(464));
            ("Markdown".to_string(), items)
        }

        PreviewData::Text { .. } => {
            let mut items = vec![
                MenuItem::new(ID_TEXT_TOGGLE_OUTLINE, "Toggle Outline / Symbols")
                    .with_action("text.toggle_outline")
                    .with_shortcut_str("T"),
                MenuItem::new(ID_TEXT_GOTO_LINE, "Go to Line...")
                    .with_action("text.goto_line")
                    .with_shortcut_str("Ctrl+G"),
                MenuItem::separator(473),
                MenuItem::new(ID_TEXT_WORD_WRAP, "Word Wrap")
                    .with_checkbox(bool_to_toggle(options.word_wrap))
                    .with_action("toggle.word_wrap")
                    .with_shortcut_str("Ctrl+W"),
            ];
            items.extend(zoom_menu_items(475));
            ("Document".to_string(), items)
        }

        PreviewData::Typst { .. } => {
            let mut items = vec![
                MenuItem::new(ID_TYPST_TOGGLE_SOURCE, "Show Source Code")
                    .with_checkbox(bool_to_toggle(options.typst_show_source))
                    .with_action("typst.toggle_source")
                    .with_shortcut_str("Ctrl+M"),
                MenuItem::new(ID_TYPST_PREV_PAGE, "Previous Page")
                    .with_action("typst.prev_page")
                    .with_shortcut_str("Page_Up"),
                MenuItem::new(ID_TYPST_NEXT_PAGE, "Next Page")
                    .with_action("typst.next_page")
                    .with_shortcut_str("Page_Down"),
                MenuItem::new(ID_TYPST_TOGGLE_TOC, "Toggle Table of Contents")
                    .with_action("typst.toggle_toc")
                    .with_shortcut_str("T"),
            ];
            items.extend(zoom_menu_items(485));
            ("Typst".to_string(), items)
        }

        PreviewData::Spreadsheet { .. } => {
            let can_prev = options.active_sheet > 0;
            let can_next = options.active_sheet + 1 < options.sheet_count;
            let mut items = vec![
                MenuItem::new(ID_SHEET_PREV_SHEET, "Previous Sheet")
                    .with_action("sheet.prev_sheet")
                    .with_shortcut_str("Ctrl+Page_Up")
                    .enabled(can_prev),
                MenuItem::new(ID_SHEET_NEXT_SHEET, "Next Sheet")
                    .with_action("sheet.next_sheet")
                    .with_shortcut_str("Ctrl+Page_Down")
                    .enabled(can_next),
                MenuItem::separator(513),
                MenuItem::new(ID_SHEET_SELECT_ALL, "Select All Cells")
                    .with_action("sheet.select_all")
                    .with_shortcut_str("Ctrl+A"),
                MenuItem::new(ID_SHEET_COPY_SELECTION, "Copy Selected Cells")
                    .with_action("sheet.copy_selection")
                    .with_shortcut_str("Ctrl+C"),
            ];
            items.extend(zoom_menu_items(516));
            ("Spreadsheet".to_string(), items)
        }

        PreviewData::Audio { .. } => {
            let play_label = if options.media_playing {
                "Pause"
            } else {
                "Play"
            };
            let mute_label = if options.audio_muted {
                "Unmute Audio"
            } else {
                "Mute Audio"
            };
            (
                "Audio".to_string(),
                vec![
                    MenuItem::new(ID_MEDIA_PLAY_PAUSE, play_label)
                        .with_action("media.toggle_pause")
                        .with_shortcut_str("P"),
                    MenuItem::new(ID_MEDIA_TOGGLE_MUTE, mute_label)
                        .with_checkbox(bool_to_toggle(options.audio_muted))
                        .with_action("media.toggle_mute")
                        .with_shortcut_str("M"),
                    MenuItem::separator(494),
                    MenuItem::new(ID_MEDIA_SEEK_BACK, "Seek Backward (-5s)")
                        .with_action("media.seek_back"),
                    MenuItem::new(ID_MEDIA_SEEK_FORWARD, "Seek Forward (+5s)")
                        .with_action("media.seek_forward"),
                ],
            )
        }

        PreviewData::Media { .. } => {
            let play_label = if options.media_playing {
                "Pause"
            } else {
                "Play"
            };
            (
                "Video".to_string(),
                vec![
                    MenuItem::new(ID_MEDIA_PLAY_PAUSE, play_label)
                        .with_action("media.toggle_pause")
                        .with_shortcut_str("P"),
                    MenuItem::new(ID_MEDIA_TOGGLE_SUBTITLES, "Toggle Subtitles")
                        .with_checkbox(bool_to_toggle(options.media_subtitles_enabled))
                        .with_action("media.toggle_subtitles"),
                    MenuItem::new(ID_MEDIA_CYCLE_SUBTITLES, "Next Subtitle Track")
                        .with_action("media.cycle_subtitles")
                        .with_shortcut_str("C"),
                    MenuItem::separator(494),
                    MenuItem::new(ID_MEDIA_SEEK_BACK, "Seek Backward (-5s)")
                        .with_action("media.seek_back"),
                    MenuItem::new(ID_MEDIA_SEEK_FORWARD, "Seek Forward (+5s)")
                        .with_action("media.seek_forward"),
                ],
            )
        }

        _ => (
            "Actions".to_string(),
            vec![
                MenuItem::new(499, "No specific actions")
                    .with_action("none")
                    .enabled(false),
            ],
        ),
    }
}

#[cfg(test)]
#[path = "context_tests.rs"]
mod tests;
