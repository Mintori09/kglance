use quick_xml::events::Event;
use quick_xml::reader::Reader;

use super::entities::{decode_html_entities, decode_html_entities_only, normalize_nfc};
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
    sup_buffer: String,
    in_ruby: bool,
    ruby_base: String,
    ruby_text: String,
    in_rt: bool,
    in_rp: bool,
    quote_stack: Vec<String>,
    span_format_stack: Vec<&'static str>,
    div_callout_stack: Vec<Option<&'static str>>,
    in_figure: bool,
    in_figure_caption: bool,
    in_callout_heading: bool,
    li_just_started: bool,
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
            sup_buffer: String::new(),
            in_ruby: false,
            ruby_base: String::new(),
            ruby_text: String::new(),
            in_rt: false,
            in_rp: false,
            quote_stack: Vec::new(),
            span_format_stack: Vec::new(),
            div_callout_stack: Vec::new(),
            in_figure: false,
            in_figure_caption: false,
            in_callout_heading: false,
            li_just_started: false,
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
                Ok(Event::Eof) | Err(_) => {
                    if self.in_ruby {
                        self.in_ruby = false;
                        self.flush_ruby_pair();
                    }
                    break;
                }
                _ => {}
            }
            buf.clear();
        }

        let decoded = decode_html_entities_only(&self.output);
        let cleaned = if decoded.contains("\\-") {
            decoded.replace("\\-", "-")
        } else {
            decoded
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
                if self.div_callout_stack.last().copied().flatten().is_some() {
                    self.in_callout_heading = true;
                    return;
                }
                if name == "h6" && self.in_figure {
                    self.in_figure_caption = true;
                    self.append_str("\n\n*");
                    return;
                }
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
            "blockquote" => {
                if let Some(id) = extract_id_or_name(e) {
                    self.append_str(&format!("\n\n<a id=\"{id}\"></a>\n\n"));
                }
                self.ensure_newline();
                self.quote_stack.push(String::with_capacity(256));
            }
            "figure" => {
                self.in_figure = true;
                if let Some(id) = extract_id_or_name(e) {
                    self.append_str(&format!("\n\n<a id=\"{id}\"></a>\n\n"));
                }
                self.ensure_newline();
            }
            "aside" => {
                let is_sidebar = has_class_or_attr(e, "data-type", "sidebar")
                    || has_class_or_attr(e, "epub:type", "sidebar")
                    || has_class_or_attr(e, "class", "sidebar");
                if is_sidebar {
                    if let Some(id) = extract_id_or_name(e) {
                        self.append_str(&format!("\n\n<a id=\"{id}\"></a>\n\n"));
                    }
                    self.ensure_newline();
                    self.quote_stack.push(String::with_capacity(256));
                } else {
                    if let Some(id) = extract_id_or_name(e) {
                        self.append_str(&format!("\n\n<a id=\"{id}\"></a>\n\n"));
                    }
                    self.ensure_newline();
                }
            }
            "div" => {
                let mut callout_kind = None;
                for attr in e.attributes().flatten() {
                    let key = attr.key.as_ref();
                    if key.eq_ignore_ascii_case("data-type")
                        || key.eq_ignore_ascii_case("epub:type")
                    {
                        let val = attr.value.to_ascii_lowercase();
                        let token = match val.as_str() {
                            "note" => Some("NOTE"),
                            "tip" => Some("TIP"),
                            "warning" => Some("WARNING"),
                            "caution" => Some("CAUTION"),
                            "important" => Some("IMPORTANT"),
                            _ => None,
                        };
                        if token.is_some() {
                            callout_kind = token;
                            break;
                        }
                    }
                }
                self.div_callout_stack.push(callout_kind);
                if let Some(kind) = callout_kind {
                    if let Some(id) = extract_id_or_name(e) {
                        self.append_str(&format!("\n\n<a id=\"{id}\"></a>\n\n"));
                    }
                    self.ensure_newline();
                    let mut q = String::with_capacity(256);
                    q.push_str(&format!("[!{kind}]\n"));
                    self.quote_stack.push(q);
                } else {
                    if let Some(id) = extract_id_or_name(e) {
                        self.append_str(&format!("\n\n<a id=\"{id}\"></a>\n\n"));
                    }
                    self.ensure_newline();
                }
            }
            "p" | "section" | "article" | "header" | "footer" | "figcaption" | "main" | "body"
            | "dt" | "dd" => {
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
                } else {
                    let mut is_bold = false;
                    let mut is_italic = false;
                    for attr in e.attributes().flatten() {
                        let key = attr.key.as_ref();
                        let val = attr.value.to_ascii_lowercase();
                        if key.eq_ignore_ascii_case("class") {
                            for class_token in val.split_whitespace() {
                                if class_token == "text_2" || class_token.ends_with("_italic") {
                                    is_italic = true;
                                } else if class_token == "text_" || class_token.ends_with("_bold") {
                                    is_bold = true;
                                } else if class_token == "text_3" {
                                    is_bold = true;
                                    is_italic = true;
                                }
                            }
                        } else if key.eq_ignore_ascii_case("style") {
                            if val.contains("font-style: italic")
                                || val.contains("font-style:italic")
                            {
                                is_italic = true;
                            }
                            if val.contains("font-weight: bold") || val.contains("font-weight:bold")
                            {
                                is_bold = true;
                            }
                        }
                    }

                    let delimiter = match (is_bold, is_italic) {
                        (true, true) => Some("***"),
                        (true, false) => Some("**"),
                        (false, true) => Some("*"),
                        (false, false) => None,
                    };

                    if let Some(delim) = delimiter {
                        self.append_str(delim);
                        self.span_format_stack.push(delim);
                    } else {
                        self.span_format_stack.push("");
                    }
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
                self.li_just_started = true;
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
                if self.sup_depth == 1 {
                    self.sup_buffer.clear();
                }
            }
            "hr" => self.append_str("\n\n---\n\n"),
            "ruby" => {
                self.in_ruby = true;
                self.ruby_base.clear();
                self.ruby_text.clear();
                self.in_rt = false;
                self.in_rp = false;
            }
            "rt" => {
                if self.in_ruby {
                    self.in_rt = true;
                    self.in_rp = false;
                }
            }
            "rp" => {
                if self.in_ruby {
                    self.in_rp = true;
                }
            }
            "rb" => {
                if self.in_ruby && !self.ruby_text.is_empty() {
                    self.flush_ruby_pair();
                }
                self.in_rt = false;
                self.in_rp = false;
            }
            "img" | "image" => self.handle_img_tag(e),
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
                if self.in_ruby {
                    self.in_ruby = false;
                    self.flush_ruby_pair();
                }
                self.in_cell = false;
                let cell_txt = clean_table_cell(&self.current_cell);
                self.table_headers.push(cell_txt);
                self.current_cell.clear();
            }
            "td" if self.in_table => {
                if self.in_ruby {
                    self.in_ruby = false;
                    self.flush_ruby_pair();
                }
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
            "span" => {
                if let Some(delim) = self.span_format_stack.pop()
                    && !delim.is_empty()
                {
                    self.append_str(delim);
                }
            }
            "sub" => {
                if self.sub_depth > 0 {
                    self.sub_depth -= 1;
                    self.append_str("~");
                }
            }
            "sup" => {
                if self.sup_depth > 0 {
                    self.sup_depth -= 1;
                    if self.sup_depth == 0 {
                        let content = self.sup_buffer.trim();
                        // If it is a footnote reference (e.g. "1", "2", "*", "i")
                        if is_footnote_label(content) {
                            self.append_str(&format!("[^{content}]"));
                        } else if !content.is_empty() {
                            self.append_str(&to_superscript_unicode(content));
                        }
                        self.sup_buffer.clear();
                    }
                }
            }
            "rt" => {
                self.in_rt = false;
            }
            "rp" => {
                self.in_rp = false;
            }
            "ruby" => {
                if self.in_ruby {
                    self.in_ruby = false;
                    self.flush_ruby_pair();
                }
            }
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                if self.in_callout_heading {
                    self.in_callout_heading = false;
                    return;
                }
                if name == "h6" && self.in_figure_caption {
                    self.in_figure_caption = false;
                    self.append_str("*\n\n");
                    return;
                }
                if self.in_ruby {
                    self.in_ruby = false;
                    self.flush_ruby_pair();
                }
                self.ensure_newline();
                self.append_str("\n");
            }
            "figure" => {
                self.in_figure = false;
                self.ensure_newline();
            }
            "blockquote" | "aside" => {
                if let Some(quote_content) = self.quote_stack.pop() {
                    let trimmed = quote_content.trim();
                    if !trimmed.is_empty() {
                        let mut formatted = String::with_capacity(trimmed.len() + 32);
                        formatted.push_str("\n\n");
                        for line in trimmed.lines() {
                            let line_trim = line.trim();
                            if line_trim.is_empty() {
                                formatted.push_str(">\n");
                            } else {
                                formatted.push_str("> ");
                                formatted.push_str(line_trim);
                                formatted.push('\n');
                            }
                        }
                        formatted.push('\n');
                        self.append_str(&formatted);
                    }
                }
                self.ensure_newline();
            }
            "div" => {
                let callout_kind = self.div_callout_stack.pop().flatten();
                if callout_kind.is_some()
                    && let Some(quote_content) = self.quote_stack.pop()
                {
                    let trimmed = quote_content.trim();
                    if !trimmed.is_empty() {
                        let mut formatted = String::with_capacity(trimmed.len() + 32);
                        formatted.push_str("\n\n");
                        for line in trimmed.lines() {
                            let line_trim = line.trim();
                            if line_trim.is_empty() {
                                formatted.push_str(">\n");
                            } else {
                                formatted.push_str("> ");
                                formatted.push_str(line_trim);
                                formatted.push('\n');
                            }
                        }
                        formatted.push('\n');
                        self.append_str(&formatted);
                    }
                }
                if self.in_ruby {
                    self.in_ruby = false;
                    self.flush_ruby_pair();
                }
                if let Some(current_quote) = self.quote_stack.last_mut() {
                    if !current_quote.is_empty() && !current_quote.ends_with("\n\n") {
                        if current_quote.ends_with('\n') {
                            current_quote.push('\n');
                        } else {
                            current_quote.push_str("\n\n");
                        }
                    }
                } else {
                    self.ensure_newline();
                }
            }
            "p" | "section" | "article" => {
                if self.in_ruby {
                    self.in_ruby = false;
                    self.flush_ruby_pair();
                }
                if let Some(current_quote) = self.quote_stack.last_mut() {
                    if !current_quote.is_empty() && !current_quote.ends_with("\n\n") {
                        if current_quote.ends_with('\n') {
                            current_quote.push('\n');
                        } else {
                            current_quote.push_str("\n\n");
                        }
                    }
                } else {
                    self.ensure_newline();
                }
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
            "img" | "image" => self.handle_img_tag(e),
            _ => {}
        }
    }

    fn handle_img_tag(&mut self, e: &quick_xml::events::BytesStart) {
        let mut src = String::new();
        let mut alt = String::new();
        let mut width: Option<u32> = None;
        let mut height: Option<u32> = None;
        let mut style = String::new();

        for attr in e.attributes().flatten() {
            let key = attr.key.as_ref();
            if key.eq_ignore_ascii_case("src")
                || key.eq_ignore_ascii_case("href")
                || key.eq_ignore_ascii_case("xlink:href")
            {
                src = attr.value.to_string();
            } else if key.eq_ignore_ascii_case("alt") {
                alt = attr.value.to_string();
            } else if key.eq_ignore_ascii_case("width") {
                width = parse_dimension(attr.value.as_ref());
            } else if key.eq_ignore_ascii_case("height") {
                height = parse_dimension(attr.value.as_ref());
            } else if key.eq_ignore_ascii_case("style") {
                style = attr.value.to_string();
            }
        }

        if width.is_none() && !style.is_empty() {
            width = extract_style_dimension(&style, "width");
        }
        if height.is_none() && !style.is_empty() {
            height = extract_style_dimension(&style, "height");
        }

        if !src.is_empty() {
            if width.is_some() || height.is_some() {
                let w_attr = width.map(|w| format!(" width=\"{w}\"")).unwrap_or_default();
                let h_attr = height
                    .map(|h| format!(" height=\"{h}\""))
                    .unwrap_or_default();
                let alt_attr = if alt.is_empty() {
                    String::new()
                } else {
                    format!(" alt=\"{alt}\"")
                };
                self.append_str(&format!(
                    "\n\n<img src=\"{src}\"{alt_attr}{w_attr}{h_attr} />\n\n"
                ));
            } else {
                let alt_text = if alt.is_empty() { "image" } else { &alt };
                self.append_str(&format!("\n\n![{alt_text}]({src})\n\n"));
            }
        }
    }

    fn handle_text(&mut self, text: &str) {
        if self.skip_depth > 0 || self.in_callout_heading {
            return;
        }

        let actual_text = if self.li_just_started {
            if text.trim().is_empty() {
                return;
            }
            self.li_just_started = false;
            text.trim_start()
        } else {
            text
        };

        if self.in_pre {
            self.pre_buffer.push_str(actual_text);
            return;
        }

        let normalized_storage: String;
        let text_to_append = if actual_text.contains('\n') || actual_text.contains('\r') {
            let has_leading = actual_text.starts_with(|c: char| c.is_whitespace() || c == '\u{a0}');
            let has_trailing = actual_text.ends_with(|c: char| c.is_whitespace() || c == '\u{a0}');
            let mut words = actual_text.split_whitespace();
            if let Some(first_word) = words.next() {
                let mut buf = String::with_capacity(actual_text.len());
                if has_leading {
                    buf.push(' ');
                }
                buf.push_str(first_word);
                for word in words {
                    buf.push(' ');
                    buf.push_str(word);
                }
                if has_trailing {
                    buf.push(' ');
                }
                normalized_storage = buf;
                normalized_storage.as_str()
            } else if has_leading || has_trailing {
                " "
            } else {
                ""
            }
        } else {
            actual_text
        };

        if text_to_append.is_empty() {
            return;
        }

        let normalized_text_storage: String;
        let text_to_append = if text_to_append.contains('´')
            || text_to_append.contains('`')
            || text_to_append.contains('ộ')
        {
            normalized_text_storage = normalize_nfc(text_to_append);
            normalized_text_storage.as_str()
        } else {
            text_to_append
        };

        if self.in_math {
            if self.in_math_annotation_tex {
                self.math_annotation_tex
                    .get_or_insert_with(String::new)
                    .push_str(text_to_append);
            } else {
                self.math_buffer.push_str(text_to_append);
            }
            return;
        }

        if self.in_ruby {
            if self.in_rp {
                return;
            }
            if self.in_rt {
                self.ruby_text.push_str(text_to_append);
                return;
            }
            if !self.ruby_text.is_empty() {
                self.flush_ruby_pair();
            }
            self.ruby_base.push_str(text_to_append);
            return;
        }

        if self.in_cell {
            self.current_cell.push_str(text_to_append);
            return;
        }

        if self.sup_depth > 0 {
            self.sup_buffer.push_str(text_to_append);
            return;
        }

        self.append_str(text_to_append);
    }

    fn flush_ruby_pair(&mut self) {
        let base = decode_html_entities(&self.ruby_base);
        let ruby = decode_html_entities(&self.ruby_text);
        let base_trimmed = base.trim();
        let ruby_trimmed = ruby.trim();
        if !base_trimmed.is_empty() && !ruby_trimmed.is_empty() {
            self.append_str(&format!(
                "<ruby>{base_trimmed}<rt>{ruby_trimmed}</rt></ruby>"
            ));
        } else if !base_trimmed.is_empty() {
            self.append_str(base_trimmed);
        }
        self.ruby_base.clear();
        self.ruby_text.clear();
    }

    fn append_str(&mut self, s: &str) {
        if self.in_cell {
            self.current_cell.push_str(s);
        } else if let Some(current_quote) = self.quote_stack.last_mut() {
            current_quote.push_str(s);
        } else {
            self.output.push_str(s);
        }
    }

    fn ensure_newline(&mut self) {
        if let Some(current_quote) = self.quote_stack.last_mut() {
            if !current_quote.is_empty() && !current_quote.ends_with('\n') {
                current_quote.push('\n');
            }
        } else if !self.output.is_empty() && !self.output.ends_with('\n') {
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

pub const CUSTOM_BULLET_CHARS: &[char] = &['♦', '•', '▪', '▫', '✦', '►', '▸', '⁃', '◇', '▶', '●'];

fn is_custom_bullet_start(s: &str) -> Option<&str> {
    let mut chars = s.chars();
    if let Some(first) = chars.next()
        && CUSTOM_BULLET_CHARS.contains(&first)
    {
        let rest = chars.as_str();
        let stripped = rest.trim_start_matches(|c: char| c.is_whitespace() || c == '\u{a0}');
        if stripped.len() < rest.len() {
            return Some(stripped);
        }
        // Also support when delimiter like `**` immediately follows the bullet before whitespace
        if let Some(after_delim) = stripped
            .strip_prefix("**")
            .or_else(|| stripped.strip_prefix('*'))
        {
            let inner =
                after_delim.trim_start_matches(|c: char| c.is_whitespace() || c == '\u{a0}');
            if inner.len() < after_delim.len() {
                return Some(stripped);
            }
        }
    }
    None
}

fn clean_markdown_output(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut in_code_fence = false;
    let mut in_table = false;
    let mut in_list = false;
    let mut in_quote = false;

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
            in_quote = false;
            continue;
        }

        let is_quote_line = trimmed_start.starts_with("> ") || trimmed == ">";
        let is_table_row = trimmed.starts_with('|') && trimmed.ends_with('|');
        let custom_bullet_rest = is_custom_bullet_start(trimmed_start);
        let is_empty_bullet = trimmed == "-"
            || trimmed == "*"
            || trimmed == "+"
            || (trimmed_start
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_digit())
                && trimmed_start.ends_with('.')
                && !trimmed_start.contains(' '));
        if is_empty_bullet {
            continue;
        }
        let is_list_item = custom_bullet_rest.is_some()
            || trimmed_start.starts_with("- ")
            || trimmed_start.starts_with("* ")
            || trimmed_start.starts_with("+ ")
            || (trimmed_start
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_digit())
                && trimmed_start.find(". ").is_some());
        let is_footnote_def = trimmed_start.starts_with("[^") && trimmed_start.find("]:").is_some();

        if is_quote_line {
            if !in_quote && !result.is_empty() {
                result.push_str("\n\n");
            } else if in_quote {
                result.push('\n');
            }
            result.push_str(trimmed);
            in_quote = true;
            in_table = false;
            in_list = false;
        } else if is_table_row {
            if !in_table && !result.is_empty() {
                result.push_str("\n\n");
            } else if in_table {
                result.push('\n');
            }
            result.push_str(trimmed);
            in_table = true;
            in_list = false;
            in_quote = false;
        } else if is_list_item {
            if !in_list && !result.is_empty() {
                result.push_str("\n\n");
            } else if in_list {
                result.push('\n');
            }
            if let Some(rest) = custom_bullet_rest {
                result.push_str("- ");
                result.push_str(rest.trim_start());
            } else {
                result.push_str(line);
            }
            in_list = true;
            in_table = false;
            in_quote = false;
        } else if is_footnote_def {
            if !result.is_empty() {
                result.push_str("\n\n");
            }
            result.push_str(trimmed);
            in_list = false;
            in_table = false;
            in_quote = false;
        } else {
            // Check if this line looks like a footnote definition formatted as `[^1]. Text` or `[^1]: Text`
            let normalized_line = if trimmed_start.starts_with("[^") {
                if let Some(bracket_end) = trimmed_start.find(']') {
                    let after = &trimmed_start[bracket_end + 1..];
                    if let Some(rest) = after.strip_prefix(". ") {
                        let label = &trimmed_start[..=bracket_end];
                        Some(format!("{label}: {rest}"))
                    } else if let Some(rest) = after.strip_prefix(": ") {
                        let label = &trimmed_start[..=bracket_end];
                        Some(format!("{label}: {rest}"))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            let line_to_push = normalized_line.as_deref().unwrap_or(trimmed);

            if !result.is_empty() {
                result.push_str("\n\n");
            }
            result.push_str(line_to_push);
            in_table = false;
            in_list = false;
            in_quote = false;
        }
    }

    result
}

fn is_footnote_label(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return false;
    }
    // Simple numeric, or standard footnote symbols like * or † or letters like a, b, i, ii
    trimmed
        .chars()
        .all(|c| c.is_alphanumeric() || c == '*' || c == '†' || c == '‡')
}

fn to_superscript_unicode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '0' => '⁰',
            '1' => '¹',
            '2' => '²',
            '3' => '³',
            '4' => '⁴',
            '5' => '⁵',
            '6' => '⁶',
            '7' => '⁷',
            '8' => '⁸',
            '9' => '⁹',
            '+' => '⁺',
            '-' => '⁻',
            '=' => '⁼',
            '(' => '⁽',
            ')' => '⁾',
            'n' => 'ⁿ',
            'i' => 'ⁱ',
            other => other,
        })
        .collect()
}

fn mathml_to_latex(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    trimmed.to_string()
}

fn parse_dimension(val: &str) -> Option<u32> {
    let trimmed = val.trim();
    let num_str = trimmed
        .strip_suffix("px")
        .or_else(|| trimmed.strip_suffix("pt"))
        .unwrap_or(trimmed)
        .trim();
    num_str.parse::<f32>().ok().and_then(|f| {
        if f > 0.0 && f <= 10000.0 {
            Some(f.round() as u32)
        } else {
            None
        }
    })
}

fn extract_style_dimension(style: &str, prop: &str) -> Option<u32> {
    for decl in style.split(';') {
        if let Some((k, v)) = decl.split_once(':')
            && k.trim().eq_ignore_ascii_case(prop)
        {
            return parse_dimension(v);
        }
    }
    None
}

pub fn strip_html_tags(html: &str) -> String {
    convert_html_to_markdown(html)
}
