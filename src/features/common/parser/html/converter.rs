use quick_xml::events::Event;
use quick_xml::reader::Reader;

use super::entities::{decode_html_entities, decode_html_entities_only, normalize_nfc};
use super::extract::{HTML_VOID_TAGS, extract_id_or_name, is_element_hidden};

use std::collections::HashMap;

use crate::features::epub::parser::css::{BlockStyle, VerticalAlign, parse_inline_style};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InlineFormat {
    Delim(&'static str),
    Underline,
    Styled {
        color: Option<[u8; 4]>,
        background: Option<[u8; 4]>,
    },
}

fn target_ends_with_exact_closing(target: &str, fmt: InlineFormat) -> bool {
    match fmt {
        InlineFormat::Delim("*") => target.ends_with('*') && !target.ends_with("**"),
        InlineFormat::Delim("**") => target.ends_with("**") && !target.ends_with("***"),
        InlineFormat::Delim("***") => target.ends_with("***") && !target.ends_with("****"),
        InlineFormat::Delim("~~") => target.ends_with("~~"),
        InlineFormat::Underline => target.ends_with("</u>"),
        InlineFormat::Styled { .. } => target.ends_with("</span>"),
        InlineFormat::Delim(d) => target.ends_with(d),
    }
}

pub fn convert_html_to_markdown(html: &str) -> String {
    let mut converter = HtmlToMarkdownConverter::new();
    converter.convert(html)
}

pub fn convert_html_to_markdown_with_stylesheet(
    html: &str,
    stylesheet_map: &HashMap<String, BlockStyle>,
) -> String {
    let mut converter = HtmlToMarkdownConverter::with_stylesheet(stylesheet_map);
    converter.convert(html)
}

pub struct HtmlToMarkdownConverter<'a> {
    output: String,
    skip_depth: usize,
    hidden_tag_stack: Vec<String>,
    in_pre: bool,
    pre_lang: Option<String>,
    pre_buffer: String,
    in_code: bool,
    pending_code_close: bool,
    in_table: bool,
    table_headers: Vec<String>,
    table_rows: Vec<Vec<String>>,
    current_row: Vec<String>,
    current_cell: String,
    current_cell_colspan: usize,
    table_column_alignments: Vec<Option<String>>,
    in_caption: bool,
    caption_buffer: String,
    table_caption: Option<String>,
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
    span_format_stack: Vec<Vec<InlineFormat>>,
    span_va_stack: Vec<Option<VerticalAlign>>,
    pending_opening_formats: Vec<InlineFormat>,
    div_callout_stack: Vec<Option<&'static str>>,
    in_figure: bool,
    in_figure_caption: bool,
    in_callout_heading: bool,
    in_heading: bool,
    li_just_started: bool,
    p_just_started: bool,
    in_dl: bool,
    in_dt: bool,
    in_dd: bool,
    has_chapter_title: bool,
    stylesheet_map: Option<&'a HashMap<String, BlockStyle>>,
}

#[derive(Clone, Copy)]
struct ListContext {
    ordered: bool,
    index: usize,
}

impl Default for HtmlToMarkdownConverter<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> HtmlToMarkdownConverter<'a> {
    pub fn new() -> Self {
        Self {
            output: String::with_capacity(1024),
            skip_depth: 0,
            hidden_tag_stack: Vec::new(),
            in_pre: false,
            pre_lang: None,
            pre_buffer: String::new(),
            in_code: false,
            pending_code_close: false,
            in_table: false,
            table_headers: Vec::new(),
            table_rows: Vec::new(),
            current_row: Vec::new(),
            current_cell: String::new(),
            current_cell_colspan: 1,
            table_column_alignments: Vec::new(),
            in_caption: false,
            caption_buffer: String::new(),
            table_caption: None,
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
            span_va_stack: Vec::new(),
            pending_opening_formats: Vec::new(),
            div_callout_stack: Vec::new(),
            in_figure: false,
            in_figure_caption: false,
            in_callout_heading: false,
            in_heading: false,
            li_just_started: false,
            p_just_started: false,
            in_dl: false,
            in_dt: false,
            in_dd: false,
            has_chapter_title: false,
            stylesheet_map: None,
        }
    }

    pub fn with_stylesheet(stylesheet_map: &'a HashMap<String, BlockStyle>) -> Self {
        Self {
            stylesheet_map: Some(stylesheet_map),
            ..Self::new()
        }
    }

    pub fn convert(&mut self, html: &str) -> String {
        let coalesced_storage;
        let html_input = if html.contains("chapterNumber") || html.contains("chapternumber") {
            coalesced_storage = coalesce_chapter_number_and_title(html);
            &coalesced_storage
        } else {
            html
        };

        let has_chapter_title =
            html_input.contains("chapterTitle") || html_input.contains("chaptertitle");
        self.has_chapter_title = has_chapter_title;

        let mut reader = Reader::from_str(html_input);
        reader.config_mut().check_end_names = false;
        reader.config_mut().trim_text(false);

        let mut buf = Vec::with_capacity(256);

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    if name.as_ref().eq_ignore_ascii_case("br") {
                        self.handle_empty_tag(name.as_ref(), e);
                    } else {
                        self.handle_start_tag(name.as_ref(), e);
                    }
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
                    self.flush_pending_code_close();
                    self.flush_pending_formats();
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
        clean_markdown_output(&decoded)
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

        let is_void = HTML_VOID_TAGS.contains(&name);

        if !self.hidden_tag_stack.is_empty() {
            if !is_void {
                self.hidden_tag_stack.push(name.to_string());
            }
            return;
        }

        if is_element_hidden(e, self.stylesheet_map) {
            if !is_void {
                self.hidden_tag_stack.push(name.to_string());
            }
            return;
        }

        if name != "code" {
            self.flush_pending_code_close();
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
                self.table_column_alignments.clear();
                self.table_caption = None;
            }
            "caption" if self.in_table => {
                self.in_caption = true;
                self.caption_buffer.clear();
            }
            "tr" if self.in_table => {
                self.current_row.clear();
            }
            "th" if self.in_table => {
                self.in_cell = true;
                self.current_cell.clear();
                self.current_cell_colspan = extract_colspan(e);
                let col_idx = self.table_headers.len();
                if let Some(align) = extract_cell_alignment(e) {
                    if col_idx >= self.table_column_alignments.len() {
                        self.table_column_alignments.resize(col_idx + 1, None);
                    }
                    if self.table_column_alignments[col_idx].is_none() {
                        self.table_column_alignments[col_idx] = Some(align);
                    }
                }
            }
            "td" if self.in_table => {
                self.in_cell = true;
                self.current_cell.clear();
                self.current_cell_colspan = extract_colspan(e);
                let col_idx = self.current_row.len();
                if let Some(align) = extract_cell_alignment(e) {
                    if col_idx >= self.table_column_alignments.len() {
                        self.table_column_alignments.resize(col_idx + 1, None);
                    }
                    if self.table_column_alignments[col_idx].is_none() {
                        self.table_column_alignments[col_idx] = Some(align);
                    }
                }
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
                } else if self.pending_code_close {
                    // Previous <code> just ended and another <code> starts immediately;
                    // merge them into the same inline code span without emitting closing/opening backticks!
                    self.pending_code_close = false;
                    self.in_code = true;
                } else {
                    self.flush_pending_code_close();
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
                self.in_heading = true;
                let hashes = if self.has_chapter_title && has_class_or_attr(e, "class", "heading-1")
                {
                    "##"
                } else if self.has_chapter_title && has_class_or_attr(e, "class", "heading-2") {
                    "###"
                } else {
                    match name {
                        "h1" => "#",
                        "h2" => "##",
                        "h3" => "###",
                        "h4" => "####",
                        "h5" => "#####",
                        "h6" => "######",
                        _ => "#",
                    }
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
            "dl" => {
                self.in_dl = true;
                if let Some(id) = extract_id_or_name(e) {
                    self.append_str(&format!("\n\n<a id=\"{id}\"></a>\n\n"));
                }
                self.ensure_newline();
            }
            "dt" => {
                self.in_dt = true;
                if let Some(id) = extract_id_or_name(e) {
                    self.append_str(&format!("\n\n<a id=\"{id}\"></a>\n\n"));
                }
                self.ensure_newline();
                self.append_str("- **");
            }
            "dd" => {
                self.in_dd = true;
                if let Some(id) = extract_id_or_name(e) {
                    self.append_str(&format!("<a id=\"{id}\"></a> "));
                }
                self.li_just_started = true;
            }
            "p" | "section" | "article" | "header" | "footer" | "figcaption" | "main" | "body" => {
                if let Some(id) = extract_id_or_name(e) {
                    self.append_str(&format!("\n\n<a id=\"{id}\"></a>\n\n"));
                }
                if self.in_dd || !self.list_stack.is_empty() {
                    self.li_just_started = true;
                } else {
                    self.ensure_newline();
                    if self.list_stack.is_empty() {
                        self.p_just_started = true;
                    }
                }
            }
            "a" => {
                if self.in_pre {
                    return;
                }
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
                    if self.in_dt {
                        self.append_str(&format!("<a id=\"{id}\"></a>"));
                    } else if self.output.ends_with('\n') || self.output.is_empty() {
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
                    if self.in_dt {
                        self.append_str(&format!("<a id=\"{id}\"></a>"));
                    } else if self.output.ends_with('\n') || self.output.is_empty() {
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
                    let mut is_underline = false;
                    let mut is_strikethrough = false;
                    let mut vertical_align = None;
                    let mut color = None;
                    let mut background_color = None;

                    if let Some(id) = extract_id_or_name(e)
                        && let Some(map) = self.stylesheet_map
                    {
                        let id_key = format!("#{id}");
                        let style_opt = map.get(&id_key).or_else(|| map.get(&id));
                        if let Some(style) = style_opt {
                            if style.is_bold {
                                is_bold = true;
                            }
                            if style.is_italic {
                                is_italic = true;
                            }
                            if style.is_underline {
                                is_underline = true;
                            }
                            if style.is_strikethrough {
                                is_strikethrough = true;
                            }
                            if style.vertical_align.is_some() {
                                vertical_align = style.vertical_align;
                            }
                            if self.stylesheet_map.is_none() {
                                if style.color.is_some() {
                                    color = style.color;
                                }
                                if style.background_color.is_some() {
                                    background_color = style.background_color;
                                }
                            }
                        }
                    }

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
                                if let Some(map) = self.stylesheet_map
                                    && let Some(style) = map.get(class_token)
                                {
                                    if style.is_bold {
                                        is_bold = true;
                                    }
                                    if style.is_italic {
                                        is_italic = true;
                                    }
                                    if style.is_underline {
                                        is_underline = true;
                                    }
                                    if style.is_strikethrough {
                                        is_strikethrough = true;
                                    }
                                    if style.vertical_align.is_some() {
                                        vertical_align = style.vertical_align;
                                    }
                                    if style.color.is_some() {
                                        color = style.color;
                                    }
                                    if style.background_color.is_some() {
                                        background_color = style.background_color;
                                    }
                                }
                            }
                        } else if key.eq_ignore_ascii_case("style") {
                            let inline_style = parse_inline_style(&attr.value);
                            if inline_style.vertical_align.is_some() {
                                vertical_align = inline_style.vertical_align;
                            }
                            if self.stylesheet_map.is_none() {
                                if inline_style.color.is_some() {
                                    color = inline_style.color;
                                }
                                if inline_style.background_color.is_some() {
                                    background_color = inline_style.background_color;
                                }
                            }
                            for decl in val.split(';') {
                                if let Some((k, v)) = decl.split_once(':') {
                                    let k = k.trim();
                                    let v = v.trim();
                                    if k == "font-style" && (v == "italic" || v == "oblique") {
                                        is_italic = true;
                                    } else if k == "font-weight"
                                        && (v == "bold"
                                            || v == "bolder"
                                            || v == "700"
                                            || v == "800"
                                            || v == "900")
                                    {
                                        is_bold = true;
                                    } else if k == "text-decoration" || k == "text-decoration-line"
                                    {
                                        if v.contains("underline") {
                                            is_underline = true;
                                        }
                                        if v.contains("line-through") {
                                            is_strikethrough = true;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    match vertical_align {
                        Some(VerticalAlign::Super) => {
                            self.sup_depth += 1;
                            if self.sup_depth == 1 {
                                self.sup_buffer.clear();
                            }
                        }
                        Some(VerticalAlign::Sub) => {
                            self.sub_depth += 1;
                        }
                        None => {}
                    }
                    self.span_va_stack.push(vertical_align);

                    let delimiter = match (is_bold, is_italic) {
                        (true, true) => Some("***"),
                        (true, false) => Some("**"),
                        (false, true) => Some("*"),
                        (false, false) => None,
                    };

                    let mut formats = Vec::new();
                    if is_strikethrough {
                        formats.push(InlineFormat::Delim("~~"));
                        self.pending_opening_formats.push(InlineFormat::Delim("~~"));
                    }
                    if let Some(delim) = delimiter {
                        formats.push(InlineFormat::Delim(delim));
                        self.pending_opening_formats
                            .push(InlineFormat::Delim(delim));
                    }
                    if is_underline {
                        formats.push(InlineFormat::Underline);
                        self.pending_opening_formats.push(InlineFormat::Underline);
                    }
                    if color.is_some() || background_color.is_some() {
                        let fmt = InlineFormat::Styled {
                            color,
                            background: background_color,
                        };
                        formats.push(fmt);
                        self.pending_opening_formats.push(fmt);
                    }
                    self.span_format_stack.push(formats);
                }
            }
            "mark" => {
                let fmt = InlineFormat::Styled {
                    color: None,
                    background: Some([255, 255, 0, 255]),
                };
                self.pending_opening_formats.push(fmt);
                self.span_format_stack.push(vec![fmt]);
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
                self.pending_opening_formats.push(InlineFormat::Delim("**"));
            }
            "i" | "em" => {
                if let Some(id) = extract_id_or_name(e) {
                    self.append_str(&format!("<a id=\"{id}\"></a> "));
                }
                self.pending_opening_formats.push(InlineFormat::Delim("*"));
            }
            "u" | "ins" => {
                if let Some(id) = extract_id_or_name(e) {
                    self.append_str(&format!("<a id=\"{id}\"></a> "));
                }
                self.pending_opening_formats.push(InlineFormat::Underline);
            }
            "del" | "s" | "strike" => {
                self.pending_opening_formats.push(InlineFormat::Delim("~~"));
            }
            "sub" => {
                self.sub_depth += 1;
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
                    if self.in_dt {
                        self.append_str(&format!("<a id=\"{id}\"></a>"));
                    } else if self.output.ends_with('\n') || self.output.is_empty() {
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

        if !self.hidden_tag_stack.is_empty() {
            if let Some(pos) = self.hidden_tag_stack.iter().rposition(|t| t == name) {
                self.hidden_tag_stack.truncate(pos);
            } else {
                self.hidden_tag_stack.pop();
            }
            return;
        }

        if name != "code" {
            self.flush_pending_code_close();
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
                    self.pending_code_close = true;
                }
            }
            "a" => {
                if !self.in_pre
                    && let Some(href) = self.link_stack.pop()
                {
                    self.append_str(&format!("]({href})"));
                }
            }
            "caption" if self.in_table => {
                self.in_caption = false;
                let cap = clean_table_cell(&self.caption_buffer);
                if !cap.is_empty() {
                    self.table_caption = Some(cap);
                }
                self.caption_buffer.clear();
            }
            "th" if self.in_table => {
                if self.in_ruby {
                    self.in_ruby = false;
                    self.flush_ruby_pair();
                }
                self.in_cell = false;
                let cell_txt = clean_table_cell(&self.current_cell);
                let span = self.current_cell_colspan.max(1);
                self.table_headers.push(cell_txt);
                for _ in 1..span {
                    self.table_headers.push(String::new());
                }
                self.current_cell.clear();
            }
            "td" if self.in_table => {
                if self.in_ruby {
                    self.in_ruby = false;
                    self.flush_ruby_pair();
                }
                self.in_cell = false;
                let cell_txt = clean_table_cell(&self.current_cell);
                let span = self.current_cell_colspan.max(1);
                self.current_row.push(cell_txt);
                for _ in 1..span {
                    self.current_row.push(String::new());
                }
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
            "b" | "strong" => self.append_closing_format(InlineFormat::Delim("**")),
            "i" | "em" => self.append_closing_format(InlineFormat::Delim("*")),
            "u" | "ins" => self.append_closing_format(InlineFormat::Underline),
            "del" | "s" | "strike" => self.append_closing_format(InlineFormat::Delim("~~")),
            "span" | "mark" => {
                if let Some(formats) = self.span_format_stack.pop() {
                    for fmt in formats.into_iter().rev() {
                        self.append_closing_format(fmt);
                    }
                }
                if let Some(va) = self.span_va_stack.pop() {
                    match va {
                        Some(VerticalAlign::Super) => {
                            if self.sup_depth > 0 {
                                self.sup_depth -= 1;
                                if self.sup_depth == 0 {
                                    let content = self.sup_buffer.trim();
                                    if is_footnote_label(content) {
                                        self.append_str(&format!("[^{content}]"));
                                    } else if !content.is_empty() {
                                        self.append_str(&to_superscript_unicode(content));
                                    }
                                    self.sup_buffer.clear();
                                }
                            }
                        }
                        Some(VerticalAlign::Sub) if self.sub_depth > 0 => {
                            self.sub_depth -= 1;
                        }
                        _ => {}
                    }
                }
            }
            "sub" => {
                if self.sub_depth > 0 {
                    self.sub_depth -= 1;
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
                self.in_heading = false;
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
            "dl" => {
                self.in_dl = false;
                self.ensure_newline();
            }
            "dt" => {
                self.in_dt = false;
                self.append_str("**: ");
            }
            "dd" => {
                self.in_dd = false;
                self.ensure_newline();
            }
            "p" | "section" | "article" => {
                self.flush_pending_formats();
                self.p_just_started = false;
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
                } else if !self.in_dd && self.list_stack.is_empty() {
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

        if self.skip_depth > 0
            || !self.hidden_tag_stack.is_empty()
            || is_element_hidden(e, self.stylesheet_map)
        {
            return;
        }

        self.flush_pending_code_close();

        if !self.in_pre
            && let Some(id) = extract_id_or_name(e)
        {
            if self.output.ends_with('\n') || self.output.is_empty() {
                self.append_str(&format!("<a id=\"{id}\"></a>\n\n"));
            } else {
                self.append_str(&format!("<a id=\"{id}\"></a> "));
            }
        }

        match name {
            "br" => {
                if self.in_heading || self.in_cell {
                    self.append_str("<br>");
                } else {
                    self.append_str("\n");
                }
            }
            "hr" => self.append_str("\n\n---\n\n"),
            "img" | "image" => self.handle_img_tag(e),
            _ => {}
        }
    }

    fn handle_img_tag(&mut self, e: &quick_xml::events::BytesStart) {
        if !self.hidden_tag_stack.is_empty() || is_element_hidden(e, self.stylesheet_map) {
            return;
        }

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
            // Check if this is an inline callout badge (e.g. inside a <dt> or <pre> callout link)
            let is_callout_context = self.in_pre
                || self.in_dt
                || !self.link_stack.is_empty()
                || (width.is_some_and(|w| w <= 24) && height.is_some_and(|h| h <= 24));
            if is_callout_context && let Some(symbol) = to_callout_symbol(&alt, &src) {
                if self.in_pre {
                    self.pre_buffer.push(' ');
                    self.pre_buffer.push_str(&symbol);
                } else {
                    self.append_str(&symbol);
                }
                return;
            }

            if self.in_pre {
                return;
            }

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
        if self.skip_depth > 0 || !self.hidden_tag_stack.is_empty() || self.in_callout_heading {
            return;
        }

        let actual_text = if self.li_just_started {
            if text.trim().is_empty() {
                return;
            }
            self.li_just_started = false;
            text.trim_start()
        } else if self.output.ends_with("<br>")
            || self.output.ends_with("<br/>")
            || self.output.ends_with("<br />")
        {
            text.trim_start_matches(|c: char| c.is_whitespace() || c == '\u{a0}')
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
                if has_leading && !self.output.ends_with(' ') && !self.output.ends_with('\n') {
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
                if !self.output.ends_with(' ') && !self.output.ends_with('\n') {
                    " "
                } else {
                    ""
                }
            } else {
                ""
            }
        } else {
            actual_text
        };

        if text_to_append.is_empty() {
            return;
        }

        self.flush_pending_code_close();

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

        let subscript_storage: String;
        let text_to_append = if self.sub_depth > 0 {
            subscript_storage = to_subscript_unicode(text_to_append);
            subscript_storage.as_str()
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
            self.append_formatted_text(text_to_append);
            return;
        }

        if self.sup_depth > 0 {
            self.sup_buffer.push_str(text_to_append);
            return;
        }

        if self.p_just_started {
            if text_to_append.trim().is_empty() {
                self.append_str(text_to_append);
                return;
            }
            self.p_just_started = false;
            let trimmed_start = text_to_append.trim_start();
            if trimmed_start.starts_with("- ")
                || trimmed_start.starts_with("+ ")
                || trimmed_start.starts_with("* ")
            {
                let leading_ws = &text_to_append[..text_to_append.len() - trimmed_start.len()];
                let escaped = format!("{leading_ws}\\{trimmed_start}");
                self.append_formatted_text(&escaped);
                return;
            }
        }

        self.append_formatted_text(text_to_append);
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

    fn current_target_mut(&mut self) -> &mut String {
        if self.in_cell {
            &mut self.current_cell
        } else if self.in_caption {
            &mut self.caption_buffer
        } else if let Some(current_quote) = self.quote_stack.last_mut() {
            current_quote
        } else {
            &mut self.output
        }
    }

    fn append_str(&mut self, s: &str) {
        self.current_target_mut().push_str(s);
    }

    fn flush_pending_code_close(&mut self) {
        if self.pending_code_close {
            self.pending_code_close = false;
            let target = self.current_target_mut();
            let trimmed_len = target
                .trim_end_matches(|c: char| c.is_whitespace() || c == '\u{a0}')
                .trim_end_matches("&nbsp;")
                .len();
            if trimmed_len < target.len() {
                let trailing = target[trimmed_len..].to_string();
                target.truncate(trimmed_len);
                target.push('`');
                target.push_str(&trailing);
            } else {
                target.push('`');
            }
        }
    }

    fn append_closing_format(&mut self, fmt: InlineFormat) {
        if let Some(pos) = self.pending_opening_formats.iter().rposition(|&f| f == fmt) {
            self.pending_opening_formats.remove(pos);
            return;
        }

        let closing_str = match fmt {
            InlineFormat::Delim(d) => d,
            InlineFormat::Underline => "</u>",
            InlineFormat::Styled { .. } => "</span>",
        };

        let target = self.current_target_mut();

        let trimmed_len = target
            .trim_end_matches(|c: char| c.is_whitespace() || c == '\u{a0}')
            .trim_end_matches("&nbsp;")
            .len();
        if trimmed_len < target.len() {
            let trailing = target[trimmed_len..].to_string();
            target.truncate(trimmed_len);
            target.push_str(closing_str);
            target.push_str(&trailing);
        } else {
            target.push_str(closing_str);
        }
    }

    fn flush_pending_formats(&mut self) {
        let pending = std::mem::take(&mut self.pending_opening_formats);
        let target = self.current_target_mut();
        for fmt in pending {
            if target_ends_with_exact_closing(target, fmt) {
                let closing_len = match fmt {
                    InlineFormat::Delim(d) => d.len(),
                    InlineFormat::Underline => "</u>".len(),
                    InlineFormat::Styled { .. } => "</span>".len(),
                };
                target.truncate(target.len() - closing_len);
            } else {
                match fmt {
                    InlineFormat::Delim(d) => target.push_str(d),
                    InlineFormat::Underline => target.push_str("<u>"),
                    InlineFormat::Styled { color, background } => {
                        let mut style = String::new();
                        if let Some([r, g, b, a]) = color {
                            if a == 255 {
                                style.push_str(&format!("color:#{r:02x}{g:02x}{b:02x};"));
                            } else {
                                style.push_str(&format!(
                                    "color:rgba({r},{g},{b},{:.2});",
                                    a as f32 / 255.0
                                ));
                            }
                        }
                        if let Some([r, g, b, a]) = background {
                            if a == 255 {
                                style
                                    .push_str(&format!("background-color:#{r:02x}{g:02x}{b:02x};"));
                            } else {
                                style.push_str(&format!(
                                    "background-color:rgba({r},{g},{b},{:.2});",
                                    a as f32 / 255.0
                                ));
                            }
                        }
                        target.push_str(&format!("<span style=\"{style}\">"));
                    }
                }
            }
        }
    }

    fn append_formatted_text(&mut self, text: &str) {
        if self.pending_opening_formats.is_empty() {
            self.append_str(text);
            return;
        }

        let mut non_ws_idx = 0;
        let len = text.len();

        while non_ws_idx < len {
            let rem = &text[non_ws_idx..];
            if rem.starts_with("&nbsp;") {
                non_ws_idx += "&nbsp;".len();
            } else if let Some(ch) = rem.chars().next() {
                if ch.is_whitespace() || ch == '\u{a0}' {
                    non_ws_idx += ch.len_utf8();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if non_ws_idx > 0 {
            self.append_str(&text[..non_ws_idx]);
        }

        if non_ws_idx < text.len() {
            self.flush_pending_formats();
            self.append_str(&text[non_ws_idx..]);
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
        if let Some(cap) = self.table_caption.take() {
            self.output.push_str(&format!("*{cap}*\n\n"));
        } else {
            self.output.push('\n');
        }

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
        for i in 0..col_count {
            let align = self
                .table_column_alignments
                .get(i)
                .and_then(|a| a.as_deref());
            match align {
                Some("center") => self.output.push_str(" :---: |"),
                Some("right") => self.output.push_str(" ---: |"),
                _ => self.output.push_str(" --- |"),
            }
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
        self.table_column_alignments.clear();
    }
}

fn extract_colspan(e: &quick_xml::events::BytesStart) -> usize {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref().eq_ignore_ascii_case("colspan")
            && let Ok(n) = attr.value.trim().parse::<usize>()
        {
            return n.max(1);
        }
    }
    1
}

fn extract_cell_alignment(e: &quick_xml::events::BytesStart) -> Option<String> {
    for attr in e.attributes().flatten() {
        let key = attr.key.as_ref();
        if key.eq_ignore_ascii_case("align") {
            let v = attr.value.trim().to_ascii_lowercase();
            if v == "center" || v == "right" || v == "left" {
                return Some(v);
            }
        } else if key.eq_ignore_ascii_case("style") {
            let lower = attr.value.to_ascii_lowercase();
            if lower.contains("text-align: center") || lower.contains("text-align:center") {
                return Some("center".to_string());
            } else if lower.contains("text-align: right") || lower.contains("text-align:right") {
                return Some("right".to_string());
            } else if lower.contains("text-align: left") || lower.contains("text-align:left") {
                return Some("left".to_string());
            }
        }
    }
    None
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

        let cleaned_line = if line.contains("****") {
            std::borrow::Cow::Owned(line.replace("****", ""))
        } else {
            std::borrow::Cow::Borrowed(line)
        };
        let line = cleaned_line.as_ref();

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

fn to_callout_symbol(alt: &str, src: &str) -> Option<String> {
    let trimmed_alt = alt.trim();
    if !trimmed_alt.is_empty()
        && let Ok(n) = trimmed_alt.parse::<u32>()
    {
        return Some(format_circled_number(n));
    }

    let file_stem = src
        .split('/')
        .next_back()
        .unwrap_or(src)
        .split('.')
        .next()
        .unwrap_or("");
    let num_part = file_stem
        .strip_prefix("callout_")
        .or_else(|| file_stem.strip_prefix("callout-"))
        .or_else(|| file_stem.strip_prefix("co_"))
        .or_else(|| file_stem.strip_prefix("co-"))
        .unwrap_or(file_stem);

    if let Ok(n) = num_part.parse::<u32>() {
        return Some(format_circled_number(n));
    }

    if !trimmed_alt.is_empty() {
        Some(format!("({trimmed_alt})"))
    } else {
        None
    }
}

fn format_circled_number(n: u32) -> String {
    match n {
        1..=20 => {
            if let Some(ch) = char::from_u32(0x2460 + n - 1) {
                ch.to_string()
            } else {
                format!("({n})")
            }
        }
        21..=35 => {
            if let Some(ch) = char::from_u32(0x3251 + n - 21) {
                ch.to_string()
            } else {
                format!("({n})")
            }
        }
        36..=50 => {
            if let Some(ch) = char::from_u32(0x32B1 + n - 36) {
                ch.to_string()
            } else {
                format!("({n})")
            }
        }
        _ => format!("({n})"),
    }
}

pub fn to_subscript_unicode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '0' => '₀',
            '1' => '₁',
            '2' => '₂',
            '3' => '₃',
            '4' => '₄',
            '5' => '₅',
            '6' => '₆',
            '7' => '₇',
            '8' => '₈',
            '9' => '₉',
            '+' => '₊',
            '-' => '₋',
            '=' => '₌',
            '(' => '₍',
            ')' => '₎',
            'a' => 'ₐ',
            'e' => 'ₑ',
            'h' => 'ₕ',
            'i' => 'ᵢ',
            'j' => 'ⱼ',
            'k' => 'ₖ',
            'l' => 'ₗ',
            'm' => 'ₘ',
            'n' => 'ₙ',
            'o' => 'ₒ',
            'p' => 'ₚ',
            'r' => 'ᵣ',
            's' => 'ₛ',
            't' => 'ₜ',
            'u' => 'ᵤ',
            'v' => 'ᵥ',
            'x' => 'ₓ',
            other => other,
        })
        .collect()
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

pub fn coalesce_chapter_number_and_title(html: &str) -> String {
    let lower = html.to_ascii_lowercase();
    let Some(tag_pos) = lower.find("chapternumber") else {
        return html.to_string();
    };

    let Some(h1_open) = lower[..tag_pos].rfind("<h") else {
        return html.to_string();
    };
    let Some(rel_gt) = lower[h1_open..].find('>') else {
        return html.to_string();
    };
    let h1_open_gt = h1_open + rel_gt;
    if h1_open_gt < tag_pos {
        return html.to_string();
    }

    if h1_open + 3 > lower.len() {
        return html.to_string();
    }
    let tag1_name = &lower[h1_open + 1..h1_open + 3];
    let h1_close_str = format!("</{tag1_name}>");
    let Some(rel_close) = lower[h1_open_gt..].find(&h1_close_str) else {
        return html.to_string();
    };
    let h1_close = h1_open_gt + rel_close;

    let num_tag = &html[h1_open..=h1_open_gt];
    let num_text = html[h1_open_gt + 1..h1_close].trim();

    let after_first = h1_close + h1_close_str.len();
    let mut curr = after_first;
    let bytes = html.as_bytes();
    while curr < bytes.len() && bytes[curr].is_ascii_whitespace() {
        curr += 1;
    }

    if curr + 2 > lower.len() || !lower[curr..].starts_with("<h") {
        return html.to_string();
    }

    let h2_open = curr;
    let Some(rel_gt2) = lower[h2_open..].find('>') else {
        return html.to_string();
    };
    let h2_open_gt = h2_open + rel_gt2;
    let h2_tag_str = &lower[h2_open..=h2_open_gt];
    if !h2_tag_str.contains("chaptertitle") {
        return html.to_string();
    }

    if h2_open + 3 > lower.len() {
        return html.to_string();
    }
    let tag2_name = &lower[h2_open + 1..h2_open + 3];
    let h2_close_str = format!("</{tag2_name}>");
    let Some(rel_close2) = lower[h2_open_gt..].find(&h2_close_str) else {
        return html.to_string();
    };
    let h2_close = h2_open_gt + rel_close2;

    let title_tag = &html[h2_open..=h2_open_gt];
    let title_text = html[h2_open_gt + 1..h2_close].trim();

    let id1 = extract_attr_id(num_tag);
    let id2 = extract_attr_id(title_tag);

    let extra_anchor = if let (Some(a), Some(b)) = (&id1, &id2)
        && a != b
    {
        format!("<a id=\"{b}\"></a>")
    } else {
        String::new()
    };

    let replacement = format!(
        "{extra_anchor}<h1 class=\"chapterTitle\" id=\"{}\">Chapter {num_text}: {title_text}</h1>",
        id1.as_deref().or(id2.as_deref()).unwrap_or("")
    );

    let mut result = String::with_capacity(html.len() + replacement.len());
    result.push_str(&html[..h1_open]);
    result.push_str(&replacement);
    result.push_str(&html[h2_close + h2_close_str.len()..]);
    result
}

fn extract_attr_id(tag: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let idx = lower.find("id=")?;
    let after_id = &tag[idx + 3..];
    let quote = after_id.chars().next()?;
    if quote == '"' || quote == '\'' {
        let remainder = &after_id[quote.len_utf8()..];
        let end_idx = remainder.find(quote)?;
        Some(remainder[..end_idx].to_string())
    } else {
        None
    }
}
