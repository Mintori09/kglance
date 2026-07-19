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

    for event in Parser::new(raw) {
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
}
