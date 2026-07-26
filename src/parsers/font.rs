use crate::parsers::{ParseError, ParsedContent, PreviewParser};
use std::path::Path;

pub struct FontParser;

impl PreviewParser for FontParser {
    fn supported_extensions(&self) -> &[&str] {
        &["ttf", "otf", "woff", "woff2"]
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let data = std::fs::read(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        let font = fontdue::Font::from_bytes(data, fontdue::FontSettings::default())
            .map_err(|e| ParseError::ParseFailed(format!("font parse: {e}")))?;

        let name = font.name().unwrap_or("Unknown").to_string();
        let glyph_count = font.glyph_count();
        let units_per_em = font.units_per_em();
        let line_metrics = font.horizontal_line_metrics(36.0);
        let asc = line_metrics.map(|lm| lm.ascent as i32).unwrap_or(0);
        let desc = line_metrics.map(|lm| lm.descent as i32).unwrap_or(0);
        let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

        let mut meta = Vec::new();
        meta.push(format!("Name: {name}"));
        meta.push(format!("Glyphs: {glyph_count}"));
        meta.push(format!("Units per EM: {units_per_em}"));
        meta.push(format!("Ascender: {asc}"));
        meta.push(format!("Descender: {desc}"));
        meta.push(format!(
            "File size: {}",
            crate::parsers::human_size(file_size)
        ));

        let metadata = meta.join("\n");
        let sample_text = "The quick brown fox jumps over the lazy dog\nABCabc 123 !@#";
        let px = 36.0;

        let (sample, w, h) = render_text(&font, sample_text, px);

        Ok(ParsedContent::Font {
            name,
            metadata,
            sample,
            sample_width: w,
            sample_height: h,
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
        image::ImageBuffer::from_pixel(width, height, image::Rgba([255, 255, 255, 255]));

    for (line_idx, line) in lines.iter().enumerate() {
        let line_y = 5.0 + line_idx as f32 * line_height;
        let mut x: f32 = 10.0;

        for (metrics, bitmap) in line {
            let gx = (x + metrics.xmin as f32) as i32;
            let gy = (line_y - metrics.ymin as f32) as i32;

            for row in 0..metrics.height {
                for col in 0..metrics.width {
                    let idx = row * metrics.width + col;
                    if idx < bitmap.len() {
                        let alpha = bitmap[idx];
                        if alpha > 0 {
                            let px = (gx + col as i32) as u32;
                            let py = (gy + row as i32) as u32;
                            if px < width && py < height {
                                img.put_pixel(px, py, image::Rgba([0, 0, 0, alpha]));
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
