pub(crate) fn try_docx_direct(path: &str) -> Result<String, crate::parsers::ParseError> {
    use std::io::Read;

    let file = std::fs::File::open(path)
        .map_err(|e| crate::parsers::ParseError::ParseFailed(e.to_string()))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| crate::parsers::ParseError::ParseFailed(e.to_string()))?;
    let mut document = archive
        .by_name("word/document.xml")
        .map_err(|_| crate::parsers::ParseError::ParseFailed("no document.xml".into()))?;
    let mut xml = String::new();
    document
        .read_to_string(&mut xml)
        .map_err(|e| crate::parsers::ParseError::ParseFailed(e.to_string()))?;

    let content = extract_docx_text(&xml);
    if content.trim().is_empty() {
        Err(crate::parsers::ParseError::ParseFailed(
            "empty document".into(),
        ))
    } else {
        Ok(content)
    }
}

fn extract_docx_text(xml: &str) -> String {
    let mut result = String::new();
    let mut in_para = false;
    let mut in_text = false;
    let bytes = xml.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'<' {
            let tag_end = xml[i..].find('>').map(|p| i + p + 1).unwrap_or(len);
            let tag = &xml[i..tag_end];
            if tag.starts_with("<w:p") {
                in_para = true;
                if !result.is_empty() && !result.ends_with('\n') {
                    result.push('\n');
                }
            } else if tag.starts_with("</w:p") {
                in_para = false;
                in_text = false;
            } else if tag.starts_with("<w:t") && !tag.starts_with("</") {
                in_text = true;
            } else if tag.starts_with("</w:t") {
                in_text = false;
            } else if tag.starts_with("<w:br") && in_para {
                result.push('\n');
            }
            i = tag_end;
        } else if in_text {
            let text_end = xml[i..].find('<').map(|p| i + p).unwrap_or(len);
            result.push_str(&xml[i..text_end]);
            i = text_end;
        } else {
            i += 1;
        }
    }

    result
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
