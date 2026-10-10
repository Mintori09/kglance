use std::path::Path;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use super::converter::strip_html_tags;
use super::entities::{decode_html_entities, normalize_nfc};

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
