use crate::core::PreviewData;
use crate::core::config::AppConfig;
use crate::ui::theme::color::AppTheme;
use dbus_global_menu::{GlobalMenu, MenuHandle, MenuItem, ToggleState};
use tokio::sync::mpsc;

pub const ID_FILE_ROOT: i32 = 100;
pub const ID_FILE_OPEN: i32 = 101;
pub const ID_FILE_OPEN_DOLPHIN: i32 = 102;
pub const ID_FILE_CLOSE: i32 = 104;

pub const ID_EDIT_ROOT: i32 = 200;
pub const ID_EDIT_COPY: i32 = 201;
pub const ID_EDIT_SELECT_ALL: i32 = 202;
pub const ID_EDIT_FIND: i32 = 203;

pub const ID_VIEW_ROOT: i32 = 300;
pub const ID_VIEW_WORD_WRAP: i32 = 301;
pub const ID_VIEW_SMOOTH_SCROLL: i32 = 303;
pub const ID_VIEW_PREFERENCES: i32 = 310;
pub const ID_VIEW_THEME_SUBMENU: i32 = 320;

pub const ID_THEME_BASE: i32 = 330;
pub const THEME_ITEMS: &[(&str, &str)] = &[
    ("Dark", "dark"),
    ("Light", "light"),
    ("Catppuccin Mocha", "catppuccin-mocha"),
    ("Catppuccin Latte", "catppuccin-latte"),
    ("Tokyo Night", "tokyo-night"),
    ("Gruvbox Dark", "gruvbox-dark"),
    ("Nord", "nord"),
    ("Dracula", "dracula"),
    ("Proper Dark", "proper-dark"),
];

pub const ID_CONTEXT_ROOT: i32 = 400;

pub fn build_root_menu(config: &AppConfig, active_theme: &AppTheme) -> MenuItem {
    let mut root = MenuItem::new(0, "Root");

    // 1. File Menu
    let mut file_menu = MenuItem::new(ID_FILE_ROOT, "File");
    let open_item = MenuItem::new(ID_FILE_OPEN, "Open in Default App")
        .with_action("file.open")
        .with_shortcut_str("Return");
    let dolphin_item = MenuItem::new(ID_FILE_OPEN_DOLPHIN, "Open Containing Folder")
        .with_action("file.open_dolphin");
    let close_item = MenuItem::new(ID_FILE_CLOSE, "Close Preview")
        .with_action("file.close")
        .with_shortcut_str("Escape");

    file_menu.children = vec![open_item, dolphin_item, close_item];

    // 2. Edit Menu
    let mut edit_menu = MenuItem::new(ID_EDIT_ROOT, "Edit");
    let copy_item = MenuItem::new(ID_EDIT_COPY, "Copy Path")
        .with_action("edit.copy")
        .with_shortcut_str("Ctrl+C");
    let select_all_item = MenuItem::new(ID_EDIT_SELECT_ALL, "Select All")
        .with_action("edit.select_all")
        .with_shortcut_str("Ctrl+A");
    let find_item = MenuItem::new(ID_EDIT_FIND, "Find...")
        .with_action("edit.find")
        .with_shortcut_str("Ctrl+F");
    edit_menu.children = vec![
        copy_item,
        select_all_item,
        MenuItem::separator(290),
        find_item,
    ];

    // 3. View Menu
    let mut view_menu = MenuItem::new(ID_VIEW_ROOT, "View");
    let wrap_state = if config.ui.word_wrap {
        ToggleState::On
    } else {
        ToggleState::Off
    };
    let toggle_wrap = MenuItem::new(ID_VIEW_WORD_WRAP, "Word Wrap")
        .with_checkbox(wrap_state)
        .with_action("toggle.word_wrap")
        .with_shortcut_str("Ctrl+W");

    let scroll_state = if config.scroll.smooth_scroll_enabled {
        ToggleState::On
    } else {
        ToggleState::Off
    };
    let toggle_scroll = MenuItem::new(ID_VIEW_SMOOTH_SCROLL, "Smooth Scroll")
        .with_checkbox(scroll_state)
        .with_action("toggle.smooth_scroll");

    let preferences = MenuItem::new(ID_VIEW_PREFERENCES, "Preferences...")
        .with_action("settings.open")
        .with_shortcut_str("Ctrl+comma");

    // Themes Submenu
    let mut theme_submenu = MenuItem::new(ID_VIEW_THEME_SUBMENU, "Theme");
    let current_theme_key = active_theme.name();
    let mut theme_children = Vec::new();
    for (idx, (label, theme_key)) in THEME_ITEMS.iter().enumerate() {
        let is_active = current_theme_key == *theme_key;
        let toggle = if is_active {
            ToggleState::On
        } else {
            ToggleState::Off
        };
        let item = MenuItem::new(ID_THEME_BASE + idx as i32, *label)
            .with_radio(toggle)
            .with_action(format!("theme.set.{theme_key}"));
        theme_children.push(item);
    }
    theme_submenu.children = theme_children;

    view_menu.children = vec![
        theme_submenu,
        MenuItem::separator(390),
        toggle_wrap,
        toggle_scroll,
        MenuItem::separator(391),
        preferences,
    ];

    // 4. Dynamic Context Menu
    let mut context_menu = MenuItem::new(ID_CONTEXT_ROOT, "Actions");
    context_menu.children = vec![
        MenuItem::new(499, "No specific actions")
            .with_action("none")
            .enabled(false),
    ];

    root.children = vec![file_menu, edit_menu, view_menu, context_menu];
    root
}

pub fn init_global_menu(
    action_tx: mpsc::UnboundedSender<String>,
    config: &AppConfig,
    active_theme: &AppTheme,
) -> Result<(GlobalMenu, MenuHandle), Box<dyn std::error::Error + Send + Sync>> {
    let root = build_root_menu(config, active_theme);
    let service = GlobalMenu::builder("org.mintori.Kglance.AppMenu")
        .object_path("/MenuBar")
        .menu(root)
        .action_sender(action_tx)
        .start()?;

    let handle = service.handle();
    Ok((service, handle))
}

pub fn sync_menu_states(handle: &MenuHandle, config: &AppConfig, active_theme: &AppTheme) {
    let wrap_state = if config.ui.word_wrap {
        ToggleState::On
    } else {
        ToggleState::Off
    };
    handle.set_toggle_state(ID_VIEW_WORD_WRAP, wrap_state);

    let scroll_state = if config.scroll.smooth_scroll_enabled {
        ToggleState::On
    } else {
        ToggleState::Off
    };
    handle.set_toggle_state(ID_VIEW_SMOOTH_SCROLL, scroll_state);

    let current_theme_key = active_theme.name();
    for (idx, (_, theme_key)) in THEME_ITEMS.iter().enumerate() {
        let is_active = current_theme_key == *theme_key;
        let toggle = if is_active {
            ToggleState::On
        } else {
            ToggleState::Off
        };
        handle.set_toggle_state(ID_THEME_BASE + idx as i32, toggle);
    }
}

pub fn sync_copy_menu_label(handle: &MenuHandle, has_selection: bool) {
    let label = if has_selection {
        "Copy Selection"
    } else {
        "Copy Path"
    };
    handle.set_label(ID_EDIT_COPY, label);
}

pub fn update_context_menu(handle: &MenuHandle, content: Option<&PreviewData>) {
    let mut actions = Vec::new();
    let mut label = "Actions";

    if let Some(content) = content {
        match content {
            PreviewData::Image { .. } => {
                label = "Image";
                actions.push(
                    MenuItem::new(401, "Rotate Clockwise")
                        .with_action("image.rotate_cw")
                        .with_shortcut_str("R"),
                );
                actions.push(
                    MenuItem::new(402, "Rotate Counter-Clockwise")
                        .with_action("image.rotate_ccw")
                        .with_shortcut_str("Shift+R"),
                );
                actions.push(
                    MenuItem::new(403, "Fit to Window")
                        .with_action("image.fit")
                        .with_shortcut_str("F"),
                );
                actions.push(
                    MenuItem::new(404, "Original Size (100%)")
                        .with_action("image.actual_size")
                        .with_shortcut_str("1"),
                );
            }
            PreviewData::Pdf { .. } => {
                label = "PDF";
                actions.push(
                    MenuItem::new(411, "Previous Page")
                        .with_action("pdf.prev_page")
                        .with_shortcut_str("Page_Up"),
                );
                actions.push(
                    MenuItem::new(412, "Next Page")
                        .with_action("pdf.next_page")
                        .with_shortcut_str("Page_Down"),
                );
                actions.push(
                    MenuItem::new(413, "Toggle Table of Contents")
                        .with_action("pdf.toggle_toc")
                        .with_shortcut_str("T"),
                );
            }
            PreviewData::Epub { .. } => {
                label = "EPUB";
                actions.push(
                    MenuItem::new(421, "Previous Chapter")
                        .with_action("epub.prev_chapter")
                        .with_shortcut_str("Page_Up"),
                );
                actions.push(
                    MenuItem::new(422, "Next Chapter")
                        .with_action("epub.next_chapter")
                        .with_shortcut_str("Page_Down"),
                );
                actions.push(
                    MenuItem::new(423, "Toggle Table of Contents")
                        .with_action("epub.toggle_toc")
                        .with_shortcut_str("T"),
                );
            }
            PreviewData::Audio { .. } => {
                label = "Audio";
                actions.push(
                    MenuItem::new(431, "Play / Pause")
                        .with_action("media.toggle_pause")
                        .with_shortcut_str("P"),
                );
                actions.push(
                    MenuItem::new(432, "Mute / Unmute")
                        .with_action("media.toggle_mute")
                        .with_shortcut_str("M"),
                );
            }
            PreviewData::Media { .. } => {
                label = "Video";
                actions.push(
                    MenuItem::new(431, "Play / Pause")
                        .with_action("media.toggle_pause")
                        .with_shortcut_str("P"),
                );
                actions.push(
                    MenuItem::new(433, "Cycle Subtitles")
                        .with_action("media.cycle_subtitles")
                        .with_shortcut_str("C"),
                );
            }
            PreviewData::Text { .. } | PreviewData::Markdown { .. } => {
                label = "Document";
                actions.push(
                    MenuItem::new(405, "Toggle Outline / Table of Contents")
                        .with_action("text.toggle_outline")
                        .with_shortcut_str("T"),
                );
            }
            PreviewData::Json { .. } => {
                label = "JSON";
                actions.push(
                    MenuItem::new(406, "Toggle Tree / Raw View")
                        .with_action("json.toggle_mode")
                        .with_shortcut_str("Ctrl+M"),
                );
            }
            _ => {
                label = "Actions";
                actions.push(
                    MenuItem::new(499, "No specific actions")
                        .with_action("none")
                        .enabled(false),
                );
            }
        }
    }

    handle.update_item(ID_CONTEXT_ROOT, move |item| {
        item.label = label.to_string();
        item.children = actions;
    });
}
