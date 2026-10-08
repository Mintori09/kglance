use crate::app::KglanceApp;
use crate::app::messages::Message;
use crate::features::markdown::update as markdown;
use iced::Task;
use url::Url;

pub fn handle_copy_text(app: &mut KglanceApp, text: String) -> Task<Message> {
    let toast = app.show_toast("Copied!");
    Task::batch(vec![iced::clipboard::write(text), toast])
}

pub fn handle_open_link(url: String) -> Task<Message> {
    if is_allowed_link(&url) {
        let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
    }
    Task::none()
}

fn is_allowed_link(link: &str) -> bool {
    match Url::parse(link) {
        Ok(url) => match url.scheme() {
            "http" | "https" | "mailto" => true,
            "file" => {
                if let Ok(path) = url.to_file_path() {
                    path.is_file()
                } else {
                    false
                }
            }
            _ => false,
        },
        Err(_) => {
            let path = std::path::Path::new(link);
            path.is_absolute() && path.is_file()
        }
    }
}

pub fn handle_theme_toggled(app: &mut KglanceApp) -> Task<Message> {
    app.state.app_theme = match app.state.app_theme {
        crate::ui::theme::AppTheme::Dark => crate::ui::theme::AppTheme::Light,
        crate::ui::theme::AppTheme::Light => crate::ui::theme::AppTheme::Nord,
        _ => crate::ui::theme::AppTheme::Dark,
    };
    Task::none()
}

pub fn handle_toast_dismissed(app: &mut KglanceApp, id: u64) -> Task<Message> {
    app.state.toasts.retain(|t| t.id != id);
    Task::none()
}

pub fn handle_markdown_sidebar_resized(app: &mut KglanceApp, width: f32) -> Task<Message> {
    use crate::ui::components::sidebar::{DEFAULT_MAX_SIDEBAR_WIDTH, DEFAULT_MIN_SIDEBAR_WIDTH};

    let new_w = width.clamp(DEFAULT_MIN_SIDEBAR_WIDTH, DEFAULT_MAX_SIDEBAR_WIDTH);
    if (app.state.markdown.sidebar_width - new_w).abs() > 1.0 {
        app.state.markdown.sidebar_width = new_w;
        if let Some(crate::core::PreviewData::Markdown { blocks, .. }) = &app.current_content {
            let content_width = app.state.markdown_content_width();
            crate::features::markdown::recompute_markdown_layout(
                &mut app.state.markdown,
                blocks,
                app.state.font_size,
                content_width,
            );
        }
    }
    Task::none()
}

pub fn handle_epub_sidebar_resized(app: &mut KglanceApp, width: f32) -> Task<Message> {
    use crate::ui::components::sidebar::{DEFAULT_MAX_SIDEBAR_WIDTH, DEFAULT_MIN_SIDEBAR_WIDTH};

    app.state.epub.sidebar_width =
        width.clamp(DEFAULT_MIN_SIDEBAR_WIDTH, DEFAULT_MAX_SIDEBAR_WIDTH);
    Task::none()
}

pub fn handle_sidebar_drag_started(app: &mut KglanceApp) -> Task<Message> {
    use crate::ui::components::sidebar::start_sidebar_drag;

    for (resizing, start_x, start_w, width) in [
        (
            &mut app.state.markdown.sidebar_resizing,
            &mut app.state.markdown.sidebar_drag_start_x,
            &mut app.state.markdown.sidebar_drag_start_width,
            app.state.markdown.sidebar_width,
        ),
        (
            &mut app.state.epub.sidebar_resizing,
            &mut app.state.epub.sidebar_drag_start_x,
            &mut app.state.epub.sidebar_drag_start_width,
            app.state.epub.sidebar_width,
        ),
        (
            &mut app.state.text.sidebar_resizing,
            &mut app.state.text.sidebar_drag_start_x,
            &mut app.state.text.sidebar_drag_start_width,
            app.state.text.sidebar_width,
        ),
        (
            &mut app.state.pdf.sidebar_resizing,
            &mut app.state.pdf.sidebar_drag_start_x,
            &mut app.state.pdf.sidebar_drag_start_width,
            app.state.pdf.sidebar_width,
        ),
    ] {
        start_sidebar_drag(resizing, start_x, start_w, width);
    }
    Task::none()
}

pub fn handle_sidebar_drag_ended(app: &mut KglanceApp) -> Task<Message> {
    use crate::ui::components::sidebar::end_sidebar_drag;

    for resizing in [
        &mut app.state.markdown.sidebar_resizing,
        &mut app.state.epub.sidebar_resizing,
        &mut app.state.text.sidebar_resizing,
        &mut app.state.pdf.sidebar_resizing,
    ] {
        end_sidebar_drag(resizing);
    }

    let win_w = app.state.current_window_size.width;
    let pdf = &mut app.state.pdf;
    let desired_w = pdf.desired_width;
    crate::features::pdf::geometry::recalculate_pdf_thumbnail_offsets(pdf);
    let sidebar_w = if pdf.sidebar_visible {
        pdf.sidebar_width + 1.0
    } else {
        0.0
    };
    let max_w = (win_w - sidebar_w - 40.0).clamp(300.0, 2400.0);
    let target_display_w = desired_w.min(max_w);
    crate::features::pdf::view::recalculate_pdf_offsets_for_width(pdf, target_display_w);
    Task::none()
}

pub fn handle_mouse_pressed(app: &mut KglanceApp, _x: f32, _y: f32) -> Task<Message> {
    app.state.pdf.auto_scroll_delta = None;
    let s = markdown::active_markdown_state_mut(app);
    s.is_mouse_held = true;
    s.is_dragging_selection = false;
    s.selection_range = None;
    s.selected_text = None;
    s.auto_scroll_delta = None;
    s.smooth_scroll.is_animating = false;
    s.smooth_scroll.velocity = 0.0;
    Task::none()
}

pub fn handle_mouse_released(app: &mut KglanceApp) -> Task<Message> {
    let _ = handle_sidebar_drag_ended(app);
    app.state.text.is_dragging_selection = false;
    app.state.text.auto_scroll_delta = None;
    app.state.text.drag_start = None;
    app.state.spreadsheet.is_dragging = false;
    app.state.spreadsheet.is_dragging_row_headers = false;
    app.state.spreadsheet.auto_scroll_delta_y = None;
    app.state.spreadsheet.auto_scroll_delta_x = None;
    app.state.pdf.is_selecting = false;
    app.state.pdf.auto_scroll_delta = None;
    crate::features::pdf::selection::handle_selection_drag_end(&mut app.state.pdf);
    let s = markdown::active_markdown_state_mut(app);
    s.is_mouse_held = false;
    s.is_dragging_selection = false;
    s.auto_scroll_delta = None;
    markdown::handle_selection_drag_end(app)
}

pub fn handle_mouse_moved(app: &mut KglanceApp, x: f32, y: f32) -> Task<Message> {
    use crate::ui::components::sidebar::{
        DEFAULT_MAX_SIDEBAR_WIDTH, DEFAULT_MIN_SIDEBAR_WIDTH, PDF_MAX_SIDEBAR_WIDTH,
        PDF_MIN_SIDEBAR_WIDTH, apply_sidebar_drag,
    };

    markdown::active_markdown_state_mut(app).drag_last_y = y;
    app.state.pdf.drag_last_x = x;
    app.state.pdf.drag_last_y = y;

    if app.state.pdf.is_selecting {
        const HEADER_HEIGHT: f32 = 40.0;
        const FOOTER_HEIGHT: f32 = 30.0;
        const MIN_CONTENT_HEIGHT: f32 = 100.0;

        let win_height = app.state.current_window_size.height;
        let top_bound = HEADER_HEIGHT;
        let bottom_bound = (win_height - FOOTER_HEIGHT).max(top_bound + MIN_CONTENT_HEIGHT);

        let overflow = if y < top_bound {
            y - top_bound
        } else if y > bottom_bound {
            y - bottom_bound
        } else {
            0.0
        };

        if overflow != 0.0 {
            let direction = overflow.signum();
            let speed = (overflow.abs() * 0.8).clamp(5.0, 40.0) * direction;
            app.state.pdf.auto_scroll_delta = Some(speed);
        } else {
            app.state.pdf.auto_scroll_delta = None;
        }

        let rel_y = (y - top_bound).max(0.0);
        let content_y = app.state.pdf.scroll_y + rel_y;
        let win_w = app.state.current_window_size.width;
        if let Some(pos) = crate::features::pdf::selection::hit_test_pdf_position(
            &app.state.pdf,
            win_w,
            x,
            content_y,
        ) {
            crate::features::pdf::selection::handle_selection_drag_update(&mut app.state.pdf, pos);
        }
    }

    if app.state.text.is_dragging_selection {
        const HEADER_HEIGHT: f32 = 40.0;
        const FOOTER_HEIGHT: f32 = 30.0;
        const MIN_CONTENT_HEIGHT: f32 = 100.0;

        let win_height = app.state.current_window_size.height;
        let top_bound = HEADER_HEIGHT;
        let bottom_bound = (win_height - FOOTER_HEIGHT).max(top_bound + MIN_CONTENT_HEIGHT);

        let overflow = if y < top_bound {
            y - top_bound
        } else if y > bottom_bound {
            y - bottom_bound
        } else {
            0.0
        };

        let sidebar_w = if app.state.text.outline_visible {
            app.state.text.sidebar_width + 4.0
        } else {
            0.0
        };
        app.state.text.drag_last_cursor = iced::Point::new((x - sidebar_w).max(0.0), y - top_bound);
        if overflow != 0.0 {
            let direction = overflow.signum();
            let speed = (overflow.abs() * 0.8).clamp(5.0, 40.0) * direction;
            app.state.text.auto_scroll_delta = Some(speed);
        } else {
            app.state.text.auto_scroll_delta = None;
        }
    }

    if markdown::active_markdown_state(app).is_dragging_selection {
        const HEADER_HEIGHT: f32 = 40.0;
        const FOOTER_HEIGHT: f32 = 30.0;
        const MIN_CONTENT_HEIGHT: f32 = 100.0;

        let win_height = app.state.current_window_size.height;
        let top_bound = HEADER_HEIGHT;
        let bottom_bound = (win_height - FOOTER_HEIGHT).max(top_bound + MIN_CONTENT_HEIGHT);

        let overflow = if y < top_bound {
            y - top_bound
        } else if y > bottom_bound {
            y - bottom_bound
        } else {
            0.0
        };

        let s = markdown::active_markdown_state_mut(app);
        if overflow != 0.0 {
            let direction = overflow.signum();
            let speed = (overflow.abs() * 0.8).clamp(5.0, 40.0) * direction;
            s.auto_scroll_delta = Some(speed);
        } else {
            s.auto_scroll_delta = None;
        }
    }

    if app.state.spreadsheet.is_dragging || app.state.spreadsheet.is_dragging_row_headers {
        const HEADER_HEIGHT: f32 = 40.0;
        const FOOTER_HEIGHT: f32 = 40.0;
        const LEFT_MARGIN: f32 = 10.0;
        const RIGHT_MARGIN: f32 = 10.0;
        const MIN_CONTENT_HEIGHT: f32 = 100.0;

        let win_width = app.state.current_window_size.width;
        let win_height = app.state.current_window_size.height;
        let top_bound = HEADER_HEIGHT;
        let bottom_bound = (win_height - FOOTER_HEIGHT).max(top_bound + MIN_CONTENT_HEIGHT);
        let left_bound = LEFT_MARGIN;
        let right_bound = (win_width - RIGHT_MARGIN).max(left_bound + 100.0);

        let overflow_y = if y < top_bound {
            y - top_bound
        } else if y > bottom_bound {
            y - bottom_bound
        } else {
            0.0
        };

        let overflow_x = if x < left_bound {
            x - left_bound
        } else if x > right_bound {
            x - right_bound
        } else {
            0.0
        };

        let s = &mut app.state.spreadsheet;
        s.drag_last_cursor = iced::Point::new(
            (x - left_bound).clamp(0.0, s.viewport_width),
            (y - top_bound).clamp(0.0, s.viewport_height),
        );

        if overflow_y != 0.0 {
            let direction = overflow_y.signum();
            let speed = (overflow_y.abs() * 0.8).clamp(5.0, 40.0) * direction;
            s.auto_scroll_delta_y = Some(speed);
        } else {
            s.auto_scroll_delta_y = None;
        }

        if overflow_x != 0.0 {
            let direction = overflow_x.signum();
            let speed = (overflow_x.abs() * 0.8).clamp(5.0, 40.0) * direction;
            s.auto_scroll_delta_x = Some(speed);
        } else {
            s.auto_scroll_delta_x = None;
        }
    }

    for (resizing, start_x, start_w, width, min, max) in [
        (
            app.state.markdown.sidebar_resizing,
            &mut app.state.markdown.sidebar_drag_start_x,
            &mut app.state.markdown.sidebar_drag_start_width,
            &mut app.state.markdown.sidebar_width,
            DEFAULT_MIN_SIDEBAR_WIDTH,
            DEFAULT_MAX_SIDEBAR_WIDTH,
        ),
        (
            app.state.epub.sidebar_resizing,
            &mut app.state.epub.sidebar_drag_start_x,
            &mut app.state.epub.sidebar_drag_start_width,
            &mut app.state.epub.sidebar_width,
            DEFAULT_MIN_SIDEBAR_WIDTH,
            DEFAULT_MAX_SIDEBAR_WIDTH,
        ),
        (
            app.state.text.sidebar_resizing,
            &mut app.state.text.sidebar_drag_start_x,
            &mut app.state.text.sidebar_drag_start_width,
            &mut app.state.text.sidebar_width,
            DEFAULT_MIN_SIDEBAR_WIDTH,
            DEFAULT_MAX_SIDEBAR_WIDTH,
        ),
    ] {
        if resizing {
            apply_sidebar_drag(start_x, start_w, width, x, min, max);
        }
    }

    let win_w = app.state.current_window_size.width;
    let pdf = &mut app.state.pdf;
    if pdf.sidebar_resizing {
        let old_width = pdf.sidebar_width;
        let desired_w = pdf.desired_width;
        apply_sidebar_drag(
            &mut pdf.sidebar_drag_start_x,
            &mut pdf.sidebar_drag_start_width,
            &mut pdf.sidebar_width,
            x,
            PDF_MIN_SIDEBAR_WIDTH,
            PDF_MAX_SIDEBAR_WIDTH,
        );
        if (pdf.sidebar_width - old_width).abs() > 0.5 {
            crate::features::pdf::geometry::recalculate_pdf_thumbnail_offsets(pdf);
            let sidebar_w = if pdf.sidebar_visible {
                pdf.sidebar_width + 1.0
            } else {
                0.0
            };
            let max_w = (win_w - sidebar_w - 40.0).clamp(300.0, 2400.0);
            let target_display_w = desired_w.min(max_w);
            if (pdf.display_width - target_display_w).abs() > 1.0 {
                crate::features::pdf::view::recalculate_pdf_offsets_for_width(
                    pdf,
                    target_display_w,
                );
            }
        }
    }
    Task::none()
}

pub fn update_current_window_size(app: &mut KglanceApp, width: f32, height: f32) -> Task<Message> {
    if width > 0.0 && height > 0.0 {
        let old_w = app.state.window_width;
        app.state.current_window_size.width = width;
        app.state.current_window_size.height = height;
        app.state.window_width = width;
        app.state.window_height = height;
        app.state.markdown.viewport_height = height;
        app.state.epub.markdown_state.viewport_height = height;

        if (old_w - width).abs() > 1.0 {
            if let Some(crate::core::PreviewData::Markdown { blocks, .. }) = &app.current_content {
                let content_width = app.state.markdown_content_width();
                crate::features::markdown::recompute_markdown_layout(
                    &mut app.state.markdown,
                    blocks,
                    app.state.font_size,
                    content_width,
                );
            } else if matches!(
                app.current_content,
                Some(crate::core::PreviewData::Text { .. })
            ) {
                app.state.text.viewport_height = height;
                let wrap_mode = if app.state.word_wrap {
                    crate::features::text::WrapMode::Word
                } else {
                    crate::features::text::WrapMode::None
                };
                app.state.text.display_map.update_geometry(
                    &app.state.text.document,
                    width,
                    app.state.font_size,
                    wrap_mode,
                );
                app.state.text.total_content_height =
                    app.state.text.display_map.total_content_height();
                let theme = app.state.app_theme;
                let scroll_y = app.state.text.scroll_y;
                crate::features::text::update_tokens_for_viewport(
                    &mut app.state.text,
                    scroll_y,
                    theme,
                );
            } else if matches!(
                app.current_content,
                Some(crate::core::PreviewData::Json { .. })
            ) {
                app.state.json.raw_text.viewport_height = height;
                let wrap_mode = if app.state.word_wrap {
                    crate::features::text::WrapMode::Word
                } else {
                    crate::features::text::WrapMode::None
                };
                app.state.json.raw_text.display_map.update_geometry(
                    &app.state.json.raw_text.document,
                    width,
                    app.state.font_size,
                    wrap_mode,
                );
                app.state.json.raw_text.total_content_height =
                    app.state.json.raw_text.display_map.total_content_height();
                let theme = app.state.app_theme;
                let scroll_y = app.state.json.raw_text.scroll_y;
                crate::features::text::update_tokens_for_viewport(
                    &mut app.state.json.raw_text,
                    scroll_y,
                    theme,
                );
            } else if matches!(
                app.current_content,
                Some(crate::core::PreviewData::Typst { .. })
            ) {
                app.state.typst.source_text.viewport_height = height;
                let wrap_mode = if app.state.word_wrap {
                    crate::features::text::WrapMode::Word
                } else {
                    crate::features::text::WrapMode::None
                };
                app.state.typst.source_text.display_map.update_geometry(
                    &app.state.typst.source_text.document,
                    width,
                    app.state.font_size,
                    wrap_mode,
                );
                app.state.typst.source_text.total_content_height = app
                    .state
                    .typst
                    .source_text
                    .display_map
                    .total_content_height();
                let theme = app.state.app_theme;
                let scroll_y = app.state.typst.source_text.scroll_y;
                crate::features::text::update_tokens_for_viewport(
                    &mut app.state.typst.source_text,
                    scroll_y,
                    theme,
                );
            }
        }
    }

    let pdf = &mut app.state.pdf;
    let desired_w = pdf.desired_width;
    let sidebar_w = if pdf.sidebar_visible {
        pdf.sidebar_width + 1.0
    } else {
        0.0
    };
    let max_w = (width - sidebar_w - 40.0).clamp(300.0, 2400.0);
    let target_display_w = desired_w.min(max_w);
    if (pdf.display_width - target_display_w).abs() > 1.0 {
        crate::features::pdf::view::recalculate_pdf_offsets_for_width(pdf, target_display_w);
    }

    Task::none()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_util::{epub_content, test_app};
    use crate::features::markdown::update::handle_selection_drag_start;

    #[test]
    fn drag_start_sets_resizing_and_clears_anchor() {
        let mut app = test_app(None);
        let _ = handle_sidebar_drag_started(&mut app);
        assert!(app.state.markdown.sidebar_resizing);
        assert!(app.state.markdown.sidebar_drag_start_x.is_none());
        assert!(app.state.epub.sidebar_resizing);
        assert!(app.state.epub.sidebar_drag_start_x.is_none());
        assert!(app.state.text.sidebar_resizing);
        assert!(app.state.text.sidebar_drag_start_x.is_none());
        assert!(app.state.pdf.sidebar_resizing);
        assert!(app.state.pdf.sidebar_drag_start_x.is_none());
    }

    #[test]
    fn mouse_moved_anchors_on_first_move_without_resizing() {
        let mut app = test_app(None);
        app.state.epub.sidebar_width = 200.0;
        let _ = handle_sidebar_drag_started(&mut app);
        let _ = handle_mouse_moved(&mut app, 250.0, 100.0);
        assert_eq!(app.state.epub.sidebar_drag_start_x, Some(250.0));
        assert_eq!(app.state.epub.sidebar_drag_start_width, 200.0);
        assert_eq!(app.state.epub.sidebar_width, 200.0);
    }

    #[test]
    fn mouse_moved_applies_delta_after_anchor() {
        let mut app = test_app(None);
        app.state.epub.sidebar_width = 200.0;
        let _ = handle_sidebar_drag_started(&mut app);
        let _ = handle_mouse_moved(&mut app, 250.0, 100.0);
        let _ = handle_mouse_moved(&mut app, 270.0, 100.0);
        assert_eq!(app.state.epub.sidebar_width, 220.0);
        assert_eq!(app.state.epub.sidebar_drag_start_x, Some(250.0));
    }

    #[test]
    fn mouse_moved_resizes_text_sidebar() {
        let mut app = test_app(None);
        app.state.text.sidebar_width = 250.0;
        let _ = handle_sidebar_drag_started(&mut app);
        let _ = handle_mouse_moved(&mut app, 300.0, 100.0);
        let _ = handle_mouse_moved(&mut app, 350.0, 100.0);
        assert_eq!(app.state.text.sidebar_width, 300.0);
        assert_eq!(app.state.text.sidebar_drag_start_x, Some(300.0));
    }

    #[test]
    fn mouse_moved_clamps_epub_width() {
        let mut app = test_app(None);
        app.state.epub.sidebar_width = 200.0;
        let _ = handle_sidebar_drag_started(&mut app);
        let _ = handle_mouse_moved(&mut app, 0.0, 100.0);
        let _ = handle_mouse_moved(&mut app, -1000.0, 100.0);
        assert_eq!(app.state.epub.sidebar_width, 140.0);
    }

    #[test]
    fn drag_end_clears_resizing() {
        let mut app = test_app(None);
        let _ = handle_sidebar_drag_started(&mut app);
        let _ = handle_sidebar_drag_ended(&mut app);
        assert!(!app.state.markdown.sidebar_resizing);
        assert!(!app.state.epub.sidebar_resizing);
        assert!(!app.state.text.sidebar_resizing);
        assert!(!app.state.pdf.sidebar_resizing);
    }

    #[test]
    fn epub_mouse_moved_sets_auto_scroll_on_epub_state() {
        let mut app = test_app(Some(epub_content(&["hello"])));
        let _ = handle_selection_drag_start(&mut app, 0, 0);
        app.state.current_window_size.height = 200.0;
        let _ = handle_mouse_moved(&mut app, 50.0, 9999.0);
        assert!(app.state.epub.markdown_state.auto_scroll_delta.is_some());
        assert!(app.state.markdown.auto_scroll_delta.is_none());
    }

    #[test]
    fn mouse_moved_clamps_pdf_width() {
        let mut app = test_app(None);
        app.state.pdf.sidebar_width = 300.0;
        let _ = handle_sidebar_drag_started(&mut app);
        let _ = handle_mouse_moved(&mut app, 100.0, 100.0);
        let _ = handle_mouse_moved(&mut app, -1000.0, 100.0);
        assert_eq!(app.state.pdf.sidebar_width, 120.0);
        assert!(app.state.pdf.sidebar_resizing);
        assert!(app.state.pdf.sidebar_drag_start_x.is_some());
    }

    #[test]
    fn pdf_mouse_moved_sets_auto_scroll_and_updates_selection() {
        let mut app = test_app(None);
        app.state.pdf.is_selecting = true;
        app.state.current_window_size.height = 200.0;
        let _ = handle_mouse_moved(&mut app, 100.0, 500.0);
        assert!(app.state.pdf.auto_scroll_delta.is_some());
        let delta = app.state.pdf.auto_scroll_delta.unwrap();
        assert!(delta > 0.0);

        let _ = handle_mouse_released(&mut app);
        assert!(!app.state.pdf.is_selecting);
        assert!(app.state.pdf.auto_scroll_delta.is_none());
    }
}
