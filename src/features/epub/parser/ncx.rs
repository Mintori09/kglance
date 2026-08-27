use super::html::{extract_attribute, extract_tag_content};

pub type NcxNavPoint = (String, u8, String, Option<String>);

pub fn extract_ncx_navpoints(ncx_xml: &str) -> Vec<NcxNavPoint> {
    let mut entries = Vec::new();
    let mut search_str = ncx_xml;

    while let Some(idx) = search_str.find("<navPoint") {
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
        let src = extract_attribute(nav_block, "src=\"");

        if let (Some(lbl), Some(src_str)) = (label, src) {
            let clean_label = lbl.trim().to_string();
            let mut parts = src_str.split('#');
            let file_part = parts.next().unwrap_or(&src_str).to_string();
            let anchor = parts.next().map(ToString::to_string);

            entries.push((clean_label, level, file_part, anchor));
        }

        search_str = &search_str[idx + 9..];
    }

    entries
}

pub fn extract_epub3_navpoints(nav_html: &str) -> Vec<NcxNavPoint> {
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;

    let mut entries = Vec::new();
    let mut reader = Reader::from_str(nav_html);
    reader.config_mut().check_end_names = false;
    reader.config_mut().trim_text(false);

    let mut buf = Vec::with_capacity(256);
    let mut inside_toc_nav = false;
    let mut list_depth: u8 = 0;
    let mut current_href: Option<String> = None;
    let mut current_text = String::new();
    let mut inside_a = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();
                if tag_name == "nav" {
                    for attr in e.attributes().flatten() {
                        let val = String::from_utf8_lossy(&attr.value).to_lowercase();
                        if val.contains("toc") {
                            inside_toc_nav = true;
                        }
                    }
                } else if inside_toc_nav {
                    if tag_name == "ol" || tag_name == "ul" {
                        list_depth += 1;
                    } else if tag_name == "a" {
                        inside_a = true;
                        current_text.clear();
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_lowercase();
                            if key == "href" {
                                current_href =
                                    Some(String::from_utf8_lossy(&attr.value).to_string());
                            }
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();
                if tag_name == "nav" && inside_toc_nav {
                    inside_toc_nav = false;
                } else if inside_toc_nav {
                    if (tag_name == "ol" || tag_name == "ul") && list_depth > 0 {
                        list_depth -= 1;
                    } else if tag_name == "a" && inside_a {
                        inside_a = false;
                        if let Some(href) = current_href.take() {
                            let clean_label = current_text.trim().to_string();
                            if !clean_label.is_empty() {
                                let mut parts = href.split('#');
                                let file_part = parts.next().unwrap_or(&href).to_string();
                                let anchor = parts.next().map(ToString::to_string);
                                entries.push((clean_label, list_depth.max(1), file_part, anchor));
                            }
                        }
                        current_text.clear();
                    }
                }
            }
            Ok(Event::Text(ref e)) => {
                if inside_toc_nav
                    && inside_a
                    && let Ok(txt) = e.decode()
                {
                    current_text.push_str(&txt);
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    entries
}
