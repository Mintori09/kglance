use std::path::Path;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

pub fn convert_html_to_markdown(html: &str) -> String {
    let mut converter = HtmlToMarkdownConverter::new();
    converter.convert(html)
}

struct HtmlToMarkdownConverter {
    output: String,
    skip_depth: usize,
    in_pre: bool,
    pre_lang: Option<String>,
    pre_buffer: String,
    in_code: bool,
    in_table: bool,
    table_headers: Vec<String>,
    table_rows: Vec<Vec<String>>,
    current_row: Vec<String>,
    current_cell: String,
    in_cell: bool,
    in_math: bool,
    math_buffer: String,
    math_annotation_tex: Option<String>,
    in_math_annotation_tex: bool,
    in_math_display: bool,
    list_stack: Vec<ListContext>,
    sub_depth: usize,
    sup_depth: usize,
}

#[derive(Clone, Copy)]
struct ListContext {
    ordered: bool,
    index: usize,
}

impl HtmlToMarkdownConverter {
    fn new() -> Self {
        Self {
            output: String::with_capacity(1024),
            skip_depth: 0,
            in_pre: false,
            pre_lang: None,
            pre_buffer: String::new(),
            in_code: false,
            in_table: false,
            table_headers: Vec::new(),
            table_rows: Vec::new(),
            current_row: Vec::new(),
            current_cell: String::new(),
            in_cell: false,
            in_math: false,
            math_buffer: String::new(),
            math_annotation_tex: None,
            in_math_annotation_tex: false,
            in_math_display: false,
            list_stack: Vec::new(),
            sub_depth: 0,
            sup_depth: 0,
        }
    }

    fn convert(&mut self, html: &str) -> String {
        let mut reader = Reader::from_str(html);
        reader.config_mut().check_end_names = false;
        reader.config_mut().trim_text(false);

        let mut buf = Vec::with_capacity(256);

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();
                    self.handle_start_tag(&name, e);
                }
                Ok(Event::End(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();
                    self.handle_end_tag(&name);
                }
                Ok(Event::Empty(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();
                    self.handle_empty_tag(&name, e);
                }
                Ok(Event::Text(ref e)) => {
                    let raw_str = String::from_utf8_lossy(e.as_ref());
                    self.handle_text(&raw_str);
                }
                Ok(Event::CData(ref e)) => {
                    let raw_str = String::from_utf8_lossy(e.as_ref());
                    self.handle_text(&raw_str);
                }
                Ok(Event::GeneralRef(ref e)) => {
                    let raw = String::from_utf8_lossy(e.as_ref());
                    if raw.starts_with('&') && raw.ends_with(';') {
                        self.handle_text(&raw);
                    } else {
                        self.handle_text(&format!("&{raw};"));
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => {
                    break;
                }
                _ => {}
            }
            buf.clear();
        }

        let decoded = decode_html_entities(&self.output);
        let normalized = normalize_nfc(&decoded);
        let cleaned = normalized.replace("\\-", "-");

        clean_markdown_output(&cleaned)
    }

    fn handle_start_tag(&mut self, name: &str, e: &quick_xml::events::BytesStart) {
        if matches!(name, "style" | "script" | "head" | "title") {
            self.skip_depth += 1;
            return;
        }

        if self.skip_depth > 0 {
            return;
        }

        match name {
            "math" => {
                self.in_math = true;
                self.math_buffer.clear();
                self.math_annotation_tex = None;
                self.in_math_annotation_tex = false;
                self.in_math_display = has_class_or_attr(e, "display", "block")
                    || has_class_or_attr(e, "mode", "display");
            }
            "annotation" if self.in_math => {
                if has_tex_encoding(e) {
                    self.in_math_annotation_tex = true;
                }
            }
            "table" => {
                self.in_table = true;
                self.table_headers.clear();
                self.table_rows.clear();
            }
            "tr" if self.in_table => {
                self.current_row.clear();
            }
            "th" if self.in_table => {
                self.in_cell = true;
                self.current_cell.clear();
            }
            "td" if self.in_table => {
                self.in_cell = true;
                self.current_cell.clear();
            }
            "pre" => {
                self.in_pre = true;
                self.pre_buffer.clear();
                if self.pre_lang.is_none() {
                    self.pre_lang = extract_code_lang(e);
                }
            }
            "code" => {
                if self.in_pre {
                    if self.pre_lang.is_none() {
                        self.pre_lang = extract_code_lang(e);
                    }
                } else {
                    self.in_code = true;
                    self.append_str("`");
                }
            }
            "span" => {
                if has_class_or_attr(e, "class", "math inline") {
                    self.in_math = true;
                    self.math_buffer.clear();
                    self.math_annotation_tex = None;
                    self.in_math_display = false;
                } else if has_class_or_attr(e, "class", "math display") {
                    self.in_math = true;
                    self.math_buffer.clear();
                    self.math_annotation_tex = None;
                    self.in_math_display = true;
                }
            }
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let hashes = match name {
                    "h1" => "#",
                    "h2" => "##",
                    "h3" => "###",
                    "h4" => "####",
                    "h5" => "#####",
                    "h6" => "######",
                    _ => "#",
                };
                let id_attr = extract_attr_from_event(e, b"id")
                    .or_else(|| extract_attr_from_event(e, b"name"));
                if let Some(id) = id_attr {
                    self.append_str(&format!("\n\n<a id=\"{id}\"></a>\n{hashes} "));
                } else {
                    self.append_str(&format!("\n\n{hashes} "));
                }
            }
            "p" | "div" | "blockquote" => {
                let id_attr = extract_attr_from_event(e, b"id")
                    .or_else(|| extract_attr_from_event(e, b"name"));
                if let Some(id) = id_attr {
                    self.append_str(&format!("\n\n<a id=\"{id}\"></a>\n"));
                }
                self.ensure_newline();
            }
            "ul" => {
                self.list_stack.push(ListContext {
                    ordered: false,
                    index: 0,
                });
            }
            "ol" => {
                self.list_stack.push(ListContext {
                    ordered: true,
                    index: 1,
                });
            }
            "li" => {
                self.ensure_newline();
                let indent_level = self.list_stack.len().saturating_sub(1);
                let indent = "  ".repeat(indent_level);
                if let Some(ctx) = self.list_stack.last_mut() {
                    if ctx.ordered {
                        let idx = ctx.index;
                        ctx.index += 1;
                        self.append_str(&format!("{indent}{idx}. "));
                    } else {
                        self.append_str(&format!("{indent}- "));
                    }
                } else {
                    self.append_str("- ");
                }
            }
            "b" | "strong" => self.append_str("**"),
            "i" | "em" => self.append_str("*"),
            "sub" => {
                self.sub_depth += 1;
                self.append_str("~");
            }
            "sup" => {
                self.sup_depth += 1;
                self.append_str("^");
            }
            "hr" => self.append_str("\n\n---\n\n"),
            _ => {}
        }
    }

    fn handle_end_tag(&mut self, name: &str) {
        if matches!(name, "style" | "script" | "head" | "title") {
            if self.skip_depth > 0 {
                self.skip_depth -= 1;
            }
            return;
        }

        if self.skip_depth > 0 {
            return;
        }

        match name {
            "math" => {
                self.in_math = false;
                self.in_math_annotation_tex = false;
                let tex = if let Some(ref t) = self.math_annotation_tex {
                    t.trim().to_string()
                } else {
                    mathml_to_latex(&self.math_buffer)
                };
                if !tex.is_empty() {
                    if self.in_math_display {
                        self.append_str(&format!("\n\n$${tex}$$\n\n"));
                    } else {
                        self.append_str(&format!("${tex}$"));
                    }
                }
                self.math_buffer.clear();
                self.math_annotation_tex = None;
            }
            "annotation" if self.in_math => {
                self.in_math_annotation_tex = false;
            }
            "span" if self.in_math => {
                self.in_math = false;
                let tex = self.math_buffer.trim();
                if !tex.is_empty() {
                    if self.in_math_display {
                        self.append_str(&format!("\n\n$${tex}$$\n\n"));
                    } else {
                        self.append_str(&format!("${tex}$"));
                    }
                }
                self.math_buffer.clear();
            }
            "pre" => {
                self.in_pre = false;
                let mut lang = self.pre_lang.take().unwrap_or_default();
                let code = self.pre_buffer.trim_end_matches('\n');
                if lang.is_empty()
                    && let Some(detected) =
                        crate::features::markdown::view::highlight::detect_code_language(code)
                {
                    lang = detected.to_string();
                }
                self.append_str(&format!("\n\n```{lang}\n{code}\n```\n\n"));
                self.pre_buffer.clear();
            }
            "code" => {
                if !self.in_pre {
                    self.in_code = false;
                    self.append_str("`");
                }
            }
            "th" if self.in_table => {
                self.in_cell = false;
                let cell_txt = clean_table_cell(&self.current_cell);
                self.table_headers.push(cell_txt);
                self.current_cell.clear();
            }
            "td" if self.in_table => {
                self.in_cell = false;
                let cell_txt = clean_table_cell(&self.current_cell);
                self.current_row.push(cell_txt);
                self.current_cell.clear();
            }
            "tr" if self.in_table => {
                if !self.current_row.is_empty() {
                    self.table_rows.push(std::mem::take(&mut self.current_row));
                }
            }
            "table" => {
                self.in_table = false;
                self.flush_table();
            }
            "ul" | "ol" => {
                self.list_stack.pop();
                self.ensure_newline();
            }
            "b" | "strong" => self.append_str("**"),
            "i" | "em" => self.append_str("*"),
            "sub" => {
                if self.sub_depth > 0 {
                    self.sub_depth -= 1;
                    self.append_str("~");
                }
            }
            "sup" => {
                if self.sup_depth > 0 {
                    self.sup_depth -= 1;
                    self.append_str("^");
                }
            }
            "p" | "div" | "blockquote" => {
                self.ensure_newline();
            }
            _ => {}
        }
    }

    fn handle_empty_tag(&mut self, name: &str, e: &quick_xml::events::BytesStart) {
        if self.skip_depth > 0 {
            return;
        }

        match name {
            "br" => self.append_str("\n"),
            "hr" => self.append_str("\n\n---\n\n"),
            "img" => {
                let mut src = String::new();
                let mut alt = String::from("image");

                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_lowercase();
                    if key == "src" {
                        src = String::from_utf8_lossy(&attr.value).to_string();
                    } else if key == "alt" {
                        alt = String::from_utf8_lossy(&attr.value).to_string();
                    }
                }

                if !src.is_empty() {
                    self.append_str(&format!("\n\n![{alt}]({src})\n\n"));
                }
            }
            _ => {}
        }
    }

    fn handle_text(&mut self, text: &str) {
        if self.skip_depth > 0 {
            return;
        }

        if self.in_pre {
            self.pre_buffer.push_str(text);
            return;
        }

        if self.in_math {
            if self.in_math_annotation_tex {
                self.math_annotation_tex
                    .get_or_insert_with(String::new)
                    .push_str(text);
            } else {
                self.math_buffer.push_str(text);
            }
            return;
        }

        if self.in_cell {
            self.current_cell.push_str(text);
            return;
        }

        self.append_str(text);
    }

    fn append_str(&mut self, s: &str) {
        if self.in_cell {
            self.current_cell.push_str(s);
        } else {
            self.output.push_str(s);
        }
    }

    fn ensure_newline(&mut self) {
        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
    }

    fn flush_table(&mut self) {
        if self.table_headers.is_empty() && self.table_rows.is_empty() {
            return;
        }

        self.ensure_newline();
        self.output.push('\n');

        let col_count = if !self.table_headers.is_empty() {
            self.table_headers.len()
        } else {
            self.table_rows.iter().map(|r| r.len()).max().unwrap_or(1)
        };

        if col_count == 0 {
            return;
        }

        // Print header row
        self.output.push('|');
        if !self.table_headers.is_empty() {
            for h in &self.table_headers {
                self.output.push_str(&format!(" {h} |"));
            }
            for _ in self.table_headers.len()..col_count {
                self.output.push_str("  |");
            }
        } else {
            for i in 0..col_count {
                self.output.push_str(&format!(" Col {} |", i + 1));
            }
        }
        self.output.push('\n');

        // Print separator
        self.output.push('|');
        for _ in 0..col_count {
            self.output.push_str(" --- |");
        }
        self.output.push('\n');

        // Print data rows
        for row in &self.table_rows {
            self.output.push('|');
            for cell in row {
                self.output.push_str(&format!(" {cell} |"));
            }
            for _ in row.len()..col_count {
                self.output.push_str("  |");
            }
            self.output.push('\n');
        }

        self.output.push('\n');
        self.table_headers.clear();
        self.table_rows.clear();
    }
}

fn clean_table_cell(raw: &str) -> String {
    let unescaped = raw.replace('|', "\\|");
    let collapsed = unescaped.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed.trim().to_string()
}

fn has_class_or_attr(e: &quick_xml::events::BytesStart, attr_name: &str, target_val: &str) -> bool {
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_lowercase();
        if key == attr_name {
            let val = String::from_utf8_lossy(&attr.value).to_lowercase();
            if val.contains(target_val) {
                return true;
            }
        }
    }
    false
}

fn has_tex_encoding(e: &quick_xml::events::BytesStart) -> bool {
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_lowercase();
        if key == "encoding" {
            let val = String::from_utf8_lossy(&attr.value).to_lowercase();
            if val.contains("tex") || val.contains("latex") {
                return true;
            }
        }
    }
    false
}

fn extract_code_lang(e: &quick_xml::events::BytesStart) -> Option<String> {
    for attr in e.attributes().flatten() {
        let key = attr.key.as_ref();
        if key.eq_ignore_ascii_case(b"class") {
            let val = String::from_utf8_lossy(&attr.value);
            for part in val.split_whitespace() {
                let trimmed = part.trim_matches('"').trim_matches('\'');
                if let Some(lang) = trimmed.strip_prefix("language-") {
                    return Some(clean_lang_token(lang));
                }
                if let Some(lang) = trimmed.strip_prefix("lang-") {
                    return Some(clean_lang_token(lang));
                }
                if let Some(lang) = trimmed.strip_prefix("highlight-") {
                    return Some(clean_lang_token(lang));
                }
                if is_known_code_lang(trimmed) {
                    return Some(clean_lang_token(trimmed));
                }
            }
        } else if key.eq_ignore_ascii_case(b"data-lang")
            || key.eq_ignore_ascii_case(b"data-language")
            || key.eq_ignore_ascii_case(b"data-code-language")
        {
            let val = String::from_utf8_lossy(&attr.value);
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                return Some(clean_lang_token(trimmed));
            }
        }
    }
    None
}

fn is_known_code_lang(token: &str) -> bool {
    matches!(
        token.to_lowercase().as_str(),
        "rust"
            | "rs"
            | "typescript"
            | "ts"
            | "javascript"
            | "js"
            | "python"
            | "py"
            | "cpp"
            | "c"
            | "c++"
            | "csharp"
            | "cs"
            | "java"
            | "go"
            | "golang"
            | "html"
            | "css"
            | "json"
            | "xml"
            | "yaml"
            | "yml"
            | "toml"
            | "sql"
            | "sh"
            | "bash"
            | "zsh"
            | "lua"
            | "kotlin"
            | "kt"
            | "swift"
            | "php"
            | "ruby"
            | "rb"
            | "dockerfile"
            | "diff"
    )
}

fn clean_lang_token(token: &str) -> String {
    token.to_lowercase()
}

fn clean_markdown_output(text: &str) -> String {
    let mut result = Vec::new();
    let mut in_code_fence = false;
    let mut code_fence_lines: Vec<String> = Vec::new();

    for line in text.lines() {
        let trimmed_start = line.trim_start();
        if trimmed_start.starts_with("```") {
            if in_code_fence {
                code_fence_lines.push(line.to_string());
                result.push(code_fence_lines.join("\n"));
                code_fence_lines.clear();
                in_code_fence = false;
            } else {
                in_code_fence = true;
                code_fence_lines.clear();
                code_fence_lines.push(line.to_string());
            }
            continue;
        }

        if in_code_fence {
            code_fence_lines.push(line.to_string());
        } else {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                result.push(trimmed.to_string());
            }
        }
    }

    if !code_fence_lines.is_empty() {
        result.push(code_fence_lines.join("\n"));
    }

    result.join("\n\n")
}

fn mathml_to_latex(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    trimmed.to_string()
}

pub fn strip_html_tags(html: &str) -> String {
    convert_html_to_markdown(html)
}

pub fn decode_html_entities(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '&' {
            let mut entity_buf = String::new();
            let mut found_semi = false;

            while let Some(&next_ch) = chars.peek() {
                if next_ch == ';' {
                    chars.next();
                    found_semi = true;
                    break;
                } else if next_ch.is_alphanumeric() || next_ch == '#' {
                    entity_buf.push(chars.next().unwrap());
                    if entity_buf.len() > 16 {
                        break;
                    }
                } else {
                    break;
                }
            }

            if found_semi {
                if let Some(decoded) = decode_named_or_numeric_entity(&entity_buf) {
                    result.push(decoded);
                    continue;
                }
                result.push('&');
                result.push_str(&entity_buf);
                result.push(';');
            } else {
                result.push('&');
                result.push_str(&entity_buf);
            }
        } else {
            result.push(ch);
        }
    }

    normalize_nfc(&result)
}

pub fn normalize_nfc(text: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    let preprocessed = text.replace("ộ\u{0301}", "ối").replace("ộ´", "ối");

    let mut transformed = String::with_capacity(preprocessed.len());
    let mut prev_char: Option<char> = None;
    for c in preprocessed.chars() {
        match c {
            '´' => transformed.push('\u{0301}'),
            '`' if prev_char.is_some_and(|p| p.is_alphabetic() && p != '`') => {
                transformed.push('\u{0300}');
            }
            other => transformed.push(other),
        }
        prev_char = Some(c);
    }

    transformed.nfc().collect()
}

fn decode_named_or_numeric_entity(entity: &str) -> Option<char> {
    if let Some(hex) = entity
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
            "ldquo" => Some('“'),
            "rdquo" => Some('”'),
            "lsquo" => Some('‘'),
            "rsquo" => Some('’'),
            _ => None,
        }
    }
}

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
                    current_heading_id = extract_attr_from_event(e, b"id");
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
                if current_heading_level.is_some()
                    && let Ok(txt) = e.decode()
                {
                    current_heading_text.push_str(&txt);
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

fn parse_heading_tag(name: &[u8]) -> Option<u8> {
    match name {
        b"h1" | b"H1" => Some(1),
        b"h2" | b"H2" => Some(2),
        b"h3" | b"H3" => Some(3),
        b"h4" | b"H4" => Some(4),
        b"h5" | b"H5" => Some(5),
        b"h6" | b"H6" => Some(6),
        _ => None,
    }
}

fn extract_attr_from_event(e: &quick_xml::events::BytesStart, attr_name: &[u8]) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref().eq_ignore_ascii_case(attr_name) {
            return String::from_utf8(attr.value.to_vec()).ok();
        }
    }
    None
}
