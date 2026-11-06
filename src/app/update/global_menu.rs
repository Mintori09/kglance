use crate::app::KglanceApp;
use crate::app::messages::{GlobalMenuMsg, Message, NavigationMsg, SettingsMsg};
use crate::core::config::ConfigManager;
use iced::Task;

fn self_is_audio(app: &KglanceApp) -> bool {
    matches!(
        app.current_content,
        Some(crate::core::PreviewData::Audio { .. })
    )
}

pub fn handle_global_menu_msg(app: &mut KglanceApp, msg: GlobalMenuMsg) -> Task<Message> {
    match msg {
        GlobalMenuMsg::ActionTriggered(action) => handle_action_triggered(app, &action),
    }
}

fn handle_action_triggered(app: &mut KglanceApp, action: &str) -> Task<Message> {
    crate::log_info!("Global Menu action triggered: {action}");

    match action {
        // File actions
        "file.open" => app.handle_open_clicked(),
        "file.open_dolphin" => {
            let path = app.state.file_name.clone();
            if !path.is_empty() {
                let _ = std::process::Command::new("dolphin")
                    .arg("--select")
                    .arg(&path)
                    .spawn();
            }
            Task::none()
        }
        "file.close" => app.handle_close(),

        // Edit actions
        "edit.copy" | "edit.copy_selection" => app
            .handle_ctrl_copy()
            .unwrap_or_else(|| app.handle_copy_path()),
        "edit.select_all" => app.handle_ctrl_a().unwrap_or_else(Task::none),
        "edit.find" => app.handle_ctrl_f().unwrap_or_else(Task::none),

        // View actions
        "settings.open" => app.update(NavigationMsg::ToggleSettingsClicked.into()),

        "toggle.word_wrap" => app.update(SettingsMsg::WordWrapChanged(!app.state.word_wrap).into()),

        "toggle.smooth_scroll" => app.update(
            SettingsMsg::SmoothScrollChanged(!app.state.scroll_config.smooth_scroll_enabled).into(),
        ),

        // Dynamic Context Actions
        "image.rotate_cw" => app.update(crate::app::messages::ImageMsg::RotateRight.into()),
        "image.rotate_ccw" => app.update(crate::app::messages::ImageMsg::RotateLeft.into()),
        "image.fit" => app.update(crate::app::messages::ImageMsg::FitToWindow.into()),
        "image.actual_size" => app.update(crate::app::messages::ImageMsg::DoubleClick.into()),

        "pdf.prev_page" => app.scroll_page(-0.85),
        "pdf.next_page" => app.scroll_page(0.85),
        "pdf.toggle_toc" => app.update(crate::app::messages::PdfMsg::SidebarToggled.into()),

        "epub.prev_chapter" => app.scroll_page(-0.85),
        "epub.next_chapter" => app.scroll_page(0.85),
        "epub.toggle_toc" => app.update(crate::app::messages::EpubMsg::SidebarToggled.into()),

        "text.toggle_outline" => app.update(crate::app::messages::TextMsg::ToggleOutline.into()),
        "json.toggle_mode" => app.update(crate::app::messages::JsonMsg::ToggleMode.into()),

        "media.toggle_pause" => {
            if self_is_audio(app) {
                app.update(crate::app::messages::AudioMsg::PlayPauseClicked.into())
            } else {
                app.update(crate::app::messages::MediaMsg::PlayPauseClicked.into())
            }
        }
        "media.toggle_mute" => {
            if self_is_audio(app) {
                crate::features::audio::update::handle_toggle_mute(app)
            } else {
                Task::none()
            }
        }
        "media.cycle_subtitles" => {
            app.update(crate::app::messages::MediaMsg::CycleSubtitleTrack.into())
        }

        action if action.starts_with("theme.set.") => {
            let theme_name = &action["theme.set.".len()..];
            let theme_setting = theme_name.to_string();
            let app_theme = ConfigManager::resolve_theme(&theme_setting);

            app.state.theme_setting = theme_setting.clone();
            app.state.app_theme = app_theme;

            app.state.for_each_text_state_mut(|st| {
                let scroll = st.scroll_y;
                st.syntax_cache.clear();
                st.cached_tokens.clear();
                crate::features::text::update_tokens_for_viewport(st, scroll, app_theme);
            });

            let mut cfg = app.load_config();
            cfg.ui.theme = Some(theme_setting);
            let _ = app.save_config(&cfg);

            if let Some(ref handle) = app.menu_handle {
                crate::dbus::global_menu::sync_menu_states(handle, &cfg, &app_theme);
            }
            Task::none()
        }

        _ => {
            crate::log_debug!("Unhandled global menu action: {action}");
            Task::none()
        }
    }
}

#[cfg(test)]
#[path = "global_menu_tests.rs"]
mod tests;
