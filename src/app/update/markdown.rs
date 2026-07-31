use crate::app::KglanceApp;
use crate::app::messages::Message;
use crate::core::PreviewData;
use crate::parsers::markdown::Block;
use iced::Task;
use iced::widget::operation;

fn markdown_block_y_offset(
    blocks: &[Block],
    target_index: usize,
    font_size: f32,
    image_sizes: &std::collections::HashMap<usize, (u32, u32)>,
) -> f32 {
    let mut y: f32 = 15.0;
    for (i, block) in blocks.iter().enumerate() {
        if i == target_index {
            return y;
        }
        y += crate::parsers::markdown::estimated_block_height(block, font_size, i, image_sizes);
    }
    0.0
}

pub fn handle_toc_toggled(app: &mut KglanceApp) -> Task<Message> {
    app.state.markdown.toc_visible = !app.state.markdown.toc_visible;
    Task::none()
}

pub fn handle_toc_toggle_collapse(app: &mut KglanceApp, idx: usize) -> Task<Message> {
    if app.state.markdown.collapsed_headings.contains(&idx) {
        app.state.markdown.collapsed_headings.remove(&idx);
    } else {
        app.state.markdown.collapsed_headings.insert(idx);
    }
    Task::none()
}

pub fn handle_toc_heading_clicked(app: &mut KglanceApp, idx: usize) -> Task<Message> {
    let y = app
        .state
        .markdown
        .toc
        .iter()
        .find(|e| e.block_index == idx)
        .map(|e| e.y_offset)
        .unwrap_or(0.0);
    operation::scroll_to("content_scroll", operation::AbsoluteOffset { x: 0.0, y })
}

pub fn handle_markdown_scrolled(app: &mut KglanceApp, y: f32) -> Task<Message> {
    app.state.markdown.scroll_y = y;
    let toc = &app.state.markdown.toc;
    if let Some(active_pos) = toc.iter().rposition(|e| e.y_offset <= y + 50.0) {
        let target_y = (active_pos as f32 * 28.0 - 100.0).max(0.0);
        operation::scroll_to(
            "toc_scroll",
            operation::AbsoluteOffset {
                x: 0.0,
                y: target_y,
            },
        )
    } else {
        Task::none()
    }
}

pub fn handle_search_toggle(app: &mut KglanceApp) -> Task<Message> {
    let s = &mut app.state.markdown;
    s.search_visible = !s.search_visible;
    if !s.search_visible {
        s.search_query.clear();
        s.search_match_count = 0;
        s.search_match_index = 0;
        s.search_match_blocks.clear();
        s.search_info.clear();
        Task::none()
    } else {
        operation::focus("md_search_input")
    }
}

pub fn handle_search_query_changed(app: &mut KglanceApp, query: String) -> Task<Message> {
    let s = &mut app.state.markdown;
    s.search_query = query.clone();
    s.search_match_index = 0;
    if query.is_empty() {
        s.search_match_count = 0;
        s.search_match_blocks.clear();
        s.search_info.clear();
    } else if let Some(PreviewData::Markdown { blocks }) = &app.current_content {
        let q = query.to_lowercase();
        let mut count = 0;
        let mut match_blocks = Vec::new();
        for (bi, block) in blocks.iter().enumerate() {
            let text = match block {
                Block::Heading { content, .. } | Block::Paragraph(content) => {
                    crate::parsers::markdown::flatten_inlines(content)
                }
                Block::CodeBlock { code, .. } => code.clone(),
                Block::Quote(inner) => inner
                    .iter()
                    .map(|ib| match ib {
                        Block::Heading { content, .. } | Block::Paragraph(content) => {
                            crate::parsers::markdown::flatten_inlines(content)
                        }
                        _ => String::new(),
                    })
                    .collect::<Vec<_>>()
                    .join(" "),
                Block::List { items, .. } => items
                    .iter()
                    .flat_map(|item| {
                        let own = crate::parsers::markdown::flatten_inlines(&item.content);
                        let sub: String = item
                            .sub_blocks
                            .iter()
                            .map(|lb| match lb {
                                Block::Heading { content, .. } | Block::Paragraph(content) => {
                                    crate::parsers::markdown::flatten_inlines(content)
                                }
                                _ => String::new(),
                            })
                            .collect::<Vec<_>>()
                            .join(" ");
                        vec![own, sub]
                    })
                    .collect::<Vec<_>>()
                    .join(" "),
                Block::Table(tbl) => tbl
                    .rows
                    .iter()
                    .flat_map(|r| r.iter())
                    .map(|cell| crate::parsers::markdown::flatten_inlines(&cell.content))
                    .collect::<Vec<_>>()
                    .join(" "),
                _ => String::new(),
            };
            let n = text.to_lowercase().matches(&q).count();
            for _ in 0..n {
                match_blocks.push(bi);
            }
            count += n;
        }
        s.search_match_count = count;
        s.search_match_blocks = match_blocks;
        s.search_info = if count > 0 {
            format!("1/{}", count)
        } else {
            String::new()
        };
    }
    Task::none()
}

pub fn handle_search_next(app: &mut KglanceApp) -> Task<Message> {
    let s = &mut app.state.markdown;
    if s.search_match_count > 0 {
        s.search_match_index = (s.search_match_index + 1) % s.search_match_count;
        s.search_info = format!("{}/{}", s.search_match_index + 1, s.search_match_count);
        let block_idx = s.search_match_blocks[s.search_match_index];
        if let Some(PreviewData::Markdown { blocks }) = &app.current_content {
            let y = markdown_block_y_offset(
                blocks,
                block_idx,
                app.state.font_size,
                &app.state.markdown.cached_image_sizes,
            );
            return operation::scroll_to("content_scroll", operation::AbsoluteOffset { x: 0.0, y });
        }
    }
    Task::none()
}

pub fn handle_search_prev(app: &mut KglanceApp) -> Task<Message> {
    let s = &mut app.state.markdown;
    if s.search_match_count > 0 {
        s.search_match_index = if s.search_match_index == 0 {
            s.search_match_count - 1
        } else {
            s.search_match_index - 1
        };
        s.search_info = format!("{}/{}", s.search_match_index + 1, s.search_match_count);
        let block_idx = s.search_match_blocks[s.search_match_index];
        if let Some(PreviewData::Markdown { blocks }) = &app.current_content {
            let y = markdown_block_y_offset(
                blocks,
                block_idx,
                app.state.font_size,
                &app.state.markdown.cached_image_sizes,
            );
            return operation::scroll_to("content_scroll", operation::AbsoluteOffset { x: 0.0, y });
        }
    }
    Task::none()
}

pub fn handle_search_closed(app: &mut KglanceApp) -> Task<Message> {
    let s = &mut app.state.markdown;
    s.search_visible = false;
    s.search_query.clear();
    s.search_match_count = 0;
    s.search_match_index = 0;
    s.search_match_blocks.clear();
    s.search_info.clear();
    Task::none()
}
