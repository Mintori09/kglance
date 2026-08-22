use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Parser, Tag, TagEnd};
use std::path::Path;

use crate::{features::image::types::ImageRef, log_debug};

use super::{AlertKind, Block, Inline, ListItem, TableBlock, TableCell};

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
                log_debug!("{}", resolved.to_string().to_string());
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

fn split_inlines_by_display_math(inlines: Vec<Inline>) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut current = Vec::new();

    for inline in inlines {
        match inline {
            Inline::DisplayMath(latex) => {
                if !current.is_empty() {
                    let has_text = current.iter().any(|i| match i {
                        Inline::Text(t) => !t.trim().is_empty(),
                        Inline::SoftBreak => false,
                        _ => true,
                    });
                    if has_text {
                        blocks.push(Block::Paragraph(std::mem::take(&mut current)));
                    } else {
                        current.clear();
                    }
                }
                blocks.push(Block::Math(latex));
            }
            _ => {
                current.push(inline);
            }
        }
    }

    if !current.is_empty() {
        let has_text = current.iter().any(|i| match i {
            Inline::Text(t) => !t.trim().is_empty(),
            Inline::SoftBreak => false,
            _ => true,
        });
        if has_text {
            blocks.push(Block::Paragraph(current));
        }
    }

    blocks
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
                Event::FootnoteReference(label) => {
                    result.push(Inline::FootnoteReference(label.to_string()));
                }
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
                    blocks.extend(self.parse_event_blocks(event));
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
                    blocks.extend(self.parse_event_blocks(event));
                }
            }
        }
        blocks
    }

    fn parse_event_blocks(&mut self, event: Event) -> Vec<Block> {
        match event {
            Event::Start(Tag::Paragraph) => {
                let content = self.parse_inlines();
                let _ = self.iter.next();
                if content.len() == 1
                    && let Inline::Image { ref alt, ref url } = content[0]
                {
                    vec![Block::Image {
                        alt: alt.clone(),
                        path: url.clone(),
                    }]
                } else {
                    split_inlines_by_display_math(content)
                }
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
                vec![Block::Heading {
                    level: lvl,
                    content,
                }]
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let block = self.parse_code_block(kind);
                vec![block]
            }
            Event::Start(Tag::List(first_num)) => {
                let ordered = first_num.is_some();
                let start_number = first_num.unwrap_or(1);
                let items = self.parse_list_items();
                vec![Block::List {
                    ordered,
                    start_number,
                    items,
                }]
            }
            Event::Start(Tag::BlockQuote(kind)) => {
                let quotes = self.parse_blocks_until(TagEnd::BlockQuote(kind));
                let alert_opt = match kind {
                    Some(pulldown_cmark::BlockQuoteKind::Note) => {
                        Some((AlertKind::Note, quotes.clone()))
                    }
                    Some(pulldown_cmark::BlockQuoteKind::Tip) => {
                        Some((AlertKind::Tip, quotes.clone()))
                    }
                    Some(pulldown_cmark::BlockQuoteKind::Important) => {
                        Some((AlertKind::Important, quotes.clone()))
                    }
                    Some(pulldown_cmark::BlockQuoteKind::Warning) => {
                        Some((AlertKind::Warning, quotes.clone()))
                    }
                    Some(pulldown_cmark::BlockQuoteKind::Caution) => {
                        Some((AlertKind::Caution, quotes.clone()))
                    }
                    _ => detect_and_convert_alert(quotes.clone()),
                };
                if let Some((alert_kind, cleaned_blocks)) = alert_opt {
                    vec![Block::Alert {
                        kind: alert_kind,
                        content: cleaned_blocks,
                    }]
                } else {
                    vec![Block::Quote(quotes)]
                }
            }
            Event::Start(Tag::FootnoteDefinition(label)) => {
                let content = self.parse_blocks_until(TagEnd::FootnoteDefinition);
                vec![Block::FootnoteDefinition {
                    label: label.to_string(),
                    content,
                }]
            }
            Event::Rule => vec![Block::HorizontalRule],
            Event::Html(text) | Event::InlineHtml(text) => vec![Block::Html(text.to_string())],
            Event::DisplayMath(t) => vec![Block::Math(t.to_string())],
            Event::InlineMath(t) => {
                vec![Block::Paragraph(vec![Inline::InlineMath(t.to_string())])]
            }
            _ => Vec::new(),
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
        } else if lang.as_deref() == Some("math")
            || lang.as_deref() == Some("latex")
            || lang.as_deref() == Some("katex")
        {
            Block::Math(code)
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
                    let inlines = self.parse_inlines();
                    let _ = self.iter.next();
                    let item_blocks = split_inlines_by_display_math(inlines);
                    for b in item_blocks {
                        match b {
                            Block::Paragraph(p_inlines) => {
                                if content.is_empty() {
                                    content = p_inlines;
                                } else {
                                    sub_blocks.push(Block::Paragraph(p_inlines));
                                }
                            }
                            other => {
                                sub_blocks.push(other);
                            }
                        }
                    }
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
                    sub_blocks.push(Block::Math(t.to_string()));
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

fn detect_and_convert_alert(mut blocks: Vec<Block>) -> Option<(AlertKind, Vec<Block>)> {
    if blocks.is_empty() {
        return None;
    }

    if let Block::Paragraph(ref mut inlines) = blocks[0] {
        if inlines.is_empty() {
            return None;
        }

        let first_inline = &inlines[0];
        if let Inline::Text(t) = first_inline {
            let trimmed = t.trim_start();
            let kind = if trimmed.starts_with("[!NOTE]") {
                AlertKind::Note
            } else if trimmed.starts_with("[!TIP]") {
                AlertKind::Tip
            } else if trimmed.starts_with("[!IMPORTANT]") {
                AlertKind::Important
            } else if trimmed.starts_with("[!WARNING]") {
                AlertKind::Warning
            } else if trimmed.starts_with("[!CAUTION]") {
                AlertKind::Caution
            } else {
                return None;
            };

            let prefix_tag = match kind {
                AlertKind::Note => "[!NOTE]",
                AlertKind::Tip => "[!TIP]",
                AlertKind::Important => "[!IMPORTANT]",
                AlertKind::Warning => "[!WARNING]",
                AlertKind::Caution => "[!CAUTION]",
            };

            let idx = t.find(prefix_tag).unwrap_or(0);
            let rem = t[idx + prefix_tag.len()..].trim_start().to_string();

            if rem.is_empty() {
                inlines.remove(0);
                if inlines
                    .first()
                    .is_some_and(|i| matches!(i, Inline::SoftBreak))
                {
                    inlines.remove(0);
                }
            } else {
                inlines[0] = Inline::Text(rem);
            }

            if inlines.is_empty() {
                blocks.remove(0);
            }

            return Some((kind, blocks));
        }
    }

    None
}

fn extract_frontmatter(content: &str) -> (Option<Vec<(String, String)>>, &str) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return (None, content);
    }

    let after_start = &trimmed[3..];
    if !after_start.starts_with('\n') && !after_start.starts_with("\r\n") {
        return (None, content);
    }

    let body_start = if let Some(rest) = after_start.strip_prefix("\r\n") {
        rest
    } else if let Some(rest) = after_start.strip_prefix('\n') {
        rest
    } else {
        return (None, content);
    };

    if let Some(end_pos) = body_start.find("\n---") {
        let frontmatter_text = &body_start[..end_pos];
        let rest_after = &body_start[end_pos + 4..];
        let remaining_markdown = if let Some(rest) = rest_after.strip_prefix("\r\n") {
            rest
        } else if let Some(rest) = rest_after.strip_prefix('\n') {
            rest
        } else {
            rest_after
        };

        let mut entries = Vec::new();
        for line in frontmatter_text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once(':') {
                let key = k.trim().to_string();
                let mut val = v.trim().to_string();
                let is_quoted = (val.starts_with('"') && val.ends_with('"'))
                    || (val.starts_with('\'') && val.ends_with('\''));
                if is_quoted && val.len() >= 2 {
                    val = val[1..val.len() - 1].to_string();
                }
                entries.push((key, val));
            }
        }

        if entries.is_empty() {
            (None, content)
        } else {
            (Some(entries), remaining_markdown)
        }
    } else {
        (None, content)
    }
}

pub fn parse_to_blocks(content: &str) -> Vec<Block> {
    let (frontmatter, rest) = extract_frontmatter(content);
    let mut stream = EventStream::new(rest);
    let mut blocks = stream.parse_blocks();
    if let Some(entries) = frontmatter {
        blocks.insert(0, Block::Frontmatter(entries));
    }
    blocks
}
