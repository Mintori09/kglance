use crate::app::KglanceApp;
use crate::app::messages::Message;
use iced::Task;

pub mod epub;
pub mod file;
pub mod grid;
pub mod image;
pub mod json;
pub mod markdown;
pub mod misc;
pub mod navigation;
pub mod pdf;
pub mod spreadsheet;

pub fn update(app: &mut KglanceApp, message: Message) -> Task<Message> {
    match message {
        Message::Action(msg) => match msg {
            crate::app::messages::ActionMsg::OpenClicked => app.handle_open_clicked(),
            crate::app::messages::ActionMsg::CopyPathClicked => app.handle_copy_path(),
            crate::app::messages::ActionMsg::BackClicked => Task::none(),
            crate::app::messages::ActionMsg::CloseRequested => app.handle_close(),
            crate::app::messages::ActionMsg::CopyContentClicked => Task::none(),
            crate::app::messages::ActionMsg::CopyCode(code) => misc::handle_copy_code(app, code),
        },
        Message::System(msg) => match msg {
            crate::app::messages::SystemMsg::ThemeToggled => misc::handle_theme_toggled(app),
            crate::app::messages::SystemMsg::DaemonOpenWindow { path } => {
                app.handle_daemon_open_window(path)
            }
            crate::app::messages::SystemMsg::DaemonOpenWithPlaylist {
                path,
                content,
                playlist,
            } => file::handle_daemon_open_with_playlist(app, path, content, playlist),
            crate::app::messages::SystemMsg::FileLoaded { path, content } => {
                file::handle_file_loaded_msg(app, path, content)
            }
            crate::app::messages::SystemMsg::WindowEvent(id, event) => {
                app.handle_window_event(id, event)
            }
            crate::app::messages::SystemMsg::KeyPressed(key, modifiers) => {
                app.handle_key_pressed(key, modifiers)
            }
            crate::app::messages::SystemMsg::ToastDismissed(id) => {
                misc::handle_toast_dismissed(app, id)
            }
            crate::app::messages::SystemMsg::OpenLink(url) => misc::handle_open_link(url),
            crate::app::messages::SystemMsg::FilePreviewError(path) => {
                file::handle_file_preview_error(app, path)
            }
            crate::app::messages::SystemMsg::FileChanged(path) => {
                file::handle_file_changed(app, path)
            }
        },
        Message::Navigation(msg) => match msg {
            crate::app::messages::NavigationMsg::PrevFileClicked => {
                navigation::handle_prev_file(app)
            }
            crate::app::messages::NavigationMsg::NextFileClicked => {
                navigation::handle_next_file(app)
            }
            crate::app::messages::NavigationMsg::HistoryBack => Task::none(),
            crate::app::messages::NavigationMsg::HistoryForward => Task::none(),
            crate::app::messages::NavigationMsg::SiblingFilesLoaded(files) => {
                navigation::handle_sibling_files_loaded(app, files)
            }
            crate::app::messages::NavigationMsg::ToggleViewMode => {
                grid::handle_toggle_view_mode(app)
            }
            crate::app::messages::NavigationMsg::FileClickedInGrid(idx) => {
                navigation::handle_file_clicked_in_grid(app, idx)
            }
            crate::app::messages::NavigationMsg::GridThumbnailLoaded { index, handle } => {
                grid::handle_grid_thumbnail_loaded(app, index, handle)
            }
            crate::app::messages::NavigationMsg::PreloadCompleted { path, content } => {
                file::handle_preload_completed(app, path, content)
            }
            crate::app::messages::NavigationMsg::ToggleSettingsClicked => {
                navigation::handle_toggle_settings(app)
            }
            crate::app::messages::NavigationMsg::FileClicked(idx) => {
                navigation::handle_file_clicked(app, idx)
            }
        },
        Message::Image(msg) => match msg {
            crate::app::messages::ImageMsg::Zoom(delta) => image::handle_zoom(app, delta),
            crate::app::messages::ImageMsg::PanDelta(dx, dy) => image::handle_pan(app, dx, dy),
            crate::app::messages::ImageMsg::DoubleClick => image::handle_double_click(app),
        },
        Message::Text(msg) => match msg {
            crate::app::messages::TextMsg::Edit(action) => misc::handle_text_edit(app, action),
            crate::app::messages::TextMsg::SearchQueryChanged(_) => Task::none(),
            crate::app::messages::TextMsg::SearchNext => Task::none(),
            crate::app::messages::TextMsg::SearchPrev => Task::none(),
            crate::app::messages::TextMsg::SearchClosed => Task::none(),
            crate::app::messages::TextMsg::WrapToggled => Task::none(),
            crate::app::messages::TextMsg::Scrolled(y) => misc::handle_text_scrolled(app, y),
        },
        Message::Media(msg) => match msg {
            crate::app::messages::MediaMsg::PlayPauseClicked => app.handle_play_pause(),
            crate::app::messages::MediaMsg::SeekClicked(pct) => app.handle_seek(pct),
            crate::app::messages::MediaMsg::SeekRelativeClicked(secs) => {
                app.handle_seek_relative(secs)
            }
            crate::app::messages::MediaMsg::VideoEventReceived(event) => {
                app.handle_video_event(event)
            }
            crate::app::messages::MediaMsg::MouseEnter => app.handle_media_mouse_enter(),
            crate::app::messages::MediaMsg::MouseLeave => app.handle_media_mouse_leave(),
            crate::app::messages::MediaMsg::VideoThumbnailLoaded { data } => {
                image::handle_video_thumbnail_loaded(app, data)
            }
        },
        Message::Pdf(msg) => match msg {
            crate::app::messages::PdfMsg::Scrolled(vp) => pdf::handle_scrolled(app, vp),
            crate::app::messages::PdfMsg::PagesLoaded(_) => pdf::handle_pages_loaded(app),
            crate::app::messages::PdfMsg::PageReady(idx, d, w, h) => {
                pdf::handle_page_ready(app, idx, d, w, h)
            }
            crate::app::messages::PdfMsg::ThumbReady(idx, d, w, h) => {
                pdf::handle_thumb_ready(app, idx, d, w, h)
            }
        },
        Message::Spreadsheet(msg) => match msg {
            crate::app::messages::SpreadsheetMsg::SheetTabClicked(idx) => {
                spreadsheet::handle_sheet_tab_clicked(app, idx)
            }
            crate::app::messages::SpreadsheetMsg::ColumnClicked(col) => {
                spreadsheet::handle_column_clicked(app, col)
            }
            crate::app::messages::SpreadsheetMsg::SearchQueryChanged(q) => {
                spreadsheet::handle_search_query_changed(app, q)
            }
            crate::app::messages::SpreadsheetMsg::SearchClosed => {
                spreadsheet::handle_search_closed(app)
            }
        },
        Message::Grid(msg) => match msg {
            crate::app::messages::GridMsg::SearchQueryChanged(q) => {
                grid::handle_grid_search_query_changed(app, q)
            }
            crate::app::messages::GridMsg::SearchClosed => grid::handle_grid_search_closed(app),
        },
        Message::Markdown(msg) => match msg {
            crate::app::messages::MarkdownMsg::TocToggled => markdown::handle_toc_toggled(app),
            crate::app::messages::MarkdownMsg::TocHeadingClicked(idx) => {
                markdown::handle_toc_heading_clicked(app, idx)
            }
            crate::app::messages::MarkdownMsg::TocToggleCollapse(idx) => {
                markdown::handle_toc_toggle_collapse(app, idx)
            }
            crate::app::messages::MarkdownMsg::Scrolled(y) => {
                markdown::handle_markdown_scrolled(app, y)
            }
            crate::app::messages::MarkdownMsg::SearchToggle => markdown::handle_search_toggle(app),
            crate::app::messages::MarkdownMsg::SearchQueryChanged(q) => {
                markdown::handle_search_query_changed(app, q)
            }
            crate::app::messages::MarkdownMsg::SearchNext => markdown::handle_search_next(app),
            crate::app::messages::MarkdownMsg::SearchPrev => markdown::handle_search_prev(app),
            crate::app::messages::MarkdownMsg::SearchClosed => markdown::handle_search_closed(app),
            crate::app::messages::MarkdownMsg::SidebarResized(w) => {
                misc::handle_markdown_sidebar_resized(app, w)
            }
            crate::app::messages::MarkdownMsg::MermaidBlockRendered { index, png_bytes } => {
                image::handle_mermaid_rendered(app, index, png_bytes)
            }
            crate::app::messages::MarkdownMsg::ImageLoaded { index, png_bytes } => {
                image::handle_markdown_image_loaded(app, index, png_bytes)
            }
        },
        Message::Epub(msg) => match msg {
            crate::app::messages::EpubMsg::SidebarToggled => epub::handle_sidebar_toggled(app),
            crate::app::messages::EpubMsg::ChapterClicked(idx) => {
                epub::handle_chapter_clicked(app, idx)
            }
            crate::app::messages::EpubMsg::ChapterToggleCollapse(idx) => {
                epub::handle_chapter_toggle_collapse(app, idx)
            }
            crate::app::messages::EpubMsg::SidebarResized(w) => {
                misc::handle_epub_sidebar_resized(app, w)
            }
        },
        Message::Json(msg) => match msg {
            crate::app::messages::JsonMsg::ToggleMode => json::handle_toggle_mode(app),
            crate::app::messages::JsonMsg::ToggleNode(idx) => json::handle_toggle_node(app, idx),
            crate::app::messages::JsonMsg::Scrolled(y) => json::handle_scrolled(app, y),
            crate::app::messages::JsonMsg::RawEdit(act) => json::handle_raw_edit(app, act),
            crate::app::messages::JsonMsg::SearchToggle => json::handle_search_toggle(app),
            crate::app::messages::JsonMsg::SearchQueryChanged(q) => {
                json::handle_search_query_changed(app, q)
            }
            crate::app::messages::JsonMsg::SearchClosed => json::handle_search_closed(app),
            crate::app::messages::JsonMsg::ExpandAll => json::handle_expand_all(app),
            crate::app::messages::JsonMsg::CollapseAll => json::handle_collapse_all(app),
            crate::app::messages::JsonMsg::CopyPath(idx) => json::handle_copy_path(app, idx),
            crate::app::messages::JsonMsg::NodeClicked(idx) => json::handle_node_clicked(app, idx),
            crate::app::messages::JsonMsg::BreadcrumbClicked(idx) => {
                json::handle_breadcrumb_clicked(app, idx)
            }
            crate::app::messages::JsonMsg::ToggleFormat => json::handle_toggle_format(app),
            crate::app::messages::JsonMsg::EditStart(idx) => json::handle_edit_start(app, idx),
            crate::app::messages::JsonMsg::EditValue(v) => json::handle_edit_value(app, v),
            crate::app::messages::JsonMsg::EditSave => json::handle_edit_save(app),
            crate::app::messages::JsonMsg::EditCancel => json::handle_edit_cancel(app),
            crate::app::messages::JsonMsg::SchemaToggle => json::handle_schema_toggle(app),
        },
        Message::Settings(msg) => match msg {
            crate::app::messages::SettingsMsg::ThemeChanged(t) => {
                app.state.theme_dark = t != "Light";
                let mut config = crate::core::config::ConfigManager::load_or_create();
                config.ui.theme = Some(t);
                let _ = crate::core::config::ConfigManager::save(&config);
                Task::none()
            }
            crate::app::messages::SettingsMsg::FontSizeChanged(s) => {
                app.state.font_size = s;
                let mut config = crate::core::config::ConfigManager::load_or_create();
                config.ui.font_size = s;
                let _ = crate::core::config::ConfigManager::save(&config);
                Task::none()
            }
            crate::app::messages::SettingsMsg::FontFamilySelected(f) => {
                app.state.font_family = Some(f.clone());
                let mut config = crate::core::config::ConfigManager::load_or_create();
                config.ui.font_family = Some(f);
                let _ = crate::core::config::ConfigManager::save(&config);
                Task::none()
            }
            crate::app::messages::SettingsMsg::FontFamilyMonoSelected(f) => {
                app.state.font_family_mono = Some(f.clone());
                let mut config = crate::core::config::ConfigManager::load_or_create();
                config.ui.font_family_mono = Some(f);
                let _ = crate::core::config::ConfigManager::save(&config);
                Task::none()
            }
            crate::app::messages::SettingsMsg::EpubFontFamilySelected(f) => {
                app.state.epub_font_family = Some(f.clone());
                let mut config = crate::core::config::ConfigManager::load_or_create();
                config.ui.epub_font_family = Some(f);
                let _ = crate::core::config::ConfigManager::save(&config);
                Task::none()
            }
            crate::app::messages::SettingsMsg::MaxTextWidthChanged(w) => {
                app.state.max_text_width = w;
                let mut config = crate::core::config::ConfigManager::load_or_create();
                config.ui.max_text_width = w;
                let _ = crate::core::config::ConfigManager::save(&config);
                Task::none()
            }
            crate::app::messages::SettingsMsg::DefaultWidthChanged(w) => {
                app.state.window_default_size.width = w as f32;
                let mut config = crate::core::config::ConfigManager::load_or_create();
                config.ui.default_width = w;
                let _ = crate::core::config::ConfigManager::save(&config);
                Task::none()
            }
            crate::app::messages::SettingsMsg::DefaultHeightChanged(h) => {
                app.state.window_default_size.height = h as f32;
                let mut config = crate::core::config::ConfigManager::load_or_create();
                config.ui.default_height = h;
                let _ = crate::core::config::ConfigManager::save(&config);
                Task::none()
            }
            crate::app::messages::SettingsMsg::MinWidthChanged(w) => {
                app.state.window_min_size.width = w as f32;
                let mut config = crate::core::config::ConfigManager::load_or_create();
                config.ui.min_width = w;
                let _ = crate::core::config::ConfigManager::save(&config);
                Task::none()
            }
            crate::app::messages::SettingsMsg::MinHeightChanged(h) => {
                app.state.window_min_size.height = h as f32;
                let mut config = crate::core::config::ConfigManager::load_or_create();
                config.ui.min_height = h;
                let _ = crate::core::config::ConfigManager::save(&config);
                Task::none()
            }
        },

        // Global Layout / Input Events
        Message::CtrlHeldChanged(held) => app.handle_ctrl_changed(held),
        Message::ShiftHeldChanged(held) => app.handle_shift_changed(held),
        Message::ModifiersUpdated(modifiers) => app.handle_modifiers_changed(modifiers),
        Message::ScrollDelta { x, y } => app.handle_scroll_delta(x, y),
        Message::SortByFieldClicked(field) => navigation::handle_sort_by_field(app, field),
        Message::SidebarDragStarted(start_x) => misc::handle_sidebar_drag_started(app, start_x),
        Message::SidebarDragEnded => misc::handle_sidebar_drag_ended(app),
        Message::MouseMoved(x, y) => misc::handle_mouse_moved(app, x, y),
    }
}
