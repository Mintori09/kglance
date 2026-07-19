use std::path::Path;

use pulldown_cmark::{Event, Parser, Tag, TagEnd};

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

fn format_unicode_table(rows: &[Vec<String>]) -> String {
    if rows.is_empty() {
        return String::new();
    }

    let num_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if num_cols == 0 {
        return String::new();
    }

    let mut col_widths = vec![0; num_cols];
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < num_cols {
                col_widths[i] = col_widths[i].max(cell.chars().count());
            }
        }
    }

    let mut table_str = String::new();
    table_str.push('\n');

    // Top border
    let mut top = String::from("┌");
    for (i, &w) in col_widths.iter().enumerate() {
        top.push_str(&"─".repeat(w + 2));
        if i < num_cols - 1 {
            top.push('┬');
        }
    }
    top.push('┐');
    table_str.push_str(&format!("`{}`\n", top));

    // Rows
    for (r_idx, row) in rows.iter().enumerate() {
        let mut line = String::from("│");
        for (i, &w) in col_widths.iter().enumerate().take(num_cols) {
            let cell_text = row.get(i).cloned().unwrap_or_default();
            let pad_len = w - cell_text.chars().count();
            line.push(' ');
            line.push_str(&cell_text);
            line.push_str(&" ".repeat(pad_len));
            line.push(' ');
            line.push('│');
        }
        table_str.push_str(&format!("`{}`\n", line));

        if r_idx == 0 && rows.len() > 1 {
            // Header divider
            let mut div = String::from("├");
            for (i, &w) in col_widths.iter().enumerate() {
                div.push_str(&"─".repeat(w + 2));
                if i < num_cols - 1 {
                    div.push('┼');
                }
            }
            div.push('┤');
            table_str.push_str(&format!("`{}`\n", div));
        } else if r_idx < rows.len() - 1 {
            // Light row divider
            let mut div = String::from("├");
            for (i, &w) in col_widths.iter().enumerate() {
                div.push_str(&"─".repeat(w + 2));
                if i < num_cols - 1 {
                    div.push('┼');
                }
            }
            div.push('┤');
            table_str.push_str(&format!("`{}`\n", div));
        }
    }

    // Bottom border
    let mut bottom = String::from("└");
    for (i, &w) in col_widths.iter().enumerate() {
        bottom.push_str(&"─".repeat(w + 2));
        if i < num_cols - 1 {
            bottom.push('┴');
        }
    }
    bottom.push('┘');
    table_str.push_str(&format!("`{}`\n", bottom));

    table_str
}

pub fn convert_markdown_to_slint_markdown(content: &str) -> String {
    let mut result = String::new();
    let parser = pulldown_cmark::Parser::new_ext(content, pulldown_cmark::Options::all());

    struct TableState {
        rows: Vec<Vec<String>>,
        current_row: Vec<String>,
        current_cell: String,
    }
    let mut table_state: Option<TableState> = None;

    for event in parser {
        if let Some(ref mut state) = table_state {
            match event {
                pulldown_cmark::Event::Start(pulldown_cmark::Tag::TableCell) => {
                    state.current_cell.clear();
                }
                pulldown_cmark::Event::End(pulldown_cmark::TagEnd::TableCell) => {
                    state.current_row.push(state.current_cell.clone());
                }
                pulldown_cmark::Event::End(pulldown_cmark::TagEnd::TableRow) => {
                    state.rows.push(state.current_row.clone());
                    state.current_row.clear();
                }
                pulldown_cmark::Event::End(pulldown_cmark::TagEnd::Table) => {
                    let formatted_table = format_unicode_table(&state.rows);
                    result.push_str(&formatted_table);
                    table_state = None;
                }
                pulldown_cmark::Event::Text(text) => {
                    state.current_cell.push_str(&text);
                }
                pulldown_cmark::Event::Code(code) => {
                    state.current_cell.push('`');
                    state.current_cell.push_str(&code);
                    state.current_cell.push('`');
                }
                _ => {}
            }
            continue;
        }

        match event {
            pulldown_cmark::Event::Start(tag) => match tag {
                pulldown_cmark::Tag::Table(_) => {
                    table_state = Some(TableState {
                        rows: Vec::new(),
                        current_row: Vec::new(),
                        current_cell: String::new(),
                    });
                }
                pulldown_cmark::Tag::Heading { level, .. } => {
                    result.push_str("\n**");
                    let prefix = match level {
                        pulldown_cmark::HeadingLevel::H1 => "# ",
                        pulldown_cmark::HeadingLevel::H2 => "## ",
                        pulldown_cmark::HeadingLevel::H3 => "### ",
                        _ => "#### ",
                    };
                    result.push_str(prefix);
                }
                pulldown_cmark::Tag::Strong => {
                    result.push_str("**");
                }
                pulldown_cmark::Tag::Emphasis => {
                    result.push('*');
                }
                pulldown_cmark::Tag::Item => {
                    result.push_str("  • ");
                }
                pulldown_cmark::Tag::CodeBlock(_) => {
                    result.push('\n');
                }
                _ => {}
            },
            pulldown_cmark::Event::End(tag) => match tag {
                pulldown_cmark::TagEnd::Heading(_) => {
                    result.push_str("**\n");
                }
                pulldown_cmark::TagEnd::Strong => {
                    result.push_str("**");
                }
                pulldown_cmark::TagEnd::Emphasis => {
                    result.push('*');
                }
                pulldown_cmark::TagEnd::Item => {
                    result.push('\n');
                }
                pulldown_cmark::TagEnd::CodeBlock => {
                    result.push('\n');
                }
                pulldown_cmark::TagEnd::Paragraph => {
                    result.push_str("\n\n");
                }
                _ => {}
            },
            pulldown_cmark::Event::Text(text) => {
                result.push_str(&text);
            }
            pulldown_cmark::Event::Code(code) => {
                result.push('`');
                result.push_str(&code);
                result.push('`');
            }
            pulldown_cmark::Event::SoftBreak | pulldown_cmark::Event::HardBreak => {
                result.push('\n');
            }
            _ => {}
        }
    }
    result
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
    fn test_convert_markdown_to_slint_markdown_and_decode() {
        let md = "# Heading 1\n\nSome text with **bold** and *italic* and `inline code`.\n\n- Item 1\n- Item 2\n\n```rust\nfn main() {}\n```";
        let slint_md = convert_markdown_to_slint_markdown(md);

        let decoded = slint::StyledText::from_markdown(&slint_md);
        assert!(
            decoded.is_ok(),
            "Failed to decode generated Slint Markdown: {:?}",
            decoded.err()
        );
    }

    #[test]
    fn test_markdown_performance_3000_lines() {
        let mut large_content = String::new();
        for i in 1..=3000 {
            if i % 50 == 0 {
                large_content.push_str(&format!("## Heading Level 2 at line {}\n\n", i));
            } else if i % 10 == 0 {
                large_content
                    .push_str("This is **bold** text and *italic* text with `inline code`.\n\n");
            } else {
                large_content.push_str("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor.\n");
            }
        }

        let start_time = std::time::Instant::now();

        // 1. Parsing
        let parser = MarkdownParser::new();
        let mut tmp = tempfile::Builder::new().suffix(".md").tempfile().unwrap();
        write!(tmp, "{}", large_content).unwrap();
        let parsed = parser.parse(tmp.path()).unwrap();

        // 2. Slint Markdown formatting
        let raw_content = match parsed {
            ParsedContent::Markdown { content, .. } => content,
            _ => panic!("Expected Markdown variant"),
        };
        let formatted = convert_markdown_to_slint_markdown(&raw_content);

        // 3. Slint StyledText parsing
        let decoded = slint::StyledText::from_markdown(&formatted);
        assert!(decoded.is_ok());

        let duration = start_time.elapsed();
        println!(
            "Total 3000-line markdown processing pipeline time: {:?}",
            duration
        );

        let limit = if cfg!(debug_assertions) { 250 } else { 50 };
        assert!(
            duration.as_millis() < limit,
            "Performance threshold exceeded: {:?}",
            duration
        );
    }

    #[test]
    fn test_convert_markdown_table_to_unicode() {
        let md = "| H1 | H2 |\n|---|---|\n| C1 | C2 |";
        let slint_md = convert_markdown_to_slint_markdown(md);

        assert!(slint_md.contains('┌'));
        assert!(slint_md.contains("H1"));
        assert!(slint_md.contains("C2"));
        assert!(slint_md.contains('└'));

        let decoded = slint::StyledText::from_markdown(&slint_md);
        assert!(decoded.is_ok());
    }
}
