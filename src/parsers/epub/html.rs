use std::path::Path;

pub fn convert_html_to_markdown(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut skip_content = false;
    let mut chars = html.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '<' {
            let mut tag_content = String::new();
            while let Some(&c) = chars.peek() {
                if c == '>' {
                    break;
                }
                tag_content.push(chars.next().unwrap());
            }

            let tag_name = tag_content
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_start_matches('/')
                .to_lowercase();

            let is_close = tag_content.starts_with('/');

            if matches!(tag_name.as_str(), "style" | "script" | "head" | "title") {
                skip_content = !is_close;
            }

            if !skip_content {
                append_markdown_for_tag(&mut result, &tag_name, is_close, &tag_content);
            }

            for c in chars.by_ref() {
                if c == '>' {
                    break;
                }
            }
        } else if !skip_content {
            result.push(ch);
        }
    }

    let decoded = decode_html_entities(&result);
    let cleaned = decoded.replace("\\-", "-");

    cleaned
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn append_markdown_for_tag(result: &mut String, tag_name: &str, is_close: bool, tag_content: &str) {
    match tag_name {
        "h1" if !is_close => result.push_str("\n\n# "),
        "h2" if !is_close => result.push_str("\n\n## "),
        "h3" if !is_close => result.push_str("\n\n### "),
        "h4" if !is_close => result.push_str("\n\n#### "),
        "h5" if !is_close => result.push_str("\n\n##### "),
        "h6" if !is_close => result.push_str("\n\n###### "),
        "p" | "div" | "br" | "tr" => {
            if !result.is_empty() && !result.ends_with('\n') {
                result.push('\n');
            }
        }
        "blockquote" if !is_close => {
            if !result.is_empty() && !result.ends_with('\n') {
                result.push('\n');
            }
        }
        "li" if !is_close => {
            if !result.is_empty() && !result.ends_with('\n') {
                result.push_str("\n- ");
            } else {
                result.push_str("- ");
            }
        }
        "hr" => result.push_str("\n\n---\n\n"),
        "b" | "strong" => result.push_str("**"),
        "i" | "em" => result.push('*'),
        "code" => result.push('`'),
        "img" => {
            if let Some(src) = extract_attribute(tag_content, "src=\"") {
                let alt =
                    extract_attribute(tag_content, "alt=\"").unwrap_or_else(|| "image".into());
                result.push_str(&format!("\n\n![{alt}]({src})\n\n"));
            }
        }
        _ => {}
    }
}

pub fn strip_html_tags(html: &str) -> String {
    convert_html_to_markdown(html)
}

pub fn decode_html_entities(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut rest = text;

    while let Some(amp_idx) = rest.find('&') {
        result.push_str(&rest[..amp_idx]);
        rest = &rest[amp_idx..];

        if let Some(semi_idx) = rest.find(';') {
            let entity = &rest[1..semi_idx];
            let decoded = decode_named_or_numeric_entity(entity);

            if let Some(ch) = decoded {
                result.push(ch);
                rest = &rest[semi_idx + 1..];
                continue;
            }
        }

        result.push('&');
        rest = &rest[1..];
    }

    result.push_str(rest);
    result
}

fn decode_named_or_numeric_entity(entity: &str) -> Option<char> {
    if let Some(hex) = entity
        .strip_prefix("#x")
        .or_else(|| entity.strip_prefix("#X"))
    {
        u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
    } else if let Some(dec) = entity.strip_prefix('#') {
        dec.parse::<u32>().ok().and_then(char::from_u32)
    } else {
        match entity {
            "quot" => Some('"'),
            "apos" => Some('\''),
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "nbsp" => Some(' '),
            "hellip" => Some('…'),
            "mdash" => Some('—'),
            "ndash" => Some('–'),
            "ldquo" | "rdquo" => Some('"'),
            "lsquo" | "rsquo" => Some('\''),
            _ => None,
        }
    }
}

pub fn extract_tag_content(xml: &str, tag_name: &str) -> Option<String> {
    let lower_xml = xml.to_lowercase();
    let start_tag = format!("<{}", tag_name.to_lowercase());
    let close_tag = format!("</{}>", tag_name.to_lowercase());

    let start_idx = lower_xml.find(&start_tag)?;
    let content_start = lower_xml[start_idx..].find('>')? + start_idx + 1;
    let end_idx = lower_xml[content_start..].find(&close_tag)? + content_start;

    let raw = xml[content_start..end_idx].trim();
    let stripped = strip_html_tags(raw);
    let decoded = decode_html_entities(&stripped);

    if decoded.is_empty() {
        None
    } else {
        Some(decoded)
    }
}

pub fn extract_attribute(text: &str, attr_prefix: &str) -> Option<String> {
    let idx = text.find(attr_prefix)?;
    let start = idx + attr_prefix.len();
    let end = text[start..].find('"')?;
    Some(text[start..start + end].to_string())
}

pub fn extract_first_paragraph_snippet(html: &str) -> Option<String> {
    let mut search_str = html;

    while let Some(start_idx) = search_str.find("<p") {
        let tag_end = search_str[start_idx..].find('>')? + start_idx + 1;
        let close_idx = search_str[tag_end..].find("</p>")? + tag_end;

        let raw_paragraph = &search_str[tag_end..close_idx];
        let stripped = strip_html_tags(raw_paragraph);
        let cleaned = decode_html_entities(&stripped);
        let trimmed = cleaned.trim();

        if !trimmed.is_empty() {
            let snippet = if trimmed.chars().count() > 40 {
                format!("{}...", trimmed.chars().take(40).collect::<String>())
            } else {
                trimmed.to_string()
            };
            return Some(snippet);
        }
        search_str = &search_str[close_idx + 4..];
    }
    None
}

pub fn extract_chapter_title_from_html(html: &str, book_title: &str) -> Option<String> {
    extract_tag_content(html, "h1")
        .or_else(|| extract_tag_content(html, "h2"))
        .or_else(|| extract_tag_content(html, "h3"))
        .or_else(|| {
            let title = extract_tag_content(html, "title")?;
            if title != book_title {
                Some(title)
            } else {
                None
            }
        })
        .or_else(|| extract_first_paragraph_snippet(html))
}

pub fn extract_filename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
        .to_string()
}
