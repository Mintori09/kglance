use std::path::Path;

use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

use crate::parser::{ImageRef, ParseError, ParsedContent, PreviewParser};

pub struct MarkdownParser;

impl MarkdownParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum Block {
    Heading { level: u8, text: String },
    Paragraph(String),
    CodeBlock { lang: String, code: String },
    Table(TableBlock),
}

#[derive(Debug, Clone)]
pub struct TableBlock {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

fn extract_images(raw: &str, parent: &Path) -> Vec<ImageRef> {
    let mut images = Vec::new();
    let mut in_image = false;
    let mut image_alt = String::new();
    let mut image_url = String::new();

    for event in Parser::new_ext(raw, pulldown_cmark::Options::all()) {
        match event {
            Event::Start(Tag::Image {
                dest_url, title: _, ..
            }) => {
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

pub fn parse_to_blocks(content: &str) -> Vec<Block> {
    let parser = pulldown_cmark::Parser::new_ext(content, pulldown_cmark::Options::all());
    let mut blocks = Vec::new();

    struct TableAccum {
        rows: Vec<Vec<String>>,
        current_row: Vec<String>,
        current_cell: String,
    }
    let mut table_accum: Option<TableAccum> = None;
    let mut code_accum: Option<(String, String)> = None;
    let mut current_text = String::new();

    let flush_text = |text: &mut String, blocks: &mut Vec<Block>| {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            blocks.push(Block::Paragraph(trimmed.to_string()));
            text.clear();
        }
    };

    for event in parser {
        if let Some(ref mut state) = table_accum {
            match event {
                Event::Start(Tag::TableCell) => {
                    state.current_cell.clear();
                }
                Event::End(TagEnd::TableCell) => {
                    state.current_row.push(state.current_cell.clone());
                }
                Event::End(TagEnd::TableRow) | Event::End(TagEnd::TableHead) => {
                    state.rows.push(state.current_row.clone());
                    state.current_row.clear();
                }
                Event::End(TagEnd::Table) => {
                    let headers = state.rows.first().cloned().unwrap_or_default();
                    let rows = if state.rows.len() > 1 {
                        state.rows[1..].to_vec()
                    } else {
                        Vec::new()
                    };
                    blocks.push(Block::Table(TableBlock { headers, rows }));
                    table_accum = None;
                }
                Event::Text(text) => {
                    state.current_cell.push_str(&text);
                }
                Event::Code(code) => {
                    state.current_cell.push('`');
                    state.current_cell.push_str(&code);
                    state.current_cell.push('`');
                }
                _ => {}
            }
            continue;
        }

        if let Some(ref mut accum) = code_accum {
            match event {
                Event::End(TagEnd::CodeBlock) => {
                    let (lang, code) = std::mem::take(accum);
                    blocks.push(Block::CodeBlock { lang, code });
                    code_accum = None;
                }
                Event::Text(t) => accum.1.push_str(&t),
                Event::SoftBreak | Event::HardBreak => accum.1.push('\n'),
                _ => {}
            }
            continue;
        }

        match event {
            Event::Start(Tag::Table(_)) => {
                flush_text(&mut current_text, &mut blocks);
                table_accum = Some(TableAccum {
                    rows: Vec::new(),
                    current_row: Vec::new(),
                    current_cell: String::new(),
                });
            }
            Event::Start(Tag::Heading { level, .. }) => {
                let prefix = match level {
                    HeadingLevel::H1 => "# ",
                    HeadingLevel::H2 => "## ",
                    HeadingLevel::H3 => "### ",
                    _ => "#### ",
                };
                current_text.push_str(prefix);
            }
            Event::End(TagEnd::Heading(_)) => {
                flush_text(&mut current_text, &mut blocks);
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                flush_text(&mut current_text, &mut blocks);
                let lang = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(l) => l.to_string(),
                    pulldown_cmark::CodeBlockKind::Indented => String::new(),
                };
                code_accum = Some((lang, String::new()));
            }
            Event::Start(Tag::List(_)) => {
                current_text.push('\n');
            }
            Event::End(TagEnd::List(_)) => {
                current_text.push('\n');
            }
            Event::Start(Tag::Item) => {
                current_text.push_str("  • ");
            }
            Event::End(TagEnd::Item) => {
                current_text.push('\n');
            }
            Event::End(TagEnd::Paragraph) => {
                flush_text(&mut current_text, &mut blocks);
            }
            Event::Text(text) => {
                current_text.push_str(&text);
            }
            Event::Code(code) => {
                current_text.push('`');
                current_text.push_str(&code);
                current_text.push('`');
            }
            Event::SoftBreak | Event::HardBreak => {
                current_text.push('\n');
            }
            _ => {}
        }
    }

    flush_text(&mut current_text, &mut blocks);
    blocks
}

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

        Ok(ParsedContent::Markdown {
            content: raw,
            images,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parses_basic_markdown() {
        let mut tmp = tempfile::Builder::new().suffix(".md").tempfile().unwrap();
        write!(tmp, "# Hello\n\nThis is **bold** and `code`").unwrap();
        let parser = MarkdownParser::new();
        let result = parser.parse(tmp.path()).unwrap();
        match result {
            ParsedContent::Markdown { content, images } => {
                assert!(content.contains("Hello"));
                assert!(images.is_empty());
            }
            _ => panic!("expected Markdown variant"),
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
    fn extracts_images() {
        let mut tmp = tempfile::Builder::new().suffix(".md").tempfile().unwrap();
        write!(tmp, "![alt](image.png)").unwrap();
        let parser = MarkdownParser::new();
        let result = parser.parse(tmp.path()).unwrap();
        match result {
            ParsedContent::Markdown { content: _, images } => {
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
                large_content.push_str(&format!("## Heading Level 2 at line {}\n\n", i));
            } else {
                large_content.push_str("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor.\n");
            }
        }

        write!(tmp, "{}", large_content).unwrap();

        let parser = MarkdownParser::new();
        let start_time = std::time::Instant::now();
        let result = parser.parse(tmp.path()).unwrap();
        let duration = start_time.elapsed();

        match result {
            ParsedContent::Markdown { content, images } => {
                assert!(content.contains("Heading Level 2 at line 3500"));
                assert_eq!(content.lines().count(), large_content.lines().count());
                assert!(images.is_empty());
                assert!(
                    duration.as_millis() < 100,
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

        write!(tmp, "{}", large_content).unwrap();

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

        write!(tmp, "{}", large_content).unwrap();

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
    fn test_parse_table_to_blocks() {
        let md = "| H1 | H2 |\n|---|---|\n| C1 | C2 |";
        let blocks = parse_to_blocks(md);
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            Block::Table(tbl) => {
                assert_eq!(tbl.headers, vec!["H1", "H2"]);
                assert_eq!(tbl.rows.len(), 1);
                assert_eq!(tbl.rows[0], vec!["C1", "C2"]);
            }
            _ => panic!("expected Table block"),
        }
    }

    #[test]
    fn test_parse_heading_paragraph_to_blocks() {
        let md = "# Hello\n\nSome text";
        let blocks = parse_to_blocks(md);
        assert!(!blocks.is_empty());
        assert!(matches!(blocks[0], Block::Paragraph(_)));
    }

    #[test]
    fn test_parse_code_block_to_blocks() {
        let md = "```rust\nfn main() {}\n```";
        let blocks = parse_to_blocks(md);
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            Block::CodeBlock { lang, code } => {
                assert_eq!(lang, "rust");
                assert!(code.contains("fn main"));
            }
            _ => panic!("expected CodeBlock block"),
        }
    }

    #[test]
    fn test_parse_table_headers_only() {
        let md = "| A | B |\n|---|---|";
        let blocks = parse_to_blocks(md);
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            Block::Table(tbl) => {
                assert_eq!(tbl.headers, vec!["A", "B"]);
                assert!(tbl.rows.is_empty());
            }
            _ => panic!("expected Table block"),
        }
    }
}
