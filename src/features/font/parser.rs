use crate::core::utils::human_size;
use std::path::Path;

use crate::features::common::parser::traits::{ParseError, PreviewParser};
use crate::features::common::parser::types::ParsedContent;
pub struct FontParser;

impl PreviewParser for FontParser {
    fn supported_extensions(&self) -> &[&str] {
        &["ttf", "otf", "woff", "woff2"]
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let mmap = crate::core::MmapFile::open(path)
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        let fontdue_font =
            fontdue::Font::from_bytes(mmap.as_bytes(), fontdue::FontSettings::default()).ok();

        let ttf_face = ttf_parser::Face::parse(mmap.as_bytes(), 0).ok();

        let (
            family,
            full_name,
            post_script_name,
            weight,
            is_italic,
            glyph_count,
            units_per_em,
            asc,
            desc,
        ) = if let Some(ref face) = ttf_face {
            let mut family = None;
            let mut subfamily = None;
            let mut full_name = None;
            let mut post_script_name = None;

            for name in face.names() {
                if !name.is_unicode() {
                    continue;
                }
                match name.name_id {
                    ttf_parser::name_id::TYPOGRAPHIC_FAMILY => {
                        if let Some(s) = name.to_string() {
                            family = Some(s);
                        }
                    }
                    ttf_parser::name_id::FAMILY => {
                        if family.is_none()
                            && let Some(s) = name.to_string()
                        {
                            family = Some(s);
                        }
                    }
                    ttf_parser::name_id::TYPOGRAPHIC_SUBFAMILY => {
                        if let Some(s) = name.to_string() {
                            subfamily = Some(s);
                        }
                    }
                    ttf_parser::name_id::SUBFAMILY => {
                        if subfamily.is_none()
                            && let Some(s) = name.to_string()
                        {
                            subfamily = Some(s);
                        }
                    }
                    ttf_parser::name_id::FULL_NAME => {
                        if let Some(s) = name.to_string() {
                            full_name = Some(s);
                        }
                    }
                    ttf_parser::name_id::POST_SCRIPT_NAME => {
                        if let Some(s) = name.to_string() {
                            post_script_name = Some(s);
                        }
                    }
                    _ => {}
                }
            }

            let fam = family
                .or_else(|| post_script_name.clone())
                .unwrap_or_else(|| "Unknown".to_string());
            let full = full_name.unwrap_or_else(|| {
                if let Some(sub) = &subfamily {
                    format!("{fam} {sub}")
                } else {
                    fam.clone()
                }
            });

            (
                fam,
                full,
                post_script_name,
                face.weight().to_number(),
                face.is_italic() || face.is_oblique(),
                face.number_of_glyphs() as usize,
                face.units_per_em() as usize,
                face.ascender() as i32,
                face.descender() as i32,
            )
        } else if let Some(ref font) = fontdue_font {
            let name = font.name().unwrap_or("Unknown").to_string();
            let line_metrics = font.horizontal_line_metrics(36.0);
            (
                name.clone(),
                name,
                None,
                400u16,
                false,
                font.glyph_count() as usize,
                font.units_per_em() as usize,
                line_metrics.map(|lm| lm.ascent as i32).unwrap_or(0),
                line_metrics.map(|lm| lm.descent as i32).unwrap_or(0),
            )
        } else {
            return Err(ParseError::ParseFailed(
                "Failed to parse font data".to_string(),
            ));
        };

        let file_size = mmap.len() as u64;

        let mut meta = Vec::new();
        meta.push(format!("Name: {full_name}"));
        if family != full_name {
            meta.push(format!("Family: {family}"));
        }
        meta.push(format!("Weight: {weight}"));
        if is_italic {
            meta.push("Style: Italic".to_string());
        }
        meta.push(format!("Glyphs: {glyph_count}"));
        meta.push(format!("Units per EM: {units_per_em}"));
        meta.push(format!("Ascender: {asc}"));
        meta.push(format!("Descender: {desc}"));
        let size_str = human_size(file_size);
        meta.push(format!("File size: {size_str}"));

        let metadata = meta.join("\n");
        let sample_text = "The quick brown fox jumps over the lazy dog\nABCabc 123 !@#";
        let px = 36.0;

        let (sample, w, h) = if let Some(ref font) = fontdue_font {
            render_text(font, sample_text, px)
        } else {
            (Vec::new(), 1, 1)
        };

        Ok(ParsedContent::Font {
            name: full_name,
            family,
            post_script_name,
            weight,
            is_italic,
            metadata,
            sample,
            sample_width: w,
            sample_height: h,
            data: mmap.to_vec(),
        })
    }
}

fn render_text(font: &fontdue::Font, text: &str, px: f32) -> (Vec<u8>, u32, u32) {
    let line_metrics = font.horizontal_line_metrics(px);
    let line_height = line_metrics.map(|lm| lm.new_line_size).unwrap_or(px * 1.4);

    let mut lines: Vec<Vec<(fontdue::Metrics, Vec<u8>)>> = Vec::new();
    let mut current_line = Vec::new();
    let mut max_line_width: f32 = 0.0;
    let mut x_offset: f32 = 0.0;

    for ch in text.chars() {
        if ch == '\n' {
            lines.push(current_line);
            current_line = Vec::new();
            x_offset = 0.0;
            continue;
        }

        let (metrics, bitmap) = font.rasterize(ch, px);
        x_offset += metrics.advance_width;
        current_line.push((metrics, bitmap));

        if x_offset > max_line_width {
            max_line_width = x_offset;
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    let num_lines = lines.len().max(1);
    let total_height = (num_lines as f32 * line_height + 10.0) as u32;
    let width = (max_line_width.ceil() as u32).max(1) + 20;
    let height = total_height.max(1);

    let mut img: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
        image::ImageBuffer::from_pixel(width, height, image::Rgba([0, 0, 0, 0]));

    let ascent = line_metrics.map(|lm| lm.ascent).unwrap_or(px * 0.8);

    for (line_idx, line) in lines.iter().enumerate() {
        let baseline_y = 5.0 + line_idx as f32 * line_height + ascent;
        let mut x: f32 = 10.0;

        for (metrics, bitmap) in line {
            let gx = (x + metrics.xmin as f32) as i32;
            let gy = (baseline_y - metrics.ymin as f32 - metrics.height as f32) as i32;

            for row in 0..metrics.height {
                for col in 0..metrics.width {
                    let idx = row * metrics.width + col;
                    if idx < bitmap.len() {
                        let alpha = bitmap[idx];
                        if alpha > 0 {
                            let px = (gx + col as i32) as u32;
                            let py = (gy + row as i32) as u32;
                            if px < width && py < height {
                                img.put_pixel(px, py, image::Rgba([230, 230, 230, alpha]));
                            }
                        }
                    }
                }
            }

            x += metrics.advance_width;
        }
    }

    (img.into_raw(), width, height)
}

pub fn resolve_font_name(name: &str) -> String {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    static CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let key = name.to_lowercase();

    {
        let guard = cache.lock().unwrap();
        if let Some(resolved) = guard.get(&key) {
            return resolved.clone();
        }
    }

    let resolved = std::process::Command::new("fc-match")
        .args([name, "--format=%{family[0]}"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if !s.is_empty() { Some(s) } else { None }
            } else {
                None
            }
        })
        .unwrap_or_else(|| name.to_string());

    let mut guard = cache.lock().unwrap();
    guard.insert(key, resolved.clone());
    resolved
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_parser_merriweather() {
        let path =
            Path::new("/home/mintori/.local/share/fonts/Merriweather/Merriweather-Regular.ttf");
        if !path.exists() {
            return;
        }
        let parser = FontParser;
        let content = parser
            .parse(path)
            .expect("failed to parse merriweather font");
        match content {
            ParsedContent::Font {
                name,
                family,
                post_script_name,
                weight,
                is_italic,
                ..
            } => {
                assert_eq!(name, "Merriweather Regular");
                assert_eq!(family, "Merriweather");
                assert_eq!(post_script_name, Some("Merriweather-Regular".to_string()));
                assert_eq!(weight, 400);
                assert!(!is_italic);
            }
            _ => panic!("expected ParsedContent::Font"),
        }
    }
}
