use std::collections::HashMap;

use super::Block;
use super::flatten::flatten_inlines_toc;
use super::layout_constants::{self as lc, heading_layout, scale_size};
use crate::core::TocEntry;

pub fn estimated_block_height(
    block: &Block,
    font_size: f32,
    block_index: usize,
    image_sizes: &HashMap<usize, (u32, u32)>,
    content_width: f32,
) -> f32 {
    let scale = |s: f32| scale_size(s, font_size);
    let line = font_size * 1.5;
    let margin = block_margin(block, font_size);
    let effective_width = if content_width > 0.0 {
        content_width
    } else {
        800.0
    };

    match block {
        Block::Heading { level, .. } => {
            let layout = heading_layout(*level, font_size);
            let div = if *level == 1 || *level == 2 {
                lc::DIVIDER_HEIGHT + lc::SECTION_SPACING
            } else {
                0.0
            };
            layout.padding_top + layout.font_size + layout.padding_bottom + div + margin
        }
        Block::Paragraph(inlines) => {
            let text = flatten_inlines_toc(inlines);
            let explicit_lines = text.lines().count().max(1);
            let available_w = effective_width.max(100.0);
            let chars_per_line = (available_w / (font_size * 0.55)).max(20.0) as usize;
            let visual_units = text
                .chars()
                .map(|c| {
                    if ('\u{2E80}'..='\u{9FFF}').contains(&c)
                        || ('\u{3040}'..='\u{30FF}').contains(&c)
                        || ('\u{AC00}'..='\u{D7AF}').contains(&c)
                        || ('\u{FF01}'..='\u{FF60}').contains(&c)
                    {
                        2
                    } else {
                        1
                    }
                })
                .sum::<usize>();
            let wrapped_lines = (visual_units / chars_per_line).max(1);
            let num_lines = explicit_lines.max(wrapped_lines) as f32;
            let pad_v = (lc::PARAGRAPH_PADDING_V * 2) as f32;
            num_lines * line + pad_v + margin
        }
        Block::CodeBlock { lang: _, code, .. } => {
            let n = code.lines().count().max(1) as f32;
            let button_font_h = scale_size(lc::CODE_LABEL_BUTTON_FONT_SIZE, font_size);
            let top_bar = button_font_h
                + (lc::CODE_BUTTON_PADDING_V * 2).max(lc::CODE_TOP_BAR_PADDING_V * 2) as f32;
            let code_font_size = scale_size(lc::CODE_LINE_FONT_SIZE, font_size);
            let code_line_h = code_font_size * 1.35;
            let pad_v = (lc::CODE_PADDING * 2) as f32;
            top_bar + pad_v + n * code_line_h + margin
        }
        Block::Table(t) => {
            let num_cols = if t.headers.is_empty() {
                t.rows.first().map_or(1, |r| r.len())
            } else {
                t.headers.len()
            }
            .max(1);
            let available_w = effective_width.max(100.0);
            let col_width = (available_w / num_cols as f32).max(50.0);
            let char_width = font_size * 0.55;
            let approx_col_chars = (col_width / char_width).max(8.0) as usize;

            let row_height = scale(28.0);
            let mut total_lines = 0.0;

            if !t.headers.is_empty() {
                let header_lines = t
                    .headers
                    .iter()
                    .map(|cell| {
                        let len = flatten_inlines_toc(&cell.content).chars().count();
                        (len / approx_col_chars + 1) as f32
                    })
                    .fold(1.0, f32::max);
                total_lines += header_lines;
            }

            for row in &t.rows {
                let row_lines = row
                    .iter()
                    .map(|cell| {
                        let len = flatten_inlines_toc(&cell.content).chars().count();
                        (len / approx_col_chars + 1) as f32
                    })
                    .fold(1.0, f32::max);
                total_lines += row_lines;
            }

            if total_lines == 0.0 {
                total_lines = 1.0;
            }

            let separator_h = if !t.rows.is_empty() {
                lc::DIVIDER_HEIGHT
            } else {
                0.0
            };
            total_lines * row_height + separator_h + margin
        }
        Block::List { items, .. } => {
            let mut total_h = 0.0;
            let available_w = (effective_width - lc::LIST_SUB_BLOCK_LEFT_PADDING).max(100.0);
            let chars_per_line = (available_w / (font_size * 0.55)).max(20.0) as usize;
            for item in items {
                let text = flatten_inlines_toc(&item.content);
                let explicit_lines = text.lines().count().max(1);
                let visual_units = text
                    .chars()
                    .map(|c| {
                        if ('\u{2E80}'..='\u{9FFF}').contains(&c)
                            || ('\u{3040}'..='\u{30FF}').contains(&c)
                            || ('\u{AC00}'..='\u{D7AF}').contains(&c)
                            || ('\u{FF01}'..='\u{FF60}').contains(&c)
                        {
                            2
                        } else {
                            1
                        }
                    })
                    .sum::<usize>();
                let wrapped_lines = (visual_units / chars_per_line).max(1);
                let n = explicit_lines.max(wrapped_lines) as f32;
                let item_pad = lc::LIST_ITEM_PADDING * 2.0;
                total_h += n * line + item_pad;
            }
            if items.len() > 1 {
                total_h += (items.len() - 1) as f32 * lc::SECTION_SPACING;
            }
            total_h.max(line) + margin
        }
        Block::Quote(b) => {
            let h: f32 = b
                .iter()
                .map(|b| {
                    estimated_block_height(b, font_size, block_index, image_sizes, effective_width)
                        * 0.9
                })
                .sum();
            let pad_v = (lc::QUOTE_CONTENT_PADDING_V * 2) as f32;
            h + pad_v + margin
        }
        Block::Alert { content, .. } => {
            let h: f32 = content
                .iter()
                .map(|b| {
                    estimated_block_height(b, font_size, block_index, image_sizes, effective_width)
                })
                .sum();
            scale(28.0) + h + 20.0 + margin
        }
        Block::FootnoteDefinition { content, .. } => {
            let h: f32 = content
                .iter()
                .map(|b| {
                    estimated_block_height(b, font_size, block_index, image_sizes, effective_width)
                })
                .sum();
            scale(16.0) + h + 8.0 + margin
        }
        Block::Frontmatter(entries) => {
            let n = entries.len() as f32;
            let pad_v = 24.0;
            pad_v + n * scale(20.0) + margin
        }
        Block::HorizontalRule => {
            let pad_v = (lc::HR_PADDING_V * 2) as f32;
            lc::DIVIDER_HEIGHT + pad_v + margin
        }
        Block::Image { .. } => {
            let max_w = lc::IMAGE_MAX_WIDTH;
            let pad_v = (lc::IMAGE_PADDING_V * 2) as f32;
            if let Some(&(w, h)) = image_sizes.get(&block_index) {
                let display_w = if w as f32 > max_w { max_w } else { w as f32 };
                let display_h = if w > 0 {
                    (h as f32 * display_w) / (w as f32)
                } else {
                    200.0
                };
                display_h + pad_v + margin
            } else {
                200.0 + pad_v + margin
            }
        }
        Block::Mermaid { .. } => 250.0 + margin,
        Block::Html(_) => {
            let pad_v = (lc::PARAGRAPH_PADDING_V * 2) as f32;
            lc::HTML_FONT_SIZE * 1.5 + pad_v + margin
        }
        Block::Math(latex) => {
            let n = latex.lines().count().max(1) as f32;
            let pad_v = (lc::MATH_PADDING * 2) as f32;
            let math_font_size = scale_size(font_size * lc::MATH_FONT_SCALE, font_size);
            pad_v + n * math_font_size * 1.5 + margin
        }
    }
}

/// Returns the bottom margin for a block, matching the view-layer spacing.
///
/// Defined here (mirroring `view::blocks::block_margin`) so that the parser
/// can estimate block heights without importing from the UI layer.
pub fn block_margin(block: &Block, font_size: f32) -> f32 {
    let base = match block {
        Block::Heading { level, .. } => match level {
            1 => lc::MARGIN_HEADING_H1,
            2 => lc::MARGIN_HEADING_H2,
            _ => lc::MARGIN_HEADING_DEFAULT,
        },
        Block::HorizontalRule => lc::MARGIN_HORIZONTAL_RULE,
        Block::CodeBlock { .. } => lc::MARGIN_CODE,
        Block::Table(_) => lc::MARGIN_TABLE,
        Block::Quote(_) => lc::MARGIN_QUOTE,
        Block::Alert { .. } => lc::MARGIN_ALERT,
        Block::FootnoteDefinition { .. } => lc::MARGIN_FOOTNOTE,
        Block::Frontmatter(_) => lc::MARGIN_FRONTMATTER,
        Block::Image { .. } => lc::MARGIN_IMAGE,
        Block::Mermaid { .. } => lc::MARGIN_MERMAID,
        Block::List { .. } => lc::MARGIN_LIST,
        Block::Paragraph(_) => lc::MARGIN_PARAGRAPH,
        Block::Html(_) => lc::MARGIN_HTML,
        Block::Math(_) => lc::MARGIN_MATH,
    };
    scale_size(base, font_size)
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
    content_width: f32,
) -> Vec<TocEntry> {
    let mut toc = Vec::new();
    let mut y: f32 = scale_size(lc::CONTENT_PADDING, font_size);
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
        y += estimated_block_height(block, font_size, i, image_sizes, content_width);
    }
    toc
}

pub fn rescale_markdown_scroll_y(
    blocks: &[Block],
    old_scroll_y: f32,
    old_font_size: f32,
    new_font_size: f32,
    image_sizes: &HashMap<usize, (u32, u32)>,
    content_width: f32,
) -> f32 {
    if blocks.is_empty() || old_scroll_y <= 0.0 {
        return 0.0;
    }

    let mut current_y = scale_size(lc::CONTENT_PADDING, old_font_size);
    let mut target_block_idx = 0;
    let mut progress = 0.0;

    for (i, block) in blocks.iter().enumerate() {
        let h = estimated_block_height(block, old_font_size, i, image_sizes, content_width);
        if old_scroll_y < current_y + h || i == blocks.len() - 1 {
            target_block_idx = i;
            let offset_inside_block = (old_scroll_y - current_y).max(0.0);
            progress = if h > 0.0 {
                (offset_inside_block / h).clamp(0.0, 1.0)
            } else {
                0.0
            };
            break;
        }
        current_y += h;
    }

    let mut new_y = scale_size(lc::CONTENT_PADDING, new_font_size);
    for (i, block) in blocks.iter().enumerate() {
        let new_h = estimated_block_height(block, new_font_size, i, image_sizes, content_width);
        if i == target_block_idx {
            return (new_y + progress * new_h).max(0.0);
        }
        new_y += new_h;
    }

    new_y.max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::markdown::parser::Inline;

    #[test]
    fn test_paragraph_height_scales_with_content_width() {
        let long_text = "This is a long test paragraph meant to test wrapping behavior across different container widths. ".repeat(5);
        let block = Block::Paragraph(vec![Inline::Text(long_text)]);
        let image_sizes = HashMap::new();

        let h_wide = estimated_block_height(&block, 14.0, 0, &image_sizes, 1200.0);
        let h_narrow = estimated_block_height(&block, 14.0, 0, &image_sizes, 300.0);

        // Narrow container must wrap more lines, resulting in a taller block
        assert!(
            h_narrow > h_wide,
            "Narrow height ({}) should be greater than wide height ({})",
            h_narrow,
            h_wide
        );
    }
}
