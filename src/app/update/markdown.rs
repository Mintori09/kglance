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
    } else if let Some(PreviewData::Markdown { blocks, .. }) = &app.current_content {
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
        if let Some(PreviewData::Markdown { blocks, .. }) = &app.current_content {
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
        if let Some(PreviewData::Markdown { blocks, .. }) = &app.current_content {
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

pub fn handle_selection_changed(app: &mut KglanceApp, selected: Option<String>) -> Task<Message> {
    app.state.markdown.selected_text = selected;
    Task::none()
}

pub fn handle_selection_drag_start(
    app: &mut KglanceApp,
    block: usize,
    offset: usize,
) -> Task<Message> {
    let start_pt = crate::core::SelectionPoint { block, offset };
    app.state.markdown.selection_range = Some(crate::core::SelectionRange {
        start: start_pt,
        end: start_pt,
    });
    app.state.markdown.is_dragging_selection = true;
    app.state.markdown.selected_text = None;
    Task::none()
}

pub fn handle_selection_drag_update(
    app: &mut KglanceApp,
    block: usize,
    offset: usize,
) -> Task<Message> {
    let pt = crate::core::SelectionPoint { block, offset };
    if app.state.markdown.selection_range.is_none() {
        app.state.markdown.selection_range =
            Some(crate::core::SelectionRange { start: pt, end: pt });
        app.state.markdown.is_dragging_selection = true;
    } else if let Some(range) = &mut app.state.markdown.selection_range {
        range.end = pt;
        update_selected_text_from_range(app);
    }
    Task::none()
}

pub fn handle_selection_drag_end(app: &mut KglanceApp) -> Task<Message> {
    app.state.markdown.is_dragging_selection = false;
    app.state.markdown.auto_scroll_delta = None;
    update_selected_text_from_range(app);
    Task::none()
}

pub fn handle_selection_clear(app: &mut KglanceApp) -> Task<Message> {
    app.state.markdown.selection_range = None;
    app.state.markdown.is_dragging_selection = false;
    app.state.markdown.auto_scroll_delta = None;
    app.state.markdown.selected_text = None;
    Task::none()
}

pub fn handle_select_all(app: &mut KglanceApp) -> Task<Message> {
    let Some(PreviewData::Markdown { blocks, .. }) = &app.current_content else {
        return Task::none();
    };
    if blocks.is_empty() {
        return Task::none();
    }
    let lines = collect_indexed_lines(blocks);
    let Some((&last_block, _)) = lines.last_key_value() else {
        return Task::none();
    };
    let range = crate::core::SelectionRange {
        start: crate::core::SelectionPoint {
            block: 0,
            offset: 0,
        },
        end: crate::core::SelectionPoint {
            block: last_block,
            offset: 999_999,
        },
    };
    app.state.markdown.selection_range = Some(range);
    app.state.markdown.is_dragging_selection = false;
    app.state.markdown.auto_scroll_delta = None;
    app.state.markdown.selected_text = build_selected_text(blocks, range);
    Task::none()
}

pub fn handle_auto_scroll_tick(app: &mut KglanceApp) -> Task<Message> {
    let Some(delta) = app.state.markdown.auto_scroll_delta else {
        return Task::none();
    };

    let new_y = (app.state.markdown.scroll_y + delta).max(0.0);
    app.state.markdown.scroll_y = new_y;

    if let Some(PreviewData::Markdown { blocks, .. }) = &app.current_content {
        let total_blocks = blocks.len();
        if total_blocks > 0 {
            let target_block = if delta > 0.0 {
                (total_blocks - 1) * 1000
            } else {
                0
            };
            let target_offset = if delta > 0.0 { 999999 } else { 0 };

            if let Some(range) = &mut app.state.markdown.selection_range {
                range.end = crate::core::SelectionPoint {
                    block: target_block,
                    offset: target_offset,
                };
                update_selected_text_from_range(app);
            }
        }
    }

    iced::widget::operation::scroll_to(
        "content_scroll",
        iced::widget::operation::AbsoluteOffset { x: 0.0, y: new_y },
    )
}

fn update_selected_text_from_range(app: &mut KglanceApp) {
    let Some(range) = app.state.markdown.selection_range else {
        app.state.markdown.selected_text = None;
        return;
    };
    let blocks_vec: Vec<Block> = match &app.current_content {
        Some(PreviewData::Markdown { blocks, .. }) => blocks.clone(),
        Some(PreviewData::Epub { chapters, .. }) => {
            chapters.iter().flat_map(|ch| ch.blocks.clone()).collect()
        }
        _ => {
            app.state.markdown.selected_text = None;
            return;
        }
    };
    app.state.markdown.selected_text = build_selected_text(&blocks_vec, range);
}

struct CopyLine {
    prefix: Option<String>,
    text: String,
    suffix: Option<String>,
    blank_before: bool,
    table_separator: Option<String>,
}

fn table_separator_line(num_cols: usize) -> String {
    let mut s = String::with_capacity(num_cols * 4 + 2);
    s.push('|');
    for _ in 0..num_cols.max(1) {
        s.push_str("---|");
    }
    s
}

fn char_boundary(text: &str, idx: usize) -> usize {
    let mut i = idx.min(text.len());
    while i > 0 && !text.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn collect_indexed_lines(blocks: &[Block]) -> std::collections::BTreeMap<usize, CopyLine> {
    let mut indexed_blocks: std::collections::BTreeMap<usize, CopyLine> =
        std::collections::BTreeMap::new();
    for (i, block) in blocks.iter().enumerate() {
        collect_indexed_plain_texts(index_block_base(i), block, &mut indexed_blocks);
    }
    indexed_blocks
}

fn build_selected_text(blocks: &[Block], range: crate::core::SelectionRange) -> Option<String> {
    let (start_pt, end_pt) =
        if (range.start.block, range.start.offset) <= (range.end.block, range.end.offset) {
            (range.start, range.end)
        } else {
            (range.end, range.start)
        };

    let indexed_blocks = collect_indexed_lines(blocks);

    let mut result = String::new();
    let mut first_line = true;
    let mut prev_blk_idx: Option<usize> = None;
    for (&blk_idx, line) in &indexed_blocks {
        if blk_idx < start_pt.block || blk_idx > end_pt.block {
            continue;
        }

        let is_start = blk_idx == start_pt.block;
        let is_end = blk_idx == end_pt.block;

        let text = &line.text;
        let (include_prefix, include_suffix, slice_start, slice_end) = if is_start && is_end {
            let start = char_boundary(text, start_pt.offset);
            let end = char_boundary(text, end_pt.offset);
            (start == 0, end >= text.len(), start, end)
        } else if is_start {
            let start = char_boundary(text, start_pt.offset);
            (start == 0, true, start, text.len())
        } else if is_end {
            let end = char_boundary(text, end_pt.offset);
            (true, end >= text.len(), 0, end)
        } else {
            (true, true, 0, text.len())
        };

        let slice = if slice_start < slice_end {
            &text[slice_start..slice_end]
        } else {
            ""
        };

        let is_table_cell_continuation = match prev_blk_idx {
            Some(prev) => {
                (blk_idx == prev + 1)
                    && (blk_idx % 1000 != 0)
                    && line.prefix.as_deref() == Some(" | ")
            }
            None => false,
        };

        if !first_line {
            if line.blank_before {
                result.push_str("\n\n");
            } else if !is_table_cell_continuation {
                result.push('\n');
            }
        }

        if let Some(separator) = &line.table_separator
            && prev_blk_idx == Some(blk_idx - 1)
        {
            result.push_str(separator);
            result.push('\n');
        }

        if include_prefix && let Some(p) = &line.prefix {
            result.push_str(p);
        }
        result.push_str(slice);
        if include_suffix && let Some(s) = &line.suffix {
            result.push_str(s);
        }
        first_line = false;
        prev_blk_idx = Some(blk_idx);
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

fn index_block_base(index: usize) -> usize {
    index * 1000
}

fn is_special_inline(inline: &crate::parsers::markdown::Inline) -> bool {
    matches!(
        inline,
        crate::parsers::markdown::Inline::Link { .. }
            | crate::parsers::markdown::Inline::InlineMath(_)
            | crate::parsers::markdown::Inline::DisplayMath(_)
    )
}

fn collect_indexed_plain_texts(
    base_idx: usize,
    block: &crate::parsers::markdown::Block,
    map: &mut std::collections::BTreeMap<usize, CopyLine>,
) {
    match block {
        crate::parsers::markdown::Block::Heading { level, content } => {
            let hashes = "#".repeat((*level as usize).min(6));
            let text = crate::parsers::markdown::flatten_inlines_visual(content);
            map.insert(
                base_idx,
                CopyLine {
                    prefix: Some(format!("{hashes} ")),
                    text,
                    suffix: None,
                    blank_before: false,
                    table_separator: None,
                },
            );
        }
        crate::parsers::markdown::Block::Paragraph(content) => {
            collect_paragraph_segments(base_idx, content, map);
        }
        crate::parsers::markdown::Block::CodeBlock { lang, code, .. } => {
            let lang_str = lang.as_deref().unwrap_or("");
            map.insert(
                base_idx,
                CopyLine {
                    prefix: None,
                    text: format!("```{lang_str}"),
                    suffix: None,
                    blank_before: false,
                    table_separator: None,
                },
            );
            let mut line_count = 0;
            for (line_idx, line) in code.split('\n').enumerate() {
                map.insert(
                    base_idx + line_idx + 1,
                    CopyLine {
                        prefix: None,
                        text: line.to_string(),
                        suffix: None,
                        blank_before: false,
                        table_separator: None,
                    },
                );
                line_count = line_idx + 1;
            }
            map.insert(
                base_idx + line_count + 1,
                CopyLine {
                    prefix: None,
                    text: "```".to_string(),
                    suffix: None,
                    blank_before: false,
                    table_separator: None,
                },
            );
        }
        crate::parsers::markdown::Block::List {
            ordered,
            start_number,
            items,
        } => {
            collect_list_plain_texts(base_idx, *ordered, *start_number, items, map, 0);
        }
        crate::parsers::markdown::Block::Quote(sub_blocks) => {
            collect_quote_plain_texts(base_idx, sub_blocks, map, "> ");
        }
        crate::parsers::markdown::Block::Mermaid { lines, .. } => {
            map.insert(
                base_idx,
                CopyLine {
                    prefix: None,
                    text: "```mermaid".to_string(),
                    suffix: None,
                    blank_before: false,
                    table_separator: None,
                },
            );
            for (line_idx, line) in lines.iter().enumerate() {
                map.insert(
                    base_idx + line_idx + 1,
                    CopyLine {
                        prefix: None,
                        text: line.clone(),
                        suffix: None,
                        blank_before: false,
                        table_separator: None,
                    },
                );
            }
            map.insert(
                base_idx + lines.len() + 1,
                CopyLine {
                    prefix: None,
                    text: "```".to_string(),
                    suffix: None,
                    blank_before: false,
                    table_separator: None,
                },
            );
        }
        crate::parsers::markdown::Block::Table(table) => {
            let col_count = if table.headers.is_empty() {
                table.rows.first().map_or(1, |r| r.len())
            } else {
                table.headers.len()
            };
            let num_cols = if col_count == 0 { 1 } else { col_count };

            for (i, header) in table.headers.iter().enumerate() {
                let text = crate::parsers::markdown::flatten_inlines_visual(&header.content);
                let prefix = if i == 0 { "| " } else { " | " };
                let suffix = if i + 1 == table.headers.len() {
                    Some(" |".to_string())
                } else {
                    None
                };
                map.insert(
                    base_idx + i + 1,
                    CopyLine {
                        prefix: Some(prefix.to_string()),
                        text,
                        suffix,
                        blank_before: false,
                        table_separator: None,
                    },
                );
            }

            for (row_idx, row_data) in table.rows.iter().enumerate() {
                let num_cells = row_data.len();
                for (j, cell) in row_data.iter().enumerate() {
                    let text = crate::parsers::markdown::flatten_inlines_visual(&cell.content);
                    let prefix = if j == 0 { "| " } else { " | " };
                    let suffix = if j + 1 == num_cells {
                        Some(" |".to_string())
                    } else {
                        None
                    };
                    let separator = if row_idx == 0 && j == 0 {
                        Some(table_separator_line(num_cols))
                    } else {
                        None
                    };
                    map.insert(
                        base_idx + num_cols + row_idx * num_cols + j + 1,
                        CopyLine {
                            prefix: Some(prefix.to_string()),
                            text,
                            suffix,
                            blank_before: false,
                            table_separator: separator,
                        },
                    );
                }
            }
        }
        _ => {}
    }
}

fn collect_paragraph_segments(
    base_idx: usize,
    inlines: &[crate::parsers::markdown::Inline],
    map: &mut std::collections::BTreeMap<usize, CopyLine>,
) {
    let has_special = inlines.iter().any(is_special_inline);
    if !has_special {
        insert_segment_line(base_idx, inlines, map);
        return;
    }

    let mut start = 0;
    for (i, inline) in inlines.iter().enumerate() {
        if !is_special_inline(inline) {
            continue;
        }
        insert_segment_line(base_idx + i + 1, &inlines[start..i], map);
        start = i + 1;
    }
    insert_segment_line(base_idx + inlines.len() + 1, &inlines[start..], map);
}

fn insert_segment_line(
    blk_idx: usize,
    inlines: &[crate::parsers::markdown::Inline],
    map: &mut std::collections::BTreeMap<usize, CopyLine>,
) {
    if inlines.is_empty() {
        return;
    }
    let text = crate::parsers::markdown::flatten_inlines_visual(inlines);
    if text.is_empty() {
        return;
    }
    map.insert(
        blk_idx,
        CopyLine {
            prefix: None,
            text,
            suffix: None,
            blank_before: false,
            table_separator: None,
        },
    );
}

fn collect_list_plain_texts(
    base_idx: usize,
    ordered: bool,
    start_number: u64,
    items: &[crate::parsers::markdown::ListItem],
    map: &mut std::collections::BTreeMap<usize, CopyLine>,
    depth: usize,
) {
    let indent = "  ".repeat(depth);
    let mut current_idx = base_idx + 1;
    for (idx, item) in items.iter().enumerate() {
        let item_blk = current_idx;
        current_idx += 1;

        let raw_prefix = if let Some(checked) = item.is_task {
            if checked {
                "[x] ".to_string()
            } else {
                "[ ] ".to_string()
            }
        } else if ordered {
            format!("{}. ", start_number + idx as u64)
        } else {
            "- ".to_string()
        };
        let prefix = format!("{indent}{raw_prefix}");
        let text = crate::parsers::markdown::flatten_inlines_visual(&item.content);
        map.insert(
            item_blk,
            CopyLine {
                prefix: Some(prefix),
                text,
                suffix: None,
                blank_before: false,
                table_separator: None,
            },
        );

        for sub_block in item.sub_blocks.iter() {
            match sub_block {
                crate::parsers::markdown::Block::List {
                    ordered: sub_ord,
                    start_number: sub_start,
                    items: sub_items,
                } => {
                    collect_list_plain_texts(
                        current_idx,
                        *sub_ord,
                        *sub_start,
                        sub_items,
                        map,
                        depth + 1,
                    );
                }
                _ => {
                    collect_indexed_plain_texts(current_idx, sub_block, map);
                }
            }
            current_idx += 10;
        }
    }
}

fn collect_quote_plain_texts(
    base_idx: usize,
    sub_blocks: &[crate::parsers::markdown::Block],
    map: &mut std::collections::BTreeMap<usize, CopyLine>,
    quote_prefix: &str,
) {
    for (i, sub_block) in sub_blocks.iter().enumerate() {
        let sub_base = base_idx + i + 1;
        let before = map.len();

        if let crate::parsers::markdown::Block::Quote(inner_blocks) = sub_block {
            let inner_prefix = format!("{quote_prefix}> ");
            collect_quote_plain_texts(sub_base, inner_blocks, map, &inner_prefix);
        } else {
            collect_indexed_plain_texts(sub_base, sub_block, map);
        }

        for k in map.keys().skip(before).copied().collect::<Vec<_>>() {
            if let Some(line) = map.get_mut(&k) {
                line.prefix = Some(match line.prefix.take() {
                    Some(p) => format!("{quote_prefix}{p}"),
                    None => quote_prefix.to_string(),
                });
            }
        }

        if i > 0
            && map.len() > before
            && let Some(&min_key) = map.keys().nth(before)
            && let Some(line) = map.get_mut(&min_key)
        {
            line.blank_before = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{build_selected_text, char_boundary};
    use crate::core::{SelectionPoint, SelectionRange};
    use crate::parsers::markdown::parse_to_blocks;

    fn range(start: usize, start_off: usize, end: usize, end_off: usize) -> SelectionRange {
        SelectionRange {
            start: SelectionPoint {
                block: start,
                offset: start_off,
            },
            end: SelectionPoint {
                block: end,
                offset: end_off,
            },
        }
    }

    #[test]
    fn selected_vietnamese_paragraph_full() {
        let md = "Xin chào, đây là đoạn tiếng Việt.";
        let blocks = parse_to_blocks(md);
        assert_eq!(blocks.len(), 1);
        let sel = build_selected_text(&blocks, range(0, 0, 0, md.len()));
        assert_eq!(sel.as_deref(), Some(md));
    }

    #[test]
    fn selected_vietnamese_paragraph_partial() {
        let md = "Xin chào bạn";
        let blocks = parse_to_blocks(md);
        let start = md.find("chào").unwrap();
        let end = start + "chào".len();
        let sel = build_selected_text(&blocks, range(0, start, 0, end));
        assert_eq!(sel.as_deref(), Some("chào"));
    }

    #[test]
    fn selected_mid_char_offset_does_not_panic() {
        let md = "xin chào";
        let blocks = parse_to_blocks(md);

        let mid = md.find('à').unwrap() + 1;
        assert!(!md.is_char_boundary(mid));
        let sel = build_selected_text(&blocks, range(0, mid, 0, mid));
        assert_eq!(sel, None);

        let end_mid = char_boundary(md, md.len() + 5);
        assert_eq!(end_mid, md.len());
    }

    #[test]
    fn selected_two_paragraphs() {
        let md = "dòng một\n\ndòng hai";
        let blocks = parse_to_blocks(md);
        assert_eq!(blocks.len(), 2);
        let sel = build_selected_text(&blocks, range(0, 0, 1000, "dòng hai".len()));
        assert_eq!(sel.as_deref(), Some("dòng một\ndòng hai"));
    }

    #[test]
    fn selected_upward_drag_across_paragraphs_full() {
        let md = "hello\n\nworld";
        let blocks = parse_to_blocks(md);
        assert_eq!(blocks.len(), 2);
        let sel = build_selected_text(&blocks, range(1000, 5, 0, 0));
        assert_eq!(sel.as_deref(), Some("hello\nworld"));
    }

    #[test]
    fn selected_upward_drag_across_paragraphs_partial() {
        let md = "hello\n\nworld";
        let blocks = parse_to_blocks(md);
        assert_eq!(blocks.len(), 2);
        let sel = build_selected_text(&blocks, range(1000, 5, 0, 2));
        assert_eq!(sel.as_deref(), Some("llo\nworld"));
    }

    #[test]
    fn selected_all_markdown() {
        let md = "# Tiêu đề\n\nĐoạn một với **đậm**.\n\n- mục 1\n- mục 2\n\n> quote";
        let blocks = parse_to_blocks(md);
        assert!(blocks.len() > 1);
        let lines = super::collect_indexed_lines(&blocks);
        let last_block = *lines.last_key_value().expect("has lines").0;
        let sel = build_selected_text(
            &blocks,
            SelectionRange {
                start: SelectionPoint {
                    block: 0,
                    offset: 0,
                },
                end: SelectionPoint {
                    block: last_block,
                    offset: 999_999,
                },
            },
        );
        let text = sel.expect("select-all must produce text");
        assert!(!text.contains("**"));
        assert!(!text.contains('`'));
        assert!(text.starts_with("# Tiêu đề"));
        assert!(text.contains("Đoạn một với đậm."));
        assert!(text.contains("- mục 1"));
        assert!(text.contains("- mục 2"));
        assert!(text.contains("> quote"));
    }

    #[test]
    fn selected_list_items() {
        let md = "- item một\n- item hai";
        let blocks = parse_to_blocks(md);
        assert_eq!(blocks.len(), 1);
        let sel = build_selected_text(&blocks, range(1, 0, 2, "item hai".len()));
        assert_eq!(sel.as_deref(), Some("- item một\n- item hai"));
    }

    #[test]
    fn selected_quote_two_paragraphs() {
        let md = "> đoạn một\n>\n> đoạn hai";
        let blocks = parse_to_blocks(md);
        assert_eq!(blocks.len(), 1);
        let sel = build_selected_text(&blocks, range(1, 0, 2, "đoạn hai".len()));
        assert_eq!(sel.as_deref(), Some("> đoạn một\n\n> đoạn hai"));
    }

    #[test]
    fn selected_table_cell_and_row() {
        let md = "| H1 | H2 |\n| --- | --- |\n| A | B |";
        let blocks = parse_to_blocks(md);
        assert_eq!(blocks.len(), 1);

        let cell = build_selected_text(&blocks, range(3, 0, 3, 1));
        assert_eq!(cell.as_deref(), Some("| A"));

        let last_cell = build_selected_text(&blocks, range(4, 0, 4, 1));
        assert_eq!(last_cell.as_deref(), Some(" | B |"));

        let full_row = build_selected_text(&blocks, range(3, 0, 4, 1));
        assert_eq!(full_row.as_deref(), Some("| A | B |"));

        let header_and_body = build_selected_text(&blocks, range(1, 0, 4, 1));
        assert_eq!(
            header_and_body.as_deref(),
            Some("| H1 | H2 |\n|---|---|\n| A | B |")
        );

        let row = build_selected_text(&blocks, range(1, 0, 2, 2));
        assert_eq!(row.as_deref(), Some("| H1 | H2 |"));
    }

    #[test]
    fn selected_heading() {
        let md = "# Tiêu đề";
        let blocks = parse_to_blocks(md);
        let sel = build_selected_text(&blocks, range(0, 0, 0, "Tiêu đề".len()));
        assert_eq!(sel.as_deref(), Some("# Tiêu đề"));
    }

    #[test]
    fn selected_inline_formatting_is_plain() {
        let md = "**đậm** chữ `code`";
        let blocks = parse_to_blocks(md);
        assert_eq!(blocks.len(), 1);
        let full = "đậm chữ code";
        let sel = build_selected_text(&blocks, range(0, 0, 0, full.len()));
        assert_eq!(sel.as_deref(), Some(full));
    }

    #[test]
    fn selected_image_renders_as_alt_text() {
        let md = "x ![alt](u) y";
        let blocks = parse_to_blocks(md);
        assert_eq!(blocks.len(), 1);
        let full = "x [alt] y";
        let sel = build_selected_text(&blocks, range(0, 0, 0, full.len()));
        assert_eq!(sel.as_deref(), Some(full));
    }

    #[test]
    fn selected_paragraph_with_link_segments() {
        let md = "a [link](u) b";
        let blocks = parse_to_blocks(md);
        assert_eq!(blocks.len(), 1);

        let sel = build_selected_text(&blocks, range(2, 0, 4, 2));
        assert_eq!(sel.as_deref(), Some("a \n b"));
    }
}
