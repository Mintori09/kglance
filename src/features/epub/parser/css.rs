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

        let mut align = None;
        let mut indent = None;
        let mut is_bold = false;
        let mut is_italic = false;
        let mut is_uppercase = false;
        let mut is_underline = false;
        let mut is_strikethrough = false;
        let mut font_scale = None;

        for decl in body.split(';') {
            let decl = decl.trim();
            if let Some((prop, val)) = decl.split_once(':') {
                let prop = prop.trim().to_ascii_lowercase();
                let val = val.trim().to_ascii_lowercase();

                if prop == "text-align" {
                    align = match val.as_str() {
                        "center" => Some(TextAlign::Center),
                        "right" => Some(TextAlign::Right),
                        "justify" => Some(TextAlign::Justify),
                        "left" => Some(TextAlign::Left),
                        _ => None,
                    };
                } else if prop == "text-indent" {
                    indent = parse_css_length(&val);
                } else if prop == "font-weight"
                    && (val == "bold"
                        || val == "bolder"
                        || val == "700"
                        || val == "800"
                        || val == "900")
                {
                    is_bold = true;
                } else if prop == "font-style" && (val == "italic" || val == "oblique") {
                    is_italic = true;
                } else if prop == "text-transform" && val == "uppercase" {
                    is_uppercase = true;
                } else if prop == "text-decoration" || prop == "text-decoration-line" {
                    if val.contains("underline") {
                        is_underline = true;
                    }
                    if val.contains("line-through") {
                        is_strikethrough = true;
                    }
                } else if prop == "font-size" {
                    font_scale = parse_font_size_scale(&val);
                }
            }
        }

        if align.is_some()
            || indent.is_some()
            || is_bold
            || is_italic
            || is_uppercase
            || is_underline
            || is_strikethrough
            || font_scale.is_some()
        {
            let style = BlockStyle {
                align: align.unwrap_or_default(),
                text_indent: indent.unwrap_or(0.0),
                is_bold,
                is_italic,
                is_uppercase,
                is_underline,
                is_strikethrough,
                font_scale,
            };

            for sel in selectors.split(',') {
                let sel = sel.trim();
                // Match class selectors, e.g. `.block_3` or `h1.block_3` or `p.block_4`, and tag selectors `h1`, `h2`...
                for part in sel.split_whitespace() {
                    if let Some(idx) = part.find('.') {
                        let class_name = part[idx + 1..]
                            .trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '-');
                        if !class_name.is_empty() {
                            map.insert(class_name.to_string(), style.clone());
                        }
                    } else {
                        let tag_name = part.trim_matches(|c: char| !c.is_alphanumeric());
                        if matches!(
                            tag_name.to_ascii_lowercase().as_str(),
                            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "blockquote" | "pre"
                        ) {
                            map.insert(tag_name.to_ascii_lowercase(), style.clone());
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
) -> BlockStyle {
    let mut style = BlockStyle::default();

    if let Some(t) = tag
        && let Some(tag_style) = stylesheet_map.get(t)
    {
        if tag_style.align != TextAlign::Left {
            style.align = tag_style.align;
        }
        if tag_style.text_indent > 0.0 {
            style.text_indent = tag_style.text_indent;
        }
        if tag_style.is_bold {
            style.is_bold = true;
        }
        if tag_style.is_italic {
            style.is_italic = true;
        }
        if tag_style.is_uppercase {
            style.is_uppercase = true;
        }
        if tag_style.is_underline {
            style.is_underline = true;
        }
        if tag_style.is_strikethrough {
            style.is_strikethrough = true;
        }
        if let Some(scale) = tag_style.font_scale {
            style.font_scale = Some(scale);
        }
    }

    for cls in classes {
        if let Some(cls_style) = stylesheet_map.get(cls) {
            if cls_style.align != TextAlign::Left {
                style.align = cls_style.align;
            }
            if cls_style.text_indent > 0.0 {
                style.text_indent = cls_style.text_indent;
            }
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

    if let Some(inline) = inline_style {
        let inline_s = parse_inline_style(inline);
        if inline_s.align != TextAlign::Left {
            style.align = inline_s.align;
        }
        if inline_s.text_indent > 0.0 {
            style.text_indent = inline_s.text_indent;
        }
        if inline_s.is_bold {
            style.is_bold = true;
        }
        if inline_s.is_italic {
            style.is_italic = true;
        }
        if inline_s.is_uppercase {
            style.is_uppercase = true;
        }
        if inline_s.is_underline {
            style.is_underline = true;
        }
        if inline_s.is_strikethrough {
            style.is_strikethrough = true;
        }
        if let Some(scale) = inline_s.font_scale {
            style.font_scale = Some(scale);
        }
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
fn parse_css_length(val: &str) -> Option<f32> {
    let val = val.trim();
    if let Some(num_str) = val.strip_suffix("pt") {
        num_str.trim().parse::<f32>().ok()
    } else if let Some(num_str) = val.strip_suffix("px") {
        num_str.trim().parse::<f32>().ok()
    } else if let Some(num_str) = val.strip_suffix("em") {
        num_str.trim().parse::<f32>().ok().map(|em| em * 16.0)
    } else if let Some(num_str) = val.strip_suffix("rem") {
        num_str.trim().parse::<f32>().ok().map(|rem| rem * 16.0)
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
            let prop = prop.trim().to_ascii_lowercase();
            let val = val.trim().to_ascii_lowercase();
            if prop == "text-align" {
                match val.as_str() {
                    "center" => style.align = TextAlign::Center,
                    "right" => style.align = TextAlign::Right,
                    "justify" => style.align = TextAlign::Justify,
                    "left" => style.align = TextAlign::Left,
                    _ => {}
                }
            } else if prop == "text-indent" {
                if let Some(indent) = parse_css_length(&val) {
                    style.text_indent = indent;
                }
            } else if prop == "font-weight"
                && (val == "bold"
                    || val == "bolder"
                    || val == "700"
                    || val == "800"
                    || val == "900")
            {
                style.is_bold = true;
            } else if prop == "font-style" && (val == "italic" || val == "oblique") {
                style.is_italic = true;
            } else if prop == "text-transform" && val == "uppercase" {
                style.is_uppercase = true;
            } else if prop == "text-decoration" || prop == "text-decoration-line" {
                if val.contains("underline") {
                    style.is_underline = true;
                }
                if val.contains("line-through") {
                    style.is_strikethrough = true;
                }
            } else if prop == "font-size"
                && let Some(scale) = parse_font_size_scale(&val)
            {
                style.font_scale = Some(scale);
            }
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
}
