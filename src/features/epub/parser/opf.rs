use std::path::{Component, Path};

use super::html::extract_attribute;

pub fn extract_opf_path(xml: &str) -> Option<String> {
    let tag = "full-path=\"";
    let start_idx = xml.find(tag)? + tag.len();
    let length = xml[start_idx..].find('"')?;
    Some(xml[start_idx..start_idx + length].to_string())
}

pub fn extract_ncx_href(opf_xml: &str) -> Option<String> {
    let mut search_str = opf_xml;

    while let Some(idx) = search_str.find("<item") {
        let tag_end = search_str[idx..].find('>')?;
        let tag = &search_str[idx..idx + tag_end];

        let is_ncx = tag.contains("application/x-dtbncx+xml")
            || tag.contains("id=\"ncx\"")
            || tag.contains("id=\"toc\"");

        if is_ncx && let Some(href) = extract_attribute(tag, "href=\"") {
            return Some(href);
        }
        search_str = &search_str[idx + tag_end..];
    }
    None
}

pub fn extract_nav_href(opf_xml: &str) -> Option<String> {
    let mut search_str = opf_xml;

    while let Some(idx) = search_str.find("<item") {
        let tag_end = search_str[idx..].find('>')?;
        let tag = &search_str[idx..idx + tag_end];

        let is_nav = tag.contains("properties=\"nav\"")
            || tag.contains("properties=\"toc\"")
            || tag.contains("id=\"nav\"")
            || tag.contains("id=\"toc\"");

        if is_nav && let Some(href) = extract_attribute(tag, "href=\"") {
            return Some(href);
        }
        search_str = &search_str[idx + tag_end..];
    }
    None
}

pub fn extract_cover_image_href(opf_xml: &str) -> Option<String> {
    let mut search_str = opf_xml;
    let mut cover_item_id = None;
    while let Some(idx) = search_str.find("<meta") {
        let tag_end = search_str[idx..].find('>').unwrap_or(0);
        let tag = &search_str[idx..idx + tag_end];
        let is_cover = tag.contains("name=\"cover\"") || tag.contains("name='cover'");
        if is_cover && let Some(content) = extract_attribute(tag, "content=\"") {
            cover_item_id = Some(content);
            break;
        }
        search_str = &search_str[idx + tag_end.max(1)..];
    }

    if let Some(id) = cover_item_id {
        let pattern = format!("id=\"{}\"", id);
        let alt_pattern = format!("id='{}'", id);
        if let Some(idx) = opf_xml
            .find(&pattern)
            .or_else(|| opf_xml.find(&alt_pattern))
        {
            let tag_start = opf_xml[..idx].rfind("<item").unwrap_or(0);
            let tag_end = opf_xml[idx..].find('>').unwrap_or(0) + idx;
            let tag_content = &opf_xml[tag_start..tag_end];
            if let Some(href) = extract_attribute(tag_content, "href=\"") {
                return Some(href);
            }
        }
    }

    let mut search_str = opf_xml;
    while let Some(idx) = search_str.find("<item") {
        let tag_end = search_str[idx..].find('>').unwrap_or(0);
        let tag = &search_str[idx..idx + tag_end];
        let is_cover_tag = tag.contains("properties=\"cover-image\"")
            || tag.contains("id=\"cover\"")
            || tag.contains("id=\"cover-image\"");
        if is_cover_tag && let Some(href) = extract_attribute(tag, "href=\"") {
            return Some(href);
        }
        search_str = &search_str[idx + tag_end.max(1)..];
    }

    None
}

pub fn extract_spine_items(xml: &str) -> Vec<String> {
    let mut item_refs = Vec::new();
    let mut search_str = xml;

    while let Some(idx) = search_str.find("<itemref") {
        let tag_end = search_str[idx..].find('>').unwrap_or(0);
        let tag = &search_str[idx..idx + tag_end];

        if let Some(idref) = extract_attribute(tag, "idref=\"") {
            item_refs.push(idref);
        }
        search_str = &search_str[idx + tag_end..];
    }

    let mut hrefs = Vec::new();
    for ref_id in item_refs {
        let pattern = format!("id=\"{}\"", ref_id);
        if let Some(idx) = xml.find(&pattern) {
            let tag_start = xml[..idx].rfind("<item").unwrap_or(0);
            let tag_end = xml[idx..].find('>').unwrap_or(0) + idx;
            let tag_content = &xml[tag_start..tag_end];

            if let Some(href) = extract_attribute(tag_content, "href=\"") {
                hrefs.push(href);
            }
        }
    }

    hrefs
}

pub fn resolve_relative_path(base_file: &str, relative: &str) -> String {
    let parent = Path::new(base_file)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let joined = if parent.as_os_str().is_empty() {
        Path::new(relative).to_path_buf()
    } else {
        parent.join(relative)
    };

    let mut result = Vec::new();
    for component in joined.components() {
        match component {
            Component::Normal(c) => {
                if let Some(s) = c.to_str() {
                    result.push(s);
                }
            }
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {}
            _ => {}
        }
    }

    result.join("/")
}
