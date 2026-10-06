#[derive(Debug, Clone, Default)]
pub struct AudioState {
    pub playing: bool,
    pub time: String,
    pub progress: f32,
    pub position_secs: f64,
    pub duration_secs: f64,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub metadata: String,
    pub cover_art: Option<iced::widget::image::Handle>,
    pub error: Option<String>,
}
