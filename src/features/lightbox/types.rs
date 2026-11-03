use crate::features::image::ImageState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightboxItem {
    pub id: String,
    pub title: Option<String>,
    pub subtitle: String,
    pub link_url: Option<String>,
    pub in_memory_bytes: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct LightboxState {
    pub items: Vec<LightboxItem>,
    pub current_index: usize,
    pub image_state: ImageState,
}
