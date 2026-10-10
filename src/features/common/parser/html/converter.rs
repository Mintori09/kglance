use quick_xml::events::Event;
use quick_xml::reader::Reader;

use super::entities::{decode_html_entities, normalize_nfc};
use super::extract::extract_id_or_name;

pub fn convert_html_to_markdown(html: &str) -> String {
    let mut converter = HtmlToMarkdownConverter::new();
    converter.convert(html)
}

pub struct HtmlToMarkdownConverter {
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
    link_stack: Vec<String>,
    sub_depth: usize,
    sup_depth: usize,
}

#[derive(Clone, Copy)]
struct ListContext {
    ordered: bool,
    index: usize,
}

impl Default for HtmlToMarkdownConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl HtmlToMarkdownConverter {
    pub fn new() -> Self {
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
            link_stack: Vec::new(),
            sub_depth: 0,
            sup_depth: 0,
        }
    }

    pub fn convert(&mut self, html: &str) -> String {
        let mut reader = Reader::from_str(html);
        reader.config_mut().check_end_names = false;
        reader.config_mut().trim_text(false);

        let mut buf = Vec::with_capacity(256);

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    self.handle_start_tag(name.as_ref(), e);
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    self.handle_end_tag(name.as_ref());
                }
                Ok(Event::Empty(ref e)) => {
                    let name = e.name();
                    self.handle_empty_tag(name.as_ref(), e);
                }
                Ok(Event::Text(ref e)) => {
                    let raw_str = e.as_ref();
                    self.handle_text(raw_str);
                }
                Ok(Event::CData(ref e)) => {
                    let raw_str = e.as_ref();
                    self.handle_text(raw_str);
                }
                Ok(Event::GeneralRef(ref e)) => {
                    let raw = e.as_ref();
                    if raw.starts_with('&') && raw.ends_with(';') {
                        self.handle_text(raw);
                    } else {
                        self.handle_text(&format!("&{raw};"));
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }

        let decoded = decode_html_entities(&self.output);
        let normalized = normalize_nfc(&decoded);
        let cleaned = if normalized.contains("\\-") {
            normalized.replace("\\-", "-")
        } else {
            normalized
        };

        clean_markdown_output(&cleaned)
    }

    fn handle_start_tag(&mut self, name: &str, e: &quick_xml::events::BytesStart) {
        let name_lower = if name.bytes().any(|b| b.is_ascii_uppercase()) {
            std::borrow::Cow::Owned(name.to_ascii_lowercase())
        } else {
            std::borrow::Cow::Borrowed(name)
        };
        let name = name_lower.as_ref();

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
                let id_attr = extract_id_or_name(e);
                if let Some(id) = id_attr {
                    self.append_str(&format!("\n\n<a id=\"{id}\"></a>\n\n{hashes} "));
                } else {
                    self.append_str(&format!("\n\n{hashes} "));
                }
            }
            "p" | "div" | "blockquote" | "section" | "article" | "header" | "footer" | "aside"
            | "figure" | "figcaption" | "main" | "body" | "dt" | "dd" => {
                if let Some(id) = extract_id_or_name(e) {
                    self.append_str(&format!("\n\n<a id=\"{id}\"></a>\n\n"));
                }
                self.ensure_newline();
            }
            "a" => {
                let mut href = None;
                for attr in e.attributes().flatten() {
                    if attr.key.as_ref().eq_ignore_ascii_case("href") {
                        let val = attr.value.to_string();
                        if !val.trim().is_empty() {
                            href = Some(val);
                        }
                    }
                }
                if let Some(id) = extract_id_or_name(e) {
                    if self.output.ends_with('\n') || self.output.is_empty() {
                        self.append_str(&format!("<a id=\"{id}\"></a>\n\n"));
                    } else {
                        self.append_str(&format!("<a id=\"{id}\"></a> "));
                    }
                }
                if let Some(h) = href {
                    self.link_stack.push(h);
                    self.append_str("[");
                }
            }
            "span" => {
                if let Some(id) = extract_id_or_name(e) {
                    if self.output.ends_with('\n') || self.output.is_empty() {
                        self.append_str(&format!("<a id=\"{id}\"></a>\n\n"));
                    } else {
                        self.append_str(&format!("<a id=\"{id}\"></a> "));
                    }
                }
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
                if let Some(id) = extract_id_or_name(e) {
                    self.append_str(&format!("<a id=\"{id}\"></a> "));
                }
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
            "b" | "strong" => {
                if let Some(id) = extract_id_or_name(e) {
                    self.append_str(&format!("<a id=\"{id}\"></a> "));
                }
                self.append_str("**");
            }
            "i" | "em" => {
                if let Some(id) = extract_id_or_name(e) {
                    self.append_str(&format!("<a id=\"{id}\"></a> "));
                }
                self.append_str("*");
            }
            "del" | "s" | "strike" => {
                self.append_str("~~");
            }
            "sub" => {
                self.sub_depth += 1;
                self.append_str("~");
            }
            "sup" => {
                self.sup_depth += 1;
                self.append_str("^");
            }
            "hr" => self.append_str("\n\n---\n\n"),
            _ => {
                if let Some(id) = extract_id_or_name(e) {
                    if self.output.ends_with('\n') || self.output.is_empty() {
                        self.append_str(&format!("<a id=\"{id}\"></a>\n\n"));
                    } else {
                        self.append_str(&format!("<a id=\"{id}\"></a> "));
                    }
                }
            }
        }
    }

    fn handle_end_tag(&mut self, name: &str) {
        let name_lower = if name.bytes().any(|b| b.is_ascii_uppercase()) {
            std::borrow::Cow::Owned(name.to_ascii_lowercase())
        } else {
            std::borrow::Cow::Borrowed(name)
        };
        let name = name_lower.as_ref();

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
            "a" => {
                if let Some(href) = self.link_stack.pop() {
                    self.append_str(&format!("]({href})"));
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
            "del" | "s" | "strike" => self.append_str("~~"),
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
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                self.ensure_newline();
                self.append_str("\n");
            }
            "p" | "div" | "blockquote" => {
                self.ensure_newline();
            }
            _ => {}
        }
    }

    fn handle_empty_tag(&mut self, name: &str, e: &quick_xml::events::BytesStart) {
        let name_lower = if name.bytes().any(|b| b.is_ascii_uppercase()) {
            std::borrow::Cow::Owned(name.to_ascii_lowercase())
        } else {
            std::borrow::Cow::Borrowed(name)
        };
        let name = name_lower.as_ref();

        if self.skip_depth > 0 {
            return;
        }

        if let Some(id) = extract_id_or_name(e) {
            if self.output.ends_with('\n') || self.output.is_empty() {
                self.append_str(&format!("<a id=\"{id}\"></a>\n\n"));
            } else {
                self.append_str(&format!("<a id=\"{id}\"></a> "));
            }
        }

        match name {
            "br" => self.append_str("\n"),
            "hr" => self.append_str("\n\n---\n\n"),
            "img" => {
                let mut src = String::new();
                let mut alt = "image".to_string();

                for attr in e.attributes().flatten() {
                    if attr.key.as_ref().eq_ignore_ascii_case("src") {
                        src = attr.value.to_string();
                    } else if attr.key.as_ref().eq_ignore_ascii_case("alt") {
                        alt = attr.value.to_string();
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
            self.table_rows.iter().map(Vec::len).max().unwrap_or(1)
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
        if attr.key.as_ref().eq_ignore_ascii_case(attr_name)
            && attr.value.to_ascii_lowercase().contains(target_val)
        {
            return true;
        }
    }
    false
}

fn has_tex_encoding(e: &quick_xml::events::BytesStart) -> bool {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref().eq_ignore_ascii_case("encoding") {
            let val = attr.value.to_ascii_lowercase();
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
        if key.eq_ignore_ascii_case("class") {
            let val = attr.value.as_ref();
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
        } else if key.eq_ignore_ascii_case("data-lang")
            || key.eq_ignore_ascii_case("data-language")
            || key.eq_ignore_ascii_case("data-code-language")
        {
            let val = attr.value.as_ref();
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
    let mut result = String::with_capacity(text.len());
    let mut in_code_fence = false;
    let mut in_table = false;
    let mut in_list = false;

    for line in text.lines() {
        let trimmed_start = line.trim_start();
        if trimmed_start.starts_with("```") {
            in_code_fence = !in_code_fence;
            if !result.is_empty() && !result.ends_with('\n') {
                result.push('\n');
            }
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if in_code_fence {
            result.push_str(line);
            result.push('\n');
            continue;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            in_table = false;
            in_list = false;
            continue;
        }

        let is_table_row = trimmed.starts_with('|') && trimmed.ends_with('|');
        let is_list_item = trimmed_start.starts_with("- ")
            || trimmed_start.starts_with("* ")
            || trimmed_start.starts_with("+ ")
            || (trimmed_start
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_digit())
                && trimmed_start.find(". ").is_some());

        if is_table_row {
            if !in_table && !result.is_empty() {
                result.push_str("\n\n");
            } else if in_table {
                result.push('\n');
            }
            result.push_str(trimmed);
            in_table = true;
            in_list = false;
        } else if is_list_item {
            if !in_list && !result.is_empty() {
                result.push_str("\n\n");
            } else if in_list {
                result.push('\n');
            }
            result.push_str(line);
            in_list = true;
            in_table = false;
        } else {
            if !result.is_empty() {
                result.push_str("\n\n");
            }
            result.push_str(trimmed);
            in_table = false;
            in_list = false;
        }
    }

    result
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
