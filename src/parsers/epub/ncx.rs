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
