use std::collections::HashMap;
use std::io::{Read, Seek};

use crate::features::epub::parser::image::extract_filename;
use crate::features::epub::parser::ncx::decode_url_component;
use crate::features::epub::parser::opf::resolve_relative_path;

/// Normalizes a comment extracted from an SVG into LaTeX math delimiters.
/// - `$X \sim P$` -> `$X \sim P$`
/// - `$$X \sim P$$` -> `$$X \sim P$$`
/// - `\[...\]` -> `$$...$$`
/// - `\(...\)` -> `$...$`
/// - `\begin{aligned}...` -> `$$\begin{aligned}...$$`
pub fn normalize_latex_comment(comment: &str) -> Option<String> {
    let trimmed = comment.trim();
    if trimmed.is_empty() {
        return None;
    }
    if (trimmed.starts_with("$$") && trimmed.ends_with("$$") && trimmed.len() >= 4)
        || (trimmed.starts_with('$') && trimmed.ends_with('$') && trimmed.len() >= 2)
    {
        return Some(trimmed.to_string());
    }
    if trimmed.starts_with("\\[") && trimmed.ends_with("\\]") && trimmed.len() >= 4 {
        let inner = trimmed[2..trimmed.len() - 2].trim();
        return Some(format!("$${inner}$$"));
    }
    if trimmed.starts_with("\\(") && trimmed.ends_with("\\)") && trimmed.len() >= 4 {
        let inner = trimmed[2..trimmed.len() - 2].trim();
        return Some(format!("${inner}$"));
    }
    if trimmed.starts_with("\\begin{") {
        return Some(format!("$${trimmed}$$"));
    }
    None
}

/// Extracts a LaTeX formula from XML comments inside an SVG string.
pub fn extract_latex_from_svg(svg_content: &str) -> Option<String> {
    let mut search_idx = 0;
    while let Some(start) = svg_content[search_idx..].find("<!--") {
        let comment_start = search_idx + start + 4;
        if let Some(end) = svg_content[comment_start..].find("-->") {
            let comment_end = comment_start + end;
            let comment = &svg_content[comment_start..comment_end];
            if let Some(latex) = normalize_latex_comment(comment) {
                return Some(latex);
            }
            search_idx = comment_end + 3;
        } else {
            break;
        }
    }
    None
}

/// Extracts an attribute value from an HTML tag string.
fn extract_attr_value<'a>(tag: &'a str, attr_name: &str) -> Option<&'a str> {
    let mut search = 0;
    while let Some(pos) = tag[search..].find(attr_name) {
        let abs_pos = search + pos;
        let is_word_boundary = abs_pos == 0 || tag.as_bytes()[abs_pos - 1].is_ascii_whitespace();
        let after = &tag[abs_pos + attr_name.len()..];
        let trimmed_after = after.trim_start();
        if is_word_boundary && trimmed_after.starts_with('=') {
            let val_part = trimmed_after[1..].trim_start();
            if let Some(quote) = val_part.chars().next()
                && (quote == '"' || quote == '\'')
            {
                let rest = &val_part[1..];
                if let Some(end_quote) = rest.find(quote) {
                    return Some(&rest[..end_quote]);
                }
            }
        }
        search = abs_pos + attr_name.len();
    }
    None
}

/// Replaces math <img> tags in HTML with their LaTeX representation.
pub fn replace_math_images_in_html<F>(html: &str, mut latex_resolver: F) -> String
where
    F: FnMut(&str) -> Option<String>,
{
    let mut result = String::with_capacity(html.len());
    let mut search_idx = 0;

    while let Some(lt_pos) = html[search_idx..].find('<') {
        let abs_lt = search_idx + lt_pos;
        result.push_str(&html[search_idx..abs_lt]);

        let rest = &html[abs_lt..];
        if rest.starts_with("<!--")
            && let Some(comment_end) = rest.find("-->")
        {
            let end_pos = abs_lt + comment_end + 3;
            result.push_str(&html[abs_lt..end_pos]);
            search_idx = end_pos;
            continue;
        }

        if rest.len() >= 4
            && rest[1..4].eq_ignore_ascii_case("img")
            && (rest.as_bytes()[4].is_ascii_whitespace()
                || rest.as_bytes()[4] == b'/'
                || rest.as_bytes()[4] == b'>')
        {
            // Find closing '>' taking quotes into account
            let mut tag_end = None;
            let mut in_quote: Option<u8> = None;
            let bytes = rest.as_bytes();
            for (i, &b) in bytes.iter().enumerate().skip(4) {
                if let Some(q) = in_quote {
                    if b == q {
                        in_quote = None;
                    }
                } else if b == b'"' || b == b'\'' {
                    in_quote = Some(b);
                } else if b == b'>' {
                    tag_end = Some(abs_lt + i);
                    break;
                }
            }

            if let Some(end_pos) = tag_end {
                let tag_str = &html[abs_lt..=end_pos];
                let class_attr = extract_attr_value(tag_str, "class");
                let src_attr = extract_attr_value(tag_str, "src");
                let alt_attr = extract_attr_value(tag_str, "alt");

                let is_math_class = class_attr.is_some_and(|c| {
                    c.contains("math-image") || c.split_whitespace().any(|cls| cls == "math")
                });

                let mut replaced_latex = None;
                if let Some(src) = src_attr
                    && (is_math_class || src.ends_with(".svg"))
                {
                    replaced_latex = latex_resolver(src);
                }

                if replaced_latex.is_none()
                    && is_math_class
                    && let Some(alt) = alt_attr
                {
                    replaced_latex = normalize_latex_comment(alt);
                }

                if let Some(latex) = replaced_latex {
                    result.push_str(&latex);
                } else {
                    result.push_str(tag_str);
                }

                search_idx = end_pos + 1;
                continue;
            }
        }

        result.push('<');
        search_idx = abs_lt + 1;
    }

    result.push_str(&html[search_idx..]);
    result
}

fn read_svg_from_archive<R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
    resolved_path: &str,
    src: &str,
) -> Option<String> {
    if let Ok(mut file) = archive.by_name(resolved_path) {
        let mut bytes = Vec::new();
        if file.read_to_end(&mut bytes).is_ok() {
            return Some(String::from_utf8_lossy(&bytes).to_string());
        }
    }

    let decoded = decode_url_component(resolved_path);
    if decoded != resolved_path
        && let Ok(mut file) = archive.by_name(&decoded)
    {
        let mut bytes = Vec::new();
        if file.read_to_end(&mut bytes).is_ok() {
            return Some(String::from_utf8_lossy(&bytes).to_string());
        }
    }

    let filename = extract_filename(src);
    let matched_entry = archive
        .file_names()
        .flatten()
        .find(|name| name.as_ref() == filename || name.ends_with(&format!("/{filename}")))
        .map(|name| name.to_string());

    if let Some(entry_name) = matched_entry
        && let Ok(mut file) = archive.by_name(&entry_name)
    {
        let mut bytes = Vec::new();
        if file.read_to_end(&mut bytes).is_ok() {
            return Some(String::from_utf8_lossy(&bytes).to_string());
        }
    }

    None
}

/// Scans HTML for math images, reads their SVG definitions from archive,
/// and replaces `<img class="math-image" ...>` with LaTeX formulas.
pub fn process_chapter_math_in_html<R: Read + Seek>(
    html: &str,
    archive: &mut zip::ZipArchive<R>,
    base_chapter_path: &str,
) -> String {
    let mut latex_map: HashMap<String, String> = HashMap::new();

    // Collect candidate math image sources
    let mut search_idx = 0;
    while let Some(pos) = html[search_idx..].find("<img") {
        let abs_pos = search_idx + pos;
        let rest = &html[abs_pos..];
        let mut tag_end = None;
        let mut in_quote: Option<u8> = None;
        let bytes = rest.as_bytes();
        for (i, &b) in bytes.iter().enumerate().skip(4) {
            if let Some(q) = in_quote {
                if b == q {
                    in_quote = None;
                }
            } else if b == b'"' || b == b'\'' {
                in_quote = Some(b);
            } else if b == b'>' {
                tag_end = Some(abs_pos + i);
                break;
            }
        }

        if let Some(end_pos) = tag_end {
            let tag_str = &html[abs_pos..=end_pos];
            let class_attr = extract_attr_value(tag_str, "class");
            let src_attr = extract_attr_value(tag_str, "src");

            let is_math = class_attr.is_some_and(|c| {
                c.contains("math-image") || c.split_whitespace().any(|cls| cls == "math")
            }) || src_attr
                .is_some_and(|s| s.contains("math_") && s.ends_with(".svg"));

            if is_math
                && let Some(src) = src_attr
                && !latex_map.contains_key(src)
            {
                let resolved_path = resolve_relative_path(base_chapter_path, src);
                if let Some(svg_content) = read_svg_from_archive(archive, &resolved_path, src)
                    && let Some(latex) = extract_latex_from_svg(&svg_content)
                {
                    latex_map.insert(src.to_string(), latex);
                }
            }
            search_idx = end_pos + 1;
        } else {
            search_idx = abs_pos + 4;
        }
    }

    if latex_map.is_empty() {
        return html.to_string();
    }

    replace_math_images_in_html(html, |src| latex_map.get(src).cloned())
}
