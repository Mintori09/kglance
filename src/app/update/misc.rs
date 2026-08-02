use crate::app::KglanceApp;
use crate::app::messages::Message;
use crate::features::markdown::update as markdown;
use crate::features::pdf::update::active_pdf_state_mut;
use iced::Task;

pub fn handle_copy_code(app: &mut KglanceApp, code: String) -> Task<Message> {
    let toast = app.show_toast("Copied!");
    Task::batch(vec![iced::clipboard::write(code), toast])
}

pub fn handle_open_link(url: String) -> Task<Message> {
    let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
    Task::none()
}

pub fn handle_theme_toggled(app: &mut KglanceApp) -> Task<Message> {
    app.state.theme_dark = !app.state.theme_dark;
    Task::none()
}

pub fn handle_toast_dismissed(app: &mut KglanceApp, id: u64) -> Task<Message> {
    app.state.toasts.retain(|t| t.id != id);
    Task::none()
}

pub fn handle_markdown_sidebar_resized(app: &mut KglanceApp, width: f32) -> Task<Message> {
    app.state.markdown.sidebar_width = width.clamp(140.0, 550.0);
    Task::none()
}

pub fn handle_epub_sidebar_resized(app: &mut KglanceApp, width: f32) -> Task<Message> {
    app.state.epub.sidebar_width = width.clamp(140.0, 550.0);
    Task::none()
}

pub fn handle_sidebar_drag_started(app: &mut KglanceApp) -> Task<Message> {
    app.state.markdown.sidebar_resizing = true;
    app.state.markdown.sidebar_drag_start_x = None;
    app.state.markdown.sidebar_drag_start_width = app.state.markdown.sidebar_width;

    app.state.epub.sidebar_resizing = true;
    app.state.epub.sidebar_drag_start_x = None;
    app.state.epub.sidebar_drag_start_width = app.state.epub.sidebar_width;

    let pdf = active_pdf_state_mut(app);
    pdf.sidebar_resizing = true;
    pdf.sidebar_drag_start_x = None;
    pdf.sidebar_drag_start_width = pdf.sidebar_width;
    Task::none()
}

pub fn handle_sidebar_drag_ended(app: &mut KglanceApp) -> Task<Message> {
    app.state.markdown.sidebar_resizing = false;
    app.state.epub.sidebar_resizing = false;
    active_pdf_state_mut(app).sidebar_resizing = false;
    markdown::active_markdown_state_mut(app).is_dragging_selection = false;
    markdown::active_markdown_state_mut(app).auto_scroll_delta = None;
    markdown::handle_selection_drag_end(app)
}

pub fn handle_mouse_moved(app: &mut KglanceApp, x: f32, y: f32) -> Task<Message> {
    markdown::active_markdown_state_mut(app).drag_last_y = y;

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

    if app.state.markdown.sidebar_resizing {
        apply_sidebar_drag(
            &mut app.state.markdown.sidebar_drag_start_x,
            &mut app.state.markdown.sidebar_drag_start_width,
            &mut app.state.markdown.sidebar_width,
            x,
            140.0,
            550.0,
        );
    }
    if app.state.epub.sidebar_resizing {
        apply_sidebar_drag(
            &mut app.state.epub.sidebar_drag_start_x,
            &mut app.state.epub.sidebar_drag_start_width,
            &mut app.state.epub.sidebar_width,
            x,
            140.0,
            550.0,
        );
    }
    let pdf = active_pdf_state_mut(app);
    if pdf.sidebar_resizing {
        apply_sidebar_drag(
            &mut pdf.sidebar_drag_start_x,
            &mut pdf.sidebar_drag_start_width,
            &mut pdf.sidebar_width,
            x,
            120.0,
            500.0,
        );
    }
    Task::none()
}

fn apply_sidebar_drag(
    start_x: &mut Option<f32>,
    start_width: &mut f32,
    width: &mut f32,
    x: f32,
    min: f32,
    max: f32,
) {
    match *start_x {
        None => {
            *start_x = Some(x);
            *start_width = *width;
        }
        Some(anchor) => {
            *width = (*start_width + (x - anchor)).clamp(min, max);
        }
    }
}

pub fn update_current_window_size(app: &mut KglanceApp, width: f32, height: f32) -> Task<Message> {
    app.state.current_window_size.width = width;
    app.state.current_window_size.height = height;
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
}
