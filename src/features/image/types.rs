#[derive(Debug)]
#[allow(dead_code)]
pub struct ImageRef {
    pub alt_text: String,
    pub path: String,
}

#[derive(Debug, Clone, Copy)]
pub enum ImageFormat {
    Png,
    Jpeg,
    WebP,
    Gif,
    Bmp,
}

#[derive(Debug, Clone, Default)]
pub struct ImageMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub software: Option<String>,
    pub creation_date: Option<String>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub date_taken: Option<String>,
    pub gps_lat: Option<String>,
    pub gps_lon: Option<String>,
    pub exposure: Option<String>,
    pub f_number: Option<String>,
    pub iso: Option<String>,
    pub focal_length: Option<String>,
}
