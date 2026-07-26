use iced::Color;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrapMode {
    NoWrap,
    CharWrap(usize),
}

#[derive(Debug, Clone)]
pub struct LogicalLine {
    pub index: usize,
    pub text: String,
}

#[derive(Debug, Clone, Default)]
pub struct LogicalDocument {
    pub lines: Vec<LogicalLine>,
}

impl LogicalDocument {
    pub fn from_text(text: &str) -> Self {
        let lines = text
            .lines()
            .enumerate()
            .map(|(idx, line)| LogicalLine {
                index: idx + 1,
                text: line.to_string(),
            })
            .collect();
        Self { lines }
    }
}

#[derive(Debug, Clone)]
pub struct LayoutConfig {
    pub viewport_width: f32,
    pub font_size: f32,
    pub tab_width: u8,
    pub wrap_mode: WrapMode,
}

#[derive(Debug, Clone)]
pub struct HighlightedSpan {
    pub text: String,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub struct VisualLine {
    pub visual_index: usize,
    pub logical_line_index: usize,
    pub line_number: Option<usize>,
    pub text: String,
    pub spans: Vec<HighlightedSpan>,
}

#[derive(Debug, Clone, Default)]
pub struct VisualDocument {
    pub visual_lines: Vec<VisualLine>,
}

pub struct TextLayoutEngine;

impl TextLayoutEngine {
    pub fn compute(doc: &LogicalDocument, config: &LayoutConfig) -> VisualDocument {
        Self::compute_highlighted(doc, config, "txt", true)
    }

    pub fn compute_highlighted(
        doc: &LogicalDocument,
        config: &LayoutConfig,
        extension: &str,
        is_dark: bool,
    ) -> VisualDocument {
        let ps = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        let syntax = ps
            .find_syntax_by_extension(extension)
            .unwrap_or_else(|| ps.find_syntax_plain_text());
        let theme = if is_dark {
            &ts.themes["base16-mocha.dark"]
        } else {
            &ts.themes["InspiredGitHub"]
        };
        let mut highlighter = HighlightLines::new(syntax, theme);

        let default_color = if is_dark {
            Color::from_rgb(0.93, 0.94, 0.96)
        } else {
            Color::from_rgb(0.12, 0.13, 0.16)
        };

        let mut visual_lines = Vec::new();
        let mut v_idx = 0;

        for line in &doc.lines {
            let mut spans = Vec::new();
            if let Ok(ranges) = highlighter.highlight_line(&format!("{}\n", line.text), &ps) {
                for (style, text_str) in ranges {
                    let fg = style.foreground;
                    let color = if fg.a == 0 {
                        default_color
                    } else {
                        Color::from_rgb8(fg.r, fg.g, fg.b)
                    };
                    spans.push(HighlightedSpan {
                        text: text_str.trim_end_matches('\n').to_string(),
                        color,
                    });
                }
            } else {
                spans.push(HighlightedSpan {
                    text: line.text.clone(),
                    color: default_color,
                });
            }

            match config.wrap_mode {
                WrapMode::NoWrap => {
                    visual_lines.push(VisualLine {
                        visual_index: v_idx,
                        logical_line_index: line.index,
                        line_number: Some(line.index),
                        text: line.text.clone(),
                        spans,
                    });
                    v_idx += 1;
                }
                WrapMode::CharWrap(max_chars) => {
                    if max_chars == 0 || line.text.chars().count() <= max_chars {
                        visual_lines.push(VisualLine {
                            visual_index: v_idx,
                            logical_line_index: line.index,
                            line_number: Some(line.index),
                            text: line.text.clone(),
                            spans,
                        });
                        v_idx += 1;
                    } else {
                        let chars: Vec<char> = line.text.chars().collect();
                        let mut first = true;
                        for chunk in chars.chunks(max_chars) {
                            let chunk_text: String = chunk.iter().collect();
                            visual_lines.push(VisualLine {
                                visual_index: v_idx,
                                logical_line_index: line.index,
                                line_number: if first { Some(line.index) } else { None },
                                text: chunk_text.clone(),
                                spans: vec![HighlightedSpan {
                                    text: chunk_text,
                                    color: default_color,
                                }],
                            });
                            first = false;
                            v_idx += 1;
                        }
                    }
                }
            }
        }

        VisualDocument { visual_lines }
    }
}
