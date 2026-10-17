use crate::app::KglanceApp;
use crate::app::messages::{CustomActionEditorMsg, Message, SettingsMsg};
use crate::core::config::ConfigManager;
use crate::core::selection_action::{CustomAction, CustomActionEditorState};
use iced::Task;

pub fn handle_settings_message(app: &mut KglanceApp, msg: SettingsMsg) -> Task<Message> {
    match msg {
        SettingsMsg::TabChanged(tab) => {
            app.state.settings_tab = tab;
            Task::none()
        }
        SettingsMsg::ThemeChanged(t) => {
            app.state.theme_setting = t.clone();
            app.state.app_theme = ConfigManager::resolve_theme(&t);
            let theme = app.state.app_theme;
            let text_scroll = app.state.text.scroll_y;
            app.state.text.syntax_cache.clear();
            app.state.text.cached_tokens.clear();
            crate::features::text::update_tokens_for_viewport(
                &mut app.state.text,
                text_scroll,
                theme,
            );
            let json_scroll = app.state.json.raw_text.scroll_y;
            app.state.json.raw_text.syntax_cache.clear();
            app.state.json.raw_text.cached_tokens.clear();
            crate::features::text::update_tokens_for_viewport(
                &mut app.state.json.raw_text,
                json_scroll,
                theme,
            );
            let typst_scroll = app.state.typst.source_text.scroll_y;
            app.state.typst.source_text.syntax_cache.clear();
            app.state.typst.source_text.cached_tokens.clear();
            crate::features::text::update_tokens_for_viewport(
                &mut app.state.typst.source_text,
                typst_scroll,
                theme,
            );
            let _ = app.save_current_config();
            Task::none()
        }
        SettingsMsg::FontSizeChanged(s) => {
            let old_size = app.state.font_size;
            app.state.font_size = s;
            let win_w = app.state.current_window_size.width;
            let theme = app.state.app_theme;
            let word_wrap = app.state.word_wrap;

            crate::features::text::rescale_text_geometry(
                &mut app.state.text,
                old_size,
                s,
                word_wrap,
                win_w,
                theme,
            );
            crate::features::text::rescale_text_geometry(
                &mut app.state.json.raw_text,
                old_size,
                s,
                word_wrap,
                win_w,
                theme,
            );
            crate::features::text::rescale_text_geometry(
                &mut app.state.typst.source_text,
                old_size,
                s,
                word_wrap,
                win_w,
                theme,
            );
            if let Some(crate::core::PreviewData::Markdown { blocks, .. }) = &app.current_content {
                let content_width = app.state.markdown_content_width();
                crate::features::markdown::rescale_and_update_markdown_layout(
                    &mut app.state.markdown,
                    blocks,
                    old_size,
                    s,
                    content_width,
                );
            } else if matches!(
                app.current_content,
                Some(crate::core::PreviewData::Epub { .. })
            ) {
                let content_width = app.state.epub_content_width();
                let blocks = if app.state.epub.reading_mode
                    == crate::core::config::EpubReadingMode::Continuous
                {
                    app.state.epub.continuous_blocks.as_slice()
                } else {
                    let active_chapter = app.state.epub.active_chapter;
                    app.state
                        .epub
                        .chapters
                        .get(active_chapter)
                        .map_or([].as_slice(), |c| c.blocks.as_slice())
                };
                if !blocks.is_empty() {
                    crate::features::markdown::rescale_and_update_markdown_layout(
                        &mut app.state.epub.markdown_state,
                        blocks,
                        old_size,
                        s,
                        content_width,
                    );
                }
            } else if matches!(
                app.current_content,
                Some(crate::core::PreviewData::Spreadsheet { .. })
            ) {
                crate::features::sheet::update::rescale_spreadsheet_geometry(
                    &mut app.state.spreadsheet,
                    old_size,
                    s,
                );
            }

            let _ = app.save_current_config();
            Task::none()
        }
        SettingsMsg::FontFamilySelected(f) => {
            app.state.font_family = Some(f.clone());
            if let Some(crate::core::PreviewData::Markdown { blocks, .. }) = &app.current_content {
                let content_width = app.state.markdown_content_width();
                let s = app.state.font_size;
                crate::features::markdown::rescale_and_update_markdown_layout(
                    &mut app.state.markdown,
                    blocks,
                    s,
                    s,
                    content_width,
                );
            } else if app.state.epub_font_family.is_none()
                && matches!(
                    app.current_content,
                    Some(crate::core::PreviewData::Epub { .. })
                )
            {
                let content_width = app.state.epub_content_width();
                let blocks = if app.state.epub.reading_mode
                    == crate::core::config::EpubReadingMode::Continuous
                {
                    app.state.epub.continuous_blocks.as_slice()
                } else {
                    let active_chapter = app.state.epub.active_chapter;
                    app.state
                        .epub
                        .chapters
                        .get(active_chapter)
                        .map_or([].as_slice(), |c| c.blocks.as_slice())
                };
                if !blocks.is_empty() {
                    let s = app.state.font_size;
                    crate::features::markdown::rescale_and_update_markdown_layout(
                        &mut app.state.epub.markdown_state,
                        blocks,
                        s,
                        s,
                        content_width,
                    );
                }
            }
            let _ = app.save_current_config();
            Task::none()
        }
        SettingsMsg::FontFamilyMonoSelected(f) => {
            app.state.font_family_mono = Some(f.clone());
            let old_size = app.state.font_size;
            let s = app.state.font_size;
            let win_w = app.state.current_window_size.width;
            let theme = app.state.app_theme;
            let word_wrap = app.state.word_wrap;

            crate::features::text::rescale_text_geometry(
                &mut app.state.text,
                old_size,
                s,
                word_wrap,
                win_w,
                theme,
            );
            crate::features::text::rescale_text_geometry(
                &mut app.state.json.raw_text,
                old_size,
                s,
                word_wrap,
                win_w,
                theme,
            );
            crate::features::text::rescale_text_geometry(
                &mut app.state.typst.source_text,
                old_size,
                s,
                word_wrap,
                win_w,
                theme,
            );
            let _ = app.save_current_config();
            Task::none()
        }
        SettingsMsg::EpubFontFamilySelected(f) => {
            app.state.epub_font_family = Some(f.clone());
            if matches!(
                app.current_content,
                Some(crate::core::PreviewData::Epub { .. })
            ) {
                let content_width = app.state.epub_content_width();
                let blocks = if app.state.epub.reading_mode
                    == crate::core::config::EpubReadingMode::Continuous
                {
                    app.state.epub.continuous_blocks.as_slice()
                } else {
                    let active_chapter = app.state.epub.active_chapter;
                    app.state
                        .epub
                        .chapters
                        .get(active_chapter)
                        .map_or([].as_slice(), |c| c.blocks.as_slice())
                };
                if !blocks.is_empty() {
                    let s = app.state.font_size;
                    crate::features::markdown::rescale_and_update_markdown_layout(
                        &mut app.state.epub.markdown_state,
                        blocks,
                        s,
                        s,
                        content_width,
                    );
                }
            }
            let _ = app.save_current_config();
            Task::none()
        }
        SettingsMsg::MaxTextWidthChanged(w) => {
            if let Some(val) = w {
                if let Some(task) =
                    crate::features::markdown::update::rescale_markdown_page_width(app, val)
                {
                    return task;
                }
            } else {
                app.state.max_text_width = None;
                let _ = app.save_current_config();
            }
            Task::none()
        }
        SettingsMsg::DefaultWidthChanged(w) => {
            let clamped_w = w.max(app.state.window_min_size.width as u32);
            app.state.window_default_size.width = clamped_w as f32;
            app.save_current_config_logged("default width changed");
            Task::none()
        }
        SettingsMsg::DefaultHeightChanged(h) => {
            let clamped_h = h.max(app.state.window_min_size.height as u32);
            app.state.window_default_size.height = clamped_h as f32;
            app.save_current_config_logged("default height changed");
            Task::none()
        }
        SettingsMsg::MinWidthChanged(w) => {
            let min_w = w as f32;
            app.state.window_min_size.width = min_w;
            if app.state.window_default_size.width < min_w {
                app.state.window_default_size.width = min_w;
            }
            app.save_current_config_logged("min width changed");
            Task::none()
        }
        SettingsMsg::MinHeightChanged(h) => {
            let min_h = h as f32;
            app.state.window_min_size.height = min_h;
            if app.state.window_default_size.height < min_h {
                app.state.window_default_size.height = min_h;
            }
            app.save_current_config_logged("min height changed");
            Task::none()
        }
        SettingsMsg::UseCurrentWindowSize => {
            let w = app.state.current_window_size.width as u32;
            let h = app.state.current_window_size.height as u32;
            let min_w = app.state.window_min_size.width as u32;
            let min_h = app.state.window_min_size.height as u32;
            if w >= min_w && h >= min_h {
                app.state.window_default_size = iced::Size::new(w as f32, h as f32);
                app.save_current_config_logged("use current window size");
            }
            Task::none()
        }
        SettingsMsg::SetWindowPreset { width, height } => {
            app.state.window_default_size = iced::Size::new(
                (width as f32).max(app.state.window_min_size.width),
                (height as f32).max(app.state.window_min_size.height),
            );
            app.save_current_config_logged("set window preset");
            Task::none()
        }
        SettingsMsg::WordWrapChanged(enabled) => {
            app.state.word_wrap = enabled;
            app.state.text.wrap = enabled;
            let wrap_mode = if enabled {
                crate::features::text::WrapMode::Word
            } else {
                crate::features::text::WrapMode::None
            };
            let win_w = if app.state.current_window_size.width > 0.0 {
                app.state.current_window_size.width
            } else if app.state.window_width > 0.0 {
                app.state.window_width
            } else {
                1024.0
            };
            let sidebar_w = if app.state.text.outline_visible && app.state.text.sidebar_width > 0.0
            {
                app.state.text.sidebar_width
            } else {
                0.0
            };
            let available_w = (win_w - sidebar_w).max(200.0);

            app.state.text.display_map.update_geometry(
                &app.state.text.document,
                available_w,
                app.state.font_size,
                wrap_mode,
            );
            let theme = app.state.app_theme;
            let text_scroll_y = app.state.text.scroll_y;
            app.state.text.total_content_height = app.state.text.display_map.total_content_height();
            crate::features::text::update_tokens_for_viewport(
                &mut app.state.text,
                text_scroll_y,
                theme,
            );

            app.state.json.raw_text.wrap = enabled;
            app.state.json.raw_text.display_map.update_geometry(
                &app.state.json.raw_text.document,
                win_w,
                app.state.font_size,
                wrap_mode,
            );
            app.state.json.raw_text.total_content_height =
                app.state.json.raw_text.display_map.total_content_height();
            let json_scroll_y = app.state.json.raw_text.scroll_y;
            crate::features::text::update_tokens_for_viewport(
                &mut app.state.json.raw_text,
                json_scroll_y,
                theme,
            );

            app.state.typst.source_text.wrap = enabled;
            app.state.typst.source_text.display_map.update_geometry(
                &app.state.typst.source_text.document,
                win_w,
                app.state.font_size,
                wrap_mode,
            );
            app.state.typst.source_text.total_content_height = app
                .state
                .typst
                .source_text
                .display_map
                .total_content_height();
            let typst_scroll_y = app.state.typst.source_text.scroll_y;
            crate::features::text::update_tokens_for_viewport(
                &mut app.state.typst.source_text,
                typst_scroll_y,
                theme,
            );

            app.save_current_config_logged("word wrap changed");
            Task::none()
        }
        SettingsMsg::JsonTreeViewChanged(enabled) => {
            app.state.json_tree_view = enabled;
            app.state.json.tree_mode = enabled;
            app.save_current_config_logged("json tree view changed");
            Task::none()
        }
        SettingsMsg::RememberWindowSizeChanged(enabled) => {
            app.state.remember_window_size = enabled;
            if enabled
                && app.state.current_window_size.width > 0.0
                && app.state.current_window_size.height > 0.0
            {
                app.state.window_default_size = iced::Size::new(
                    app.state
                        .current_window_size
                        .width
                        .max(app.state.window_min_size.width),
                    app.state
                        .current_window_size
                        .height
                        .max(app.state.window_min_size.height),
                );
            }
            app.save_current_config_logged("remember window size changed");
            Task::none()
        }
        SettingsMsg::EpubReadingModeChanged(mode) => {
            let task = crate::features::epub::update::handle_reading_mode_changed(app, mode);
            app.state.epub_reading_mode = mode;
            if !app.state.file_name.is_empty()
                && let Some(pos) = app.state.read_positions.get(&app.state.file_name)
            {
                app.state.read_positions.insert(
                    app.state.file_name.clone(),
                    crate::core::ReadPosition {
                        reading_mode: Some(mode),
                        ..pos
                    },
                );
                app.state.read_positions_dirty = true;
            }
            app.save_current_config_logged("epub reading mode changed");
            task
        }
        SettingsMsg::RubyDisplayModeChanged(mode) => {
            app.state.ruby_display_mode = mode;
            app.save_current_config_logged("ruby display mode changed");
            Task::none()
        }
        SettingsMsg::PreferMermaidCliChanged(enabled) => {
            app.state.prefer_mermaid_cli = enabled;
            app.save_current_config_logged("prefer mermaid cli changed");
            Task::none()
        }
        SettingsMsg::MaxMemoryMbChanged(mb) => {
            app.state.cache_config.max_memory_mb = mb;
            app.state
                .cache
                .set_max_bytes(mb.saturating_mul(1024 * 1024));
            app.save_current_config_logged("max memory mb changed");
            Task::none()
        }
        SettingsMsg::MaxDiskCacheMbChanged(mb) => {
            app.state.cache_config.max_disk_cache_mb = mb;
            app.save_current_config_logged("max disk cache mb changed");
            Task::none()
        }
        SettingsMsg::SmoothScrollChanged(enabled) => {
            app.state.scroll_config.smooth_scroll_enabled = enabled;
            app.state.apply_scroll_controllers();
            app.save_current_config_logged("smooth scroll changed");
            Task::none()
        }
        SettingsMsg::ScrollFrictionChanged(friction) => {
            app.state.scroll_config.friction = friction;
            app.state.apply_scroll_controllers();
            app.save_current_config_logged("scroll friction changed");
            Task::none()
        }
        SettingsMsg::ScrollSpringStiffnessChanged(stiffness) => {
            app.state.scroll_config.spring_stiffness = stiffness;
            app.state.apply_scroll_controllers();
            app.save_current_config_logged("scroll spring stiffness changed");
            Task::none()
        }
        SettingsMsg::SelectionMenuToggled(enabled) => {
            app.state.selection_menu_config.enabled = enabled;
            app.save_current_config_logged("selection menu toggled");
            Task::none()
        }
        SettingsMsg::CustomAction(action_msg) => {
            match action_msg {
                CustomActionEditorMsg::OpenAdd => {
                    app.state.action_editor = Some(CustomActionEditorState {
                        editing_index: None,
                        name: String::new(),
                        command: String::new(),
                        popup: true,
                        toast: false,
                        stream: false,
                    });
                }
                CustomActionEditorMsg::OpenEdit(idx) => {
                    if let Some(action) = app.state.selection_menu_config.custom_actions.get(idx) {
                        app.state.action_editor = Some(CustomActionEditorState {
                            editing_index: Some(idx),
                            name: action.name.clone(),
                            command: action.command.clone(),
                            popup: action.popup,
                            toast: action.toast,
                            stream: action.stream,
                        });
                    }
                }
                CustomActionEditorMsg::Cancel => {
                    app.state.action_editor = None;
                }
                CustomActionEditorMsg::NameChanged(name) => {
                    if let Some(editor) = &mut app.state.action_editor {
                        editor.name = name;
                    }
                }
                CustomActionEditorMsg::CommandChanged(command) => {
                    if let Some(editor) = &mut app.state.action_editor {
                        editor.command = command;
                    }
                }
                CustomActionEditorMsg::PopupToggled(popup) => {
                    if let Some(editor) = &mut app.state.action_editor {
                        editor.popup = popup;
                    }
                }
                CustomActionEditorMsg::ToastToggled(toast) => {
                    if let Some(editor) = &mut app.state.action_editor {
                        editor.toast = toast;
                    }
                }
                CustomActionEditorMsg::StreamToggled(stream) => {
                    if let Some(editor) = &mut app.state.action_editor {
                        editor.stream = stream;
                    }
                }
                CustomActionEditorMsg::Save => {
                    if let Some(editor) = app.state.action_editor.take() {
                        let name = editor.name.trim().to_string();
                        let command = editor.command.trim().to_string();
                        if !name.is_empty() && !command.is_empty() {
                            let action = CustomAction {
                                name,
                                command,
                                enabled: true,
                                popup: editor.popup,
                                toast: editor.toast,
                                stream: editor.stream,
                            };
                            if let Some(idx) = editor.editing_index {
                                if idx < app.state.selection_menu_config.custom_actions.len() {
                                    app.state.selection_menu_config.custom_actions[idx] = action;
                                }
                            } else {
                                app.state.selection_menu_config.custom_actions.push(action);
                            }
                            app.state.action_editor = None;
                            let _ = app.save_current_config();
                        }
                    }
                }
                CustomActionEditorMsg::Delete(idx) => {
                    if idx < app.state.selection_menu_config.custom_actions.len() {
                        app.state.selection_menu_config.custom_actions.remove(idx);
                        if app
                            .state
                            .action_editor
                            .as_ref()
                            .is_some_and(|editor| editor.editing_index == Some(idx))
                        {
                            app.state.action_editor = None;
                        }
                        let _ = app.save_current_config();
                    }
                }
                CustomActionEditorMsg::ToggleEnabled { index, enabled } => {
                    if let Some(action) = app
                        .state
                        .selection_menu_config
                        .custom_actions
                        .get_mut(index)
                    {
                        action.enabled = enabled;
                        let _ = app.save_current_config();
                    }
                }
            }
            Task::none()
        }
        SettingsMsg::TrashEnabledChanged(enabled) => {
            app.state.trash_config.enabled = enabled;
            let _ = app.save_current_config();
            Task::none()
        }
        SettingsMsg::TrashConfirmOnDeleteChanged(confirm) => {
            app.state.trash_config.confirm_on_delete = confirm;
            let _ = app.save_current_config();
            Task::none()
        }
        SettingsMsg::TrashUndoTimeoutChanged(timeout) => {
            app.state.trash_config.undo_timeout_seconds = timeout;
            let _ = app.save_current_config();
            Task::none()
        }
        SettingsMsg::KeybindingTrashFileChanged(shortcut) => {
            app.state.keybinding_config.trash_file = shortcut;
            let _ = app.save_current_config();
            Task::none()
        }
        SettingsMsg::KeybindingUndoTrashChanged(shortcut) => {
            app.state.keybinding_config.undo_trash = shortcut;
            let _ = app.save_current_config();
            Task::none()
        }
        SettingsMsg::ResetToDefaults => {
            let default_config = crate::core::config::AppConfig::default();
            app.state.selection_menu_config = default_config.selection_menu.clone();
            app.state.trash_config = default_config.trash.clone();
            app.state.keybinding_config = default_config.keybindings.clone();
            let old_size = app.state.font_size;
            app.state.font_size = default_config.ui.font_size;
            app.state.default_font_size = default_config.ui.font_size;
            app.state.font_family = default_config.ui.font_family.clone();
            app.state.font_family_mono = default_config.ui.font_family_mono.clone();
            app.state.epub_font_family = default_config.ui.epub_font_family.clone();
            let epub_task = if matches!(
                app.current_content,
                Some(crate::core::PreviewData::Epub { .. })
            ) {
                crate::features::epub::update::handle_reading_mode_changed(
                    app,
                    default_config.ui.epub_reading_mode,
                )
            } else {
                app.state.epub_reading_mode = default_config.ui.epub_reading_mode;
                app.state.epub.reading_mode = default_config.ui.epub_reading_mode;
                if !app.state.file_name.is_empty()
                    && let Some(pos) = app.state.read_positions.get(&app.state.file_name)
                {
                    app.state.read_positions.insert(
                        app.state.file_name.clone(),
                        crate::core::ReadPosition {
                            reading_mode: Some(default_config.ui.epub_reading_mode),
                            ..pos
                        },
                    );
                    app.state.read_positions_dirty = true;
                }
                Task::none()
            };
            app.state.ruby_display_mode = default_config.ui.ruby_display_mode;
            app.state.max_text_width = default_config.ui.max_text_width;
            app.state.window_default_size = iced::Size::new(
                default_config.ui.default_width as f32,
                default_config.ui.default_height as f32,
            );
            app.state.window_min_size = iced::Size::new(
                default_config.ui.min_width as f32,
                default_config.ui.min_height as f32,
            );
            app.state.prefer_mermaid_cli = default_config.ui.prefer_mermaid_cli;
            app.state.word_wrap = default_config.ui.word_wrap;
            app.state.text.wrap = default_config.ui.word_wrap;
            app.state.json_tree_view = default_config.ui.json_tree_view;
            app.state.json.tree_mode = default_config.ui.json_tree_view;
            app.state.remember_window_size = default_config.ui.remember_window_size;
            app.state.cache_config = default_config.cache.clone();
            app.state.apply_scroll_config(&default_config.scroll);
            app.state.cache.set_max_bytes(
                default_config
                    .cache
                    .max_memory_mb
                    .saturating_mul(1024 * 1024),
            );

            let theme_str = default_config
                .ui
                .theme
                .clone()
                .unwrap_or_else(|| "dark".into());
            app.state.theme_setting = theme_str.clone();
            app.state.app_theme = ConfigManager::resolve_theme(&theme_str);

            let win_w = if app.state.current_window_size.width > 0.0 {
                app.state.current_window_size.width
            } else if app.state.window_width > 0.0 {
                app.state.window_width
            } else {
                1024.0
            };
            let theme = app.state.app_theme;
            let word_wrap = app.state.word_wrap;
            crate::features::text::rescale_text_geometry(
                &mut app.state.text,
                old_size,
                default_config.ui.font_size,
                word_wrap,
                win_w,
                theme,
            );
            crate::features::text::rescale_text_geometry(
                &mut app.state.json.raw_text,
                old_size,
                default_config.ui.font_size,
                word_wrap,
                win_w,
                theme,
            );
            crate::features::text::rescale_text_geometry(
                &mut app.state.typst.source_text,
                old_size,
                default_config.ui.font_size,
                word_wrap,
                win_w,
                theme,
            );
            crate::features::sheet::update::rescale_spreadsheet_geometry(
                &mut app.state.spreadsheet,
                old_size,
                default_config.ui.font_size,
            );

            let _ = app.save_config(&default_config);
            epub_task
        }
    }
}

#[cfg(test)]
#[path = "settings_tests.rs"]
mod tests;
