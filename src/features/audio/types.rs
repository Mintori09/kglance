#[derive(Debug, Clone)]
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
    pub volume: f32,
    pub muted: bool,
    pub volume_before_mute: f32,
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            playing: false,
            time: String::new(),
            progress: 0.0,
            position_secs: 0.0,
            duration_secs: 0.0,
            title: String::new(),
            artist: String::new(),
            album: String::new(),
            metadata: String::new(),
            cover_art: None,
            error: None,
            volume: 1.0,
            muted: false,
            volume_before_mute: 1.0,
        }
    }
}
