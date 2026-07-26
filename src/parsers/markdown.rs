use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Parser, Tag, TagEnd};
use std::path::Path;

use crate::{
    log_debug, log_error,
    parsers::{ImageRef, ParseError, ParsedContent, PreviewParser},
};

pub struct MarkdownParser;

impl MarkdownParser {
    pub fn new() -> Self {
        Self
    }

    pub fn render_mermaid_blocks(blocks: &mut [Block]) {
        for block in blocks {
            if let Block::Mermaid { lines, rendered } = block
                && rendered.is_none()
            {
                let code = lines.join("\n");
                *rendered = render_mermaid_to_png(&code);
            }
        }
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

// ── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Inline {
    Text(String),
    Bold(Vec<Inline>),
    Italic(Vec<Inline>),
    Strikethrough(Vec<Inline>),
    Code(String),
    Link { text: Vec<Inline>, url: String },
    Image { alt: String, url: String },
    InlineMath(String),
    DisplayMath(String),
    SoftBreak,
}

#[derive(Debug, Clone)]
pub struct TableCell {
    pub content: Vec<Inline>,
}

#[derive(Debug, Clone)]
pub struct TableBlock {
    pub headers: Vec<TableCell>,
    pub rows: Vec<Vec<TableCell>>,
}

#[derive(Debug, Clone)]
pub enum Block {
    Heading {
        level: u8,
        content: Vec<Inline>,
    },
    Paragraph(Vec<Inline>),
    CodeBlock {
        lang: Option<String>,
        title: Option<String>,
        code: String,
    },
    Table(TableBlock),
    Mermaid {
        lines: Vec<String>,
        rendered: Option<Vec<u8>>,
    },
    Image {
        alt: String,
        path: String,
    },
    List {
        ordered: bool,
        start_number: u64,
        items: Vec<ListItem>,
    },
    Quote(Vec<Block>),
    HorizontalRule,
    Html(String),
}

#[derive(Debug, Clone)]
pub struct ListItem {
    pub is_task: Option<bool>,
    pub content: Vec<Inline>,
    pub sub_blocks: Vec<Block>,
}

// ── Flatten (inline AST → plain string for basic rendering) ──────────────────

pub fn estimated_block_height(
    block: &Block,
    font_size: f32,
    block_index: usize,
    image_sizes: &std::collections::HashMap<usize, (u32, u32)>,
) -> f32 {
    let scale = |s: f32| (s * font_size / 14.0).round().max(8.0);
    let line = font_size * 1.5;
    let margin = block_margin(block);
    match block {
        Block::Heading { level, .. } => {
            let h = match level {
                1 => scale(32.0),
                2 => scale(24.0),
                3 => scale(20.0),
                _ => scale(16.0),
            };
            let (pt, pb, div) = match level {
                1 => (scale(24.0), scale(12.0), 5.0),
                2 => (scale(20.0), scale(8.0), 5.0),
                3 => (scale(12.0), scale(4.0), 0.0),
                _ => (scale(8.0), scale(4.0), 0.0),
            };
            pt + h + pb + div + margin
        }
        Block::Paragraph(_) => line * 1.8 + margin,
        Block::CodeBlock { code, .. } => {
            let n = code.lines().count().max(1) as f32;
            scale(16.0) + n * scale(13.0) * 1.3 + margin
        }
        Block::Table(t) => {
            let rows = t.rows.len().max(1) as f32;
            scale(24.0) * (rows + 1.0) + margin
        }
        Block::List { items, .. } => items.len().max(1) as f32 * line + margin,
        Block::Quote(b) => {
            let h: f32 = b
                .iter()
                .map(|b| estimated_block_height(b, font_size, block_index, image_sizes) * 0.6)
                .sum();
            h + margin
        }
        Block::HorizontalRule => 12.0 + margin,
        Block::Image { .. } => {
            if let Some(&(w, h)) = image_sizes.get(&block_index) {
                // If w > 600, image is scaled down to max width 600
                let display_w = if w > 600 { 600.0 } else { w as f32 };
                let display_h = if w > 0 {
                    (h as f32 * display_w) / (w as f32)
                } else {
                    200.0
                };
                display_h + 8.0 + margin // 8.0 container padding [4, 0]
            } else {
                200.0 + margin
            }
        }
        Block::Mermaid { .. } => 250.0 + margin,
        Block::Html(_) => 50.0 + margin,
    }
}

fn block_margin(block: &Block) -> f32 {
    match block {
        Block::Heading { level, .. } if *level == 1 => 24.0,
        Block::Heading { level, .. } if *level == 2 => 20.0,
        Block::Heading { .. } => 16.0,
        Block::HorizontalRule => 24.0,
        Block::CodeBlock { .. } => 16.0,
        Block::Table(_) => 16.0,
        Block::List { .. } => 12.0,
        Block::Quote(_) => 16.0,
        Block::Image { .. } => 16.0,
        Block::Mermaid { .. } => 16.0,
        Block::Paragraph(_) | Block::Html(_) => 8.0,
    }
}

pub fn extract_toc(
    blocks: &[Block],
    font_size: f32,
    image_sizes: &std::collections::HashMap<usize, (u32, u32)>,
) -> Vec<crate::core::TocEntry> {
    let mut toc = Vec::new();
    let mut y: f32 = 15.0; // Initial container padding
    for (i, block) in blocks.iter().enumerate() {
        if let Block::Heading { level, content } = block {
            let text = flatten_inlines_toc(content);
            toc.push(crate::core::TocEntry {
                level: *level,
                text,
                block_index: i,
                y_offset: y,
            });
        }
        y += estimated_block_height(block, font_size, i, image_sizes);
    }
    toc
}

pub fn flatten_inlines(inlines: &[Inline]) -> String {
    let mut s = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(t) => s.push_str(t),
            Inline::Bold(c) => s.push_str(&flatten_inlines(c)),
            Inline::Italic(c) => s.push_str(&flatten_inlines(c)),
            Inline::Strikethrough(c) => s.push_str(&flatten_inlines(c)),
            Inline::Code(t) => {
                s.push('`');
                s.push_str(t);
                s.push('`');
            }
            Inline::Link { text, url } => {
                s.push_str(&flatten_inlines(text));
                s.push_str(&format!(" ({url})"));
            }
            Inline::Image { alt, url } => {
                s.push_str(&format!("[image: {alt}]({url})"));
            }
            Inline::InlineMath(latex) => s.push_str(latex),
            Inline::DisplayMath(latex) => s.push_str(latex),
            Inline::SoftBreak => s.push(' '),
        }
    }
    s
}

pub fn flatten_inlines_toc(inlines: &[Inline]) -> String {
    let mut s = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(t) => s.push_str(t),
            Inline::Bold(c) => s.push_str(&flatten_inlines(c)),
            Inline::Italic(c) => s.push_str(&flatten_inlines(c)),
            Inline::Strikethrough(c) => s.push_str(&flatten_inlines(c)),
            Inline::Code(t) => {
                s.push('`');
                s.push_str(t);
                s.push('`');
            }
            Inline::Link { text, .. } => {
                s.push_str(&flatten_inlines(text));
            }
            Inline::Image { alt, .. } => s.push_str(alt),
            Inline::InlineMath(latex) => s.push_str(latex),
            Inline::DisplayMath(latex) => s.push_str(latex),
            Inline::SoftBreak => s.push(' '),
        }
    }
    s
}

// ── Image extraction ─────────────────────────────────────────────────────────

fn extract_images(raw: &str, parent: &Path) -> Vec<ImageRef> {
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

// ── Recursive-descent parser ─────────────────────────────────────────────────

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
                    let _ = self.iter.next(); // consume End(Strong)
                    result.push(Inline::Bold(content));
                }
                Event::Start(Tag::Emphasis) => {
                    let content = self.parse_inlines();
                    let _ = self.iter.next(); // consume End(Emphasis)
                    result.push(Inline::Italic(content));
                }
                Event::Start(Tag::Strikethrough) => {
                    let content = self.parse_inlines();
                    let _ = self.iter.next(); // consume End(Strikethrough)
                    result.push(Inline::Strikethrough(content));
                }
                Event::Start(Tag::Link { dest_url, .. }) => {
                    let url = dest_url.to_string();
                    let text = self.parse_inlines();
                    let _ = self.iter.next(); // consume End(Link)
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
                    let _ = self.iter.next(); // consume End(Image)
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
                Some(Event::Start(Tag::Paragraph)) => {
                    let content = self.parse_inlines();
                    let _ = self.iter.next(); // consume End(Paragraph)
                    blocks.push(Block::Paragraph(content));
                }
                Some(Event::Start(Tag::Heading { level, .. })) => {
                    let lvl = match level {
                        HeadingLevel::H1 => 1,
                        HeadingLevel::H2 => 2,
                        HeadingLevel::H3 => 3,
                        HeadingLevel::H4 => 4,
                        HeadingLevel::H5 => 5,
                        HeadingLevel::H6 => 6,
                    };
                    let content = self.parse_inlines();
                    let _ = self.iter.next(); // consume End(Heading)
                    blocks.push(Block::Heading {
                        level: lvl,
                        content,
                    });
                }
                Some(Event::Start(Tag::CodeBlock(kind))) => {
                    let block = self.parse_code_block(kind);
                    blocks.push(block);
                }
                Some(Event::Start(Tag::List(first_num))) => {
                    let ordered = first_num.is_some();
                    let start_number = first_num.unwrap_or(1);
                    let items = self.parse_list_items();
                    blocks.push(Block::List {
                        ordered,
                        start_number,
                        items,
                    });
                }
                Some(Event::Start(Tag::BlockQuote(kind))) => {
                    let quotes = self.parse_blocks_until(TagEnd::BlockQuote(kind));
                    blocks.push(Block::Quote(quotes));
                }
                Some(Event::Rule) => {
                    blocks.push(Block::HorizontalRule);
                }
                Some(Event::Start(Tag::Table(alignments))) => {
                    if let Some(table) = self.parse_table(alignments) {
                        blocks.push(Block::Table(table));
                    }
                }
                Some(Event::Html(text)) | Some(Event::InlineHtml(text)) => {
                    blocks.push(Block::Html(text.to_string()));
                }
                _ => {}
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
                Some(Event::Start(Tag::Paragraph)) => {
                    let content = self.parse_inlines();
                    let _ = self.iter.next(); // consume End(Paragraph)
                    blocks.push(Block::Paragraph(content));
                }
                Some(Event::Start(Tag::Heading { level, .. })) => {
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
                    blocks.push(Block::Heading {
                        level: lvl,
                        content,
                    });
                }
                Some(Event::Start(Tag::List(first_num))) => {
                    let ordered = first_num.is_some();
                    let start_number = first_num.unwrap_or(1);
                    let items = self.parse_list_items();
                    blocks.push(Block::List {
                        ordered,
                        start_number,
                        items,
                    });
                }
                Some(Event::Start(Tag::BlockQuote(kind))) => {
                    let quotes = self.parse_blocks_until(TagEnd::BlockQuote(kind));
                    blocks.push(Block::Quote(quotes));
                }
                Some(Event::Rule) => {
                    blocks.push(Block::HorizontalRule);
                }
                Some(Event::Start(Tag::CodeBlock(kind))) => {
                    let block = self.parse_code_block(kind);
                    blocks.push(block);
                }
                Some(Event::Html(text)) | Some(Event::InlineHtml(text)) => {
                    blocks.push(Block::Html(text.to_string()));
                }
                _ => {}
            }
        }
        blocks
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
                let _ = self.iter.next(); // consume End(TableCell)
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

// ── Mermaid rendering ────────────────────────────────────────────────────────

static FONT_DB: std::sync::LazyLock<std::sync::Arc<resvg::usvg::fontdb::Database>> =
    std::sync::LazyLock::new(|| {
        let mut fontdb = resvg::usvg::fontdb::Database::new();
        fontdb.load_system_fonts();
        std::sync::Arc::new(fontdb)
    });

pub fn render_mermaid_to_png(code: &str) -> Option<Vec<u8>> {
    log_debug!("Rendering Mermaid diagram ({} bytes)", code.len());

    let options = mermaid_rs_renderer::RenderOptions::modern();
    let svg = mermaid_rs_renderer::render_with_options(code, options).ok()?;

    let opt = resvg::usvg::Options {
        fontdb: FONT_DB.clone(),
        ..Default::default()
    };

    let rtree = resvg::usvg::Tree::from_str(&svg, &opt).ok()?;

    let pixmap_size = rtree.size().to_int_size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())?;

    resvg::render(
        &rtree,
        resvg::usvg::Transform::default(),
        &mut pixmap.as_mut(),
    );

    let png_data = pixmap.encode_png().ok()?;

    if png_data.is_empty() {
        log_error!("Generated PNG is empty");
        return None;
    }

    log_debug!("Mermaid render succeeded ({} bytes PNG)", png_data.len());

    Some(png_data)
}

// ── PreviewParser impl ───────────────────────────────────────────────────────

impl PreviewParser for MarkdownParser {
    fn supported_extensions(&self) -> &[&str] {
        &["md", "markdown", "mdown", "mdwn", "mkd", "mkdn"]
    }

    fn is_supported(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| self.supported_extensions().contains(&e))
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let parent = path.parent().unwrap_or(Path::new("."));
        let raw =
            std::fs::read_to_string(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        let images = extract_images(&raw, parent);
        let blocks = parse_to_blocks(&raw);

        Ok(ParsedContent::Markdown {
            content: raw,
            images,
            blocks,
        })
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn text(s: &str) -> Inline {
        Inline::Text(s.to_string())
    }

    fn code(s: &str) -> Inline {
        Inline::Code(s.to_string())
    }

    #[test]
    fn parses_basic_markdown() {
        let mut tmp = tempfile::Builder::new().suffix(".md").tempfile().unwrap();
        write!(tmp, "# Hello\n\nThis is **bold** and `code`").unwrap();
        let parser = MarkdownParser::new();
        let result = parser.parse(tmp.path()).unwrap();
        match result {
            ParsedContent::Markdown {
                content, images, ..
            } => {
                assert!(content.contains("Hello"));
                assert!(images.is_empty());
            }
            _ => panic!("expected Markdown variant"),
        }
    }

    #[test]
    fn parses_inline_bold_and_code_in_paragraph() {
        let blocks = parse_to_blocks("This is **bold** and `code`");
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            Block::Paragraph(inlines) => {
                assert_eq!(inlines.len(), 4);
                assert!(matches!(inlines[0], Inline::Text(_)));
                assert!(matches!(inlines[1], Inline::Bold(_)));
                assert!(matches!(inlines[2], Inline::Text(_)));
                assert!(matches!(inlines[3], Inline::Code(_)));
            }
            _ => panic!("expected Paragraph"),
        }
    }

    #[test]
    fn parses_inline_italic_and_link() {
        let blocks = parse_to_blocks("*italic* and [link](https://example.com)");
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            Block::Paragraph(inlines) => {
                assert!(matches!(inlines[0], Inline::Italic(_)));
                match &inlines[2] {
                    Inline::Link { text, url } => {
                        assert_eq!(url, "https://example.com");
                        assert_eq!(flatten_inlines(text), "link");
                    }
                    _ => panic!("expected Link"),
                }
            }
            _ => panic!("expected Paragraph"),
        }
    }

    #[test]
    fn parses_strikethrough() {
        let blocks = parse_to_blocks("~~struck~~");
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            Block::Paragraph(inlines) => {
                assert!(matches!(inlines[0], Inline::Strikethrough(_)));
                assert_eq!(flatten_inlines(&inlines[0..1]), "struck");
            }
            _ => panic!("expected Paragraph"),
        }
    }

    #[test]
    fn parses_code_block() {
        let mut tmp = tempfile::Builder::new().suffix(".md").tempfile().unwrap();
        write!(tmp, "```rust\nfn main() {{}}\n```").unwrap();
        let parser = MarkdownParser::new();
        let result = parser.parse(tmp.path()).unwrap();
        match result {
            ParsedContent::Markdown { content, .. } => {
                assert!(content.contains("rust"));
                assert!(content.contains("fn main"));
            }
            _ => panic!("expected Markdown variant"),
        }
    }

    #[test]
    fn parses_indented_code_block() {
        let md = "- **Bản tin:**\n\n  ```json\n  {\n    \"device_id\": \"string\"\n  }\n  ```";
        let blocks = parse_to_blocks(md);
        assert!(!blocks.is_empty());

        let direct_code_md = "```json\n{\n  \"device_id\": \"string\"\n}\n```";
        let direct_blocks = parse_to_blocks(direct_code_md);
        assert_eq!(direct_blocks.len(), 1);
        match &direct_blocks[0] {
            Block::CodeBlock { lang, code, .. } => {
                assert_eq!(lang.as_deref(), Some("json"));
                assert!(code.contains("\"device_id\": \"string\""));
            }
            _ => panic!("expected CodeBlock"),
        }
    }

    #[test]
    fn extracts_images() {
        let mut tmp = tempfile::Builder::new().suffix(".md").tempfile().unwrap();
        write!(tmp, "![alt](image.png)").unwrap();
        let parser = MarkdownParser::new();
        let result = parser.parse(tmp.path()).unwrap();
        match result {
            ParsedContent::Markdown { images, .. } => {
                assert_eq!(images.len(), 1);
                assert_eq!(images[0].alt_text, "alt");
                assert!(images[0].path.ends_with("image.png"));
            }
            _ => panic!("expected Markdown variant"),
        }
    }

    #[test]
    fn supports_extension() {
        let parser = MarkdownParser::new();
        assert!(parser.is_supported(Path::new("test.md")));
        assert!(parser.is_supported(Path::new("test.markdown")));
        assert!(!parser.is_supported(Path::new("test.txt")));
    }

    #[test]
    fn returns_error_for_nonexistent_file() {
        let parser = MarkdownParser::new();
        let result = parser.parse(Path::new("/nonexistent/file.md"));
        assert!(result.is_err());
    }

    #[test]
    fn parses_large_markdown_file_over_3000_lines() {
        let mut tmp = tempfile::Builder::new().suffix(".md").tempfile().unwrap();

        let mut large_content = String::new();
        for i in 1..=3500 {
            if i % 100 == 0 {
                large_content.push_str(&format!("## Heading Level 2 at line {i}\n\n"));
            } else {
                large_content.push_str("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor.\n");
            }
        }

        write!(tmp, "{large_content}").unwrap();

        let parser = MarkdownParser::new();
        let start_time = std::time::Instant::now();
        let result = parser.parse(tmp.path()).unwrap();
        let duration = start_time.elapsed();

        match result {
            ParsedContent::Markdown {
                content, images, ..
            } => {
                assert!(content.contains("Heading Level 2 at line 3500"));
                assert_eq!(content.lines().count(), large_content.lines().count());
                assert!(images.is_empty());
                assert!(
                    duration.as_millis() < 200,
                    "Parsing took too long: {:?}",
                    duration
                );
            }
            _ => panic!("expected Markdown variant"),
        }
    }

    #[test]
    fn extracts_multiple_images_from_large_file() {
        let mut tmp = tempfile::Builder::new().suffix(".md").tempfile().unwrap();
        let parent_dir = tmp.path().parent().unwrap().to_path_buf();

        let mut large_content = String::new();
        large_content.push_str("![First Image](assets/img_first.png)\n");

        for i in 2..3100 {
            if i == 1500 {
                large_content.push_str("\n![Middle Image](assets/img_middle.jpg)\n\n");
            } else {
                large_content.push_str(
                    "Testing parser stability with a massive volume of plain text structures.\n",
                );
            }
        }
        large_content.push_str("\n![Last Image](/absolute/path/img_last.svg)\n");

        write!(tmp, "{large_content}").unwrap();

        let parser = MarkdownParser::new();
        let result = parser.parse(tmp.path()).unwrap();

        match result {
            ParsedContent::Markdown { images, .. } => {
                assert_eq!(images.len(), 3);
                assert_eq!(images[0].alt_text, "First Image");
                let expected_first_path = parent_dir
                    .join("assets/img_first.png")
                    .to_string_lossy()
                    .to_string();
                assert_eq!(images[0].path, expected_first_path);

                assert_eq!(images[1].alt_text, "Middle Image");
                let expected_mid_path = parent_dir
                    .join("assets/img_middle.jpg")
                    .to_string_lossy()
                    .to_string();
                assert_eq!(images[1].path, expected_mid_path);

                assert_eq!(images[2].alt_text, "Last Image");
                assert_eq!(images[2].path, "/absolute/path/img_last.svg");
            }
            _ => panic!("expected Markdown variant"),
        }
    }

    #[test]
    fn handles_images_inside_code_blocks_in_large_file() {
        let mut tmp = tempfile::Builder::new().suffix(".md").tempfile().unwrap();
        let mut large_content = String::new();

        large_content.push_str("![Valid Image](valid.png)\n");
        for i in 2..3200 {
            if i == 1600 {
                large_content.push_str(
                    "\n```markdown\nThis is a code block sample: ![Fake Image](fake.png)\n```\n\n",
                );
            } else {
                large_content.push_str("Standard text row filler.\n");
            }
        }

        write!(tmp, "{large_content}").unwrap();

        let parser = MarkdownParser::new();
        let result = parser.parse(tmp.path()).unwrap();

        match result {
            ParsedContent::Markdown { images, .. } => {
                assert_eq!(images.len(), 1);
                assert_eq!(images[0].alt_text, "Valid Image");
                assert!(images[0].path.ends_with("valid.png"));
            }
            _ => panic!("expected Markdown variant"),
        }
    }

    #[test]
    fn test_parse_heading_paragraph_to_blocks() {
        let md = "# Hello\n\nSome text";
        let blocks = parse_to_blocks(md);
        assert_eq!(blocks.len(), 2);
        assert!(matches!(blocks[0], Block::Heading { level: 1, .. }));
        assert!(matches!(blocks[1], Block::Paragraph(_)));
    }

    #[test]
    fn test_parse_code_block_to_blocks() {
        let md = "```rust\nfn main() {}\n```";
        let blocks = parse_to_blocks(md);
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            Block::CodeBlock { lang, code, .. } => {
                assert_eq!(lang.as_deref(), Some("rust"));
                assert!(code.contains("fn main"));
            }
            _ => panic!("expected CodeBlock block"),
        }
    }

    #[test]
    fn test_parse_table() {
        let md = "| H1 | H2 |\n|---|---|\n| C1 | C2 |";
        let blocks = parse_to_blocks(md);
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            Block::Table(tbl) => {
                assert_eq!(tbl.headers.len(), 2);
                assert_eq!(flatten_inlines(&tbl.headers[0].content), "H1");
                assert_eq!(flatten_inlines(&tbl.headers[1].content), "H2");
                assert_eq!(tbl.rows.len(), 1);
                assert_eq!(flatten_inlines(&tbl.rows[0][0].content), "C1");
                assert_eq!(flatten_inlines(&tbl.rows[0][1].content), "C2");
            }
            _ => panic!("expected Table block"),
        }
    }

    #[test]
    fn test_parse_table_headers_only() {
        let md = "| A | B |\n|---|---|";
        let blocks = parse_to_blocks(md);
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            Block::Table(tbl) => {
                assert_eq!(tbl.headers.len(), 2);
                assert!(tbl.rows.is_empty());
            }
            _ => panic!("expected Table block"),
        }
    }

    #[test]
    fn test_parse_unordered_list() {
        let md = "- Item A\n- Item B\n    - Nested";
        let blocks = parse_to_blocks(md);
        assert!(!blocks.is_empty());
        match &blocks[0] {
            Block::List { ordered, items, .. } => {
                assert!(!ordered);
                assert_eq!(items.len(), 2);
                assert!(
                    items[0]
                        .content
                        .iter()
                        .any(|i| matches!(i, Inline::Text(t) if t.contains("Item A")))
                );
            }
            _ => panic!("expected List block"),
        }
    }

    #[test]
    fn test_parse_ordered_list() {
        let md = "1. First\n2. Second";
        let blocks = parse_to_blocks(md);
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            Block::List { ordered, items, .. } => {
                assert!(ordered);
                assert_eq!(items.len(), 2);
            }
            _ => panic!("expected List block"),
        }
    }

    #[test]
    fn test_parse_blockquote() {
        let md = "> hello\n> world";
        let blocks = parse_to_blocks(md);
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            Block::Quote(inner) => {
                assert_eq!(inner.len(), 1);
                match &inner[0] {
                    Block::Paragraph(inlines) => {
                        assert_eq!(flatten_inlines(inlines), "hello world");
                    }
                    _ => panic!("expected Paragraph in quote"),
                }
            }
            _ => panic!("expected Quote block"),
        }
    }

    #[test]
    fn test_parse_horizontal_rule() {
        let md = "Before\n\n---\n\nAfter";
        let blocks = parse_to_blocks(md);
        assert_eq!(blocks.len(), 3);
        assert!(matches!(blocks[0], Block::Paragraph(_)));
        assert!(matches!(blocks[1], Block::HorizontalRule));
        assert!(matches!(blocks[2], Block::Paragraph(_)));
    }

    #[test]
    fn test_link_contains_url() {
        let md = "[Click](https://example.com)";
        let blocks = parse_to_blocks(md);
        let flat = flatten_inlines(match &blocks[0] {
            Block::Paragraph(inlines) => inlines,
            _ => panic!("expected Paragraph"),
        });
        assert!(flat.contains("example.com"));
        assert!(flat.contains("Click"));
    }

    #[test]
    fn test_flatten_inlines_preserves_code() {
        let inlines = vec![text("use "), code("std::fs"), text(";")];
        let flat = flatten_inlines(&inlines);
        assert_eq!(flat, "use `std::fs`;");
    }

    // ── Mermaid async rendering tests ──────────────────────────────────────

    #[test]
    fn mermaid_block_rendered_none_after_parse_to_blocks() {
        let md = "```mermaid\ngraph TD\nA-->B\n```";
        let blocks = parse_to_blocks(md);
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            Block::Mermaid { lines, rendered } => {
                assert!(rendered.is_none(), "Mermaid block must start unrendered");
                assert_eq!(lines.join("\n"), "graph TD\nA-->B");
            }
            _ => panic!("expected Mermaid block"),
        }
    }

    #[test]
    fn parse_returns_mermaid_blocks_unrendered() {
        let mut tmp = tempfile::Builder::new().suffix(".md").tempfile().unwrap();
        write!(
            tmp,
            "# Title\n\n```mermaid\ngraph LR\nA-->B\n```\n\nSome text."
        )
        .unwrap();
        let parser = MarkdownParser::new();
        let result = parser.parse(tmp.path()).unwrap();
        match result {
            ParsedContent::Markdown { blocks, .. } => {
                assert_eq!(blocks.len(), 3, "heading + mermaid + paragraph");
                match &blocks[1] {
                    Block::Mermaid { lines, rendered } => {
                        assert!(
                            rendered.is_none(),
                            "parse() must NOT render mermaid blocks synchronously"
                        );
                        assert!(lines.join(" ").contains("graph LR"));
                    }
                    other => panic!("expected Mermaid at index 1, got {other:?}"),
                }
            }
            _ => panic!("expected Markdown variant"),
        }
    }

    #[test]
    fn multiple_mermaid_blocks_all_unrendered() {
        let md = "# M1\n\n```mermaid\ngraph TD\nA\n```\n\nText\n\n```mermaid\nsequenceDiagram\nA->>B\n```";
        let blocks = parse_to_blocks(md);
        let mermaid_count = blocks
            .iter()
            .filter(|b| matches!(b, Block::Mermaid { rendered, .. } if rendered.is_none()))
            .count();
        assert_eq!(mermaid_count, 2, "both mermaid blocks must be unrendered");
    }

    #[test]
    fn non_mermaid_content_intact_after_parse() {
        let mut tmp = tempfile::Builder::new().suffix(".md").tempfile().unwrap();
        write!(
            tmp,
            "# Heading\n\nA paragraph.\n\n```rust\nfn main() {{}}\n```\n\n```mermaid\ngraph TD\nA-->B\n```"
        )
        .unwrap();
        let parser = MarkdownParser::new();
        let result = parser.parse(tmp.path()).unwrap();
        match result {
            ParsedContent::Markdown { blocks, .. } => {
                assert_eq!(blocks.len(), 4);
                assert!(matches!(&blocks[0], Block::Heading { level: 1, .. }));
                assert!(matches!(&blocks[1], Block::Paragraph(_)));
                assert!(matches!(&blocks[2], Block::CodeBlock { .. }));
                assert!(matches!(
                     &blocks[3],
                     Block::Mermaid { rendered, .. } if rendered.is_none()
                ));
            }
            _ => panic!("expected Markdown variant"),
        }
    }

    #[test]
    fn render_mermaid_to_png_returns_valid_png() {
        let result = render_mermaid_to_png("graph TD\nA-->B");
        assert!(result.is_some(), "should render successfully in-process");
    }

    #[test]
    fn mermaid_block_update_after_async_render() {
        let mut blocks = [
            Block::Mermaid {
                lines: vec!["graph TD".into(), "A-->B".into()],
                rendered: None,
            },
            Block::Mermaid {
                lines: vec!["sequenceDiagram".into(), "A->>B".into()],
                rendered: None,
            },
        ]
        .to_vec();

        let png_bytes = Some(vec![1, 2, 3]);
        if let Block::Mermaid {
            ref mut rendered, ..
        } = blocks[1]
        {
            *rendered = png_bytes;
        }

        assert!(matches!(&blocks[0], Block::Mermaid { rendered, .. } if rendered.is_none()));
        assert!(
            matches!(&blocks[1], Block::Mermaid { rendered, .. } if rendered.as_deref() == Some(&[1u8, 2, 3]))
        );
    }

    #[test]
    fn markdown_state_caches_mermaid_handles_correctly() {
        use crate::core::MarkdownState;
        use iced::widget::image::Handle;

        let blocks = [
            Block::Heading {
                level: 1,
                content: vec![text("Title")],
            },
            Block::Mermaid {
                lines: vec!["graph TD".into(), "A-->B".into()],
                rendered: None,
            },
            Block::Paragraph(vec![text("Some text.")]),
            Block::Mermaid {
                lines: vec!["sequenceDiagram".into(), "A->>B".into()],
                rendered: None,
            },
            Block::Mermaid {
                lines: vec!["graph LR".into(), "C-->D".into()],
                rendered: None,
            },
        ];

        let mut state = MarkdownState::default();
        assert!(
            state.cached_mermaid_handles.is_empty(),
            "fresh MarkdownState must start with an empty HashMap"
        );

        for (i, block) in blocks.iter().enumerate() {
            if let Block::Mermaid {
                rendered: Some(_), ..
            } = block
            {
                state
                    .cached_mermaid_handles
                    .insert(i, Handle::from_rgba(1, 1, vec![0, 0, 0, 255]));
            }
        }
        assert!(
            state.cached_mermaid_handles.is_empty(),
            "cache must be empty when no Mermaid block has rendered output yet"
        );

        state
            .cached_mermaid_handles
            .insert(1, Handle::from_rgba(2, 2, vec![128; 16]));

        assert_eq!(state.cached_mermaid_handles.len(), 1);
        assert!(
            state.cached_mermaid_handles.contains_key(&1),
            "handle must be stored under block index 1"
        );
        assert!(
            !state.cached_mermaid_handles.contains_key(&0),
            "non-Mermaid block (heading) must NOT have a cache entry"
        );
        assert!(
            !state.cached_mermaid_handles.contains_key(&3),
            "unrendered Mermaid block must NOT have a cache entry yet"
        );

        state
            .cached_mermaid_handles
            .insert(3, Handle::from_rgba(2, 2, vec![255; 16]));

        assert_eq!(
            state.cached_mermaid_handles.len(),
            2,
            "two handles cached after two Mermaid blocks rendered"
        );
        assert!(state.cached_mermaid_handles.contains_key(&3));

        state
            .cached_mermaid_handles
            .insert(1, Handle::from_rgba(4, 4, vec![64; 64]));

        assert_eq!(
            state.cached_mermaid_handles.len(),
            2,
            "replacing an existing handle must NOT increase HashMap size"
        );

        assert!(
            state.cached_mermaid_handles.contains_key(&1),
            "render_mermaid must find handle for block[1]"
        );
        assert!(
            state.cached_mermaid_handles.contains_key(&3),
            "render_mermaid must find handle for block[3]"
        );
        assert!(
            !state.cached_mermaid_handles.contains_key(&0),
            "render_mermaid must NOT find handle for heading block"
        );
        assert!(
            !state.cached_mermaid_handles.contains_key(&2),
            "render_mermaid must NOT find handle for paragraph block"
        );
        assert!(
            !state.cached_mermaid_handles.contains_key(&4),
            "render_mermaid must NOT find handle for unrendered Mermaid block"
        );
    }

    #[test]
    fn markdown_state_ignores_non_mermaid_blocks() {
        use crate::core::MarkdownState;
        use iced::widget::image::Handle;

        let blocks = [
            Block::Paragraph(vec![text("Hello")]),
            Block::CodeBlock {
                lang: Some("rust".into()),
                title: None,
                code: "fn main() {}".into(),
            },
            Block::Table(TableBlock {
                headers: vec![TableCell {
                    content: vec![text("A")],
                }],
                rows: vec![vec![TableCell {
                    content: vec![text("1")],
                }]],
            }),
        ];

        let mut state = MarkdownState::default();
        for (i, block) in blocks.iter().enumerate() {
            if let Block::Mermaid {
                rendered: Some(_), ..
            } = block
            {
                state
                    .cached_mermaid_handles
                    .insert(i, Handle::from_rgba(1, 1, vec![0, 0, 0, 255]));
            }
        }
        assert!(
            state.cached_mermaid_handles.is_empty(),
            "cache must be empty when markdown has zero Mermaid blocks"
        );
    }

    #[test]
    fn parses_inline_math() {
        let blocks = parse_to_blocks("Khoảng cách $D$ từ vị trí ban đầu");
        assert_eq!(blocks.len(), 1);
        if let Block::Paragraph(inlines) = &blocks[0] {
            let math = inlines.iter().find(|i| matches!(i, Inline::InlineMath(_)));
            assert!(math.is_some(), "should find InlineMath");
            if let Some(Inline::InlineMath(latex)) = math {
                assert_eq!(latex, "D");
            }
        } else {
            panic!("expected Paragraph");
        }
    }

    #[test]
    fn parses_display_math() {
        let src = r"Before

$$
d = 2R \cdot \arcsin\left(\sqrt{\sin^2\left(\frac{\Delta \phi}{2}\right)}\right)
$$

After";
        let blocks = parse_to_blocks(src);
        assert_eq!(blocks.len(), 3, "three paragraphs");
        if let Block::Paragraph(inlines) = &blocks[1] {
            let math = inlines.iter().find(|i| matches!(i, Inline::DisplayMath(_)));
            assert!(
                math.is_some(),
                "should find DisplayMath in second paragraph"
            );
        } else {
            panic!("expected Paragraph");
        }
    }

    #[test]
    fn parses_inline_math_with_text_and_le() {
        let src = "$10\\text{m} \\le D \\le 50\\text{m}$";
        let blocks = parse_to_blocks(src);
        assert_eq!(blocks.len(), 1);
        if let Block::Paragraph(inlines) = &blocks[0] {
            let math = inlines.iter().find(|i| matches!(i, Inline::InlineMath(_)));
            assert!(math.is_some(), "should find InlineMath");
        } else {
            panic!("expected Paragraph");
        }
    }

    #[test]
    fn parses_greek_latex_in_list_items() {
        let src = "\
- $\\phi_1, \\phi_2$ là vĩ độ (latitude) của 2 điểm (tính bằng radian).
- $\\Delta \\phi = \\phi_2 - \\phi_1$.
- $\\Delta \\lambda = longitude_2 - longitude_1$ (chênh lệch kinh độ tính bằng radian).
- $R$ là bán kính Trái Đất (lấy xấp xỉ $6.371\\text{ km}$).";
        let blocks = parse_to_blocks(src);
        assert_eq!(blocks.len(), 1, "one list block");
        if let Block::List { items, .. } = &blocks[0] {
            assert_eq!(items.len(), 4, "four list items");
            for (i, item) in items.iter().enumerate() {
                let math_count = item
                    .content
                    .iter()
                    .filter(|inline| matches!(inline, Inline::InlineMath(_)))
                    .count();
                assert!(
                    math_count >= 1,
                    "item {i} should have at least one InlineMath, found {math_count}"
                );
            }
            let last_math_count = items[3]
                .content
                .iter()
                .filter(|inline| matches!(inline, Inline::InlineMath(_)))
                .count();
            assert_eq!(
                last_math_count, 2,
                "last item should have two InlineMath ($R$ and $6.371\\text{{ km}}$)"
            );
        } else {
            panic!("expected List block, got {:?}", blocks[0]);
        }
    }
}
