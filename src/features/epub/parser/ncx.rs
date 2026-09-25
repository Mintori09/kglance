use quick_xml::events::Event;
use quick_xml::reader::Reader;

pub type NcxNavPoint = (String, u8, String, Option<String>);

struct NavPointFrame {
    level: u8,
    label: Option<String>,
    src: Option<String>,
    emitted: bool,
}

pub fn extract_ncx_navpoints(ncx_xml: &str) -> Vec<NcxNavPoint> {
    let mut entries = Vec::new();
    let mut reader = Reader::from_str(ncx_xml);
    reader.config_mut().check_end_names = false;
    reader.config_mut().trim_text(false);

    let mut stack: Vec<NavPointFrame> = Vec::new();
    let mut inside_label_text = false;
    let mut current_text = String::new();
    let mut buf = Vec::with_capacity(256);

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.name().as_ref().eq_ignore_ascii_case("navpoint") {
                    let level = (stack.len() as u8 + 1).max(1);
                    stack.push(NavPointFrame {
                        level,
                        label: None,
                        src: None,
                        emitted: false,
                    });
                } else if e.name().as_ref().eq_ignore_ascii_case("text") {
                    inside_label_text = true;
                    current_text.clear();
                }
            }
            Ok(Event::Empty(ref e)) => {
                if e.name().as_ref().eq_ignore_ascii_case("content") {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref().eq_ignore_ascii_case("src")
                            && let Some(frame) = stack.last_mut()
                        {
                            frame.src = Some(attr.value.to_string());
                            try_emit_navpoint(frame, &mut entries);
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref().eq_ignore_ascii_case("text") {
                    inside_label_text = false;
                    let clean = current_text.trim().to_string();
                    if let Some(frame) = stack.last_mut() {
                        frame.label = Some(clean);
                        try_emit_navpoint(frame, &mut entries);
                    }
                } else if e.name().as_ref().eq_ignore_ascii_case("navpoint") {
                    stack.pop();
                }
            }
            Ok(Event::Text(ref e)) => {
                if inside_label_text {
                    current_text.push_str(e.as_ref());
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

fn try_emit_navpoint(frame: &mut NavPointFrame, entries: &mut Vec<NcxNavPoint>) {
    if frame.emitted {
        return;
    }
    if let (Some(label), Some(src)) = (&frame.label, &frame.src)
        && !label.is_empty()
    {
        let mut parts = src.split('#');
        let file_part = parts.next().unwrap_or(src).to_string();
        let anchor = parts.next().map(ToString::to_string);
        entries.push((label.clone(), frame.level, file_part, anchor));
        frame.emitted = true;
    }
}

pub fn extract_epub3_navpoints(nav_html: &str) -> Vec<NcxNavPoint> {
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
                if e.name().as_ref().eq_ignore_ascii_case("nav") {
                    for attr in e.attributes().flatten() {
                        if attr.value.to_ascii_lowercase().contains("toc") {
                            inside_toc_nav = true;
                        }
                    }
                } else if inside_toc_nav {
                    if e.name().as_ref().eq_ignore_ascii_case("ol")
                        || e.name().as_ref().eq_ignore_ascii_case("ul")
                    {
                        list_depth += 1;
                    } else if e.name().as_ref().eq_ignore_ascii_case("a") {
                        inside_a = true;
                        current_text.clear();
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref().eq_ignore_ascii_case("href") {
                                current_href = Some(attr.value.to_string());
                            }
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref().eq_ignore_ascii_case("nav") && inside_toc_nav {
                    inside_toc_nav = false;
                } else if inside_toc_nav {
                    if (e.name().as_ref().eq_ignore_ascii_case("ol")
                        || e.name().as_ref().eq_ignore_ascii_case("ul"))
                        && list_depth > 0
                    {
                        list_depth -= 1;
                    } else if e.name().as_ref().eq_ignore_ascii_case("a") && inside_a {
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
                if inside_toc_nav && inside_a {
                    current_text.push_str(e.as_ref());
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
