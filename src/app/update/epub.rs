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
        let chapter = &app.state.epub.chapters[idx];
        let font_size = app.state.font_size;
        let mut target_y: f32 = 0.0;

        if let Some(ref anc) = chapter.anchor {
            let mut y_accum: f32 = 0.0;
            for (b_idx, block) in chapter.blocks.iter().enumerate() {
                let text_flat = match block {
                    crate::parsers::markdown::Block::Heading { content, .. }
                    | crate::parsers::markdown::Block::Paragraph(content) => {
                        crate::parsers::markdown::flatten_inlines(content)
                    }
                    _ => String::new(),
                };
                if text_flat.contains(anc) || text_flat.contains(&chapter.title) {
                    target_y = y_accum;
                    break;
                }
                y_accum += crate::parsers::markdown::estimated_block_height(
                    block,
                    font_size,
                    b_idx,
                    &app.state.markdown.cached_image_sizes,
                );
            }
        } else {
            let mut y_accum: f32 = 0.0;
            for (b_idx, block) in chapter.blocks.iter().enumerate() {
                let text_flat = match block {
                    crate::parsers::markdown::Block::Heading { content, .. }
                    | crate::parsers::markdown::Block::Paragraph(content) => {
                        crate::parsers::markdown::flatten_inlines(content)
                    }
                    _ => String::new(),
                };
                if !chapter.title.is_empty() && text_flat.contains(&chapter.title) {
                    target_y = y_accum;
                    break;
                }
                y_accum += crate::parsers::markdown::estimated_block_height(
                    block,
                    font_size,
                    b_idx,
                    &app.state.markdown.cached_image_sizes,
                );
            }
        }

        if target_y > 0.0 {
            return operation::scroll_to(
                "content_scroll",
                operation::AbsoluteOffset {
                    x: 0.0,
                    y: target_y,
                },
            );
        } else {
            return operation::snap_to(
                "content_scroll",
                operation::RelativeOffset { x: 0.0, y: 0.0 },
            );
        }
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
