use crate::app::KglanceApp;
use crate::app::messages::Message;
use crate::features::markdown::view::components::image::{
    handle_markdown_image_loaded, handle_mermaid_rendered,
};
use iced::Task;

pub mod file;
pub mod grid;
pub mod misc;
pub mod navigation;

pub fn update(app: &mut KglanceApp, message: Message) -> Task<Message> {
    match message {
        Message::None => Task::none(),
        Message::Action(msg) => match msg {
            crate::app::messages::ActionMsg::OpenClicked => app.handle_open_clicked(),
            crate::app::messages::ActionMsg::CopyPathClicked => app.handle_copy_path(),
            crate::app::messages::ActionMsg::BackClicked => Task::none(),
            crate::app::messages::ActionMsg::CloseRequested => app.handle_close(),
            crate::app::messages::ActionMsg::CopyContentClicked => Task::none(),
            crate::app::messages::ActionMsg::CopyCode(code) => misc::handle_copy_code(app, code),
        },
        Message::System(msg) => match msg {
            crate::app::messages::SystemMsg::WindowResized(width, height) => {
                misc::update_current_window_size(app, width, height)
            }
            crate::app::messages::SystemMsg::ThemeToggled => misc::handle_theme_toggled(app),
            crate::app::messages::SystemMsg::DaemonOpenWindow { path } => {
                app.handle_daemon_open_window(path)
            }
            crate::app::messages::SystemMsg::DaemonOpenWithPlaylist {
                path,
                content,
                playlist,
            } => file::handle_daemon_open_with_playlist(app, path, content, playlist),
            crate::app::messages::SystemMsg::DaemonUpdateWindow { path, content } => {
                file::handle_daemon_update_window(app, path, content)
            }
            crate::app::messages::SystemMsg::DaemonUpdateWithPlaylist {
                path,
                content,
                playlist,
            } => file::handle_daemon_update_with_playlist(app, path, content, playlist),
            crate::app::messages::SystemMsg::FileLoaded {
                path,
                content,
                generation_id,
            } => file::handle_file_loaded_msg(app, path, content, generation_id),
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
            crate::app::messages::SystemMsg::ActiveFileModified(path) => {
                file::handle_active_file_modified(app, path)
            }
            crate::app::messages::SystemMsg::ActiveFileDeleted(path) => {
                file::handle_active_file_deleted(app, path)
            }
            crate::app::messages::SystemMsg::DirectoryChanged(dir) => {
                file::handle_directory_changed(app, dir)
            }
            crate::app::messages::SystemMsg::ReadPositionsTick => {
                if app.state.read_positions_dirty {
                    let _ = app.state.read_positions.save();
                    app.state.read_positions_dirty = false;
                }
                iced::Task::none()
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
            crate::app::messages::NavigationMsg::DirectorySyncCompleted {
                dir,
                files,
                generation_id,
            } => file::handle_directory_sync_completed(app, &dir, files, generation_id),
            crate::app::messages::NavigationMsg::ToggleViewMode => {
                grid::handle_toggle_view_mode(app)
            }
            crate::app::messages::NavigationMsg::FileClickedInGrid(idx) => {
                navigation::handle_file_clicked_in_grid(app, idx)
            }
            crate::app::messages::NavigationMsg::GridThumbnailLoaded { path, handle } => {
                grid::handle_grid_thumbnail_loaded(app, &path, handle)
            }
            crate::app::messages::NavigationMsg::PreloadCompleted {
                path,
                content,
                decoded_cache,
            } => file::handle_preload_completed(app, path, content, decoded_cache),
            crate::app::messages::NavigationMsg::ToggleSettingsClicked => {
                navigation::handle_toggle_settings(app)
            }
            crate::app::messages::NavigationMsg::FileClicked(idx) => {
                navigation::handle_file_clicked(app, idx)
            }
            crate::app::messages::NavigationMsg::FolderScrolled(vp) => {
                app.state.folder.scroll_y = vp.absolute_offset().y;
                let vh = vp.bounds().height;
                if vh > 0.0 {
                    app.state.folder.viewport_height = vh;
                }
                Task::none()
            }
        },
        Message::Image(msg) => match msg {
            crate::app::messages::ImageMsg::Zoom { factor, cursor } => {
                crate::features::image::update::handle_zoom(app, factor, cursor)
            }
            crate::app::messages::ImageMsg::PanDelta(dx, dy) => {
                crate::features::image::update::handle_pan(app, dx, dy)
            }
            crate::app::messages::ImageMsg::FitToWindow => {
                crate::features::image::update::handle_fit_to_window(app)
            }
            crate::app::messages::ImageMsg::DoubleClick => {
                crate::features::image::update::handle_double_click(app)
            }
            crate::app::messages::ImageMsg::Decoded {
                load_id,
                handle,
                cached_preview,
                width,
                height,
            } => crate::features::image::update::handle_decoded(
                app,
                load_id,
                handle,
                cached_preview,
                width,
                height,
            ),
            crate::app::messages::ImageMsg::ToggleInfo => {
                crate::features::image::update::handle_toggle_info(app)
            }
            crate::app::messages::ImageMsg::CloseInfo => {
                crate::features::image::update::handle_close_info(app)
            }
        },

        Message::Text(msg) => match msg {
            crate::app::messages::TextMsg::SelectionChanged(selection) => {
                crate::features::text::update::handle_selection_changed(app, selection)
            }
            crate::app::messages::TextMsg::SelectionDragStarted(pos) => {
                crate::features::text::update::handle_selection_drag_started(app, pos)
            }
            crate::app::messages::TextMsg::SelectionDragEnded => {
                crate::features::text::update::handle_selection_drag_ended(app)
            }
            crate::app::messages::TextMsg::AutoScroll(delta, cursor) => {
                crate::features::text::update::handle_auto_scroll(app, delta, cursor)
            }
            crate::app::messages::TextMsg::AutoScrollTick => {
                crate::features::text::update::handle_auto_scroll_tick(app)
            }
            crate::app::messages::TextMsg::CopyRequested(text) => {
                crate::features::text::update::handle_copy_requested(app, text)
            }
            crate::app::messages::TextMsg::TokensReady(result) => {
                crate::features::text::update::handle_tokens_ready(app, result)
            }
            crate::app::messages::TextMsg::SearchQueryChanged(query) => {
                crate::features::text::update::handle_search_query_changed(app, query)
            }
            crate::app::messages::TextMsg::SearchNext => {
                crate::features::text::update::handle_search_next(app)
            }
            crate::app::messages::TextMsg::SearchPrev => {
                crate::features::text::update::handle_search_prev(app)
            }
            crate::app::messages::TextMsg::SearchClosed => {
                crate::features::text::update::handle_search_closed(app)
            }
            crate::app::messages::TextMsg::WrapToggled => {
                crate::features::text::update::handle_wrap_toggled(app)
            }
            crate::app::messages::TextMsg::Scrolled(viewport) => {
                crate::features::text::update::handle_text_scrolled(app, viewport)
            }
            crate::app::messages::TextMsg::WheelScrolled(delta) => {
                crate::features::text::update::handle_wheel_scrolled(app, delta)
            }
            crate::app::messages::TextMsg::ToggleOutline => {
                crate::features::text::update::handle_toggle_outline(app)
            }
            crate::app::messages::TextMsg::SymbolClicked(line) => {
                crate::features::text::update::handle_symbol_clicked(app, line)
            }
            crate::app::messages::TextMsg::GotoLineToggle => {
                crate::features::text::update::handle_goto_line_toggle(app)
            }
            crate::app::messages::TextMsg::GotoLineQueryChanged(query) => {
                crate::features::text::update::handle_goto_line_query_changed(app, query)
            }
            crate::app::messages::TextMsg::GotoLineSubmitted => {
                crate::features::text::update::handle_goto_line_submitted(app)
            }
            crate::app::messages::TextMsg::GotoLineClosed => {
                crate::features::text::update::handle_goto_line_closed(app)
            }
        },
        Message::Media(msg) => match msg {
            crate::app::messages::MediaMsg::PlayPauseClicked => {
                crate::features::video::update::handle_play_pause(app)
            }
            crate::app::messages::MediaMsg::SeekClicked(pct) => {
                crate::features::video::update::handle_seek(app, pct)
            }
            crate::app::messages::MediaMsg::SeekRelativeClicked(secs) => {
                crate::features::video::update::handle_seek_relative(app, secs)
            }
            crate::app::messages::MediaMsg::VideoNewFrame => {
                crate::features::video::update::handle_video_new_frame(app)
            }
            crate::app::messages::MediaMsg::VideoEndOfStream => {
                crate::features::video::update::handle_video_end_of_stream(app)
            }
            crate::app::messages::MediaMsg::MouseEnter => {
                crate::features::video::update::handle_media_mouse_enter(app)
            }
            crate::app::messages::MediaMsg::MouseLeave => {
                crate::features::video::update::handle_media_mouse_leave(app)
            }
            crate::app::messages::MediaMsg::VideoThumbnailLoaded { data } => {
                crate::features::image::update::handle_video_thumbnail_loaded(app, data)
            }
        },
        Message::Pdf(msg) => match msg {
            crate::app::messages::PdfMsg::Scrolled(vp) => {
                crate::features::pdf::update::handle_scrolled(app, vp)
            }
            crate::app::messages::PdfMsg::SidebarScrolled(vp) => {
                crate::features::pdf::update::handle_sidebar_scrolled(app, vp)
            }
            crate::app::messages::PdfMsg::PagesLoaded(_) => {
                crate::features::pdf::update::handle_pages_loaded(app)
            }
            crate::app::messages::PdfMsg::PageReady(idx, d, w, h) => {
                crate::features::pdf::update::handle_page_ready(app, idx, d, w, h)
            }
            crate::app::messages::PdfMsg::ThumbReady(idx, d, w, h) => {
                crate::features::pdf::update::handle_thumb_ready(app, idx, d, w, h)
            }
            crate::app::messages::PdfMsg::SidebarToggled => {
                crate::features::pdf::update::handle_sidebar_toggled(app)
            }
            crate::app::messages::PdfMsg::SetSidebarMode(m) => {
                crate::features::pdf::update::handle_set_sidebar_mode(app, m)
            }
            crate::app::messages::PdfMsg::ThumbnailClicked(idx) => {
                crate::features::pdf::update::handle_thumbnail_clicked(app, idx)
            }
            crate::app::messages::PdfMsg::TocItemClicked(idx) => {
                crate::features::pdf::update::handle_toc_item_clicked(app, idx)
            }
            crate::app::messages::PdfMsg::SidebarResized(w) => {
                crate::features::pdf::update::handle_sidebar_resized(app, w)
            }
            crate::app::messages::PdfMsg::WheelScrolled(delta) => {
                crate::features::pdf::update::handle_wheel_scrolled(app, delta)
            }
        },
        Message::Typst(msg) => match msg {
            crate::app::messages::TypstMsg::Scrolled(vp) => {
                crate::features::typst::update::handle_scrolled(app, vp)
            }
            crate::app::messages::TypstMsg::PagesLoaded => {
                crate::features::typst::update::handle_pages_loaded(app)
            }
            crate::app::messages::TypstMsg::PageReady(idx, d, w, h) => {
                crate::features::typst::update::handle_page_ready(app, idx, d, w, h)
            }
            crate::app::messages::TypstMsg::CompileError => {
                crate::features::typst::update::handle_compile_error(app)
            }
            crate::app::messages::TypstMsg::ToggleSource => {
                crate::features::typst::update::handle_toggle_source(app)
            }
            crate::app::messages::TypstMsg::SelectionChanged(sel) => {
                app.state.typst.source_text.selection = sel;
                iced::Task::none()
            }
            crate::app::messages::TypstMsg::SourceScrolled(vp) => {
                crate::features::typst::update::handle_source_scrolled(app, vp)
            }
            crate::app::messages::TypstMsg::SourceWheelScrolled(delta) => {
                crate::features::typst::update::handle_source_wheel_scrolled(app, delta)
            }
        },
        Message::Spreadsheet(msg) => match msg {
            crate::app::messages::SpreadsheetMsg::SheetTabClicked(idx) => {
                crate::features::csv::update::handle_sheet_tab_clicked(app, idx)
            }
            crate::app::messages::SpreadsheetMsg::ColumnClicked(col) => {
                crate::features::csv::update::handle_column_clicked(app, col)
            }
            crate::app::messages::SpreadsheetMsg::SearchQueryChanged(q) => {
                crate::features::csv::update::handle_search_query_changed(app, q)
            }
            crate::app::messages::SpreadsheetMsg::SearchClosed => {
                crate::features::csv::update::handle_search_closed(app)
            }
            crate::app::messages::SpreadsheetMsg::Scrolled(vp) => {
                crate::features::csv::update::handle_spreadsheet_scrolled(app, vp)
            }
            crate::app::messages::SpreadsheetMsg::WheelScrolled(delta) => {
                crate::features::csv::update::handle_wheel_scrolled(app, delta)
            }
        },
        Message::Grid(msg) => match msg {
            crate::app::messages::GridMsg::SearchQueryChanged(q) => {
                grid::handle_grid_search_query_changed(app, q)
            }
            crate::app::messages::GridMsg::SearchClosed => grid::handle_grid_search_closed(app),
        },
        Message::Markdown(msg) => match msg {
            crate::app::messages::MarkdownMsg::TocToggled => {
                crate::features::markdown::update::handle_toc_toggled(app)
            }
            crate::app::messages::MarkdownMsg::TocHeadingClicked(idx) => {
                crate::features::markdown::update::handle_toc_heading_clicked(app, idx)
            }
            crate::app::messages::MarkdownMsg::TocToggleCollapse(idx) => {
                crate::features::markdown::update::handle_toc_toggle_collapse(app, idx)
            }
            crate::app::messages::MarkdownMsg::Scrolled {
                y,
                viewport_height,
                content_height,
            } => crate::features::markdown::update::handle_markdown_scrolled(
                app,
                y,
                viewport_height,
                content_height,
            ),
            crate::app::messages::MarkdownMsg::SearchToggle => {
                crate::features::markdown::update::handle_search_toggle(app)
            }
            crate::app::messages::MarkdownMsg::SearchQueryChanged(q) => {
                crate::features::markdown::update::handle_search_query_changed(app, q)
            }
            crate::app::messages::MarkdownMsg::SearchNext => {
                crate::features::markdown::update::handle_search_next(app)
            }
            crate::app::messages::MarkdownMsg::SearchPrev => {
                crate::features::markdown::update::handle_search_prev(app)
            }
            crate::app::messages::MarkdownMsg::SearchClosed => {
                crate::features::markdown::update::handle_search_closed(app)
            }
            crate::app::messages::MarkdownMsg::SidebarResized(w) => {
                misc::handle_markdown_sidebar_resized(app, w)
            }
            crate::app::messages::MarkdownMsg::MermaidBlockRendered {
                generation_id,
                index,
                png_bytes,
            } => handle_mermaid_rendered(app, generation_id, index, png_bytes),
            crate::app::messages::MarkdownMsg::ImageLoaded {
                generation_id,
                index,
                png_bytes,
            } => handle_markdown_image_loaded(app, generation_id, index, png_bytes),
            crate::app::messages::MarkdownMsg::SelectionChanged(s) => {
                crate::features::markdown::update::handle_selection_changed(app, s)
            }
            crate::app::messages::MarkdownMsg::SelectionDragStart { block, offset } => {
                crate::features::markdown::update::handle_selection_drag_start(app, block, offset)
            }
            crate::app::messages::MarkdownMsg::SelectionDragUpdate { block, offset } => {
                crate::features::markdown::update::handle_selection_drag_update(app, block, offset)
            }
            crate::app::messages::MarkdownMsg::SelectionDragEnd => {
                crate::features::markdown::update::handle_selection_drag_end(app)
            }
            crate::app::messages::MarkdownMsg::SelectionClear => {
                crate::features::markdown::update::handle_selection_clear(app)
            }
            crate::app::messages::MarkdownMsg::AutoScrollTick => {
                crate::features::markdown::update::handle_auto_scroll_tick(app)
            }
            crate::app::messages::MarkdownMsg::SmoothWheelScrolled(delta) => {
                crate::features::markdown::update::handle_smooth_wheel_scrolled(app, delta)
            }
            crate::app::messages::MarkdownMsg::TouchpadGestureEnded(event_time) => {
                crate::features::markdown::update::handle_touchpad_gesture_ended(app, event_time)
            }
        },
        Message::Epub(msg) => match msg {
            crate::app::messages::EpubMsg::SidebarToggled => {
                crate::features::epub::update::handle_sidebar_toggled(app)
            }
            crate::app::messages::EpubMsg::ChapterClicked(idx) => {
                crate::features::epub::update::handle_chapter_clicked(app, idx)
            }
            crate::app::messages::EpubMsg::ChapterToggleCollapse(idx) => {
                crate::features::epub::update::handle_chapter_toggle_collapse(app, idx)
            }
            crate::app::messages::EpubMsg::SidebarResized(w) => {
                misc::handle_epub_sidebar_resized(app, w)
            }
        },
        Message::Json(msg) => match msg {
            crate::app::messages::JsonMsg::ToggleMode => {
                crate::features::json::update::handle_toggle_mode(app)
            }
            crate::app::messages::JsonMsg::ToggleNode(idx) => {
                crate::features::json::update::handle_toggle_node(app, idx)
            }
            crate::app::messages::JsonMsg::Scrolled(vp) => {
                crate::features::json::update::handle_scrolled(app, vp)
            }
            crate::app::messages::JsonMsg::WheelScrolled(delta) => {
                crate::features::json::update::handle_wheel_scrolled(app, delta)
            }
            crate::app::messages::JsonMsg::RawScrolled(vp) => {
                crate::features::json::update::handle_raw_scrolled(app, vp)
            }
            crate::app::messages::JsonMsg::RawWheelScrolled(delta) => {
                crate::features::json::update::handle_raw_wheel_scrolled(app, delta)
            }
            crate::app::messages::JsonMsg::RawSelectionChanged(sel) => {
                crate::features::json::update::handle_raw_selection_changed(app, sel)
            }
            crate::app::messages::JsonMsg::SearchToggle => {
                crate::features::json::update::handle_search_toggle(app)
            }
            crate::app::messages::JsonMsg::SearchQueryChanged(q) => {
                crate::features::json::update::handle_search_query_changed(app, q)
            }
            crate::app::messages::JsonMsg::SearchNext => {
                crate::features::json::update::handle_search_next(app)
            }
            crate::app::messages::JsonMsg::SearchPrev => {
                crate::features::json::update::handle_search_prev(app)
            }
            crate::app::messages::JsonMsg::SearchClosed => {
                crate::features::json::update::handle_search_closed(app)
            }
            crate::app::messages::JsonMsg::ExpandAll => {
                crate::features::json::update::handle_expand_all(app)
            }
            crate::app::messages::JsonMsg::CollapseAll => {
                crate::features::json::update::handle_collapse_all(app)
            }
            crate::app::messages::JsonMsg::CopyPath(idx) => {
                crate::features::json::update::handle_copy_path(app, idx)
            }
            crate::app::messages::JsonMsg::CopyValue(idx) => {
                crate::features::json::update::handle_copy_value(app, idx)
            }
            crate::app::messages::JsonMsg::CopyKey(idx) => {
                crate::features::json::update::handle_copy_key(app, idx)
            }
            crate::app::messages::JsonMsg::CopySubtree(idx) => {
                crate::features::json::update::handle_copy_subtree(app, idx)
            }
            crate::app::messages::JsonMsg::NodeClicked(idx) => {
                crate::features::json::update::handle_node_clicked(app, idx)
            }
            crate::app::messages::JsonMsg::BreadcrumbClicked(idx) => {
                crate::features::json::update::handle_breadcrumb_clicked(app, idx)
            }
            crate::app::messages::JsonMsg::ToggleFormat => {
                crate::features::json::update::handle_toggle_format(app)
            }
        },
        Message::Settings(msg) => match msg {
            crate::app::messages::SettingsMsg::ThemeChanged(t) => {
                app.state.theme_setting = t.clone();
                app.state.app_theme = crate::core::config::ConfigManager::resolve_theme(&t);
                let mut config = crate::core::config::ConfigManager::load_or_create();
                config.ui.theme = Some(t);
                let _ = crate::core::config::ConfigManager::save(&config);
                Task::none()
            }
            crate::app::messages::SettingsMsg::FontSizeChanged(s) => {
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
            crate::app::messages::SettingsMsg::WordWrapChanged(enabled) => {
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
                let sidebar_w =
                    if app.state.text.outline_visible && app.state.text.sidebar_width > 0.0 {
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
                app.state.text.total_content_height =
                    app.state.text.display_map.total_content_height();
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

                let mut config = crate::core::config::ConfigManager::load_or_create();
                config.ui.word_wrap = enabled;
                let _ = crate::core::config::ConfigManager::save(&config);
                Task::none()
            }
            crate::app::messages::SettingsMsg::JsonTreeViewChanged(enabled) => {
                app.state.json_tree_view = enabled;
                app.state.json.tree_mode = enabled;
                let mut config = crate::core::config::ConfigManager::load_or_create();
                config.ui.json_tree_view = enabled;
                let _ = crate::core::config::ConfigManager::save(&config);
                Task::none()
            }
        },

        // Global Layout / Input Events
        Message::SmoothScrollTick(now) => match app.current_content {
            Some(
                crate::core::PreviewData::Markdown { .. } | crate::core::PreviewData::Epub { .. },
            ) => crate::features::markdown::update::handle_smooth_scroll_tick(app, now),
            Some(crate::core::PreviewData::Pdf { .. }) => {
                crate::features::pdf::update::handle_smooth_scroll_tick(app, now)
            }
            Some(crate::core::PreviewData::Typst { .. }) => {
                if app.state.typst.show_source || app.state.typst.error.is_some() {
                    let theme = app.state.app_theme;
                    let (task, finished) =
                        crate::features::text::update::advance_text_state_smooth_scroll(
                            &mut app.state.typst.source_text,
                            theme,
                            "typst_source_scroll",
                            now,
                        );
                    if finished {
                        app.record_read_position();
                    }
                    task
                } else {
                    crate::features::pdf::update::handle_smooth_scroll_tick(app, now)
                }
            }
            Some(crate::core::PreviewData::Text { .. }) => {
                crate::features::text::update::handle_smooth_scroll_tick(app, now)
            }
            Some(crate::core::PreviewData::Json { .. }) => {
                if !app.state.json.tree_mode {
                    let theme = app.state.app_theme;
                    let (task, finished) =
                        crate::features::text::update::advance_text_state_smooth_scroll(
                            &mut app.state.json.raw_text,
                            theme,
                            "json_raw_scroll",
                            now,
                        );
                    if finished {
                        app.record_read_position();
                    }
                    task
                } else {
                    crate::features::json::update::handle_smooth_scroll_tick(app, now)
                }
            }
            Some(crate::core::PreviewData::Spreadsheet { .. }) => {
                crate::features::csv::update::handle_smooth_scroll_tick(app, now)
            }
            _ => Task::none(),
        },
        Message::CtrlHeldChanged(held) => app.handle_ctrl_changed(held),
        Message::ShiftHeldChanged(held) => app.handle_shift_changed(held),
        Message::ModifiersUpdated(modifiers) => app.handle_modifiers_changed(modifiers),
        Message::ScrollDelta { x, y } => app.handle_scroll_delta(x, y),
        Message::SortByFieldClicked(field) => navigation::handle_sort_by_field(app, field),
        Message::SidebarDragStarted => misc::handle_sidebar_drag_started(app),
        Message::SidebarDragEnded => misc::handle_sidebar_drag_ended(app),
        Message::MouseMoved(x, y) => misc::handle_mouse_moved(app, x, y),
        Message::MousePressed(x, y) => misc::handle_mouse_pressed(app, x, y),
        Message::MouseReleased => misc::handle_mouse_released(app),
    }
}
