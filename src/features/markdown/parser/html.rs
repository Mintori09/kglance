use std::path::Path;

use crate::features::common::parser::html::decode_html_entities;
use crate::features::image::types::ImageRef;
use crate::parsers::markdown::Inline;

pub(crate) fn extract_html_attribute(tag: &str, attr: &str) -> Option<String> {
    let lower_tag = tag.to_ascii_lowercase();
    let lower_attr = attr.to_ascii_lowercase();
    let mut search_start = 0;
    while let Some(attr_idx) = lower_tag[search_start..].find(&lower_attr) {
        let abs_idx = search_start + attr_idx;
        let valid_prefix = abs_idx == 0
            || lower_tag.as_bytes()[abs_idx - 1].is_ascii_whitespace()
            || lower_tag.as_bytes()[abs_idx - 1] == b'<'
            || lower_tag.as_bytes()[abs_idx - 1] == b'/';
        if !valid_prefix {
            search_start = abs_idx + lower_attr.len();
            continue;
        }

        let after_attr = &lower_tag[abs_idx + lower_attr.len()..];
        let trimmed_after = after_attr.trim_start();
        if !trimmed_after.starts_with('=') {
            search_start = abs_idx + lower_attr.len();
            continue;
        }

        let eq_idx_in_remainder = after_attr.find('=')?;
        let val_slice_lower = &after_attr[eq_idx_in_remainder + 1..];
        let leading_spaces = val_slice_lower.len() - val_slice_lower.trim_start().len();

        let val_start_abs = abs_idx + lower_attr.len() + eq_idx_in_remainder + 1 + leading_spaces;
        let remainder = &tag[val_start_abs..];
        let quote_char = remainder.chars().next()?;
        if quote_char == '"' || quote_char == '\'' {
            let after_quote = &remainder[1..];
            let end_pos = after_quote.find(quote_char)?;
            return Some(after_quote[..end_pos].to_string());
        } else {
            let end_pos = remainder
                .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
                .unwrap_or(remainder.len());
            return Some(remainder[..end_pos].to_string());
        }
    }
    None
}

pub(crate) fn parse_dimension_str(val: &str) -> Option<u32> {
    let trimmed = val.trim();
    let num_str = trimmed
        .strip_suffix("px")
        .or_else(|| trimmed.strip_suffix("pt"))
        .unwrap_or(trimmed)
        .trim();
    num_str.parse::<f32>().ok().and_then(|f| {
        if f > 0.0 && f <= 10000.0 {
            Some(f.round() as u32)
        } else {
            None
        }
    })
}

pub(crate) fn extract_css_property(style: &str, prop: &str) -> Option<String> {
    for decl in style.split(';') {
        if let Some((k, v)) = decl.split_once(':')
            && k.trim().eq_ignore_ascii_case(prop)
        {
            return Some(v.trim().to_string());
        }
    }
    None
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
        let width = extract_html_attribute(trimmed, "width")
            .as_deref()
            .and_then(parse_dimension_str)
            .or_else(|| {
                extract_html_attribute(trimmed, "style")
                    .as_deref()
                    .and_then(|s| extract_css_property(s, "width"))
                    .as_deref()
                    .and_then(parse_dimension_str)
            });
        let height = extract_html_attribute(trimmed, "height")
            .as_deref()
            .and_then(parse_dimension_str)
            .or_else(|| {
                extract_html_attribute(trimmed, "style")
                    .as_deref()
                    .and_then(|s| extract_css_property(s, "height"))
                    .as_deref()
                    .and_then(parse_dimension_str)
            });

        if !src.is_empty() {
            let is_remote = crate::core::net::is_remote_url(&src);
            let resolved = if is_remote || src.starts_with('/') {
                src.clone()
            } else if let Some(parent) = parent {
                parent.join(&src).to_string_lossy().to_string()
            } else {
                src.clone()
            };

            images.push(ImageRef {
                alt_text: alt.clone(),
                path: resolved.clone(),
            });

            result.push(Inline::Image {
                alt,
                url: resolved,
                width,
                height,
            });
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
        || lower.starts_with("</ruby")
        || lower.starts_with("</rt")
        || lower.starts_with("</rp")
        || lower.starts_with("</rb")
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

pub(super) fn parse_ruby_string(raw: &str) -> Vec<Inline> {
    let mut results = Vec::new();
    let mut current_base = String::new();
    let mut current_ruby = String::new();
    let mut in_rt = false;
    let mut in_rp = false;

    let flush_pair = |base: &mut String, ruby: &mut String, res: &mut Vec<Inline>| {
        let b_trimmed = base.trim();
        let r_trimmed = ruby.trim();
        if !b_trimmed.is_empty() && !r_trimmed.is_empty() {
            res.push(Inline::Ruby {
                text: decode_entity_if_present(b_trimmed),
                ruby: decode_entity_if_present(r_trimmed),
            });
        } else if !b_trimmed.is_empty() {
            let decoded = decode_entity_if_present(b_trimmed);
            let mut text = String::new();
            if base.starts_with(char::is_whitespace) {
                text.push(' ');
            }
            text.push_str(&decoded);
            if base.ends_with(char::is_whitespace) {
                text.push(' ');
            }
            res.push(Inline::Text(text));
        }
        base.clear();
        ruby.clear();
    };

    let mut i = 0;
    let bytes = raw.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'<' {
            if let Some(tag_end) = raw[i..].find('>') {
                let tag = &raw[i..i + tag_end + 1];
                let tag_lower = tag.to_ascii_lowercase();
                if tag_lower.starts_with("<ruby") || tag_lower.starts_with("</ruby") {
                    flush_pair(&mut current_base, &mut current_ruby, &mut results);
                    in_rt = false;
                    in_rp = false;
                } else if tag_lower.starts_with("<rt") {
                    in_rt = true;
                    in_rp = false;
                } else if tag_lower.starts_with("</rt") {
                    in_rt = false;
                } else if tag_lower.starts_with("<rp") {
                    in_rp = true;
                } else if tag_lower.starts_with("</rp") {
                    in_rp = false;
                } else if tag_lower.starts_with("<rb") {
                    if !current_ruby.is_empty() {
                        flush_pair(&mut current_base, &mut current_ruby, &mut results);
                    }
                    in_rt = false;
                    in_rp = false;
                }
                i += tag_end + 1;
                continue;
            } else {
                // Unterminated tag, advance
            }
        }

        let ch = raw[i..].chars().next().unwrap_or(' ');
        if !in_rp {
            if in_rt {
                current_ruby.push(ch);
            } else {
                if !current_ruby.is_empty() {
                    flush_pair(&mut current_base, &mut current_ruby, &mut results);
                }
                current_base.push(ch);
            }
        }
        i += ch.len_utf8();
    }

    flush_pair(&mut current_base, &mut current_ruby, &mut results);

    results
}
