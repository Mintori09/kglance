use crate::parsers::{ParseError, ParsedContent, PreviewParser};
use std::io::Read;
use std::path::Path;

pub struct EpubParser;

impl PreviewParser for EpubParser {
    fn supported_extensions(&self) -> &[&str] {
        &["epub"]
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let file = std::fs::File::open(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        // 1. Read container.xml to locate the OPF path
        let opf_path = {
            let mut container = archive
                .by_name("META-INF/container.xml")
                .map_err(|_| ParseError::ParseFailed("Missing META-INF/container.xml".into()))?;
            let mut bytes = Vec::new();
            container
                .read_to_end(&mut bytes)
                .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
            let container_xml = read_bytes_to_string(&bytes);
            extract_opf_path(&container_xml).ok_or_else(|| {
                ParseError::ParseFailed("Could not locate OPF file in container.xml".into())
            })?
        };

        // 2. Read the OPF file
        let (title, author, spine_items, ncx_path) = {
            let mut opf_file = archive
                .by_name(&opf_path)
                .map_err(|_| ParseError::ParseFailed(format!("Missing OPF file: {opf_path}")))?;
            let mut bytes = Vec::new();
            opf_file
                .read_to_end(&mut bytes)
                .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
            let opf_xml = read_bytes_to_string(&bytes);

            let title = extract_tag_content(&opf_xml, "dc:title")
                .unwrap_or_else(|| "Unknown Title".to_string());
            let author = extract_tag_content(&opf_xml, "dc:creator")
                .unwrap_or_else(|| "Unknown Author".to_string());
            let spine_items = extract_spine_items(&opf_xml);
            let ncx_href = extract_ncx_href(&opf_xml);
            (title, author, spine_items, ncx_href)
        };

        // 3. Read NCX TOC if present to build file -> title map
        // 3. Read NCX TOC if present to build list of TOC entries (label, target_file, anchor)
        let mut ncx_entries = Vec::new();
        if let Some(ncx_href) = ncx_path {
            let resolved_ncx = resolve_relative_path(&opf_path, &ncx_href);
            if let Ok(mut ncx_file) = archive.by_name(&resolved_ncx) {
                let mut bytes = Vec::new();
                if ncx_file.read_to_end(&mut bytes).is_ok() {
                    let ncx_xml = read_bytes_to_string(&bytes);
                    ncx_entries = extract_ncx_navpoints(&ncx_xml);
                }
            }
        }

        // Pre-read and parse all spine files into HTML & Markdown blocks map
        let mut spine_cache = std::collections::HashMap::new();
        for item_path in &spine_items {
            let resolved_path = resolve_relative_path(&opf_path, item_path);
            if let Ok(mut content_file) = archive.by_name(&resolved_path) {
                let mut bytes = Vec::new();
                if content_file.read_to_end(&mut bytes).is_ok() {
                    let html = read_bytes_to_string(&bytes);
                    let markdown_text = convert_html_to_markdown(&html);
                    let blocks = crate::parsers::markdown::parse_to_blocks(&markdown_text);
                    let relative_filename = Path::new(item_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(item_path)
                        .to_string();
                    spine_cache.insert(relative_filename, (html, blocks));
                }
            }
        }

        // 4. Build chapters: If NCX entries exist, create chapters for NCX navpoints
        let mut chapters = Vec::new();

        if !ncx_entries.is_empty() {
            let ncx_len = ncx_entries.len();
            for i in 0..ncx_len {
                let (label, level, file_part, anchor) = &ncx_entries[i];
                let filename = Path::new(file_part)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(file_part);

                if let Some((_html, blocks)) = spine_cache.get(filename)
                    && !blocks.is_empty()
                {
                    let clean_title = decode_html_entities(label);

                    // Find start index of this chapter's anchor or title in blocks
                    let start_block_idx = if let Some(anc) = anchor {
                        blocks
                            .iter()
                            .position(|b| {
                                let text = match b {
                                    crate::parsers::markdown::Block::Heading {
                                        content, ..
                                    }
                                    | crate::parsers::markdown::Block::Paragraph(content) => {
                                        crate::parsers::markdown::flatten_inlines(content)
                                    }
                                    _ => String::new(),
                                };
                                text.contains(anc) || text.contains(&clean_title)
                            })
                            .unwrap_or(0)
                    } else {
                        0
                    };

                    // Find end index if the next NCX entry points to the same HTML file
                    let end_block_idx = if i + 1 < ncx_len {
                        let (next_file_part, next_anchor) =
                            (&ncx_entries[i + 1].2, &ncx_entries[i + 1].3);
                        let next_filename = Path::new(next_file_part)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(next_file_part);

                        if next_filename == filename {
                            if let Some(next_anc) = next_anchor {
                                blocks[start_block_idx..]
                                        .iter()
                                        .position(|b| {
                                            let text = match b {
                                                crate::parsers::markdown::Block::Heading {
                                                    content,
                                                    ..
                                                }
                                                | crate::parsers::markdown::Block::Paragraph(
                                                    content,
                                                ) => crate::parsers::markdown::flatten_inlines(
                                                    content,
                                                ),
                                                _ => String::new(),
                                            };
                                            text.contains(next_anc)
                                        })
                                        .map(|pos| start_block_idx + pos)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let chapter_blocks = match end_block_idx {
                        Some(end_idx) if end_idx > start_block_idx => {
                            blocks[start_block_idx..end_idx].to_vec()
                        }
                        _ => blocks[start_block_idx..].to_vec(),
                    };

                    if !chapter_blocks.is_empty() {
                        chapters.push((clean_title, *level, anchor.clone(), chapter_blocks));
                    } else {
                        chapters.push((
                            clean_title,
                            *level,
                            anchor.clone(),
                            blocks[start_block_idx..].to_vec(),
                        ));
                    }
                }
            }
        }

        // Fallback to spine items if chapters is still empty
        if chapters.is_empty() {
            for (idx, item_path) in spine_items.into_iter().enumerate() {
                let relative_filename = Path::new(&item_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&item_path);

                if let Some((html, blocks)) = spine_cache.get(relative_filename) {
                    let mut chapter_title = extract_tag_content(html, "h1")
                        .or_else(|| extract_tag_content(html, "h2"))
                        .or_else(|| extract_tag_content(html, "h3"))
                        .or_else(|| {
                            let tag_title = extract_tag_content(html, "title");
                            if let Some(t) = tag_title
                                && t != title
                            {
                                return Some(t);
                            }
                            None
                        })
                        .or_else(|| extract_first_paragraph_snippet(html))
                        .unwrap_or_else(|| format!("Chapter {}", idx + 1));

                    if chapter_title == title {
                        chapter_title = extract_first_paragraph_snippet(html)
                            .unwrap_or_else(|| format!("Chapter {}", idx + 1));
                    }

                    chapter_title = decode_html_entities(&chapter_title);
                    if !blocks.is_empty() {
                        chapters.push((chapter_title, 1, None, blocks.clone()));
                    }
                }
            }
        }

        if chapters.is_empty() {
            chapters.push((
                "Chapter 1".to_string(),
                1,
                None,
                vec![crate::parsers::markdown::Block::Paragraph(vec![
                    crate::parsers::markdown::Inline::Text(
                        "[No readable content found in EPUB]".to_string(),
                    ),
                ])],
            ));
        }

        // Extract all image files from zip archive
        let mut images = std::collections::HashMap::new();
        for i in 0..archive.len() {
            if let Ok(mut file) = archive.by_index(i) {
                let name = file.name().to_string();
                let lower = name.to_lowercase();
                if lower.ends_with(".jpg")
                    || lower.ends_with(".jpeg")
                    || lower.ends_with(".png")
                    || lower.ends_with(".gif")
                    || lower.ends_with(".webp")
                    || lower.ends_with(".bmp")
                    || lower.ends_with(".svg")
                {
                    let mut buf = Vec::new();
                    if file.read_to_end(&mut buf).is_ok() {
                        let filename = Path::new(&name)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(&name)
                            .to_string();
                        images.insert(name.clone(), buf.clone());
                        images.insert(filename, buf);
                    }
                }
            }
        }

        Ok(ParsedContent::Epub {
            title,
            author,
            chapters,
            images,
        })
    }
}

fn extract_opf_path(xml: &str) -> Option<String> {
    let tag = "full-path=\"";
    let idx = xml.find(tag)?;
    let start = idx + tag.len();
    let end = xml[start..].find('"')?;
    Some(xml[start..start + end].to_string())
}

fn extract_tag_content(xml: &str, tag_name: &str) -> Option<String> {
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

fn extract_ncx_href(opf_xml: &str) -> Option<String> {
    let mut search_str = opf_xml;
    while let Some(idx) = search_str.find("<item") {
        let tag_end = search_str[idx..].find('>')?;
        let tag = &search_str[idx..idx + tag_end];
        if (tag.contains("application/x-dtbncx+xml")
            || tag.contains("id=\"ncx\"")
            || tag.contains("id=\"toc\""))
            && let Some(href_idx) = tag.find("href=\"")
        {
            let start = href_idx + 6;
            if let Some(end) = tag[start..].find('"') {
                return Some(tag[start..start + end].to_string());
            }
        }
        search_str = &search_str[idx + tag_end..];
    }
    None
}

fn extract_ncx_navpoints(ncx_xml: &str) -> Vec<(String, u8, String, Option<String>)> {
    let mut entries = Vec::new();
    let mut search_str = ncx_xml;

    while let Some(idx) = search_str.find("<navPoint") {
        // Calculate hierarchy depth by checking how many open <navPoint tags exist before this point without closing </navPoint
        let prefix = &ncx_xml[..idx];
        let open_count = prefix.matches("<navPoint").count();
        let close_count = prefix.matches("</navPoint>").count();
        let level = (open_count.saturating_sub(close_count) as u8).max(1);

        let np_end = search_str[idx..]
            .find("</navPoint>")
            .map(|e| idx + e + 11)
            .unwrap_or(search_str.len());
        let nav_block = &search_str[idx..np_end];

        let label = extract_tag_content(nav_block, "text");
        let src = if let Some(src_idx) = nav_block.find("src=\"") {
            let start = src_idx + 5;
            nav_block[start..]
                .find('"')
                .map(|end| nav_block[start..start + end].to_string())
        } else {
            None
        };

        if let (Some(lbl), Some(s)) = (label, src) {
            let clean_label = lbl.trim().to_string();
            let mut parts = s.split('#');
            let file_part = parts.next().unwrap_or(&s).to_string();
            let anchor = parts.next().map(|a| a.to_string());

            entries.push((clean_label, level, file_part, anchor));
        }

        search_str = &search_str[idx + 9..];
    }

    entries
}

fn extract_first_paragraph_snippet(html: &str) -> Option<String> {
    let mut search_str = html;
    while let Some(start_idx) = search_str.find("<p") {
        let tag_end = search_str[start_idx..].find('>')? + start_idx + 1;
        let close_idx = search_str[tag_end..].find("</p>")? + tag_end;
        let raw_p = &search_str[tag_end..close_idx];
        let stripped = strip_html_tags(raw_p);
        let decoded = decode_html_entities(&stripped);
        let cleaned = decoded.trim();
        if !cleaned.is_empty() {
            let mut snippet = cleaned.to_string();
            if snippet.chars().count() > 40 {
                snippet = snippet.chars().take(40).collect::<String>() + "...";
            }
            return Some(snippet);
        }
        search_str = &search_str[close_idx + 4..];
    }
    None
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

fn read_bytes_to_string(bytes: &[u8]) -> String {
    // Strip UTF-8 BOM if present
    let data = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &bytes[3..]
    } else {
        bytes
    };

    // Decode with lossy UTF-8 conversion
    String::from_utf8_lossy(data).into_owned()
}

pub fn decode_html_entities(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut rest = text;

    while let Some(amp_idx) = rest.find('&') {
        result.push_str(&rest[..amp_idx]);
        rest = &rest[amp_idx..];

        if let Some(semi_idx) = rest.find(';') {
            let entity = &rest[1..semi_idx];
            let decoded = if let Some(hex) = entity
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
            };

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

fn convert_html_to_markdown(html: &str) -> String {
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

            if tag_name == "style"
                || tag_name == "script"
                || tag_name == "head"
                || tag_name == "title"
            {
                skip_content = !is_close;
            }

            if !skip_content {
                match tag_name.as_str() {
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
                        if let Some(src_idx) = tag_content.find("src=\"") {
                            let start = src_idx + 5;
                            if let Some(end) = tag_content[start..].find('"') {
                                let src = &tag_content[start..start + end];
                                let alt = if let Some(alt_idx) = tag_content.find("alt=\"") {
                                    let a_start = alt_idx + 5;
                                    tag_content[a_start..]
                                        .find('"')
                                        .map(|a_end| &tag_content[a_start..a_start + a_end])
                                        .unwrap_or("image")
                                } else {
                                    "image"
                                };
                                result.push_str(&format!("\n\n![{alt}]({src})\n\n"));
                            }
                        }
                    }
                    _ => {}
                }
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
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn strip_html_tags(html: &str) -> String {
    convert_html_to_markdown(html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_html_entities() {
        let text = "&#x201C;Cough. Let&#x2019;s talk.&#x201D; \\-Haha";
        assert_eq!(decode_html_entities(text), "“Cough. Let’s talk.” \\-Haha");
    }

    #[test]
    fn test_convert_html_to_markdown() {
        let html = "<h1>Header 1</h1><p>&#x201C;Hello&#x201D; \\-World</p>";
        let md = convert_html_to_markdown(html);
        assert_eq!(md, "# Header 1\n\n“Hello” -World");
    }

    #[test]
    fn test_extract_ncx_navpoints() {
        let ncx_xml = r#"
            <ncx xmlns="http://www.daisy.org/z3986/2005/ncx/">
              <navMap>
                <navPoint id="n1">
                  <navLabel><text>LỜI NÓI ĐẦU</text></navLabel>
                  <content src="index_split_001.html"/>
                </navPoint>
                <navPoint id="n2">
                  <navLabel><text>CHƯƠNG 1 - NHỮNG NGUYÊN TẮC CƠ BẢN</text></navLabel>
                  <content src="index_split_001.html#filepos32808"/>
                </navPoint>
              </navMap>
            </ncx>
        "#;
        let navpoints = extract_ncx_navpoints(ncx_xml);
        assert_eq!(navpoints.len(), 2);
        assert_eq!(navpoints[0].0, "LỜI NÓI ĐẦU");
        assert_eq!(navpoints[1].0, "CHƯƠNG 1 - NHỮNG NGUYÊN TẮC CƠ BẢN");
        assert_eq!(navpoints[1].3.as_deref(), Some("filepos32808"));
    }

    #[test]
    fn test_convert_html_img_to_markdown() {
        let html = r#"<p>Paragraph text</p><img src="images/00001.jpg" alt="Habit Loop" class="calibre_59"/>"#;
        let md = convert_html_to_markdown(html);
        assert_eq!(md, "Paragraph text\n\n![Habit Loop](images/00001.jpg)");
    }

    #[test]
    fn test_parse_epub_with_embedded_image() {
        use std::io::Write;
        let temp_dir = std::env::temp_dir();
        let test_epub_path = temp_dir.join("test_kglance_image.epub");

        let file = std::fs::File::create(&test_epub_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);

        let options = zip::write::SimpleFileOptions::default();

        zip.start_file("META-INF/container.xml", options).unwrap();
        zip.write_all(r#"<?xml version="1.0"?><container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#.as_bytes()).unwrap();

        zip.start_file("content.opf", options).unwrap();
        zip.write_all(r#"<?xml version="1.0"?><package><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>Test EPUB</dc:title></metadata><manifest><item id="item1" href="page.html" media-type="application/xhtml+xml"/></manifest><spine><itemref idref="item1"/></spine></package>"#.as_bytes()).unwrap();

        zip.start_file("page.html", options).unwrap();
        zip.write_all(r#"<html><body><h1>Chapter 1</h1><p><img src="images/sample.png" alt="Sample"/></p></body></html>"#.as_bytes()).unwrap();

        zip.start_file("images/sample.png", options).unwrap();
        zip.write_all(&[137, 80, 78, 71, 13, 10, 26, 10]).unwrap(); // PNG header bytes

        zip.finish().unwrap();

        let parser = EpubParser;
        let result = parser.parse(&test_epub_path).unwrap();

        if let ParsedContent::Epub {
            chapters, images, ..
        } = result
        {
            assert_eq!(chapters.len(), 1);
            assert!(images.contains_key("images/sample.png"));
            assert!(images.contains_key("sample.png"));
            assert_eq!(
                images.get("sample.png").unwrap(),
                &[137, 80, 78, 71, 13, 10, 26, 10]
            );
        } else {
            panic!("Expected ParsedContent::Epub variant");
        }

        let _ = std::fs::remove_file(test_epub_path);
    }
}
