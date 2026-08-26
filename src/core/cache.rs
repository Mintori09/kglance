use iced::widget::image;
use std::sync::Arc;

use crate::core::preview::PreviewData;
use crate::features::json::JsonNode;

#[derive(Debug, Clone)]
pub enum CachedContent {
    Preview(Arc<PreviewData>),

    DecodedImage {
        handle: image::Handle,
        width: u32,
        height: u32,
    },

    ParsedJson {
        nodes: Vec<JsonNode>,
        pretty: String,
        has_error: bool,
    },

    MediaThumbnail {
        handle: image::Handle,
        width: u32,
        height: u32,
    },

    FontSample {
        handle: image::Handle,
        width: u32,
        height: u32,
    },
}

impl CachedContent {
    #[inline]
    pub fn from_preview(preview: Arc<PreviewData>) -> Self {
        Self::Preview(preview)
    }

    #[inline]
    pub fn as_preview(&self) -> Option<&Arc<PreviewData>> {
        match self {
            Self::Preview(data) => Some(data),
            _ => None,
        }
    }

    #[inline]
    pub fn as_decoded_image(&self) -> Option<(&image::Handle, u32, u32)> {
        match self {
            Self::DecodedImage {
                handle,
                width,
                height,
            } => Some((handle, *width, *height)),
            _ => None,
        }
    }

    #[inline]
    pub fn as_parsed_json(&self) -> Option<(&[JsonNode], &str, bool)> {
        match self {
            Self::ParsedJson {
                nodes,
                pretty,
                has_error,
            } => Some((nodes.as_slice(), pretty.as_str(), *has_error)),
            _ => None,
        }
    }
}

impl From<PreviewData> for CachedContent {
    #[inline]
    fn from(data: PreviewData) -> Self {
        Self::Preview(Arc::new(data))
    }
}

impl From<Arc<PreviewData>> for CachedContent {
    #[inline]
    fn from(data: Arc<PreviewData>) -> Self {
        Self::Preview(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lru::LruCache;
    use std::num::NonZeroUsize;

    #[test]
    fn test_cached_content_conversion_and_lru() {
        let mut cache: LruCache<String, CachedContent> =
            LruCache::new(NonZeroUsize::new(5).unwrap());
        let preview = Arc::new(PreviewData::Text {
            content: "hello".to_string(),
            line_numbers: "1".to_string(),
            language: "text".to_string(),
        });
        cache.put(
            "file1.txt".to_string(),
            CachedContent::from_preview(preview.clone()),
        );

        let entry = cache.get("file1.txt").expect("should exist");
        assert!(entry.as_preview().is_some());
    }
}
