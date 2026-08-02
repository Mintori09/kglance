use std::path::Path;

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

pub fn resolve_relative_path(base_opf: &str, relative: &str) -> String {
    if let Some(parent) = Path::new(base_opf).parent() {
        if parent.as_os_str().is_empty() {
            relative.to_string()
        } else {
            format!("{}/{}", parent.to_string_lossy(), relative)
        }
    } else {
        relative.to_string()
    }
}
