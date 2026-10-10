use crate::app::KglanceApp;
use crate::app::messages::{Message, SettingsMsg};
use crate::core::config::ConfigManager;
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
            let mut config = app.load_config();
            config.ui.theme = Some(t);
            let _ = app.save_config(&config);
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

            let mut config = app.load_config();
            config.ui.font_size = s;
            let _ = app.save_config(&config);
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
            }
            let mut config = app.load_config();
            config.ui.font_family = Some(f);
            let _ = app.save_config(&config);
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
            let mut config = app.load_config();
            config.ui.font_family_mono = Some(f);
            let _ = app.save_config(&config);
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
            let mut config = app.load_config();
            config.ui.epub_font_family = Some(f);
            let _ = app.save_config(&config);
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
                let mut config = app.load_config();
                config.ui.max_text_width = None;
                let _ = app.save_config(&config);
            }
            Task::none()
        }
        SettingsMsg::DefaultWidthChanged(w) => {
            let clamped_w = w.max(app.state.window_min_size.width as u32);
            app.state.window_default_size.width = clamped_w as f32;
            let mut config = app.load_config();
            config.ui.default_width = clamped_w;
            let _ = app.save_config(&config);
            Task::none()
        }
        SettingsMsg::DefaultHeightChanged(h) => {
            let clamped_h = h.max(app.state.window_min_size.height as u32);
            app.state.window_default_size.height = clamped_h as f32;
            let mut config = app.load_config();
            config.ui.default_height = clamped_h;
            let _ = app.save_config(&config);
            Task::none()
        }
        SettingsMsg::MinWidthChanged(w) => {
            app.state.window_min_size.width = w as f32;
            let mut config = app.load_config();
            config.ui.min_width = w;
            let _ = app.save_config(&config);
            Task::none()
        }
        SettingsMsg::MinHeightChanged(h) => {
            app.state.window_min_size.height = h as f32;
            let mut config = app.load_config();
            config.ui.min_height = h;
            let _ = app.save_config(&config);
            Task::none()
        }
        SettingsMsg::UseCurrentWindowSize => {
            let w = app.state.current_window_size.width as u32;
            let h = app.state.current_window_size.height as u32;
            let min_w = app.state.window_min_size.width as u32;
            let min_h = app.state.window_min_size.height as u32;
            if w >= min_w && h >= min_h {
                app.state.window_default_size = iced::Size::new(w as f32, h as f32);
                let mut config = app.load_config();
                config.ui.default_width = w;
                config.ui.default_height = h;
                let _ = app.save_config(&config);
            }
            Task::none()
        }
        SettingsMsg::SetWindowPreset { width, height } => {
            app.state.window_default_size = iced::Size::new(width as f32, height as f32);
            let mut config = app.load_config();
            config.ui.default_width = width;
            config.ui.default_height = height;
            let _ = app.save_config(&config);
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

            let mut config = app.load_config();
            config.ui.word_wrap = enabled;
            let _ = app.save_config(&config);
            Task::none()
        }
        SettingsMsg::JsonTreeViewChanged(enabled) => {
            app.state.json_tree_view = enabled;
            app.state.json.tree_mode = enabled;
            let mut config = app.load_config();
            config.ui.json_tree_view = enabled;
            let _ = app.save_config(&config);
            Task::none()
        }
        SettingsMsg::EpubReadingModeChanged(mode) => {
            app.state.epub_reading_mode = mode;
            app.state.epub.reading_mode = mode;
            let mut config = app.load_config();
            config.ui.epub_reading_mode = mode;
            let _ = app.save_config(&config);
            Task::none()
        }
        SettingsMsg::PreferMermaidCliChanged(enabled) => {
            app.state.prefer_mermaid_cli = enabled;
            let mut config = app.load_config();
            config.ui.prefer_mermaid_cli = enabled;
            let _ = app.save_config(&config);
            Task::none()
        }
        SettingsMsg::MaxMemoryMbChanged(mb) => {
            app.state.cache_config.max_memory_mb = mb;
            app.state
                .cache
                .set_max_bytes(mb.saturating_mul(1024 * 1024));
            let mut config = app.load_config();
            config.cache.max_memory_mb = mb;
            let _ = app.save_config(&config);
            Task::none()
        }
        SettingsMsg::MaxDiskCacheMbChanged(mb) => {
            app.state.cache_config.max_disk_cache_mb = mb;
            let mut config = app.load_config();
            config.cache.max_disk_cache_mb = mb;
            let _ = app.save_config(&config);
            Task::none()
        }
        SettingsMsg::SmoothScrollChanged(enabled) => {
            app.state.scroll_config.smooth_scroll_enabled = enabled;
            app.state.apply_scroll_controllers();
            let mut config = app.load_config();
            config.scroll.smooth_scroll_enabled = enabled;
            let _ = app.save_config(&config);
            Task::none()
        }
        SettingsMsg::ScrollFrictionChanged(friction) => {
            app.state.scroll_config.friction = friction;
            app.state.apply_scroll_controllers();
            let mut config = app.load_config();
            config.scroll.friction = friction;
            let _ = app.save_config(&config);
            Task::none()
        }
        SettingsMsg::ScrollSpringStiffnessChanged(stiffness) => {
            app.state.scroll_config.spring_stiffness = stiffness;
            app.state.apply_scroll_controllers();
            let mut config = app.load_config();
            config.scroll.spring_stiffness = stiffness;
            let _ = app.save_config(&config);
            Task::none()
        }
        SettingsMsg::ResetToDefaults => {
            let default_config = crate::core::config::AppConfig::default();
            let old_size = app.state.font_size;
            app.state.font_size = default_config.ui.font_size;
            app.state.default_font_size = default_config.ui.font_size;
            app.state.font_family = default_config.ui.font_family.clone();
            app.state.font_family_mono = default_config.ui.font_family_mono.clone();
            app.state.epub_font_family = default_config.ui.epub_font_family.clone();
            app.state.epub_reading_mode = default_config.ui.epub_reading_mode;
            app.state.epub.reading_mode = default_config.ui.epub_reading_mode;
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
            Task::none()
        }
    }
}

#[cfg(test)]
#[path = "settings_tests.rs"]
mod tests;
