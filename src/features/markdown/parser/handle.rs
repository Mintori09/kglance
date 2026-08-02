use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Parser, Tag, TagEnd};
use std::path::Path;

use crate::features::image::types::ImageRef;

use super::{Block, Inline, ListItem, TableBlock, TableCell};

pub(super) fn extract_images(raw: &str, parent: &Path) -> Vec<ImageRef> {
    let mut images = Vec::new();
    let mut in_image = false;
    let mut image_alt = String::new();
    let mut image_url = String::new();

    for event in Parser::new_ext(raw, pulldown_cmark::Options::all()) {
        match event {
            Event::Start(Tag::Image { dest_url, .. }) => {
                in_image = true;
                image_url = dest_url.to_string();
                image_alt.clear();
            }
            Event::End(TagEnd::Image) => {
                in_image = false;
                let resolved = if image_url.starts_with('/') {
                    image_url.clone()
                } else {
                    parent.join(&image_url).to_string_lossy().to_string()
                };
                images.push(ImageRef {
                    alt_text: image_alt.clone(),
                    path: resolved,
                });
            }
            Event::Text(text) if in_image => {
                image_alt.push_str(&text);
            }
            _ => {}
        }
    }
    images
}

struct EventStream<'a> {
    iter: std::iter::Peekable<Parser<'a>>,
}

impl<'a> EventStream<'a> {
    fn new(content: &'a str) -> Self {
        Self {
            iter: Parser::new_ext(content, pulldown_cmark::Options::all()).peekable(),
        }
    }

    fn parse_inlines(&mut self) -> Vec<Inline> {
        let mut result = Vec::new();
        loop {
            match self.iter.peek() {
                None => break,
                Some(Event::End(_)) => break,
                _ => {}
            }
            match self.iter.next().unwrap() {
                Event::Text(t) => result.push(Inline::Text(t.to_string())),
                Event::Code(t) => result.push(Inline::Code(t.to_string())),
                Event::SoftBreak => {
                    result.push(Inline::Text(" ".to_string()));
                }
                Event::HardBreak => {
                    result.push(Inline::SoftBreak);
                }
                Event::Start(Tag::Strong) => {
                    let content = self.parse_inlines();
                    let _ = self.iter.next();
                    result.push(Inline::Bold(content));
                }
                Event::Start(Tag::Emphasis) => {
                    let content = self.parse_inlines();
                    let _ = self.iter.next();
                    result.push(Inline::Italic(content));
                }
                Event::Start(Tag::Strikethrough) => {
                    let content = self.parse_inlines();
                    let _ = self.iter.next();
                    result.push(Inline::Strikethrough(content));
                }
                Event::Start(Tag::Link { dest_url, .. }) => {
                    let url = dest_url.to_string();
                    let text = self.parse_inlines();
                    let _ = self.iter.next();
                    result.push(Inline::Link { text, url });
                }
                Event::Start(Tag::Image { dest_url, .. }) => {
                    let url = dest_url.to_string();
                    let mut alt = String::new();
                    loop {
                        match self.iter.peek() {
                            Some(Event::End(TagEnd::Image)) => break,
                            Some(Event::Text(t)) => alt.push_str(t),
                            _ => {}
                        }
                        self.iter.next();
                    }
                    let _ = self.iter.next();
                    result.push(Inline::Image { alt, url });
                }
                Event::InlineMath(t) => result.push(Inline::InlineMath(t.to_string())),
                Event::DisplayMath(t) => result.push(Inline::DisplayMath(t.to_string())),
                _ => {
                    self.iter.next();
                }
            }
        }
        result
    }

    fn parse_blocks(&mut self) -> Vec<Block> {
        let mut blocks = Vec::new();
        loop {
            match self.iter.next() {
                None => break,
                Some(Event::Start(Tag::Table(alignments))) => {
                    if let Some(table) = self.parse_table(alignments) {
                        blocks.push(Block::Table(table));
                    }
                }
                Some(event) => {
                    if let Some(block) = self.parse_one_block(event) {
                        blocks.push(block);
                    }
                }
            }
        }
        blocks
    }

    fn parse_blocks_until(&mut self, end: TagEnd) -> Vec<Block> {
        let mut blocks = Vec::new();
        loop {
            match self.iter.peek() {
                None => break,
                Some(Event::End(tag)) if *tag == end => {
                    self.iter.next();
                    break;
                }
                _ => {}
            }
            match self.iter.next() {
                None => break,
                Some(event) => {
                    if let Some(block) = self.parse_one_block(event) {
                        blocks.push(block);
                    }
                }
            }
        }
        blocks
    }

    fn parse_one_block(&mut self, event: Event) -> Option<Block> {
        match event {
            Event::Start(Tag::Paragraph) => {
                let content = self.parse_inlines();
                let _ = self.iter.next();
                Some(Block::Paragraph(content))
            }
            Event::Start(Tag::Heading { level, .. }) => {
                let lvl = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                let content = self.parse_inlines();
                let _ = self.iter.next();
                Some(Block::Heading {
                    level: lvl,
                    content,
                })
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let block = self.parse_code_block(kind);
                Some(block)
            }
            Event::Start(Tag::List(first_num)) => {
                let ordered = first_num.is_some();
                let start_number = first_num.unwrap_or(1);
                let items = self.parse_list_items();
                Some(Block::List {
                    ordered,
                    start_number,
                    items,
                })
            }
            Event::Start(Tag::BlockQuote(kind)) => {
                let quotes = self.parse_blocks_until(TagEnd::BlockQuote(kind));
                Some(Block::Quote(quotes))
            }
            Event::Rule => Some(Block::HorizontalRule),
            Event::Html(text) | Event::InlineHtml(text) => Some(Block::Html(text.to_string())),
            _ => None,
        }
    }

    fn parse_code_block(&mut self, kind: CodeBlockKind) -> Block {
        let (lang, title) = match kind {
            CodeBlockKind::Fenced(info) => {
                let raw = info.trim();
                let parts: Vec<&str> = raw.splitn(2, ' ').collect();
                let lang = if parts[0].is_empty() {
                    None
                } else {
                    Some(parts[0].to_string())
                };
                let title = parts.get(1).and_then(|s| {
                    let s = s.trim();
                    if let Some(rest) = s.strip_prefix("title=\"") {
                        rest.strip_suffix('"').map(|t| t.to_string())
                    } else if !s.is_empty() {
                        Some(s.to_string())
                    } else {
                        None
                    }
                });
                (lang, title)
            }
            CodeBlockKind::Indented => (None, None),
        };

        let mut code = String::new();
        let mut base_indent = None;

        loop {
            match self.iter.peek() {
                Some(Event::End(TagEnd::CodeBlock)) => {
                    self.iter.next();
                    break;
                }
                Some(_) => match self.iter.next() {
                    Some(Event::Text(t)) => {
                        let mut first = true;
                        for line in t.lines() {
                            if !first {
                                code.push('\n');
                            }
                            first = false;
                            let current_base = *base_indent
                                .get_or_insert_with(|| line.len() - line.trim_start().len());
                            let trimmed_line = line
                                .char_indices()
                                .nth(current_base)
                                .map(|(idx, _)| &line[idx..])
                                .unwrap_or(line.trim_start());
                            code.push_str(trimmed_line);
                        }
                        if t.ends_with('\n') {
                            code.push('\n');
                            base_indent = None;
                        }
                    }
                    Some(Event::SoftBreak) | Some(Event::HardBreak) => {
                        base_indent = None;
                        code.push('\n');
                    }
                    _ => {}
                },
                None => break,
            }
        }

        if lang.as_deref() == Some("mermaid") {
            Block::Mermaid {
                lines: code.lines().map(|l| l.trim().to_string()).collect(),
                rendered: None,
            }
        } else {
            Block::CodeBlock { lang, title, code }
        }
    }

    fn parse_list_items(&mut self) -> Vec<ListItem> {
        let mut items = Vec::new();
        loop {
            match self.iter.peek() {
                Some(Event::End(TagEnd::List(_))) => {
                    self.iter.next();
                    break;
                }
                Some(Event::Start(Tag::Item)) => {
                    self.iter.next();
                    let item = self.parse_single_item();
                    items.push(item);
                }
                _ => break,
            }
        }
        items
    }

    fn parse_single_item(&mut self) -> ListItem {
        let mut is_task = None;
        let mut content = Vec::new();
        let mut sub_blocks = Vec::new();
        loop {
            match self.iter.peek() {
                Some(Event::End(TagEnd::Item)) => {
                    self.iter.next();
                    break;
                }
                Some(Event::End(TagEnd::List(_))) => break,
                _ => {}
            }
            match self.iter.next() {
                Some(Event::TaskListMarker(checked)) => {
                    is_task = Some(checked);
                }
                Some(Event::Start(Tag::Paragraph)) => {
                    let mut inlines = self.parse_inlines();
                    let _ = self.iter.next();
                    content.append(&mut inlines);
                }
                Some(Event::Start(Tag::List(first_num))) => {
                    let ordered = first_num.is_some();
                    let start_number = first_num.unwrap_or(1);
                    let items = self.parse_list_items();
                    sub_blocks.push(Block::List {
                        ordered,
                        start_number,
                        items,
                    });
                }
                Some(Event::Text(t)) => content.push(Inline::Text(t.to_string())),
                Some(Event::Code(t)) => content.push(Inline::Code(t.to_string())),
                Some(Event::SoftBreak) => {
                    content.push(Inline::Text(" ".to_string()));
                }
                Some(Event::HardBreak) => {
                    content.push(Inline::SoftBreak);
                }
                Some(Event::Start(Tag::Strong)) => {
                    let c = self.parse_inlines();
                    let _ = self.iter.next();
                    content.push(Inline::Bold(c));
                }
                Some(Event::Start(Tag::Emphasis)) => {
                    let c = self.parse_inlines();
                    let _ = self.iter.next();
                    content.push(Inline::Italic(c));
                }
                Some(Event::Start(Tag::Strikethrough)) => {
                    let c = self.parse_inlines();
                    let _ = self.iter.next();
                    content.push(Inline::Strikethrough(c));
                }
                Some(Event::Start(Tag::Link { dest_url, .. })) => {
                    let url = dest_url.to_string();
                    let text = self.parse_inlines();
                    let _ = self.iter.next();
                    content.push(Inline::Link { text, url });
                }
                Some(Event::Start(Tag::Image { dest_url, .. })) => {
                    let url = dest_url.to_string();
                    let mut alt = String::new();
                    loop {
                        match self.iter.peek() {
                            Some(Event::End(TagEnd::Image)) => break,
                            Some(Event::Text(t)) => alt.push_str(t),
                            _ => {}
                        }
                        self.iter.next();
                    }
                    let _ = self.iter.next();
                    content.push(Inline::Image { alt, url });
                }
                Some(Event::Start(Tag::CodeBlock(kind))) => {
                    let block = self.parse_code_block(kind);
                    sub_blocks.push(block);
                }
                Some(Event::InlineMath(t)) => {
                    content.push(Inline::InlineMath(t.to_string()));
                }
                Some(Event::DisplayMath(t)) => {
                    content.push(Inline::DisplayMath(t.to_string()));
                }
                _ => {}
            }
        }
        ListItem {
            is_task,
            content,
            sub_blocks,
        }
    }

    fn parse_table(&mut self, _alignments: Vec<pulldown_cmark::Alignment>) -> Option<TableBlock> {
        let mut headers = Vec::new();
        let mut rows = Vec::new();

        loop {
            match self.iter.peek() {
                Some(Event::End(TagEnd::Table)) => {
                    self.iter.next();
                    break;
                }
                None => break,
                _ => {}
            }
            match self.iter.next()? {
                Event::Start(Tag::TableHead) => {
                    headers = self.parse_table_row_cells();
                }
                Event::Start(Tag::TableRow) => {
                    let row_cells = self.parse_table_row_cells();
                    if !row_cells.is_empty() {
                        rows.push(row_cells);
                    }
                }
                _ => {}
            }
        }

        Some(TableBlock { headers, rows })
    }

    fn parse_table_row_cells(&mut self) -> Vec<TableCell> {
        let mut cells = Vec::new();
        loop {
            match self.iter.peek() {
                Some(Event::End(TagEnd::TableHead)) | Some(Event::End(TagEnd::TableRow)) => {
                    self.iter.next();
                    break;
                }
                Some(Event::End(TagEnd::Table)) => break,
                _ => {}
            }
            if let Some(Event::Start(Tag::TableCell)) = self.iter.next() {
                let content = self.parse_inlines();
                let _ = self.iter.next();
                cells.push(TableCell { content });
            }
        }
        cells
    }
}

pub fn parse_to_blocks(content: &str) -> Vec<Block> {
    let mut stream = EventStream::new(content);
    stream.parse_blocks()
}
