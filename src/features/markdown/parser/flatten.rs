use super::Inline;

#[derive(Clone, Copy)]
enum FlattenMode {
    Markdown,
    Visual,
    Plain,
    Toc,
}

pub fn flatten_inlines(inlines: &[Inline]) -> String {
    flatten_inlines_with(inlines, FlattenMode::Markdown)
}

pub fn flatten_inlines_visual(inlines: &[Inline]) -> String {
    flatten_inlines_with(inlines, FlattenMode::Visual)
}

#[allow(dead_code)]
pub fn flatten_inlines_plain(inlines: &[Inline]) -> String {
    flatten_inlines_with(inlines, FlattenMode::Plain)
}

pub fn flatten_inlines_toc(inlines: &[Inline]) -> String {
    flatten_inlines_with(inlines, FlattenMode::Toc)
}

fn flatten_inlines_with(inlines: &[Inline], mode: FlattenMode) -> String {
    let mut s = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(t) => s.push_str(t),
            Inline::Bold(c) => flatten_emphasis(&mut s, c, "**", mode),
            Inline::Italic(c) => flatten_emphasis(&mut s, c, "_", mode),
            Inline::Strikethrough(c) => flatten_emphasis(&mut s, c, "~~", mode),
            Inline::Code(t) => {
                if matches!(mode, FlattenMode::Markdown | FlattenMode::Toc) {
                    s.push('`');
                    s.push_str(t);
                    s.push('`');
                } else {
                    s.push_str(t);
                }
            }
            Inline::Link { text, url } => {
                if matches!(mode, FlattenMode::Markdown) {
                    s.push_str(&format!(
                        "[{}]({url})",
                        flatten_inlines_with(text, FlattenMode::Markdown)
                    ));
                } else {
                    s.push_str(&flatten_inlines_with(
                        text,
                        if matches!(mode, FlattenMode::Toc) {
                            FlattenMode::Markdown
                        } else {
                            mode
                        },
                    ));
                }
            }
            Inline::Image { alt, url } => match mode {
                FlattenMode::Markdown => s.push_str(&format!("![{alt}]({url})")),
                FlattenMode::Visual => {
                    s.push('[');
                    s.push_str(alt);
                    s.push(']');
                }
                FlattenMode::Plain | FlattenMode::Toc => s.push_str(alt),
            },
            Inline::InlineMath(latex) | Inline::DisplayMath(latex) => match mode {
                FlattenMode::Visual | FlattenMode::Plain => {
                    s.push_str(&crate::features::markdown::view::components::inline_spans::render_latex_to_text(latex));
                }
                _ => s.push_str(latex),
            },
            Inline::SoftBreak => s.push(' '),
        }
    }
    s
}

fn flatten_emphasis(s: &mut String, content: &[Inline], marker: &str, mode: FlattenMode) {
    let inner = flatten_inlines_with(
        content,
        if matches!(mode, FlattenMode::Markdown | FlattenMode::Toc) {
            FlattenMode::Markdown
        } else {
            mode
        },
    );
    if matches!(mode, FlattenMode::Markdown) {
        s.push_str(marker);
        s.push_str(&inner);
        s.push_str(marker);
    } else {
        s.push_str(&inner);
    }
}
