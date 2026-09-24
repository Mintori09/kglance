use iced::Font;
use iced::advanced::graphics::text::Paragraph;
use iced::widget::text::Span;
use lru::LruCache;
use rustc_hash::FxHasher;
use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParagraphKey {
    pub spans_hash: u64,
    pub font_size_bits: u32,
    pub max_width_bits: u32,
}

#[derive(Clone)]
pub struct CachedParagraph {
    pub paragraph: Paragraph,
    pub plain_text: String,
    pub size: iced::Size,
}

#[inline]
pub fn hash_spans(spans: &[Span<'_, (), Font>]) -> u64 {
    let mut hasher = FxHasher::default();
    spans.len().hash(&mut hasher);
    for span in spans {
        span.text.hash(&mut hasher);
        span.font.hash(&mut hasher);
        if let Some(color) = span.color {
            color.into_rgba8().hash(&mut hasher);
        }
        if let Some(size) = span.size {
            size.0.to_bits().hash(&mut hasher);
        }
    }
    hasher.finish()
}

thread_local! {
    static PARAGRAPH_CACHE: RefCell<LruCache<ParagraphKey, CachedParagraph>> =
        RefCell::new(LruCache::new(NonZeroUsize::new(1024).unwrap()));
}

#[inline]
pub fn get_cached_paragraph(key: &ParagraphKey) -> Option<CachedParagraph> {
    PARAGRAPH_CACHE.with(|cache| cache.borrow_mut().get(key).cloned())
}

#[inline]
pub fn insert_cached_paragraph(key: ParagraphKey, value: CachedParagraph) {
    PARAGRAPH_CACHE.with(|cache| {
        cache.borrow_mut().put(key, value);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_spans_consistency() {
        let span1: Span<'static, (), Font> = Span::new("Hello World");
        let span2: Span<'static, (), Font> = Span::new("Hello World");
        let span_diff: Span<'static, (), Font> = Span::new("Different");

        let h1 = hash_spans(&[span1]);
        let h2 = hash_spans(&[span2]);
        let h3 = hash_spans(&[span_diff]);

        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }
}
