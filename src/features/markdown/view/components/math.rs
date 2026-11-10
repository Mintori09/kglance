use super::style::STYLE;
use crate::app::Message;
use crate::ui::theme::scale_size;
use crate::ui::types::RenderContext;
use iced::widget::{Space, container, row, svg as svg_widget, text};
use iced::{Alignment, Element, Length};
use lru::LruCache;
use rustc_hash::FxHasher;
use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct MathCacheKey {
    pub(crate) latex_hash: u64,
    pub(crate) font_size_bits: u32,
    pub(crate) color_rgb: (u8, u8, u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CachedMathResult {
    Success(svg_widget::Handle),
    Error(String),
}

const MATH_CACHE_CAPACITY: NonZeroUsize = match NonZeroUsize::new(512) {
    Some(val) => val,
    None => unreachable!(),
};

thread_local! {
    static MATH_SVG_CACHE: RefCell<LruCache<MathCacheKey, CachedMathResult>> =
        RefCell::new(LruCache::new(MATH_CACHE_CAPACITY));
}

#[cfg(test)]
static CACHE_HITS: AtomicUsize = AtomicUsize::new(0);
#[cfg(test)]
static CACHE_MISSES: AtomicUsize = AtomicUsize::new(0);

#[cfg(test)]
#[inline]
pub(crate) fn math_cache_stats() -> (usize, usize) {
    (
        CACHE_HITS.load(Ordering::Relaxed),
        CACHE_MISSES.load(Ordering::Relaxed),
    )
}

#[cfg(test)]
pub(crate) fn reset_math_cache() {
    MATH_SVG_CACHE.with(|cache| cache.borrow_mut().clear());
    CACHE_HITS.store(0, Ordering::Relaxed);
    CACHE_MISSES.store(0, Ordering::Relaxed);
}

/// Extracts equation tag from LaTeX string, e.g. `\tag{3.1}` or `\tag*{3.1}`.
///
/// Returns `(sanitized_latex_without_tag, Option<formatted_tag>)`.
/// For example, `\tag{3.1}` yields tag `Some("(3.1)")`, while `\tag*{3.1}` yields `Some("3.1")`.
pub(crate) fn extract_tag_from_latex(latex: &str) -> (String, Option<String>) {
    let mut rest = latex;
    let mut cleaned = String::with_capacity(latex.len());
    let mut extracted_tag: Option<String> = None;

    while let Some(tag_pos) = rest.find("\\tag") {
        let (before, after_tag) = rest.split_at(tag_pos);
        cleaned.push_str(before);

        let remainder = &after_tag["\\tag".len()..];
        let (is_starred, body) = match remainder.strip_prefix('*') {
            Some(stripped) => (true, stripped.trim_start()),
            None => (false, remainder.trim_start()),
        };

        if let Some(inner) = body.strip_prefix('{') {
            let mut depth = 1usize;
            let mut end = 0usize;
            for (i, ch) in inner.char_indices() {
                match ch {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            end = i;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            let tag_content = &inner[..end];
            if extracted_tag.is_none() {
                let tag_str = if is_starred {
                    tag_content.trim().to_string()
                } else {
                    format!("({})", tag_content.trim())
                };
                extracted_tag = Some(tag_str);
            }
            rest = &inner[end + 1..];
        } else {
            // Not followed by brace - keep literal \tag to preserve user text
            cleaned.push_str("\\tag");
            rest = remainder;
        }
    }

    cleaned.push_str(rest);
    (cleaned, extracted_tag)
}

/// Transliterates Vietnamese accented characters into unaccented ASCII equivalents.
pub(crate) fn remove_vietnamese_diacritics(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        let replacement = match ch {
            'à' | 'á' | 'ả' | 'ã' | 'ạ' | 'ă' | 'ằ' | 'ắ' | 'ẳ' | 'ẵ' | 'ặ' | 'â' | 'ầ' | 'ấ'
            | 'ẩ' | 'ẫ' | 'ậ' => "a",
            'À' | 'Á' | 'Ả' | 'Ã' | 'Ạ' | 'Ă' | 'Ằ' | 'Ắ' | 'Ẳ' | 'Ẵ' | 'Ặ' | 'Â' | 'Ầ' | 'Ấ'
            | 'Ẩ' | 'Ẫ' | 'Ậ' => "A",
            'è' | 'é' | 'ẻ' | 'ẽ' | 'ẹ' | 'ê' | 'ề' | 'ế' | 'ể' | 'ễ' | 'ệ' => {
                "e"
            }
            'È' | 'É' | 'Ẻ' | 'Ẽ' | 'Ẹ' | 'Ê' | 'Ề' | 'Ế' | 'Ể' | 'Ễ' | 'Ệ' => {
                "E"
            }
            'ì' | 'í' | 'ỉ' | 'ĩ' | 'ị' => "i",
            'Ì' | 'Í' | 'Ỉ' | 'Ĩ' | 'Ị' => "I",
            'ò' | 'ó' | 'ỏ' | 'õ' | 'ọ' | 'ô' | 'ồ' | 'ố' | 'ổ' | 'ỗ' | 'ộ' | 'ơ' | 'ờ' | 'ớ'
            | 'ở' | 'ỡ' | 'ợ' => "o",
            'Ò' | 'Ó' | 'Ỏ' | 'Õ' | 'Ọ' | 'Ô' | 'Ồ' | 'Ố' | 'Ổ' | 'Ỗ' | 'Ộ' | 'Ơ' | 'Ờ' | 'Ớ'
            | 'Ở' | 'Ỡ' | 'Ợ' => "O",
            'ù' | 'ú' | 'ủ' | 'ũ' | 'ụ' | 'ư' | 'ừ' | 'ứ' | 'ử' | 'ữ' | 'ự' => {
                "u"
            }
            'Ù' | 'Ú' | 'Ủ' | 'Ũ' | 'Ụ' | 'Ư' | 'Ừ' | 'Ứ' | 'Ử' | 'Ữ' | 'Ự' => {
                "U"
            }
            'ỳ' | 'ý' | 'ỷ' | 'ỹ' | 'ỵ' => "y",
            'Ỳ' | 'Ý' | 'Ỷ' | 'Ỹ' | 'Ỵ' => "Y",
            'đ' => "d",
            'Đ' => "D",
            _ => {
                out.push(ch);
                continue;
            }
        };
        out.push_str(replacement);
    }
    out
}

/// Desugars AMS-LaTeX `\substack{...}` into `\begin{matrix} ... \end{matrix}`
/// which is supported by `pulldown-latex`.
pub(crate) fn desugar_substack(latex: &str) -> String {
    let mut out = String::with_capacity(latex.len());
    let mut rest = latex;

    while let Some(pos) = rest.find("\\substack") {
        let (before, after) = rest.split_at(pos);
        out.push_str(before);

        let remainder = &after["\\substack".len()..];
        let trimmed = remainder.trim_start();
        if let Some(inner) = trimmed.strip_prefix('{') {
            let mut depth = 1usize;
            let mut end = 0usize;
            for (i, ch) in inner.char_indices() {
                match ch {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            end = i;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            if depth == 0 {
                let content = &inner[..end];
                out.push_str(r"\begin{matrix}");
                out.push_str(content);
                out.push_str(r"\end{matrix}");
                rest = &inner[end + 1..];
                continue;
            }
        }

        // If not followed by balanced '{', retain literal '\substack'
        out.push_str("\\substack");
        rest = remainder;
    }

    out.push_str(rest);
    out
}

pub(crate) fn sanitize_for_iced_math(latex: &str) -> String {
    let (without_tag, _) = extract_tag_from_latex(latex);
    let desugared = desugar_substack(&without_tag);
    let mut out = String::with_capacity(desugared.len());
    let mut rest = desugared.as_str();

    while let Some(pos) = rest.find("\\text{") {
        let (before, after) = rest.split_at(pos);
        out.push_str(&before.replace("&&", r"\quad "));

        let body = &after["\\text{".len()..];
        let mut depth = 1usize;
        let mut end = 0usize;
        for (i, ch) in body.char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = i;
                        break;
                    }
                }
                _ => {}
            }
        }

        let text_content = &body[..end];
        let unaccented = remove_vietnamese_diacritics(text_content);
        if unaccented.is_ascii() {
            out.push_str(r"\text{");
            out.push_str(&unaccented);
            out.push('}');
        }
        rest = &body[end + 1..];
    }

    out.push_str(&rest.replace("&&", r"\quad "));
    out
}

pub(crate) fn enhance_svg_stroke(svg_bytes: &[u8], hex_color: &str, stroke_width: f32) -> Vec<u8> {
    if let Ok(mut s) = String::from_utf8(svg_bytes.to_vec()) {
        let target_open = format!(r#"<g fill="{hex_color}">"#);
        let enhanced_open = format!(
            r#"<g fill="{hex_color}" stroke="{hex_color}" stroke-width="{stroke_width:.2}" stroke-linejoin="round" stroke-linecap="round">"#
        );
        if s.contains(&target_open) {
            s = s.replace(&target_open, &enhanced_open);
            return s.into_bytes();
        }
    }
    svg_bytes.to_vec()
}

#[inline]
pub(crate) fn hash_latex(latex: &str) -> u64 {
    let mut hasher = FxHasher::default();
    latex.hash(&mut hasher);
    hasher.finish()
}

pub(crate) fn get_or_render_math_svg(
    latex: &str,
    font_size: f32,
    color_rgb: (u8, u8, u8),
) -> CachedMathResult {
    let latex_hash = hash_latex(latex);
    let key = MathCacheKey {
        latex_hash,
        font_size_bits: font_size.to_bits(),
        color_rgb,
    };

    let cached = MATH_SVG_CACHE.with(|cache| cache.borrow_mut().get(&key).cloned());
    if let Some(res) = cached {
        #[cfg(test)]
        CACHE_HITS.fetch_add(1, Ordering::Relaxed);
        return res;
    }

    #[cfg(test)]
    CACHE_MISSES.fetch_add(1, Ordering::Relaxed);

    let (r, g, b) = color_rgb;
    let math_color = iced_math::Color::rgb(r, g, b);
    let hex_color = format!("#{r:02x}{g:02x}{b:02x}");
    let sanitized = sanitize_for_iced_math(latex);

    let renderer = iced_math::MathRenderer::new()
        .font_size(font_size)
        .display_style(true)
        .color(math_color);

    let result = match renderer.to_svg(&sanitized) {
        Ok(bytes) => {
            let final_bytes = enhance_svg_stroke(&bytes, &hex_color, STYLE.math.stroke_width);
            CachedMathResult::Success(svg_widget::Handle::from_memory(final_bytes))
        }
        Err(_) => CachedMathResult::Error(sanitized),
    };

    MATH_SVG_CACHE.with(|cache| {
        cache.borrow_mut().put(key, result.clone());
    });

    result
}

pub(crate) fn render_math_block<'a>(
    latex: &'a str,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let _math_idx = ctx.alloc_block_index();
    let text_color = ctx.theme.palette().base.text;
    let r = (text_color.r * 255.0).round() as u8;
    let g = (text_color.g * 255.0).round() as u8;
    let b = (text_color.b * 255.0).round() as u8;
    let math_font_size = scale_size(ctx.font_size * STYLE.math.font_scale, ctx.font_size);

    let (cleaned_latex, tag) = extract_tag_from_latex(latex);

    let math_element: Element<'a, Message> =
        match get_or_render_math_svg(&cleaned_latex, math_font_size, (r, g, b)) {
            CachedMathResult::Success(handle) => svg_widget::Svg::new(handle)
                .width(Length::Shrink)
                .height(Length::Shrink)
                .into(),
            CachedMathResult::Error(sanitized) => text::Text::new(sanitized)
                .font(iced::Font::MONOSPACE)
                .color(ctx.theme.palette().roles.danger)
                .into(),
        };

    if let Some(tag_text) = tag {
        let tag_element = text::Text::new(tag_text)
            .size(math_font_size)
            .color(ctx.theme.palette().base.text_dim);

        // Three-column row: Left spacer, centered formula container, right-aligned equation tag
        let formula_container = container(math_element).center_x(Length::Fill);

        let row_content = row![
            Space::new().width(Length::Shrink),
            formula_container,
            container(tag_element).align_right(Length::Shrink)
        ]
        .width(Length::Fill)
        .align_y(Alignment::Center);

        container(row_content)
            .width(Length::Fill)
            .padding(STYLE.math.padding)
            .into()
    } else {
        container(math_element)
            .center_x(Length::Fill)
            .padding(STYLE.math.padding)
            .into()
    }
}

#[cfg(test)]
#[path = "math_tests.rs"]
mod tests;
