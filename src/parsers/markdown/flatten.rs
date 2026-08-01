use super::Inline;

pub fn flatten_inlines(inlines: &[Inline]) -> String {
    let mut s = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(t) => s.push_str(t),
            Inline::Bold(c) => {
                s.push_str("**");
                s.push_str(&flatten_inlines(c));
                s.push_str("**");
            }
            Inline::Italic(c) => {
                s.push('_');
                s.push_str(&flatten_inlines(c));
                s.push('_');
            }
            Inline::Strikethrough(c) => {
                s.push_str("~~");
                s.push_str(&flatten_inlines(c));
                s.push_str("~~");
            }
            Inline::Code(t) => {
                s.push('`');
                s.push_str(t);
                s.push('`');
            }
            Inline::Link { text, url } => {
                s.push_str(&format!("[{}]({url})", flatten_inlines(text)));
            }
            Inline::Image { alt, url } => {
                s.push_str(&format!("![{alt}]({url})"));
            }
            Inline::InlineMath(latex) => s.push_str(latex),
            Inline::DisplayMath(latex) => s.push_str(latex),
            Inline::SoftBreak => s.push(' '),
        }
    }
    s
}

pub fn flatten_inlines_visual(inlines: &[Inline]) -> String {
    let mut s = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(t) => s.push_str(t),
            Inline::Bold(c) => s.push_str(&flatten_inlines_visual(c)),
            Inline::Italic(c) => s.push_str(&flatten_inlines_visual(c)),
            Inline::Strikethrough(c) => s.push_str(&flatten_inlines_visual(c)),
            Inline::Code(t) => s.push_str(t),
            Inline::Link { text, .. } => s.push_str(&flatten_inlines_visual(text)),
            Inline::Image { alt, .. } => {
                s.push('[');
                s.push_str(alt);
                s.push(']');
            }
            Inline::InlineMath(latex) => s.push_str(latex),
            Inline::DisplayMath(latex) => s.push_str(latex),
            Inline::SoftBreak => s.push(' '),
        }
    }
    s
}

#[allow(dead_code)]
pub fn flatten_inlines_plain(inlines: &[Inline]) -> String {
    let mut s = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(t) => s.push_str(t),
            Inline::Bold(c) => s.push_str(&flatten_inlines_plain(c)),
            Inline::Italic(c) => s.push_str(&flatten_inlines_plain(c)),
            Inline::Strikethrough(c) => s.push_str(&flatten_inlines_plain(c)),
            Inline::Code(t) => s.push_str(t),
            Inline::Link { text, .. } => s.push_str(&flatten_inlines_plain(text)),
            Inline::Image { alt, .. } => s.push_str(alt),
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
