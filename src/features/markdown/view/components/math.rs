use super::style::STYLE;
use crate::app::Message;
use crate::ui::theme::scale_size;
use crate::ui::types::RenderContext;
use iced::widget::{container, svg as svg_widget, text};
use iced::{Element, Length};
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

pub(crate) fn sanitize_for_iced_math(latex: &str) -> String {
    let mut out = String::with_capacity(latex.len());
    let mut rest = latex;

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
        if text_content.is_ascii() {
            out.push_str(r"\text{");
            out.push_str(text_content);
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

    let math_element: Element<'a, Message> =
        match get_or_render_math_svg(latex, math_font_size, (r, g, b)) {
            CachedMathResult::Success(handle) => svg_widget::Svg::new(handle)
                .width(Length::Shrink)
                .height(Length::Shrink)
                .into(),
            CachedMathResult::Error(sanitized) => text::Text::new(sanitized)
                .font(iced::Font::MONOSPACE)
                .color(ctx.theme.palette().roles.danger)
                .into(),
        };

    container(math_element)
        .center_x(Length::Fill)
        .padding(STYLE.math.padding)
        .into()
}

#[cfg(test)]
#[path = "math_tests.rs"]
mod tests;
