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
            Inline::Text(t) => {
                if matches!(mode, FlattenMode::Markdown) {
                    s.push_str(t);
                } else {
                    s.push_str(&strip_anchor_tags(t));
                }
            }
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
            Inline::FootnoteReference(label) => {
                s.push_str(&format!("[^{label}]"));
            }
            Inline::SoftBreak => s.push(' '),
        }
    }
    s
}

pub fn strip_anchor_tags(s: &str) -> String {
    if !s.contains("<a") && !s.contains("<A") && !s.contains("</a") && !s.contains("</A") {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let len = s.len();
    let mut i = 0;

    while i < len {
        if i + 4 <= len && s[i..i + 4].eq_ignore_ascii_case("</a>") {
            i += 4;
            continue;
        }
        if i + 3 <= len && s[i..i + 3].eq_ignore_ascii_case("<a>") {
            i += 3;
            continue;
        }
        if i + 3 <= len
            && (s[i..i + 3].eq_ignore_ascii_case("<a ")
                || s[i..i + 3].eq_ignore_ascii_case("<a\n")
                || s[i..i + 3].eq_ignore_ascii_case("<a\t"))
            && let Some(end_offset) = s[i..].find('>')
        {
            i += end_offset + 1;
            continue;
        }
        if let Some(ch) = s[i..].chars().next() {
            out.push(ch);
            i += ch.len_utf8();
        } else {
            break;
        }
    }
    out
}

pub fn strip_anchor_tags_cow<'a>(s: &'a str) -> std::borrow::Cow<'a, str> {
    if !s.contains("<a") && !s.contains("<A") && !s.contains("</a") && !s.contains("</A") {
        return std::borrow::Cow::Borrowed(s);
    }
    if is_empty_anchor_html(s) {
        return std::borrow::Cow::Borrowed("");
    }
    std::borrow::Cow::Owned(strip_anchor_tags(s))
}

pub fn is_empty_anchor_html(html: &str) -> bool {
    let trimmed = html.trim();

    let is_anchor = trimmed.eq_ignore_ascii_case("<a>")
        || trimmed.starts_with("<a ")
        || trimmed.starts_with("<a\n")
        || trimmed.starts_with("<a\t")
        || trimmed.starts_with("<A ")
        || trimmed.starts_with("<A\n")
        || trimmed.starts_with("<A\t")
        || trimmed.eq_ignore_ascii_case("</a>");

    if !is_anchor {
        return false;
    }

    if trimmed.eq_ignore_ascii_case("</a>") {
        return true;
    }

    if trimmed.ends_with("/>") {
        return true;
    }

    if trimmed.to_ascii_lowercase().ends_with("</a>") {
        let Some(open_end) = trimmed.find('>') else {
            return false;
        };
        let inner_start = open_end + 1;
        let inner_end = trimmed.len() - "</a>".len();
        return inner_start <= inner_end && trimmed[inner_start..inner_end].trim().is_empty();
    }

    trimmed.ends_with('>')
}

pub fn is_standalone_anchor_block(block: &super::Block) -> bool {
    match block {
        super::Block::Html(h) => is_empty_anchor_html(h),
        super::Block::Paragraph(content) => {
            let text = flatten_inlines(content);
            is_empty_anchor_html(&text) || strip_anchor_tags(&text).trim().is_empty()
        }
        _ => false,
    }
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
