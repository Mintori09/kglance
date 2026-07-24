use std::io::Read;
use std::path::Path;

use crate::core::preview::PreviewContent;
use crate::features::text::content::TextContent;
use crate::parsers::ParseError;

pub struct EpubParser;

pub(crate) fn parse_epub(
    path: &Path,
) -> Result<Box<dyn PreviewContent<crate::app::Message>>, ParseError> {
    let file = std::fs::File::open(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| ParseError::ParseFailed(e.to_string()))?;

    let opf_path = {
        let mut container = archive
            .by_name("META-INF/container.xml")
            .map_err(|_| ParseError::ParseFailed("Missing META-INF/container.xml".into()))?;
        let mut container_xml = String::new();
        container
            .read_to_string(&mut container_xml)
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        extract_opf_path(&container_xml).ok_or_else(|| {
            ParseError::ParseFailed("Could not locate OPF file in container.xml".into())
        })?
    };

    let (title, author, spine_items) = {
        let mut opf_file = archive
            .by_name(&opf_path)
            .map_err(|_| ParseError::ParseFailed(format!("Missing OPF file: {opf_path}")))?;
        let mut opf_xml = String::new();
        opf_file
            .read_to_string(&mut opf_xml)
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        let title = extract_tag_content(&opf_xml, "dc:title")
            .unwrap_or_else(|| "Unknown Title".to_string());
        let author = extract_tag_content(&opf_xml, "dc:creator")
            .unwrap_or_else(|| "Unknown Author".to_string());
        let spine_items = extract_spine_items(&opf_xml);
        (title, author, spine_items)
    };

    let mut book_content = format!("Title: {title}\nAuthor: {author}\n\n");

    let mut text_extracted = false;
    for item_path in spine_items {
        let resolved_path = resolve_relative_path(&opf_path, &item_path);
        if let Ok(mut content_file) = archive.by_name(&resolved_path) {
            let mut html = String::new();
            if content_file.read_to_string(&mut html).is_ok() {
                let plain_text = strip_html_tags(&html);
                if !plain_text.trim().is_empty() {
                    book_content.push_str(&plain_text);
                    text_extracted = true;
                    break;
                }
            }
        }
    }

    if !text_extracted {
        book_content.push_str("[No readable content found in the first chapter]");
    }

    let line_count = book_content.lines().count();
    Ok(Box::new(TextContent {
        content: book_content,
        language: "EPUB".into(),
        line_count,
        highlighted_html: None,
    }))
}

fn extract_opf_path(xml: &str) -> Option<String> {
    let tag = "full-path=\"";
    let idx = xml.find(tag)?;
    let start = idx + tag.len();
    let end = xml[start..].find('"')?;
    Some(xml[start..start + end].to_string())
}

fn extract_tag_content(xml: &str, tag_name: &str) -> Option<String> {
    let start_tag = format!("<{}", tag_name);
    let close_tag = format!("</{}>", tag_name);
    let start_idx = xml.find(&start_tag)?;
    let content_start = xml[start_idx..].find('>')? + start_idx + 1;
    let end_idx = xml[content_start..].find(&close_tag)? + content_start;
    Some(xml[content_start..end_idx].trim().to_string())
}

fn extract_spine_items(xml: &str) -> Vec<String> {
    let mut item_refs = Vec::new();
    let mut search_str = xml;
    while let Some(idx) = search_str.find("<itemref") {
        let tag_end = search_str[idx..].find('>').unwrap_or(0);
        let tag = &search_str[idx..idx + tag_end];
        if let Some(idref_idx) = tag.find("idref=\"") {
            let start = idref_idx + 7;
            if let Some(end) = tag[start..].find('"') {
                item_refs.push(tag[start..start + end].to_string());
            }
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
            if let Some(href_idx) = tag_content.find("href=\"") {
                let start = href_idx + 6;
                if let Some(end) = tag_content[start..].find('"') {
                    hrefs.push(tag_content[start..start + end].to_string());
                }
            }
        }
    }
    hrefs
}

fn resolve_relative_path(base_opf: &str, relative: &str) -> String {
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

fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut skip_content = false;
    let mut tag_name = String::new();
    let bytes = html.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'<' {
            tag_name.clear();
            i += 1;
            while i < len && bytes[i] != b'>' && !bytes[i].is_ascii_whitespace() {
                tag_name.push(bytes[i] as char);
                i += 1;
            }
            let name_lower = tag_name.to_lowercase();
            if name_lower == "style" || name_lower == "script" {
                skip_content = true;
            } else if name_lower == "/style" || name_lower == "/script" {
                skip_content = false;
            }

            if (name_lower == "p"
                || name_lower == "/p"
                || name_lower == "br"
                || name_lower == "br/"
                || name_lower == "h1"
                || name_lower == "/h1"
                || name_lower == "h2"
                || name_lower == "/h2"
                || name_lower == "h3"
                || name_lower == "/h3"
                || name_lower == "h4"
                || name_lower == "/h4"
                || name_lower == "h5"
                || name_lower == "/h5"
                || name_lower == "h6"
                || name_lower == "/h6"
                || name_lower == "div"
                || name_lower == "/div")
                && !result.is_empty()
                && !result.ends_with('\n')
            {
                result.push('\n');
            }

            while i < len && bytes[i] != b'>' {
                i += 1;
            }
            if i < len {
                i += 1;
            }
        } else {
            if !skip_content {
                result.push(bytes[i] as char);
            }
            i += 1;
        }
    }

    result = result
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&");

    result
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_opf_path() {
        let container = r#"<container><rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#;
        assert_eq!(
            extract_opf_path(container),
            Some("OEBPS/content.opf".to_string())
        );
    }

    #[test]
    fn test_strip_html_tags() {
        let html = "<html><head><style>body { color: red; }</style></head><body><h1>Title</h1><p>Hello &nbsp; world!</p></body></html>";
        assert_eq!(strip_html_tags(html), "Title\n\nHello   world!");
    }
}
