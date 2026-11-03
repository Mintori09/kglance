use std::path::Path;

use memchr::{memchr, memmem};
use quick_xml::events::Event;
use quick_xml::reader::Reader;

use super::converter::strip_html_tags;
use super::entities::{decode_html_entities, normalize_nfc};

pub fn extract_tag_content(xml: &str, tag_name: &str) -> Option<String> {
    let clean_tag = tag_name
        .trim()
        .trim_start_matches('<')
        .trim_end_matches('>');
    if clean_tag.is_empty() {
        return None;
    }
    let tag_bytes = clean_tag.as_bytes();
    let xml_bytes = xml.as_bytes();

    let mut pos = 0;
    while pos < xml_bytes.len() {
        let rel_lt = memchr(b'<', &xml_bytes[pos..])?;
        let lt_pos = pos + rel_lt;

        // Skip comments: <!-- ... -->
        if xml_bytes[lt_pos..].starts_with(b"<!--") {
            let rel_comment_end = memmem::find(&xml_bytes[lt_pos + 4..], b"-->")?;
            pos = lt_pos + 4 + rel_comment_end + 3;
            continue;
        }

        // Skip CDATA: <![CDATA[ ... ]]>
        if xml_bytes[lt_pos..].starts_with(b"<![CDATA[") {
            let rel_cdata_end = memmem::find(&xml_bytes[lt_pos + 9..], b"]]>")?;
            pos = lt_pos + 9 + rel_cdata_end + 3;
            continue;
        }

        // Skip closing tags, DOCTYPE, processing instructions (</, <!, <?)
        let next_pos = lt_pos + 1;
        if next_pos < xml_bytes.len() {
            let next_b = xml_bytes[next_pos];
            if next_b == b'/' || next_b == b'!' || next_b == b'?' {
                let rel_gt = memchr(b'>', &xml_bytes[next_pos..])?;
                pos = next_pos + rel_gt + 1;
                continue;
            }
        } else {
            return None;
        }

        // Check if tag name matches case-insensitively
        let after_tag = next_pos + tag_bytes.len();
        if xml_bytes.len() < after_tag {
            return None;
        }

        if !xml_bytes[next_pos..after_tag].eq_ignore_ascii_case(tag_bytes) {
            pos = next_pos;
            continue;
        }

        // Ensure the tag name is followed by a valid delimiter: '>', '/', or whitespace
        if after_tag < xml_bytes.len() {
            let delim = xml_bytes[after_tag];
            if delim != b'>' && delim != b'/' && !delim.is_ascii_whitespace() {
                pos = next_pos;
                continue;
            }
        }

        // Find the end of this start tag '>' (taking quotes into account)
        let mut scan = after_tag;
        let mut quote: Option<u8> = None;
        let mut tag_close_gt: Option<usize> = None;

        while scan < xml_bytes.len() {
            let b = xml_bytes[scan];
            match b {
                b'"' | b'\'' => {
                    if quote == Some(b) {
                        quote = None;
                    } else if quote.is_none() {
                        quote = Some(b);
                    }
                }
                b'>' if quote.is_none() => {
                    tag_close_gt = Some(scan);
                    break;
                }
                b'<' if quote.is_none() => {
                    break;
                }
                _ => {}
            }
            scan += 1;
        }

        let Some(gt_pos) = tag_close_gt else {
            pos = next_pos;
            continue;
        };

        // Check if self-closing tag, e.g. <dc:title/> or <dc:title id="x" />
        let mut prev = gt_pos;
        while prev > after_tag && xml_bytes[prev - 1].is_ascii_whitespace() {
            prev -= 1;
        }
        if prev > after_tag && xml_bytes[prev - 1] == b'/' {
            pos = gt_pos + 1;
            continue;
        }

        let content_start = gt_pos + 1;

        // Find the matching close tag </clean_tag ...>
        let mut close_scan = content_start;
        let mut end_idx: Option<usize> = None;
        let mut close_tag_end: Option<usize> = None;

        while close_scan < xml_bytes.len() {
            let Some(rel_close_lt) = memchr(b'<', &xml_bytes[close_scan..]) else {
                break;
            };
            let close_lt = close_scan + rel_close_lt;

            // Skip comments inside content
            if xml_bytes[close_lt..].starts_with(b"<!--")
                && let Some(rel_end) = memmem::find(&xml_bytes[close_lt + 4..], b"-->")
            {
                close_scan = close_lt + 4 + rel_end + 3;
                continue;
            }

            // Skip CDATA inside content
            if xml_bytes[close_lt..].starts_with(b"<![CDATA[")
                && let Some(rel_end) = memmem::find(&xml_bytes[close_lt + 9..], b"]]>")
            {
                close_scan = close_lt + 9 + rel_end + 3;
                continue;
            }

            // Check for closing tag: </clean_tag
            if close_lt + 1 < xml_bytes.len() && xml_bytes[close_lt + 1] == b'/' {
                let close_tag_start = close_lt + 2;
                let close_tag_end_candidate = close_tag_start + tag_bytes.len();
                if xml_bytes.len() >= close_tag_end_candidate
                    && xml_bytes[close_tag_start..close_tag_end_candidate]
                        .eq_ignore_ascii_case(tag_bytes)
                {
                    let mut after_close = close_tag_end_candidate;
                    while after_close < xml_bytes.len()
                        && xml_bytes[after_close].is_ascii_whitespace()
                    {
                        after_close += 1;
                    }
                    if after_close < xml_bytes.len() && xml_bytes[after_close] == b'>' {
                        end_idx = Some(close_lt);
                        close_tag_end = Some(after_close + 1);
                        break;
                    }
                }
            }

            close_scan = close_lt + 1;
        }

        let end_idx = end_idx?;

        let raw = xml[content_start..end_idx].trim();
        let stripped = strip_html_tags(raw);
        let decoded = decode_html_entities(&stripped);

        if !decoded.is_empty() {
            return Some(decoded);
        }

        pos = close_tag_end.unwrap_or(end_idx + 1);
    }

    None
}

pub fn extract_attribute(text: &str, attr_prefix: &str) -> Option<String> {
    let idx = text.find(attr_prefix)?;
    let start = idx + attr_prefix.len();
    let quote = attr_prefix.chars().last()?;
    let quote_char = if quote == '\'' || quote == '"' {
        quote
    } else {
        '"'
    };
    let end = text[start..].find(quote_char)?;
    Some(text[start..start + end].to_string())
}

pub fn extract_first_paragraph_snippet(html: &str) -> Option<String> {
    let mut search_str = html;

    while let Some(start_idx) = search_str.find("<p") {
        let remainder = &search_str[start_idx + 2..];
        let next_char = remainder.chars().next();
        let is_valid_p_tag = matches!(next_char, Some(' ' | '\t' | '\n' | '\r' | '>' | '/'));

        if !is_valid_p_tag {
            search_str = &search_str[start_idx + 2..];
            continue;
        }

        let Some(rel_tag_end) = search_str[start_idx..].find('>') else {
            break;
        };
        let tag_end = start_idx + rel_tag_end + 1;

        let Some(rel_close_idx) = search_str[tag_end..].find("</p>") else {
            search_str = &search_str[tag_end..];
            continue;
        };
        let close_idx = tag_end + rel_close_idx;

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
            let lower = title.trim().to_lowercase();
            let is_generic = lower.is_empty()
                || lower == "untitled"
                || lower == "unknown"
                || lower == "không xác định"
                || lower == "chưa có tiêu đề";
            if title != book_title && !is_generic {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlHeading {
    pub title: String,
    pub level: u8,
    pub id: Option<String>,
}

pub fn extract_headings_from_html(html: &str) -> Vec<HtmlHeading> {
    let mut reader = Reader::from_str(html);
    reader.config_mut().check_end_names = false;
    reader.config_mut().trim_text(false);

    let mut headings = Vec::new();
    let mut buf = Vec::with_capacity(256);
    let mut current_heading_level: Option<u8> = None;
    let mut current_heading_id: Option<String> = None;
    let mut current_heading_text = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                if let Some(level) = parse_heading_tag(name.as_ref()) {
                    current_heading_level = Some(level);
                    current_heading_id = extract_id_or_name(e);
                    current_heading_text.clear();
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                if let Some(level) = parse_heading_tag(name.as_ref())
                    && current_heading_level == Some(level)
                {
                    let decoded = decode_html_entities(&current_heading_text);
                    let clean_title = normalize_nfc(&decoded).trim().to_string();
                    if !clean_title.is_empty() {
                        headings.push(HtmlHeading {
                            title: clean_title,
                            level,
                            id: current_heading_id.take(),
                        });
                    }
                    current_heading_level = None;
                    current_heading_text.clear();
                }
            }
            Ok(Event::Text(ref e)) => {
                if current_heading_level.is_some() {
                    current_heading_text.push_str(e.as_ref());
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    headings
}

fn parse_heading_tag(name: &str) -> Option<u8> {
    match name {
        "h1" | "H1" => Some(1),
        "h2" | "H2" => Some(2),
        "h3" | "H3" => Some(3),
        "h4" | "H4" => Some(4),
        "h5" | "H5" => Some(5),
        "h6" | "H6" => Some(6),
        _ => None,
    }
}

pub fn extract_classes(e: &quick_xml::events::BytesStart) -> Vec<String> {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref().eq_ignore_ascii_case("class") {
            let val = attr.value.to_string();
            return val
                .split_whitespace()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }
    Vec::new()
}

pub fn extract_inline_style_attr(e: &quick_xml::events::BytesStart) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref().eq_ignore_ascii_case("style") {
            let val = attr.value.to_string();
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

/// Information extracted from an HTML block tag (`<h1..6>` or `<p>`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RawHtmlBlockMeta {
    pub tag: String,
    pub classes: Vec<String>,
    pub inline_style: Option<String>,
    pub id: Option<String>,
    pub is_empty: bool,
    pub child_classes: Vec<String>,
}

/// Scans HTML to extract metadata for all block-level elements (`<p>`, `<h1>`..`<h6>`).
pub fn extract_html_blocks_meta(html: &str) -> Vec<RawHtmlBlockMeta> {
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;

    let mut reader = Reader::from_str(html);
    reader.config_mut().check_end_names = false;
    reader.config_mut().trim_text(false);

    let mut metas = Vec::new();
    let mut buf = Vec::with_capacity(256);

    let mut current_block: Option<RawHtmlBlockMeta> = None;
    let mut text_count = 0usize;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                let tag_str = name.as_ref().to_ascii_lowercase();

                if matches!(
                    tag_str.as_str(),
                    "p" | "h1"
                        | "h2"
                        | "h3"
                        | "h4"
                        | "h5"
                        | "h6"
                        | "hr"
                        | "table"
                        | "pre"
                        | "ul"
                        | "ol"
                ) {
                    if let Some(mut prev) = current_block.take() {
                        prev.is_empty = text_count == 0 && !matches!(prev.tag.as_str(), "hr");
                        metas.push(prev);
                    }
                    let is_empty = !matches!(tag_str.as_str(), "hr");
                    current_block = Some(RawHtmlBlockMeta {
                        tag: tag_str,
                        classes: extract_classes(e),
                        inline_style: extract_inline_style_attr(e),
                        id: extract_id_or_name(e),
                        is_empty,
                        child_classes: Vec::new(),
                    });
                    text_count = 0;
                } else if let Some(ref mut block) = current_block {
                    if matches!(
                        tag_str.as_str(),
                        "span" | "em" | "strong" | "b" | "i" | "a" | "div"
                    ) {
                        for c in extract_classes(e) {
                            if !block.child_classes.contains(&c) {
                                block.child_classes.push(c);
                            }
                        }
                    }
                    if matches!(tag_str.as_str(), "img" | "image") {
                        text_count += 1;
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                let name = e.name();
                let tag_str = name.as_ref().to_ascii_lowercase();

                if matches!(
                    tag_str.as_str(),
                    "p" | "h1"
                        | "h2"
                        | "h3"
                        | "h4"
                        | "h5"
                        | "h6"
                        | "hr"
                        | "table"
                        | "pre"
                        | "ul"
                        | "ol"
                ) {
                    if let Some(mut prev) = current_block.take() {
                        prev.is_empty = text_count == 0 && !matches!(prev.tag.as_str(), "hr");
                        metas.push(prev);
                    }
                    let is_empty = !matches!(tag_str.as_str(), "hr");
                    metas.push(RawHtmlBlockMeta {
                        tag: tag_str,
                        classes: extract_classes(e),
                        inline_style: extract_inline_style_attr(e),
                        id: extract_id_or_name(e),
                        is_empty,
                        child_classes: Vec::new(),
                    });
                    text_count = 0;
                } else if let Some(ref mut block) = current_block {
                    if matches!(
                        tag_str.as_str(),
                        "span" | "em" | "strong" | "b" | "i" | "a" | "div"
                    ) {
                        for c in extract_classes(e) {
                            if !block.child_classes.contains(&c) {
                                block.child_classes.push(c);
                            }
                        }
                    }
                    if matches!(tag_str.as_str(), "img" | "image") {
                        text_count += 1;
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                let tag_str = name.as_ref().to_ascii_lowercase();

                if let Some(ref cur) = current_block
                    && cur.tag == tag_str
                {
                    let mut block = current_block.take().unwrap();
                    block.is_empty = text_count == 0 && !matches!(block.tag.as_str(), "hr");
                    metas.push(block);
                    text_count = 0;
                }
            }
            Ok(Event::Text(ref e)) => {
                let s = e.as_ref();
                let clean = s.replace('\u{a0}', " ");
                if !clean.trim().is_empty() {
                    text_count += clean.trim().len();
                }
            }
            Ok(Event::Eof) | Err(_) => {
                if let Some(mut prev) = current_block.take() {
                    prev.is_empty = text_count == 0;
                    metas.push(prev);
                }
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    metas
}

pub fn extract_id_or_name(e: &quick_xml::events::BytesStart) -> Option<String> {
    for attr in e.attributes().flatten() {
        let key = attr.key.as_ref();
        if key.eq_ignore_ascii_case("id")
            || key.eq_ignore_ascii_case("name")
            || key.eq_ignore_ascii_case("xml:id")
        {
            let val = attr.value.to_string();
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
#[path = "extract_tests.rs"]
mod tests;
