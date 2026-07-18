use std::path::Path;

use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::parser::{ImageRef, ParsedContent, ParseError, PreviewParser};

pub struct MarkdownParser {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

#[allow(dead_code)]
impl MarkdownParser {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    fn highlight_code(&self, lang: Option<&str>, code: &str) -> String {
        let syntax = lang
            .and_then(|l| self.syntax_set.find_syntax_by_token(l))
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let theme = &self.theme_set.themes["InspiredGitHub"];

        let mut highlighter = HighlightLines::new(syntax, theme);
        let mut highlighted = String::new();
        for line in LinesWithEndings::from(code) {
            let ranges = highlighter
                .highlight_line(line, &self.syntax_set)
                .unwrap_or_default();
            for (style, text) in &ranges {
                let color = style.foreground;
                let hex = if color.a == 0 {
                    String::from("inherit")
                } else {
                    format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b)
                };
                if hex != "inherit" {
                    highlighted.push_str(&format!("<font color='{}'>{}</font>", hex, text));
                } else {
                    highlighted.push_str(text);
                }
            }
        }

        highlighted
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
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
        let raw = std::fs::read_to_string(path)
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        let mut output = String::new();
        let mut images: Vec<ImageRef> = Vec::new();
        let mut in_code_block = false;
        let mut code_block_lang: Option<String> = None;
        let mut code_block_text = String::new();
        let mut image_alt = String::new();
        let mut image_url = String::new();

        let parser = Parser::new(&raw);
        for event in parser {
            match event {
                Event::Start(Tag::CodeBlock(kind)) => {
                    in_code_block = true;
                    code_block_text.clear();
                    code_block_lang = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(info) => {
                            info.split_whitespace().next().map(|s| s.to_string())
                        }
                        pulldown_cmark::CodeBlockKind::Indented => None,
                    };
                }
                Event::End(TagEnd::CodeBlock) => {
                    in_code_block = false;
                    let lang_str = code_block_lang.as_deref();
                    let highlighted = self.highlight_code(lang_str, &code_block_text);
                    output.push_str(&format!("\n```{}\n{}\n```\n",
                        code_block_lang.as_deref().unwrap_or(""),
                        highlighted));
                }
                Event::Text(text) => {
                    if in_code_block {
                        code_block_text.push_str(&text);
                    } else {
                        output.push_str(&text);
                    }
                }
                Event::Start(Tag::Image { link_type: _, dest_url, title: _, .. }) => {
                    image_url = dest_url.to_string();
                    image_alt.clear();
                }
                Event::End(TagEnd::Image) => {
                    let resolved = if image_url.starts_with('/') {
                        image_url.clone()
                    } else {
                        parent.join(&image_url).to_string_lossy().to_string()
                    };
                    images.push(ImageRef {
                        alt_text: image_alt.clone(),
                        path: resolved,
                    });
                    output.push_str(&format!("![{}]({})", image_alt, image_url));
                }
                Event::End(TagEnd::Paragraph) => {
                    output.push('\n');
                }
                Event::SoftBreak => {
                    output.push(' ');
                }
                Event::HardBreak => {
                    output.push('\n');
                }
                _ => {}
            }
        }

        Ok(ParsedContent::Markdown {
            content: output,
            images,
        })
    }
}
