use std::collections::HashMap;

/// Text alignment extracted from CSS or HTML attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAlign {
    Super,
    Sub,
}

impl TextAlign {
    pub fn to_horizontal_align(self) -> iced::alignment::Horizontal {
        match self {
            TextAlign::Left | TextAlign::Justify => iced::alignment::Horizontal::Left,
            TextAlign::Center => iced::alignment::Horizontal::Center,
            TextAlign::Right => iced::alignment::Horizontal::Right,
        }
    }
}

/// Style properties extracted for blocks (paragraphs, headings).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BlockStyle {
    pub align: TextAlign,
    pub text_indent: f32,
    pub is_bold: bool,
    pub is_italic: bool,
    pub is_uppercase: bool,
    pub is_underline: bool,
    pub is_strikethrough: bool,
    pub is_hidden: bool,
    pub list_style_type_none: bool,
    pub font_scale: Option<f32>,
    pub line_height: Option<f32>,
    pub font_family: Option<String>,
    pub vertical_align: Option<VerticalAlign>,
    pub margin_top: Option<f32>,
    pub margin_bottom: Option<f32>,
    pub margin_left: Option<f32>,
    pub margin_right: Option<f32>,
    pub padding_top: Option<f32>,
    pub padding_bottom: Option<f32>,
    pub padding_left: Option<f32>,
    pub padding_right: Option<f32>,
    pub color: Option<[u8; 4]>,
    pub background_color: Option<[u8; 4]>,
}

impl BlockStyle {
    pub fn effective_margin_left(&self) -> f32 {
        (self.margin_left.unwrap_or(0.0) + self.padding_left.unwrap_or(0.0)).clamp(0.0, 250.0)
    }

    pub fn effective_margin_right(&self) -> f32 {
        (self.margin_right.unwrap_or(0.0) + self.padding_right.unwrap_or(0.0)).clamp(0.0, 250.0)
    }

    pub fn effective_margin_top(&self) -> f32 {
        (self.margin_top.unwrap_or(0.0) + self.padding_top.unwrap_or(0.0)).clamp(0.0, 80.0)
    }

    pub fn effective_margin_bottom(&self) -> f32 {
        (self.margin_bottom.unwrap_or(0.0) + self.padding_bottom.unwrap_or(0.0)).clamp(0.0, 80.0)
    }
}

/// Parses CSS box shorthand (1 to 4 values for top, right, bottom, left).
pub fn parse_box_sides(val: &str) -> (Option<f32>, Option<f32>, Option<f32>, Option<f32>) {
    let parts: Vec<&str> = val
        .split_whitespace()
        .map(|s| s.trim_matches(','))
        .filter(|s| !s.is_empty() && !s.starts_with('!'))
        .collect();

    let parse_len = |s: &str| -> Option<f32> {
        let s = s.trim();
        if s.eq_ignore_ascii_case("auto") {
            None
        } else {
            parse_css_length(s)
        }
    };

    match parts.as_slice() {
        [all] => {
            let v = parse_len(all);
            (v, v, v, v)
        }
        [tb, lr] => {
            let v_tb = parse_len(tb);
            let v_lr = parse_len(lr);
            (v_tb, v_lr, v_tb, v_lr)
        }
        [top, lr, bottom] => {
            let v_top = parse_len(top);
            let v_lr = parse_len(lr);
            let v_bottom = parse_len(bottom);
            (v_top, v_lr, v_bottom, v_lr)
        }
        [top, right, bottom, left, ..] => (
            parse_len(top),
            parse_len(right),
            parse_len(bottom),
            parse_len(left),
        ),
        [] => (None, None, None, None),
    }
}

/// Applies a single CSS property declaration to `BlockStyle`. Returns `true` if recognized.
fn apply_declaration(prop: &str, val: &str, style: &mut BlockStyle) -> bool {
    let prop = prop.trim().to_ascii_lowercase();
    let val = val.trim();
    match prop.as_str() {
        "text-align" => match val.to_ascii_lowercase().as_str() {
            "center" => {
                style.align = TextAlign::Center;
                true
            }
            "right" => {
                style.align = TextAlign::Right;
                true
            }
            "justify" => {
                style.align = TextAlign::Justify;
                true
            }
            "left" => {
                style.align = TextAlign::Left;
                true
            }
            _ => false,
        },
        "text-indent" => {
            if let Some(indent) = parse_css_length(val) {
                style.text_indent = indent;
                true
            } else {
                false
            }
        }
        "font-weight"
            if val.eq_ignore_ascii_case("bold")
                || val.eq_ignore_ascii_case("bolder")
                || val == "700"
                || val == "800"
                || val == "900" =>
        {
            style.is_bold = true;
            true
        }
        "font-style"
            if val.eq_ignore_ascii_case("italic") || val.eq_ignore_ascii_case("oblique") =>
        {
            style.is_italic = true;
            true
        }
        "text-transform" if val.eq_ignore_ascii_case("uppercase") => {
            style.is_uppercase = true;
            true
        }
        "display" => {
            if val.eq_ignore_ascii_case("none") {
                style.is_hidden = true;
                true
            } else {
                false
            }
        }
        "list-style" | "list-style-type" => {
            if val.to_ascii_lowercase().contains("none") {
                style.list_style_type_none = true;
                true
            } else {
                false
            }
        }
        "text-decoration" | "text-decoration-line" => {
            let val_lower = val.to_ascii_lowercase();
            let mut matched = false;
            if val_lower.contains("underline") {
                style.is_underline = true;
                matched = true;
            }
            if val_lower.contains("line-through") {
                style.is_strikethrough = true;
                matched = true;
            }
            matched
        }
        "font-size" => {
            if let Some(scale) = parse_font_size_scale(val) {
                style.font_scale = Some(scale);
                true
            } else {
                false
            }
        }
        "line-height" => {
            if let Some(lh) = parse_line_height(val) {
                style.line_height = Some(lh);
                true
            } else {
                false
            }
        }
        "font-family" => {
            if let Some(ff) = parse_font_family(val) {
                style.font_family = Some(ff);
                true
            } else {
                false
            }
        }
        "vertical-align" => match val.trim().to_ascii_lowercase().as_str() {
            "super" => {
                style.vertical_align = Some(VerticalAlign::Super);
                true
            }
            "sub" => {
                style.vertical_align = Some(VerticalAlign::Sub);
                true
            }
            _ => false,
        },
        "margin" => {
            let (t, r, b, l) = parse_box_sides(val);
            if t.is_some() || r.is_some() || b.is_some() || l.is_some() || val.contains("auto") {
                style.margin_top = t;
                style.margin_right = r;
                style.margin_bottom = b;
                style.margin_left = l;
                true
            } else {
                false
            }
        }
        "margin-top" => {
            if let Some(v) = parse_css_length(val) {
                style.margin_top = Some(v);
                true
            } else {
                false
            }
        }
        "margin-bottom" => {
            if let Some(v) = parse_css_length(val) {
                style.margin_bottom = Some(v);
                true
            } else {
                false
            }
        }
        "margin-left" => {
            if let Some(v) = parse_css_length(val) {
                style.margin_left = Some(v);
                true
            } else {
                false
            }
        }
        "margin-right" => {
            if let Some(v) = parse_css_length(val) {
                style.margin_right = Some(v);
                true
            } else {
                false
            }
        }
        "padding" => {
            let (t, r, b, l) = parse_box_sides(val);
            if t.is_some() || r.is_some() || b.is_some() || l.is_some() {
                style.padding_top = t;
                style.padding_right = r;
                style.padding_bottom = b;
                style.padding_left = l;
                true
            } else {
                false
            }
        }
        "padding-top" => {
            if let Some(v) = parse_css_length(val) {
                style.padding_top = Some(v);
                true
            } else {
                false
            }
        }
        "padding-bottom" => {
            if let Some(v) = parse_css_length(val) {
                style.padding_bottom = Some(v);
                true
            } else {
                false
            }
        }
        "padding-left" => {
            if let Some(v) = parse_css_length(val) {
                style.padding_left = Some(v);
                true
            } else {
                false
            }
        }
        "padding-right" => {
            if let Some(v) = parse_css_length(val) {
                style.padding_right = Some(v);
                true
            } else {
                false
            }
        }
        "color" => {
            if let Some(c) = parse_css_color(val) {
                style.color = Some(c);
                true
            } else {
                false
            }
        }
        "background-color" | "background" => {
            if let Some(c) = parse_css_color(val) {
                style.background_color = Some(c);
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Copies specified properties from `source` onto `target`, obeying CSS cascade.
pub fn merge_block_style(target: &mut BlockStyle, source: &BlockStyle) {
    if source.align != TextAlign::Left {
        target.align = source.align;
    }
    if source.text_indent > 0.0 {
        target.text_indent = source.text_indent;
    }
    if source.is_bold {
        target.is_bold = true;
    }
    if source.is_italic {
        target.is_italic = true;
    }
    if source.is_uppercase {
        target.is_uppercase = true;
    }
    if source.is_underline {
        target.is_underline = true;
    }
    if source.is_strikethrough {
        target.is_strikethrough = true;
    }
    if source.is_hidden {
        target.is_hidden = true;
    }
    if source.list_style_type_none {
        target.list_style_type_none = true;
    }
    if let Some(scale) = source.font_scale {
        target.font_scale = Some(scale);
    }
    if let Some(lh) = source.line_height {
        target.line_height = Some(lh);
    }
    if let Some(ref ff) = source.font_family {
        target.font_family = Some(ff.clone());
    }
    if let Some(va) = source.vertical_align {
        target.vertical_align = Some(va);
    }
    if let Some(m) = source.margin_top {
        target.margin_top = Some(m);
    }
    if let Some(m) = source.margin_bottom {
        target.margin_bottom = Some(m);
    }
    if let Some(m) = source.margin_left {
        target.margin_left = Some(m);
    }
    if let Some(m) = source.margin_right {
        target.margin_right = Some(m);
    }
    if let Some(p) = source.padding_top {
        target.padding_top = Some(p);
    }
    if let Some(p) = source.padding_bottom {
        target.padding_bottom = Some(p);
    }
    if let Some(p) = source.padding_left {
        target.padding_left = Some(p);
    }
    if let Some(p) = source.padding_right {
        target.padding_right = Some(p);
    }
    if let Some(c) = source.color {
        target.color = Some(c);
    }
    if let Some(bg) = source.background_color {
        target.background_color = Some(bg);
    }
}

/// Parses CSS text content and maps class selectors (`.class_name`) to `BlockStyle`.
pub fn parse_stylesheet(css: &str) -> HashMap<String, BlockStyle> {
    let mut map = HashMap::new();
    let mut rest = css;

    while let Some(start_brace) = rest.find('{') {
        let selectors = &rest[..start_brace];
        rest = &rest[start_brace + 1..];

        let Some(end_brace) = rest.find('}') else {
            break;
        };
        let body = &rest[..end_brace];
        rest = &rest[end_brace + 1..];

        let mut style = BlockStyle::default();
        let mut has_properties = false;

        for decl in body.split(';') {
            let decl = decl.trim();
            if let Some((prop, val)) = decl.split_once(':')
                && apply_declaration(prop, val, &mut style)
            {
                has_properties = true;
            }
        }

        if has_properties {
            for sel in selectors.split(',') {
                let sel = sel.trim();
                if sel.is_empty() {
                    continue;
                }

                // In CSS, declarations apply to the subject (the rightmost compound selector).
                // Do not apply child or descendant styles to ancestor containers (e.g. #sbo-rt-content or #book-content).
                // Strip attribute selectors like [title="..."] to safely split on combinators and whitespace.
                let mut stripped = String::with_capacity(sel.len());
                let mut in_bracket = false;
                for c in sel.chars() {
                    if c == '[' {
                        in_bracket = true;
                    } else if c == ']' {
                        in_bracket = false;
                    } else if !in_bracket {
                        stripped.push(c);
                    }
                }

                let target_part = stripped
                    .split(|c: char| c.is_whitespace() || c == '>' || c == '+' || c == '~')
                    .rfind(|s| !s.is_empty())
                    .unwrap_or(sel);

                if let Some(idx) = target_part.rfind('#') {
                    let id_name = target_part[idx + 1..]
                        .trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '-');
                    if !id_name.is_empty() {
                        let key = format!("#{id_name}");
                        if let Some(existing) = map.get_mut(&key) {
                            merge_block_style(existing, &style);
                        } else {
                            map.insert(key, style.clone());
                        }
                    }
                } else if let Some(idx) = target_part.rfind('.') {
                    let class_name = target_part[idx + 1..]
                        .trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '-');
                    if !class_name.is_empty() {
                        let key = format!(".{class_name}");
                        if let Some(existing) = map.get_mut(&key) {
                            merge_block_style(existing, &style);
                        } else {
                            map.insert(key, style.clone());
                        }
                        if let Some(existing) = map.get_mut(class_name) {
                            merge_block_style(existing, &style);
                        } else {
                            map.insert(class_name.to_string(), style.clone());
                        }
                    }
                } else {
                    let tag_name = target_part.trim_matches(|c: char| !c.is_alphanumeric());
                    let tag_lower = tag_name.to_ascii_lowercase();
                    if matches!(
                        tag_lower.as_str(),
                        "body"
                            | "h1"
                            | "h2"
                            | "h3"
                            | "h4"
                            | "h5"
                            | "h6"
                            | "p"
                            | "blockquote"
                            | "pre"
                            | "ul"
                            | "ol"
                    ) {
                        // Crucial: Only map to the global tag style (tag:<name> and <name>) if this
                        // selector is a general tag selector (e.g. "p", "h1", or scoped only by
                        // root container IDs like "#sbo-rt-content p" or "#book-content #sbo-rt-content p").
                        // Specific/descendant selectors (e.g. "blockquote p", "div.epigraph p",
                        // "section.preface[title=...] p", "p[data-type=attribution]") must NOT pollute
                        // the base tag style.
                        let parts: Vec<&str> = sel.split_whitespace().collect();
                        let is_base_tag_rule = !parts.is_empty()
                            && parts.last().is_some_and(|last| {
                                last.trim_matches(|c: char| !c.is_alphanumeric())
                                    .eq_ignore_ascii_case(&tag_lower)
                            })
                            && parts[..parts.len() - 1].iter().all(|ancestor| {
                                matches!(*ancestor, "#sbo-rt-content" | "#book-content")
                            });

                        if is_base_tag_rule {
                            let tag_key = format!("tag:{tag_lower}");
                            if let Some(existing) = map.get_mut(&tag_key) {
                                merge_block_style(existing, &style);
                            } else {
                                map.insert(tag_key, style.clone());
                            }
                            if let Some(existing) = map.get_mut(&tag_lower) {
                                merge_block_style(existing, &style);
                            } else {
                                map.insert(tag_lower, style.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    map
}

#[inline]
fn with_prefixed<R>(prefix: &str, s: &str, f: impl FnOnce(&str) -> R) -> R {
    let total_len = prefix.len() + s.len();
    if total_len <= 64 {
        let mut buf = [0u8; 64];
        buf[..prefix.len()].copy_from_slice(prefix.as_bytes());
        buf[prefix.len()..total_len].copy_from_slice(s.as_bytes());
        if let Ok(key) = std::str::from_utf8(&buf[..total_len]) {
            return f(key);
        }
    }
    let key = format!("{prefix}{s}");
    f(&key)
}

/// Resolves effective style for an HTML element given its tag, classes, child classes, optional id, and optional inline style attribute.
pub fn resolve_block_style(
    tag: Option<&str>,
    classes: &[String],
    child_classes: &[String],
    id: Option<&str>,
    inline_style: Option<&str>,
    stylesheet_map: &HashMap<String, BlockStyle>,
    is_heading_candidate: bool,
) -> BlockStyle {
    let mut style = BlockStyle::default();

    if let Some(t) = tag {
        let tag_style = with_prefixed("tag:", t, |k| stylesheet_map.get(k));
        if let Some(tag_style) = tag_style {
            merge_block_style(&mut style, tag_style);
        } else if let Some(tag_style) = stylesheet_map.get(t) {
            let mut clean_tag_style = tag_style.clone();
            // If this tag style has color, ensure it came from a tag rule rather than a colliding class rule
            let has_class_collision = with_prefixed(".", t, |k| stylesheet_map.contains_key(k));
            if clean_tag_style.color.is_some() && has_class_collision {
                clean_tag_style.color = None;
            }
            merge_block_style(&mut style, &clean_tag_style);
        }
    }

    for cls in classes {
        let cls_style =
            with_prefixed(".", cls, |k| stylesheet_map.get(k)).or_else(|| stylesheet_map.get(cls));
        if let Some(cls_style) = cls_style {
            merge_block_style(&mut style, cls_style);
        }
    }

    if let Some(id_name) = id {
        let id_style = with_prefixed("#", id_name, |k| stylesheet_map.get(k))
            .or_else(|| stylesheet_map.get(id_name));
        if let Some(id_style) = id_style {
            merge_block_style(&mut style, id_style);
        }
    }

    if is_heading_candidate {
        for cls in child_classes {
            let cls_style = with_prefixed(".", cls, |k| stylesheet_map.get(k))
                .or_else(|| stylesheet_map.get(cls));
            if let Some(cls_style) = cls_style {
                if cls_style.is_bold {
                    style.is_bold = true;
                }
                if cls_style.is_italic {
                    style.is_italic = true;
                }
                if cls_style.is_uppercase {
                    style.is_uppercase = true;
                }
                if cls_style.is_underline {
                    style.is_underline = true;
                }
                if cls_style.is_strikethrough {
                    style.is_strikethrough = true;
                }
                if let Some(scale) = cls_style.font_scale {
                    style.font_scale = Some(scale);
                }
                if let Some(lh) = cls_style.line_height {
                    style.line_height = Some(lh);
                }
                if let Some(ref ff) = cls_style.font_family {
                    style.font_family = Some(ff.clone());
                }
                if let Some(va) = cls_style.vertical_align {
                    style.vertical_align = Some(va);
                }
                if let Some(c) = cls_style.color {
                    style.color = Some(c);
                }
                if let Some(bg) = cls_style.background_color {
                    style.background_color = Some(bg);
                }
            }
        }
    }

    if let Some(inline) = inline_style {
        let inline_s = parse_inline_style(inline);
        merge_block_style(&mut style, &inline_s);
    }

    // Centered or right-aligned blocks should not have paragraph text-indent
    if style.align == TextAlign::Center || style.align == TextAlign::Right {
        style.text_indent = 0.0;
    }

    style
}

/// Parses a CSS font-size declaration into a relative multiplier scale (clamped between 0.5 and 3.0).
pub fn parse_font_size_scale(val: &str) -> Option<f32> {
    let val = val.trim();
    let scale = if let Some(s) = val.strip_suffix("em") {
        s.trim().parse::<f32>().ok()?
    } else if let Some(s) = val.strip_suffix("rem") {
        s.trim().parse::<f32>().ok()?
    } else if let Some(s) = val.strip_suffix('%') {
        s.trim().parse::<f32>().ok()? / 100.0
    } else if let Some(s) = val.strip_suffix("pt") {
        let pt = s.trim().parse::<f32>().ok()?;
        pt / 12.0
    } else if let Some(s) = val.strip_suffix("px") {
        let px = s.trim().parse::<f32>().ok()?;
        px / 16.0
    } else if val == "larger" || val == "large" {
        1.2
    } else if val == "x-large" {
        1.4
    } else if val == "xx-large" {
        1.7
    } else if val == "smaller" || val == "small" {
        0.85
    } else {
        return None;
    };

    Some(scale.clamp(0.5, 3.0))
}

/// Parses a CSS line-height declaration into a relative multiplier scale (clamped between 0.8 and 3.0).
pub fn parse_line_height(val: &str) -> Option<f32> {
    let mut val = val.trim();
    if val.is_empty() {
        return None;
    }
    if let Some(stripped) = val.strip_suffix("!important") {
        val = stripped.trim();
    } else if let Some((before, _)) = val.split_once("!important") {
        val = before.trim();
    }
    if val.is_empty() {
        return None;
    }

    let lh = if let Some(s) = val.strip_suffix('%') {
        s.trim().parse::<f32>().ok()? / 100.0
    } else if let Some(s) = val.strip_suffix("rem") {
        s.trim().parse::<f32>().ok()?
    } else if let Some(s) = val.strip_suffix("em") {
        s.trim().parse::<f32>().ok()?
    } else if let Some(s) = val.strip_suffix("px") {
        let px = s.trim().parse::<f32>().ok()?;
        px / 16.0
    } else if let Some(s) = val.strip_suffix("pt") {
        let pt = s.trim().parse::<f32>().ok()?;
        pt / 12.0
    } else if val.eq_ignore_ascii_case("normal") {
        1.55
    } else {
        val.parse::<f32>().ok()?
    };

    if !lh.is_finite() || lh <= 0.0 {
        return None;
    }

    Some(lh.clamp(0.8, 3.0))
}

/// Parses a CSS font-family declaration into a clean family name.
pub fn parse_font_family(val: &str) -> Option<String> {
    let val = val.trim();
    if val.is_empty() {
        return None;
    }
    // Take the first family if multiple are comma-separated
    let first = val.split(',').next().unwrap_or(val).trim();
    let cleaned = first.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
    if cleaned.is_empty()
        || cleaned.eq_ignore_ascii_case("inherit")
        || cleaned.eq_ignore_ascii_case("initial")
        || cleaned.eq_ignore_ascii_case("unset")
    {
        None
    } else {
        Some(cleaned.to_string())
    }
}

/// Parses a CSS length dimension into an approximate pixel/point value.
pub fn parse_css_length(val: &str) -> Option<f32> {
    let val = val.trim();
    if let Some(num_str) = val.strip_suffix("pt") {
        num_str.trim().parse::<f32>().ok()
    } else if let Some(num_str) = val.strip_suffix("px") {
        num_str.trim().parse::<f32>().ok()
    } else if let Some(num_str) = val.strip_suffix("em") {
        num_str.trim().parse::<f32>().ok().map(|em| em * 16.0)
    } else if let Some(num_str) = val.strip_suffix("rem") {
        num_str.trim().parse::<f32>().ok().map(|rem| rem * 16.0)
    } else if let Some(num_str) = val.strip_suffix('%') {
        num_str
            .trim()
            .parse::<f32>()
            .ok()
            .map(|pct| (pct * 6.5).max(0.0))
    } else {
        val.parse::<f32>().ok()
    }
}

/// Parses CSS color strings (#rgb, #rgba, #rrggbb, #rrggbbaa, rgb(), rgba(), named colors).
/// Returns [r, g, b, a].
pub fn parse_css_color(val: &str) -> Option<[u8; 4]> {
    let mut val = val.trim();
    if val.is_empty() {
        return None;
    }

    // Strip !important modifier if present
    if let Some(stripped) = val.strip_suffix("!important") {
        val = stripped.trim();
    } else if let Some((before, _)) = val.split_once("!important") {
        val = before.trim();
    }
    if val.is_empty() {
        return None;
    }

    let lower = val.to_ascii_lowercase();

    // 1. Hex format (#rgb, #rgba, #rrggbb, #rrggbbaa)
    if let Some(hex) = lower.strip_prefix('#') {
        let hex = hex.trim();
        return match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
                Some([r * 17, g * 17, b * 17, 255])
            }
            4 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
                let a = u8::from_str_radix(&hex[3..4], 16).ok()?;
                Some([r * 17, g * 17, b * 17, a * 17])
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some([r, g, b, 255])
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some([r, g, b, a])
            }
            _ => None,
        };
    }

    // 2. rgb(...) and rgba(...)
    if lower.starts_with("rgb(") && lower.ends_with(')') {
        let inner = &lower[4..lower.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
        if parts.len() == 3 {
            let r = parse_color_component(parts[0])?;
            let g = parse_color_component(parts[1])?;
            let b = parse_color_component(parts[2])?;
            return Some([r, g, b, 255]);
        }
    } else if lower.starts_with("rgba(") && lower.ends_with(')') {
        let inner = &lower[5..lower.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
        if parts.len() == 4 {
            let r = parse_color_component(parts[0])?;
            let g = parse_color_component(parts[1])?;
            let b = parse_color_component(parts[2])?;
            let a = parse_alpha_component(parts[3])?;
            return Some([r, g, b, a]);
        }
    }

    // 3. Named colors
    match lower.as_str() {
        "black" => Some([0, 0, 0, 255]),
        "white" => Some([255, 255, 255, 255]),
        "red" => Some([255, 0, 0, 255]),
        "green" => Some([0, 128, 0, 255]),
        "blue" => Some([0, 0, 255, 255]),
        "yellow" => Some([255, 255, 0, 255]),
        "gray" | "grey" => Some([128, 128, 128, 255]),
        "orange" => Some([255, 165, 0, 255]),
        "purple" => Some([128, 0, 128, 255]),
        "brown" => Some([165, 42, 42, 255]),
        "gold" => Some([255, 215, 0, 255]),
        "navy" => Some([0, 0, 128, 255]),
        "teal" => Some([0, 128, 128, 255]),
        "cyan" => Some([0, 255, 255, 255]),
        "magenta" => Some([255, 0, 255, 255]),
        "maroon" => Some([128, 0, 0, 255]),
        "silver" => Some([192, 192, 192, 255]),
        "olive" => Some([128, 128, 0, 255]),
        _ => None,
    }
}

fn parse_color_component(s: &str) -> Option<u8> {
    let s = s.trim();
    if let Some(pct) = s.strip_suffix('%') {
        let val = pct.trim().parse::<f32>().ok()?;
        Some(((val / 100.0) * 255.0).round().clamp(0.0, 255.0) as u8)
    } else {
        let val = s.parse::<f32>().ok()?;
        Some(val.round().clamp(0.0, 255.0) as u8)
    }
}

fn parse_alpha_component(s: &str) -> Option<u8> {
    let s = s.trim();
    if let Some(pct) = s.strip_suffix('%') {
        let val = pct.trim().parse::<f32>().ok()?;
        Some(((val / 100.0) * 255.0).round().clamp(0.0, 255.0) as u8)
    } else {
        let val = s.parse::<f32>().ok()?;
        if val <= 1.0 {
            Some((val * 255.0).round().clamp(0.0, 255.0) as u8)
        } else {
            Some(val.round().clamp(0.0, 255.0) as u8)
        }
    }
}

/// Parses inline style string e.g. `text-align: center; font-weight: bold;`
pub fn parse_inline_style(style_str: &str) -> BlockStyle {
    let mut style = BlockStyle::default();
    for decl in style_str.split(';') {
        let decl = decl.trim();
        if let Some((prop, val)) = decl.split_once(':') {
            apply_declaration(prop, val, &mut style);
        }
    }
    style
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_stylesheet() {
        let css = r#"
            .block_3 {
              font-weight: bold;
              text-align: center;
              margin: 0 0 18pt;
            }
            .block_1 {
              text-align: right;
              text-indent: 28.4pt;
            }
            .block_4 {
              text-align: justify;
              text-indent: 28.4pt;
            }
        "#;
        let map = parse_stylesheet(css);
        assert_eq!(
            map.get("block_3"),
            Some(&BlockStyle {
                align: TextAlign::Center,
                text_indent: 0.0,
                is_bold: true,
                is_italic: false,
                is_uppercase: false,
                is_underline: false,
                is_strikethrough: false,
                font_scale: None,
                margin_top: Some(0.0),
                margin_right: Some(0.0),
                margin_bottom: Some(18.0),
                margin_left: Some(0.0),
                ..Default::default()
            })
        );
        assert_eq!(
            map.get("block_1"),
            Some(&BlockStyle {
                align: TextAlign::Right,
                text_indent: 28.4,
                is_bold: false,
                is_italic: false,
                is_uppercase: false,
                is_underline: false,
                is_strikethrough: false,
                font_scale: None,
                ..Default::default()
            })
        );
        assert_eq!(
            map.get("block_4"),
            Some(&BlockStyle {
                align: TextAlign::Justify,
                text_indent: 28.4,
                is_bold: false,
                is_italic: false,
                is_uppercase: false,
                is_underline: false,
                is_strikethrough: false,
                font_scale: None,
                ..Default::default()
            })
        );
    }

    #[test]
    fn test_parse_inline_style() {
        let style = parse_inline_style(
            "text-align: center; text-indent: 1.5em; font-weight: bold; font-style: italic; text-transform: uppercase; font-size: 1.2em",
        );
        assert_eq!(style.align, TextAlign::Center);
        assert_eq!(style.text_indent, 24.0);
        assert!(style.is_bold);
        assert!(style.is_italic);
        assert!(style.is_uppercase);
        assert!(!style.is_underline);
        assert!(!style.is_strikethrough);
        assert_eq!(style.font_scale, Some(1.2));

        let style_decor = parse_inline_style("text-decoration: underline line-through");
        assert!(style_decor.is_underline);
        assert!(style_decor.is_strikethrough);
    }

    #[test]
    fn test_parse_stylesheet_text_decoration() {
        let css = r#"
            .underlined {
                text-decoration: underline;
            }
            .struck {
                text-decoration-line: line-through;
            }
        "#;
        let map = parse_stylesheet(css);
        assert_eq!(
            map.get("underlined"),
            Some(&BlockStyle {
                align: TextAlign::Left,
                text_indent: 0.0,
                is_bold: false,
                is_italic: false,
                is_uppercase: false,
                is_underline: true,
                is_strikethrough: false,
                font_scale: None,
                ..Default::default()
            })
        );
        assert_eq!(
            map.get("struck"),
            Some(&BlockStyle {
                align: TextAlign::Left,
                text_indent: 0.0,
                is_bold: false,
                is_italic: false,
                is_uppercase: false,
                is_underline: false,
                is_strikethrough: true,
                font_scale: None,
                ..Default::default()
            })
        );
    }

    #[test]
    fn test_parse_font_size_scale() {
        let s1 = parse_font_size_scale("1.5em");
        assert_eq!(s1, Some(1.5));
        let s2 = parse_font_size_scale("120%");
        assert_eq!(s2, Some(1.2));
        let s3 = parse_font_size_scale("larger");
        assert_eq!(s3, Some(1.2));
    }

    #[test]
    fn test_parse_box_sides() {
        // 1 param
        assert_eq!(
            parse_box_sides("10px"),
            (Some(10.0), Some(10.0), Some(10.0), Some(10.0))
        );
        // 2 params
        assert_eq!(
            parse_box_sides("10px 20px"),
            (Some(10.0), Some(20.0), Some(10.0), Some(20.0))
        );
        // 3 params
        assert_eq!(
            parse_box_sides("10px 20px 30px"),
            (Some(10.0), Some(20.0), Some(30.0), Some(20.0))
        );
        // 4 params
        assert_eq!(
            parse_box_sides("10px 20px 30px 40px"),
            (Some(10.0), Some(20.0), Some(30.0), Some(40.0))
        );
        // auto
        assert_eq!(
            parse_box_sides("0 auto"),
            (Some(0.0), None, Some(0.0), None)
        );
        // %
        let (t, r, b, l) = parse_box_sides("5%");
        assert_eq!(t, Some(32.5));
        assert_eq!(r, Some(32.5));
        assert_eq!(b, Some(32.5));
        assert_eq!(l, Some(32.5));
    }

    #[test]
    fn test_parse_stylesheet_margins_and_padding() {
        let css = r#"
            .box1 {
                margin: 10px 20px;
                padding: 5px;
            }
            .box2 {
                margin-top: 15px;
                margin-bottom: 25px;
                padding-left: 8px;
                padding-right: 12px;
            }
        "#;
        let map = parse_stylesheet(css);
        let b1 = map.get("box1").expect("box1 should exist");
        assert_eq!(b1.margin_top, Some(10.0));
        assert_eq!(b1.margin_right, Some(20.0));
        assert_eq!(b1.margin_bottom, Some(10.0));
        assert_eq!(b1.margin_left, Some(20.0));
        assert_eq!(b1.padding_top, Some(5.0));
        assert_eq!(b1.padding_right, Some(5.0));
        assert_eq!(b1.padding_bottom, Some(5.0));
        assert_eq!(b1.padding_left, Some(5.0));

        let b2 = map.get("box2").expect("box2 should exist");
        assert_eq!(b2.margin_top, Some(15.0));
        assert_eq!(b2.margin_bottom, Some(25.0));
        assert_eq!(b2.margin_left, None);
        assert_eq!(b2.padding_left, Some(8.0));
        assert_eq!(b2.padding_right, Some(12.0));
    }

    #[test]
    fn test_resolve_block_style_cascade() {
        let css = r#"
            p {
                margin: 10px;
                padding: 5px;
            }
            .content {
                margin-top: 20px;
                padding: 10px;
            }
        "#;
        let map = parse_stylesheet(css);
        let resolved = resolve_block_style(
            Some("p"),
            &["content".to_string()],
            &[],
            None,
            Some("margin-right: 30px; padding-bottom: 15px;"),
            &map,
            false,
        );

        // margin-top: class (20) overrides tag (10)
        assert_eq!(resolved.margin_top, Some(20.0));
        // margin-right: inline (30) overrides class/tag (10)
        assert_eq!(resolved.margin_right, Some(30.0));
        // margin-bottom: from tag (10)
        assert_eq!(resolved.margin_bottom, Some(10.0));
        // margin-left: from tag (10)
        assert_eq!(resolved.margin_left, Some(10.0));

        // padding-top: class (10) overrides tag (5)
        assert_eq!(resolved.padding_top, Some(10.0));
        // padding-bottom: inline (15) overrides class (10)
        assert_eq!(resolved.padding_bottom, Some(15.0));

        // Test effective margins
        assert_eq!(resolved.effective_margin_left(), 20.0); // 10 margin + 10 padding
        assert_eq!(resolved.effective_margin_top(), 30.0); // 20 margin + 10 padding
        assert_eq!(resolved.effective_margin_right(), 40.0); // 30 margin + 10 padding
        assert_eq!(resolved.effective_margin_bottom(), 25.0); // 10 margin + 15 padding
    }

    #[test]
    fn test_id_selector_and_cascade() {
        let css = r#"
            p {
                font-weight: normal;
                text-align: left;
            }
            .note {
                font-style: italic;
                text-align: center;
            }
            #special {
                font-weight: bold;
                text-align: right;
            }
        "#;
        let map = parse_stylesheet(css);
        assert!(map.contains_key("#special"));

        // tag < class < id < inline
        let res = resolve_block_style(
            Some("p"),
            &["note".to_string()],
            &[],
            Some("special"),
            None,
            &map,
            false,
        );
        // id overrides class for align (right vs center)
        assert_eq!(res.align, TextAlign::Right);
        // inherited from class
        assert!(res.is_italic);
        // inherited from id
        assert!(res.is_bold);

        // inline overrides id
        let res_inline = resolve_block_style(
            Some("p"),
            &["note".to_string()],
            &[],
            Some("special"),
            Some("text-align: justify;"),
            &map,
            false,
        );
        assert_eq!(res_inline.align, TextAlign::Justify);
    }

    #[test]
    fn test_display_none_and_list_style_type_none_parsing() {
        let css = r#"
            .hidden-box {
                display: none;
            }
            ul.plain {
                list-style-type: none;
            }
            ol.no-bullet {
                list-style: none;
            }
        "#;
        let map = parse_stylesheet(css);

        let hidden_style = map.get("hidden-box").expect("hidden-box should exist");
        assert!(hidden_style.is_hidden);

        let plain_ul = map.get("plain").expect("plain should exist");
        assert!(plain_ul.list_style_type_none);

        let no_bullet_ol = map.get("no-bullet").expect("no-bullet should exist");
        assert!(no_bullet_ol.list_style_type_none);

        // Test inline parsing
        let inline_hidden = parse_inline_style("display: none; color: red;");
        assert!(inline_hidden.is_hidden);

        let inline_list = parse_inline_style("list-style: none;");
        assert!(inline_list.list_style_type_none);
    }

    #[test]
    fn test_parse_line_height() {
        assert_eq!(parse_line_height("normal"), Some(1.55));
        assert_eq!(parse_line_height("invalid"), None);
        assert_eq!(parse_line_height("1.5"), Some(1.5));
        assert_eq!(parse_line_height("150%"), Some(1.5));
        assert_eq!(parse_line_height("1.2em"), Some(1.2));
        assert_eq!(parse_line_height("1.25rem"), Some(1.25));
        assert_eq!(parse_line_height("24px"), Some(1.5)); // 24 / 16 = 1.5
        assert_eq!(parse_line_height("18pt"), Some(1.5)); // 18 / 12 = 1.5

        // Clamping between 0.8 and 3.0
        assert_eq!(parse_line_height("0.2"), Some(0.8));
        assert_eq!(parse_line_height("5.0"), Some(3.0));
        assert_eq!(parse_line_height("10%"), Some(0.8));
        assert_eq!(parse_line_height("500%"), Some(3.0));

        // !important and boundary / invalid values
        assert_eq!(parse_line_height("1.6 !important"), Some(1.6));
        assert_eq!(parse_line_height("120%!important"), Some(1.2));
        assert_eq!(parse_line_height("0"), None);
        assert_eq!(parse_line_height("-1.5"), None);
        assert_eq!(parse_line_height("NaN"), None);
        assert_eq!(parse_line_height("!important"), None);
    }

    #[test]
    fn test_parse_font_family() {
        assert_eq!(parse_font_family(""), None);
        assert_eq!(parse_font_family("inherit"), None);
        assert_eq!(
            parse_font_family("\"Times New Roman\", serif"),
            Some("Times New Roman".to_string())
        );
        assert_eq!(
            parse_font_family("'Helvetica Neue', Arial, sans-serif"),
            Some("Helvetica Neue".to_string())
        );
        assert_eq!(
            parse_font_family("Georgia, serif"),
            Some("Georgia".to_string())
        );
    }

    #[test]
    fn test_body_selector_and_typography_cascade() {
        let css = r#"
            body {
                line-height: 1.6;
                font-family: "Noto Serif", serif;
            }
            .sup-text {
                vertical-align: super;
            }
            .sub-text {
                vertical-align: sub;
            }
        "#;
        let map = parse_stylesheet(css);
        assert!(map.contains_key("body"));
        let body = map.get("body").unwrap();
        assert_eq!(body.line_height, Some(1.6));
        assert_eq!(body.font_family, Some("Noto Serif".to_string()));

        let sup_style = map.get("sup-text").unwrap();
        assert_eq!(sup_style.vertical_align, Some(VerticalAlign::Super));

        let sub_style = map.get("sub-text").unwrap();
        assert_eq!(sub_style.vertical_align, Some(VerticalAlign::Sub));
    }

    #[test]
    fn test_parse_css_color() {
        assert_eq!(parse_css_color("#fff"), Some([255, 255, 255, 255]));
        assert_eq!(parse_css_color("#0000"), Some([0, 0, 0, 0]));
        assert_eq!(parse_css_color("#0070c0"), Some([0, 112, 192, 255]));
        assert_eq!(parse_css_color("#4f81bd80"), Some([0x4f, 0x81, 0xbd, 0x80]));
        assert_eq!(parse_css_color("rgb(10, 20, 30)"), Some([10, 20, 30, 255]));
        assert_eq!(
            parse_css_color("rgba(10, 20, 30, 0.5)"),
            Some([10, 20, 30, 128])
        );
        assert_eq!(parse_css_color("blue"), Some([0, 0, 255, 255]));
        assert_eq!(parse_css_color("red !important"), Some([255, 0, 0, 255]));
        assert_eq!(parse_css_color("invalid"), None);
    }

    #[test]
    fn test_with_prefixed_stack_and_heap() {
        let res_small = with_prefixed("tag:", "h1", |s| s.to_string());
        assert_eq!(res_small, "tag:h1");

        let res_class = with_prefixed(".", "chapter-title", |s| s.to_string());
        assert_eq!(res_class, ".chapter-title");

        let res_id = with_prefixed("#", "sec_1", |s| s.to_string());
        assert_eq!(res_id, "#sec_1");

        let long_str = "a".repeat(100);
        let res_large = with_prefixed("tag:", &long_str, |s| s.to_string());
        assert_eq!(res_large, format!("tag:{long_str}"));
    }

    #[test]
    fn test_resolve_block_style_with_prefixed_selectors() {
        let mut map = HashMap::new();
        map.insert(
            "tag:h1".to_string(),
            BlockStyle {
                is_bold: true,
                ..Default::default()
            },
        );
        map.insert(
            "tag:h2".to_string(),
            BlockStyle {
                is_italic: true,
                ..Default::default()
            },
        );
        map.insert(
            "#chapter_intro".to_string(),
            BlockStyle {
                font_scale: Some(1.25),
                ..Default::default()
            },
        );
        map.insert(
            ".title-lead".to_string(),
            BlockStyle {
                align: TextAlign::Center,
                ..Default::default()
            },
        );

        let style = resolve_block_style(
            Some("h1"),
            &["title-lead".to_string()],
            &[],
            Some("chapter_intro"),
            None,
            &map,
            true,
        );

        assert!(style.is_bold);
        assert_eq!(style.font_scale, Some(1.25));
        assert_eq!(style.align, TextAlign::Center);
    }
}
