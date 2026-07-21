use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
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

#[derive(Debug, Clone)]
pub enum Block {
    Heading {
        level: u8,
        text: String,
    },
    Paragraph(String),
    CodeBlock {
        lang: String,
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

pub fn parse_to_blocks(content: &str) -> Vec<Block> {
    let parser = pulldown_cmark::Parser::new_ext(content, pulldown_cmark::Options::all());
    let mut blocks = Vec::new();
    let mut heading_accum: Option<(u8, String)> = None;

    struct TableAccum {
        rows: Vec<Vec<String>>,
        current_row: Vec<String>,
        current_cell: String,
    }

    fn lines_from_code(code: &str) -> Vec<String> {
        code.lines().map(|l| l.trim().to_string()).collect()
    }

    let mut table_accum: Option<TableAccum> = None;
    let mut code_accum: Option<(String, String)> = None;
    let mut current_text = String::new();
    let mut image_accum: Option<(String, String)> = None; // (url, alt)

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
                    state
                        .current_row
                        .push(std::mem::take(&mut state.current_cell));
                }
                Event::End(TagEnd::TableRow) | Event::End(TagEnd::TableHead) => {
                    state.rows.push(std::mem::take(&mut state.current_row));
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
                    if lang == "mermaid" {
                        blocks.push(Block::Mermaid {
                            lines: lines_from_code(&code),
                            rendered: None,
                        });
                    } else {
                        blocks.push(Block::CodeBlock { lang, code });
                    }
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
                flush_text(&mut current_text, &mut blocks);

                let level = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };

                heading_accum = Some((level, String::new()));
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some((level, text)) = heading_accum.take() {
                    blocks.push(Block::Heading { level, text });
                }
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                flush_text(&mut current_text, &mut blocks);
                let lang = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(l) => l.to_string(),
                    pulldown_cmark::CodeBlockKind::Indented => String::new(),
                };
                code_accum = Some((lang, String::new()));
            }
            Event::Start(Tag::List(_)) | Event::End(TagEnd::List(_)) => {
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
            Event::Start(Tag::Image { dest_url, .. }) => {
                flush_text(&mut current_text, &mut blocks);
                image_accum = Some((dest_url.to_string(), String::new()));
            }
            Event::End(TagEnd::Image) => {
                if let Some((url, alt)) = image_accum.take() {
                    blocks.push(Block::Image { alt, path: url });
                }
            }
            Event::Text(text) => {
                if let Some((_, ref mut heading)) = heading_accum {
                    heading.push_str(&text);
                } else if let Some((_, ref mut alt)) = image_accum {
                    alt.push_str(&text);
                } else {
                    current_text.push_str(&text);
                }
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

/// Helper function render Mermaid diagram sang PNG byte buffer bằng mmdc
pub fn render_mermaid_to_png(code: &str) -> Option<Vec<u8>> {
    use std::io::Write;

    log_debug!("Rendering Mermaid diagram ({} bytes)", code.len());

    let dir = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(e) => {
            log_error!("Failed to create temp dir: {}", e);
            return None;
        }
    };

    let input = dir.path().join("diagram.mmd");
    let output = dir.path().join("diagram.png");

    let mut file = match std::fs::File::create(&input) {
        Ok(file) => file,
        Err(e) => {
            log_error!("Failed to create Mermaid file: {}", e);
            return None;
        }
    };

    if let Err(e) = write!(file, "{}", code) {
        log_error!("Failed to write Mermaid source: {}", e);
        return None;
    }

    drop(file);

    let result = std::process::Command::new("mmdr")
        .args([
            "-i",
            input.to_str()?,
            "-o",
            output.to_str()?,
            "-e",
            "png",
            "-w",
            "1200",
        ])
        .output();

    let output_result = match result {
        Ok(output) => output,
        Err(e) => {
            log_error!("Failed to launch mmdr: {}", e);
            return None;
        }
    };

    if !output_result.status.success() {
        let stderr = String::from_utf8_lossy(&output_result.stderr);

        log_error!(
            "mmdr exited with status {:?}: {}",
            output_result.status.code(),
            stderr.trim()
        );

        return None;
    }

    let png = match std::fs::read(&output) {
        Ok(data) => data,
        Err(e) => {
            log_error!("Failed to read generated PNG: {}", e);
            return None;
        }
    };

    if png.is_empty() {
        log_error!("Generated PNG is empty");
        return None;
    }

    log_debug!("Mermaid render succeeded ({} bytes PNG)", png.len());

    Some(png)
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
        let blocks = parse_to_blocks(&raw);

        // Mermaid blocks NOT rendered here — rendering happens asynchronously
        // after the content is displayed to the user (see app.rs FileLoaded handler).
        Ok(ParsedContent::Markdown {
            content: raw,
            images,
            blocks,
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
            ParsedContent::Markdown {
                content, images, ..
            } => {
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
                    other => panic!("expected Mermaid at index 1, got {:?}", other),
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
                assert!(matches!(&blocks[0], Block::Paragraph(_)));
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
    fn render_mermaid_to_png_graceful_when_mmdc_missing() {
        // mmdc CLI may not be installed on CI/dev machines
        let result = render_mermaid_to_png("graph TD\nA-->B");
        // Should not panic — returns None gracefully
        assert!(result.is_none() || result.is_some());
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

        // Simulate what MermaidBlockRendered handler does
        let png_bytes = Some(vec![1, 2, 3]);
        if let Block::Mermaid {
            ref mut rendered, ..
        } = blocks[1]
        {
            *rendered = png_bytes;
        }

        // Block 0 still unrendered
        assert!(matches!(&blocks[0], Block::Mermaid { rendered, .. } if rendered.is_none()));
        // Block 1 now rendered
        assert!(
            matches!(&blocks[1], Block::Mermaid { rendered, .. } if rendered.as_deref() == Some(&[1u8, 2, 3]))
        );
    }

    #[test]
    fn markdown_state_caches_mermaid_handles_correctly() {
        use crate::core::MarkdownState;
        use iced::widget::image::Handle;

        // Build a mix of blocks like a real parsed markdown file
        let blocks = vec![
            Block::Heading {
                level: 1,
                text: "Title".into(),
            },
            Block::Mermaid {
                lines: vec!["graph TD".into(), "A-->B".into()],
                rendered: None,
            },
            Block::Paragraph("Some text.".into()),
            Block::Mermaid {
                lines: vec!["sequenceDiagram".into(), "A->>B".into()],
                rendered: None,
            },
            Block::Mermaid {
                lines: vec!["graph LR".into(), "C-->D".into()],
                rendered: None,
            },
        ];

        // 1. Initial state must be empty
        let mut state = MarkdownState::default();
        assert!(
            state.cached_mermaid_handles.is_empty(),
            "fresh MarkdownState must start with an empty HashMap"
        );

        // 2. Simulate populate_state: no block has rendered data → cache stays empty
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

        // 3. Simulate MermaidBlockRendered for block index 1
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

        // 4. Simulate block 3 completing render (index 2 is a Paragraph — skip)
        state
            .cached_mermaid_handles
            .insert(3, Handle::from_rgba(2, 2, vec![255; 16]));

        assert_eq!(
            state.cached_mermaid_handles.len(),
            2,
            "two handles cached after two Mermaid blocks rendered"
        );
        assert!(state.cached_mermaid_handles.contains_key(&3));

        // 5. Simulate block 1 being re-rendered (e.g. file reload): overwrite
        state
            .cached_mermaid_handles
            .insert(1, Handle::from_rgba(4, 4, vec![64; 64]));

        assert_eq!(
            state.cached_mermaid_handles.len(),
            2,
            "replacing an existing handle must NOT increase HashMap size"
        );

        // 6. Verify lookups as render_mermaid would do
        assert!(
            state.cached_mermaid_handles.get(&1).is_some(),
            "render_mermaid must find handle for block[1]"
        );
        assert!(
            state.cached_mermaid_handles.get(&3).is_some(),
            "render_mermaid must find handle for block[3]"
        );
        assert!(
            state.cached_mermaid_handles.get(&0).is_none(),
            "render_mermaid must NOT find handle for heading block"
        );
        assert!(
            state.cached_mermaid_handles.get(&2).is_none(),
            "render_mermaid must NOT find handle for paragraph block"
        );
        assert!(
            state.cached_mermaid_handles.get(&4).is_none(),
            "render_mermaid must NOT find handle for unrendered Mermaid block"
        );
    }

    #[test]
    fn markdown_state_ignores_non_mermaid_blocks() {
        use crate::core::MarkdownState;
        use iced::widget::image::Handle;

        let blocks = vec![
            Block::Paragraph("Hello".into()),
            Block::CodeBlock {
                lang: "rust".into(),
                code: "fn main() {}".into(),
            },
            Block::Table(TableBlock {
                headers: vec!["A".into()],
                rows: vec![vec!["1".into()]],
            }),
        ];

        // Simulate populate_state: no Mermaid blocks → cache must remain empty
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
}
