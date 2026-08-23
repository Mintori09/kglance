use std::collections::HashMap;

use super::Block;
use super::flatten::flatten_inlines_toc;
use crate::core::TocEntry;

pub fn estimated_block_height(
    block: &Block,
    font_size: f32,
    block_index: usize,
    image_sizes: &HashMap<usize, (u32, u32)>,
) -> f32 {
    let scale = |s: f32| (s * font_size / 14.0).round().max(8.0);
    let line = font_size * 1.5;
    let margin = block_margin(block);
    match block {
        Block::Heading { level, .. } => {
            let h = match level {
                1 => scale(32.0),
                2 => scale(24.0),
                3 => scale(20.0),
                _ => scale(16.0),
            };
            let (pt, pb, div) = match level {
                1 => (24.0, 12.0, 5.0),
                2 => (20.0, 8.0, 5.0),
                3 => (12.0, 4.0, 0.0),
                _ => (8.0, 4.0, 0.0),
            };
            pt + h + pb + div + margin
        }
        Block::Paragraph(inlines) => {
            let text = flatten_inlines_toc(inlines);
            let explicit_lines = text.lines().count().max(1);
            let chars_per_line = ((800.0 - 32.0) / (font_size * 0.55)).max(40.0) as usize;
            let wrapped_lines = (text.len() / chars_per_line).max(1);
            let num_lines = explicit_lines.max(wrapped_lines) as f32;
            num_lines * line + 4.0 + margin
        }
        Block::CodeBlock { lang, code, .. } => {
            let n = code.lines().count().max(1) as f32;
            let top_bar = if lang.is_some() { 28.0 } else { 24.0 };
            let code_line_h = scale(13.0) * 1.35;
            top_bar + 20.0 + n * code_line_h + margin
        }
        Block::Table(t) => {
            let num_cols = if t.headers.is_empty() {
                t.rows.first().map_or(1, |r| r.len())
            } else {
                t.headers.len()
            }
            .max(1);
            let approx_col_chars = (75 / num_cols).max(12);

            let row_height = scale(28.0);
            let mut total_lines = 0.0;

            if !t.headers.is_empty() {
                let header_lines = t
                    .headers
                    .iter()
                    .map(|cell| {
                        let len = flatten_inlines_toc(&cell.content).len();
                        (len / approx_col_chars + 1) as f32
                    })
                    .fold(1.0, f32::max);
                total_lines += header_lines;
            }

            for row in &t.rows {
                let row_lines = row
                    .iter()
                    .map(|cell| {
                        let len = flatten_inlines_toc(&cell.content).len();
                        (len / approx_col_chars + 1) as f32
                    })
                    .fold(1.0, f32::max);
                total_lines += row_lines;
            }

            if total_lines == 0.0 {
                total_lines = 1.0;
            }

            total_lines * row_height + margin
        }
        Block::List { items, .. } => {
            let mut total_h = 0.0;
            let chars_per_line = ((800.0 - 64.0) / (font_size * 0.55)).max(30.0) as usize;
            for item in items {
                let text = flatten_inlines_toc(&item.content);
                let explicit_lines = text.lines().count().max(1);
                let wrapped_lines = (text.len() / chars_per_line).max(1);
                let n = explicit_lines.max(wrapped_lines) as f32;
                total_h += n * line + 8.0;
            }
            total_h.max(line) + margin
        }
        Block::Quote(b) => {
            let h: f32 = b
                .iter()
                .map(|b| estimated_block_height(b, font_size, block_index, image_sizes) * 0.9)
                .sum();
            h + 16.0 + margin
        }
        Block::Alert { content, .. } => {
            let h: f32 = content
                .iter()
                .map(|b| estimated_block_height(b, font_size, block_index, image_sizes))
                .sum();
            scale(28.0) + h + 16.0 + margin
        }
        Block::FootnoteDefinition { content, .. } => {
            let h: f32 = content
                .iter()
                .map(|b| estimated_block_height(b, font_size, block_index, image_sizes))
                .sum();
            scale(16.0) + h + margin
        }
        Block::Frontmatter(entries) => {
            let n = entries.len() as f32;
            scale(32.0) + n * scale(20.0) + margin
        }
        Block::HorizontalRule => 12.0 + margin,
        Block::Image { .. } => {
            if let Some(&(w, h)) = image_sizes.get(&block_index) {
                let display_w = if w > 600 { 600.0 } else { w as f32 };
                let display_h = if w > 0 {
                    (h as f32 * display_w) / (w as f32)
                } else {
                    200.0
                };
                display_h + 8.0 + margin
            } else {
                200.0 + margin
            }
        }
        Block::Mermaid { .. } => 250.0 + margin,
        Block::Html(_) => 50.0 + margin,
        Block::Math(latex) => {
            let n = latex.lines().count().max(1) as f32;
            scale(20.0) + n * scale(16.0) * 1.5 + margin
        }
    }
}

fn block_margin(block: &Block) -> f32 {
    match block {
        Block::Heading { level, .. } if *level == 1 => 24.0,
        Block::Heading { level, .. } if *level == 2 => 20.0,
        Block::Heading { .. } => 16.0,
        Block::HorizontalRule => 24.0,
        Block::CodeBlock { .. } => 16.0,
        Block::Table(_) => 16.0,
        Block::List { .. } => 12.0,
        Block::Quote(_) => 16.0,
        Block::Alert { .. } => 16.0,
        Block::FootnoteDefinition { .. } => 12.0,
        Block::Frontmatter(_) => 24.0,
        Block::Image { .. } => 16.0,
        Block::Mermaid { .. } => 16.0,
        Block::Math(_) => 16.0,
        Block::Paragraph(_) | Block::Html(_) => 8.0,
    }
}

pub fn slugify(text: &str) -> String {
    let mut slug = String::with_capacity(text.len());
    let mut last_dash = false;
    for c in text.chars() {
        if c.is_alphanumeric() {
            slug.extend(c.to_lowercase());
            last_dash = false;
        } else if (c == ' ' || c == '-' || c == '_') && !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }
    if slug.ends_with('-') {
        slug.pop();
    }
    if slug.starts_with('-') {
        slug.remove(0);
    }
    slug
}

pub fn extract_toc(
    blocks: &[Block],
    font_size: f32,
    image_sizes: &HashMap<usize, (u32, u32)>,
) -> Vec<TocEntry> {
    let mut toc = Vec::new();
    let mut y: f32 = 15.0;
    for (i, block) in blocks.iter().enumerate() {
        if let Block::Heading { level, content } = block {
            let text = flatten_inlines_toc(content);
            toc.push(TocEntry {
                level: *level,
                text,
                block_index: i,
                y_offset: y,
            });
        }
        y += estimated_block_height(block, font_size, i, image_sizes);
    }
    toc
}
