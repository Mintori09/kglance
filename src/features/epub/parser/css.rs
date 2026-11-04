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
    pub font_scale: Option<f32>,
    pub margin_top: Option<f32>,
    pub margin_bottom: Option<f32>,
    pub margin_left: Option<f32>,
    pub margin_right: Option<f32>,
    pub padding_top: Option<f32>,
    pub padding_bottom: Option<f32>,
    pub padding_left: Option<f32>,
    pub padding_right: Option<f32>,
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
        _ => false,
    }
}

/// Copies specified properties from `source` onto `target`, obeying CSS cascade.
fn merge_block_style(target: &mut BlockStyle, source: &BlockStyle) {
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
    if let Some(scale) = source.font_scale {
        target.font_scale = Some(scale);
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
                // Match class selectors, e.g. `.block_3` or `h1.block_3` or `p.block_4`, and tag selectors `h1`, `h2`...
                for part in sel.split_whitespace() {
                    if let Some(idx) = part.find('.') {
                        let class_name = part[idx + 1..]
                            .trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '-');
                        if !class_name.is_empty() {
                            if let Some(existing) = map.get_mut(class_name) {
                                merge_block_style(existing, &style);
                            } else {
                                map.insert(class_name.to_string(), style.clone());
                            }
                        }
                    } else {
                        let tag_name = part.trim_matches(|c: char| !c.is_alphanumeric());
                        if matches!(
                            tag_name.to_ascii_lowercase().as_str(),
                            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "blockquote" | "pre"
                        ) {
                            if let Some(existing) = map.get_mut(tag_name) {
                                merge_block_style(existing, &style);
                            } else {
                                map.insert(tag_name.to_ascii_lowercase(), style.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    map
}

/// Resolves effective style for an HTML element given its tag, classes, child classes, and optional inline style attribute.
pub fn resolve_block_style(
    tag: Option<&str>,
    classes: &[String],
    child_classes: &[String],
    inline_style: Option<&str>,
    stylesheet_map: &HashMap<String, BlockStyle>,
    is_heading_candidate: bool,
) -> BlockStyle {
    let mut style = BlockStyle::default();

    if let Some(t) = tag
        && let Some(tag_style) = stylesheet_map.get(t)
    {
        merge_block_style(&mut style, tag_style);
    }

    for cls in classes {
        if let Some(cls_style) = stylesheet_map.get(cls) {
            merge_block_style(&mut style, cls_style);
        }
    }

    if is_heading_candidate {
        for cls in child_classes {
            if let Some(cls_style) = stylesheet_map.get(cls) {
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
}
