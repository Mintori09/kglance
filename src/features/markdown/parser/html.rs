use std::path::Path;

use crate::features::common::parser::html::decode_html_entities;
use crate::features::image::types::ImageRef;
use crate::parsers::markdown::Inline;

pub(super) fn extract_html_attribute(tag: &str, attr: &str) -> Option<String> {
    let lower_tag = tag.to_ascii_lowercase();
    let lower_attr = format!("{attr}=");
    let pos = lower_tag.find(&lower_attr)?;
    let remainder = &tag[pos + lower_attr.len()..];
    let quote_char = remainder.chars().next()?;
    if quote_char == '"' || quote_char == '\'' {
        let after_quote = &remainder[1..];
        let end_pos = after_quote.find(quote_char)?;
        Some(after_quote[..end_pos].to_string())
    } else {
        let end_pos = remainder
            .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
            .unwrap_or(remainder.len());
        Some(remainder[..end_pos].to_string())
    }
}

pub(super) fn decode_entity_if_present(s: &str) -> String {
    if s.contains('&') {
        decode_html_entities(s)
    } else {
        s.to_string()
    }
}

pub(super) fn handle_inline_html<F>(
    html: &str,
    result: &mut Vec<Inline>,
    parent: Option<&Path>,
    images: &mut Vec<ImageRef>,
    mut parse_until: F,
) where
    F: FnMut(&[&str]) -> Vec<Inline>,
{
    let trimmed = html.trim();
    let lower = trimmed.to_ascii_lowercase();

    if lower == "<br>" || lower == "<br/>" || lower == "<br />" {
        result.push(Inline::SoftBreak);
        return;
    }

    if lower.starts_with("<!--") && lower.ends_with("-->") {
        return;
    }

    if lower.starts_with("<b") && (lower == "<b>" || lower.starts_with("<b "))
        || lower.starts_with("<strong") && (lower == "<strong>" || lower.starts_with("<strong "))
    {
        let content = parse_until(&["</b>", "</strong>"]);
        result.push(Inline::Bold(content));
        return;
    }

    if lower.starts_with("<i") && (lower == "<i>" || lower.starts_with("<i "))
        || lower.starts_with("<em") && (lower == "<em>" || lower.starts_with("<em "))
    {
        let content = parse_until(&["</i>", "</em>"]);
        result.push(Inline::Italic(content));
        return;
    }

    if lower.starts_with("<del") && (lower == "<del>" || lower.starts_with("<del "))
        || lower.starts_with("<s") && (lower == "<s>" || lower.starts_with("<s "))
        || lower.starts_with("<strike") && (lower == "<strike>" || lower.starts_with("<strike "))
    {
        let content = parse_until(&["</del>", "</s>", "</strike>"]);
        result.push(Inline::Strikethrough(content));
        return;
    }

    if lower.starts_with("<code") && (lower == "<code>" || lower.starts_with("<code "))
        || lower.starts_with("<kbd") && (lower == "<kbd>" || lower.starts_with("<kbd "))
        || lower.starts_with("<tt") && (lower == "<tt>" || lower.starts_with("<tt "))
        || lower.starts_with("<samp") && (lower == "<samp>" || lower.starts_with("<samp "))
    {
        let content = parse_until(&["</code>", "</kbd>", "</tt>", "</samp>"]);
        let text = super::flatten::flatten_inlines_plain(&content);
        result.push(Inline::Code(text));
        return;
    }

    if lower.starts_with("<a") && (lower == "<a>" || lower.starts_with("<a ")) {
        let href = extract_html_attribute(trimmed, "href");
        let id = extract_html_attribute(trimmed, "id")
            .or_else(|| extract_html_attribute(trimmed, "name"));
        let content = parse_until(&["</a>"]);
        if let Some(url) = href {
            result.push(Inline::Link { text: content, url });
        } else if let Some(id_val) = id {
            result.push(Inline::Text(format!("<a id=\"{id_val}\"></a>")));
            if !content.is_empty() {
                result.extend(content);
            }
        } else if !content.is_empty() {
            result.extend(content);
        }
        return;
    }

    if lower.starts_with("<img") {
        let src = extract_html_attribute(trimmed, "src").unwrap_or_default();
        let alt = extract_html_attribute(trimmed, "alt").unwrap_or_default();
        if !src.is_empty() {
            if let Some(parent) = parent {
                let resolved = if src.starts_with('/') {
                    src.clone()
                } else {
                    parent.join(&src).to_string_lossy().to_string()
                };
                images.push(ImageRef {
                    alt_text: alt.clone(),
                    path: resolved,
                });
            }
            result.push(Inline::Image { alt, url: src });
        }
        return;
    }

    if lower == "</a>"
        || lower == "</b>"
        || lower == "</strong>"
        || lower == "</i>"
        || lower == "</em>"
        || lower == "</code>"
        || lower == "</kbd>"
        || lower == "</del>"
        || lower == "</s>"
        || lower == "</strike>"
        || lower == "</span>"
        || lower == "</div>"
        || lower == "</p>"
    {
        return;
    }

    if lower.starts_with("<span")
        || lower.starts_with("<div")
        || lower.starts_with("<p")
        || lower.starts_with("<font")
        || lower.starts_with("<mark")
        || lower.starts_with("<small")
    {
        return;
    }

    if trimmed.starts_with('&') && trimmed.ends_with(';') {
        let decoded = decode_html_entities(trimmed);
        result.push(Inline::Text(decoded));
        return;
    }

    result.push(Inline::Text(trimmed.to_string()));
}
