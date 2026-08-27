use crate::app::KglanceApp;
use crate::app::messages::Message;
use iced::Task;
use iced::widget::operation;

pub fn handle_sidebar_toggled(app: &mut KglanceApp) -> Task<Message> {
    app.state.epub.sidebar_visible = !app.state.epub.sidebar_visible;
    Task::none()
}

pub fn handle_chapter_clicked(app: &mut KglanceApp, idx: usize) -> Task<Message> {
    if idx < app.state.epub.chapters.len() {
        app.state.epub.active_chapter = idx;
        app.record_read_position();

        if let Some(crate::core::PreviewData::Epub { images, .. }) = &app.current_content {
            crate::features::epub::state::ensure_chapter_images(
                &mut app.state.epub.markdown_state,
                &app.state.epub.chapters,
                idx,
                images,
            );
        }

        return operation::snap_to(
            "content_scroll",
            operation::RelativeOffset { x: 0.0, y: 0.0 },
        );
    }
    Task::none()
}

pub fn handle_chapter_toggle_collapse(app: &mut KglanceApp, idx: usize) -> Task<Message> {
    if app.state.epub.collapsed_chapters.contains(&idx) {
        app.state.epub.collapsed_chapters.remove(&idx);
    } else {
        app.state.epub.collapsed_chapters.insert(idx);
    }
    Task::none()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_util::{epub_content, test_app};

    #[test]
    fn test_handle_chapter_clicked_switches_active_chapter() {
        let mut app = test_app(Some(epub_content(&[
            "Chapter 1 content",
            "Chapter 2 content",
        ])));
        assert_eq!(app.state.epub.active_chapter, 0);

        let _ = handle_chapter_clicked(&mut app, 1);
        assert_eq!(app.state.epub.active_chapter, 1);
    }

    #[test]
    fn test_handle_chapter_toggle_collapse() {
        let mut app = test_app(Some(epub_content(&["Ch 1"])));
        assert!(!app.state.epub.collapsed_chapters.contains(&0));

        let _ = handle_chapter_toggle_collapse(&mut app, 0);
        assert!(app.state.epub.collapsed_chapters.contains(&0));

        let _ = handle_chapter_toggle_collapse(&mut app, 0);
        assert!(!app.state.epub.collapsed_chapters.contains(&0));
    }
}
