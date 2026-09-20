use iced::widget::image;
use std::sync::Arc;

use crate::core::preview::PreviewData;
use crate::features::json::JsonNode;

#[derive(Debug, Clone)]
pub enum CachedContent {
    Preview(Arc<PreviewData>),

    DecodedImage {
        preview: Option<Arc<PreviewData>>,
        handle: image::Handle,
        width: u32,
        height: u32,
    },

    ParsedJson {
        preview: Option<Arc<PreviewData>>,
        nodes: Vec<JsonNode>,
        pretty: String,
        has_error: bool,
    },

    MediaThumbnail {
        preview: Option<Arc<PreviewData>>,
        handle: image::Handle,
        width: u32,
        height: u32,
    },

    FontSample {
        preview: Option<Arc<PreviewData>>,
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
            Self::DecodedImage { preview, .. } => preview.as_ref(),
            Self::ParsedJson { preview, .. } => preview.as_ref(),
            Self::MediaThumbnail { preview, .. } => preview.as_ref(),
            Self::FontSample { preview, .. } => preview.as_ref(),
        }
    }

    #[inline]
    pub fn as_decoded_image(&self) -> Option<(&image::Handle, u32, u32)> {
        match self {
            Self::DecodedImage {
                handle,
                width,
                height,
                ..
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
                ..
            } => Some((nodes.as_slice(), pretty.as_str(), *has_error)),
            _ => None,
        }
    }

    pub fn estimated_bytes(&self) -> usize {
        let preview_bytes = self.as_preview().map_or(0, |p| p.estimated_bytes());
        match self {
            Self::Preview(_) => preview_bytes,
            Self::DecodedImage { width, height, .. }
            | Self::MediaThumbnail { width, height, .. }
            | Self::FontSample { width, height, .. } => {
                let raw = (*width as usize)
                    .saturating_mul(*height as usize)
                    .saturating_mul(4);
                raw.saturating_add(preview_bytes)
            }
            Self::ParsedJson { nodes, pretty, .. } => {
                let json_bytes = pretty
                    .len()
                    .saturating_add(nodes.len().saturating_mul(std::mem::size_of::<JsonNode>()))
                    .saturating_add(128);
                json_bytes.saturating_add(preview_bytes)
            }
        }
    }
}

impl PreviewData {
    pub fn estimated_bytes(&self) -> usize {
        match self {
            PreviewData::Image { data, .. } => data.len().saturating_add(128),
            PreviewData::Text {
                content,
                line_numbers,
                ..
            } => content
                .len()
                .saturating_add(line_numbers.len())
                .saturating_add(256),
            PreviewData::Markdown { raw_text, .. } => (raw_text.len().saturating_mul(2)).max(512),
            PreviewData::Pdf { data, .. } => data.len().saturating_add(1024),
            PreviewData::Typst { data, source, .. } => {
                data.len().saturating_add(source.len()).saturating_add(512)
            }
            PreviewData::Media {
                thumbnail_or_waveform,
                metadata,
                ..
            } => thumbnail_or_waveform
                .len()
                .saturating_add(metadata.len())
                .saturating_add(256),
            PreviewData::Folder { rows, .. } => rows.len().saturating_mul(256).saturating_add(128),
            PreviewData::Spreadsheet { sheets, .. } => {
                let mut cell_bytes = 0usize;
                for s in sheets {
                    for row in &s.rows {
                        for cell in row {
                            cell_bytes = cell_bytes.saturating_add(cell.len().saturating_add(8));
                        }
                    }
                }
                cell_bytes.max(512)
            }
            PreviewData::Json {
                content, pretty, ..
            } => content
                .len()
                .saturating_add(pretty.len())
                .saturating_add(512),
            PreviewData::Epub { images, .. } => {
                let img_bytes: usize = images.values().map(|b| b.len()).sum();
                img_bytes.saturating_add(4096)
            }
            PreviewData::Font { sample, .. } => sample.len().saturating_add(256),
            PreviewData::Error(msg) => msg.len().saturating_add(64),
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

use lru::LruCache;

/// Entry stored in the LRU cache, pairing content with its precomputed byte size
/// to avoid O(N) recomputation on eviction.
#[derive(Debug, Clone)]
struct CacheEntry {
    content: CachedContent,
    size_bytes: usize,
}

#[derive(Debug)]
pub struct MemoryCache {
    lru: LruCache<String, CacheEntry>,
    current_bytes: usize,
    max_bytes: usize,
}

impl MemoryCache {
    pub fn new(max_bytes: usize) -> Self {
        Self {
            lru: LruCache::unbounded(),
            current_bytes: 0,
            max_bytes,
        }
    }

    #[inline]
    pub fn current_bytes(&self) -> usize {
        self.current_bytes
    }

    #[inline]
    pub fn max_bytes(&self) -> usize {
        self.max_bytes
    }

    #[inline]
    pub fn set_max_bytes(&mut self, max_bytes: usize) {
        self.max_bytes = max_bytes;
        self.trim_to_budget();
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.current_bytes >= self.max_bytes
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.lru.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.lru.len()
    }

    #[inline]
    pub fn contains(&self, key: &str) -> bool {
        self.lru.contains(key)
    }

    #[inline]
    pub fn get(&mut self, key: &str) -> Option<&CachedContent> {
        self.lru.get(key).map(|entry| &entry.content)
    }

    #[inline]
    pub fn peek(&self, key: &str) -> Option<&CachedContent> {
        self.lru.peek(key).map(|entry| &entry.content)
    }

    pub fn put(&mut self, key: String, content: CachedContent) {
        let new_bytes = content.estimated_bytes();
        let entry = CacheEntry {
            content,
            size_bytes: new_bytes,
        };

        if let Some(old) = self.lru.put(key, entry) {
            self.current_bytes = self.current_bytes.saturating_sub(old.size_bytes);
        }
        self.current_bytes = self.current_bytes.saturating_add(new_bytes);

        self.trim_to_budget();
    }

    pub fn clear(&mut self) {
        self.lru.clear();
        self.current_bytes = 0;
    }

    fn trim_to_budget(&mut self) {
        while self.current_bytes > self.max_bytes {
            if let Some((_k, evicted)) = self.lru.pop_lru() {
                self.current_bytes = self.current_bytes.saturating_sub(evicted.size_bytes);
            } else {
                break;
            }
        }
    }
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::new(crate::core::config::DEFAULT_CACHE_MAX_MEMORY_MB * 1024 * 1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cached_content_conversion_and_lru() {
        let mut cache = MemoryCache::new(1024 * 1024);
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
        assert!(cache.current_bytes() > 0);
    }

    #[test]
    fn test_memory_cache_eviction_on_budget_exceeded() {
        let mut cache = MemoryCache::new(500); // 500 bytes budget

        let preview1 = Arc::new(PreviewData::Text {
            content: "a".repeat(200),
            line_numbers: "1".to_string(),
            language: "text".to_string(),
        });
        cache.put(
            "file1.txt".to_string(),
            CachedContent::from_preview(preview1),
        );
        assert!(cache.contains("file1.txt"));

        let preview2 = Arc::new(PreviewData::Text {
            content: "b".repeat(200),
            line_numbers: "1".to_string(),
            language: "text".to_string(),
        });
        cache.put(
            "file2.txt".to_string(),
            CachedContent::from_preview(preview2),
        );
        assert!(cache.contains("file2.txt"));

        // Third item pushes total over 500 bytes, file1 (oldest) should be evicted
        let preview3 = Arc::new(PreviewData::Text {
            content: "c".repeat(200),
            line_numbers: "1".to_string(),
            language: "text".to_string(),
        });
        cache.put(
            "file3.txt".to_string(),
            CachedContent::from_preview(preview3),
        );

        assert!(!cache.contains("file1.txt"));
        assert!(cache.contains("file3.txt"));
    }
}
