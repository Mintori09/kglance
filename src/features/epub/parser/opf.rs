use std::path::Path;

use super::html::extract_attribute;

pub fn extract_opf_path(xml: &str) -> Option<String> {
    if let Some(idx) = xml.find("full-path=\"") {
        let start_idx = idx + "full-path=\"".len();
        let length = xml[start_idx..].find('"')?;
        return Some(xml[start_idx..start_idx + length].to_string());
    }
    if let Some(idx) = xml.find("full-path='") {
        let start_idx = idx + "full-path='".len();
        let length = xml[start_idx..].find('\'')?;
        return Some(xml[start_idx..start_idx + length].to_string());
    }
    None
}

pub fn extract_ncx_href(opf_xml: &str) -> Option<String> {
    let mut search_str = opf_xml;

    while let Some(idx) = search_str.find("<item") {
        let tag_end = search_str[idx..].find('>')?;
        let tag = &search_str[idx..idx + tag_end];

        let is_ncx = tag.contains("application/x-dtbncx+xml")
            || tag.contains("id=\"ncx\"")
            || tag.contains("id='ncx'")
            || tag.contains("id=\"toc\"")
            || tag.contains("id='toc'");

        if is_ncx
            && let Some(href) =
                extract_attribute(tag, "href=\"").or_else(|| extract_attribute(tag, "href='"))
        {
            return Some(href);
        }
        search_str = &search_str[idx + tag_end..];
    }
    None
}

pub fn extract_nav_href(opf_xml: &str) -> Option<String> {
    let mut search_str = opf_xml;

    while let Some(idx) = search_str.find("<item") {
        let tag_end = search_str[idx..].find('>').unwrap_or(0);
        let tag = &search_str[idx..idx + tag_end];

        let has_nav_prop = extract_attribute(tag, "properties=\"")
            .or_else(|| extract_attribute(tag, "properties='"))
            .map(|props| {
                props
                    .split_whitespace()
                    .any(|p| p.eq_ignore_ascii_case("nav") || p.eq_ignore_ascii_case("toc"))
            })
            .unwrap_or(false);

        let has_nav_id = extract_attribute(tag, "id=\"")
            .or_else(|| extract_attribute(tag, "id='"))
            .map(|id| id.eq_ignore_ascii_case("nav") || id.eq_ignore_ascii_case("toc"))
            .unwrap_or(false);

        if (has_nav_prop || has_nav_id)
            && let Some(href) =
                extract_attribute(tag, "href=\"").or_else(|| extract_attribute(tag, "href='"))
        {
            return Some(href);
        }
        search_str = &search_str[idx + tag_end.max(1)..];
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
        let has_cover_prop = extract_attribute(tag, "properties=\"")
            .or_else(|| extract_attribute(tag, "properties='"))
            .map(|props| {
                props
                    .split_whitespace()
                    .any(|p| p.eq_ignore_ascii_case("cover-image"))
            })
            .unwrap_or(false);
        let is_cover_tag =
            has_cover_prop || tag.contains("id=\"cover\"") || tag.contains("id=\"cover-image\"");
        if is_cover_tag
            && let Some(href) =
                extract_attribute(tag, "href=\"").or_else(|| extract_attribute(tag, "href='"))
        {
            return Some(href);
        }
        search_str = &search_str[idx + tag_end.max(1)..];
    }

    None
}

pub fn extract_css_items(xml: &str) -> Vec<String> {
    let mut hrefs = Vec::new();
    let mut search_str = xml;

    while let Some(idx) = search_str.find("<item") {
        let tag_end = search_str[idx..].find('>').unwrap_or(0);
        let tag = &search_str[idx..idx + tag_end];

        if (tag.contains("media-type=\"text/css\"") || tag.contains("media-type='text/css'"))
            && let Some(href) = extract_attribute(tag, "href=\"")
        {
            hrefs.push(href);
        }
        search_str = &search_str[idx + tag_end.max(1)..];
    }

    hrefs
}

pub fn extract_spine_items(xml: &str) -> Vec<String> {
    let mut item_refs = Vec::new();
    let mut search_str = xml;

    while let Some(idx) = search_str.find("<itemref") {
        let tag_end = search_str[idx..].find('>').unwrap_or(0);
        let tag = &search_str[idx..idx + tag_end];

        if let Some(idref) =
            extract_attribute(tag, "idref=\"").or_else(|| extract_attribute(tag, "idref='"))
        {
            item_refs.push(idref);
        }
        search_str = &search_str[idx + tag_end.max(1)..];
    }

    let mut hrefs = Vec::new();
    for ref_id in item_refs {
        let pattern_double = format!("id=\"{}\"", ref_id);
        let pattern_single = format!("id='{}'", ref_id);
        let found_idx = xml
            .find(&pattern_double)
            .or_else(|| xml.find(&pattern_single));

        if let Some(idx) = found_idx {
            let tag_start = xml[..idx].rfind("<item").unwrap_or(0);
            let tag_end = xml[idx..].find('>').unwrap_or(0) + idx;
            let tag_content = &xml[tag_start..tag_end];

            if let Some(href) = extract_attribute(tag_content, "href=\"")
                .or_else(|| extract_attribute(tag_content, "href='"))
            {
                hrefs.push(href);
            }
        }
    }

    hrefs
}

pub fn resolve_relative_path(base_file: &str, relative: &str) -> String {
    use std::path::Component;

    let parent = Path::new(base_file)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let clean_relative = relative.trim_start_matches('/');
    let joined = if parent.as_os_str().is_empty() {
        Path::new(clean_relative).to_path_buf()
    } else {
        parent.join(clean_relative)
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
