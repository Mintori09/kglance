use crate::features::video::subtitles::SubtitleTrack;

#[derive(Debug, Clone)]
pub struct MediaState {
    pub playing: bool,
    pub volume: f32,
    pub muted: bool,
    pub volume_before_mute: f32,
    pub time: String,
    pub progress: f32,
    pub metadata: String,
    pub has_video: bool,
    pub show_controls: bool,
    pub position_secs: f64,
    pub duration_secs: f64,
    /// True when playback reached the end of stream; cleared on next play press.
    pub video_ended: bool,
    /// Set when video/audio loading fails; cleared on new file load.
    pub error: Option<String>,
    /// Subtitles enabled flag
    pub subtitles_enabled: bool,
    /// Discovered or extracted subtitle tracks
    pub subtitle_tracks: Vec<SubtitleTrack>,
    /// Index of currently active subtitle track
    pub active_subtitle_track: Option<usize>,
    /// Text of subtitle active at current position
    pub current_subtitle_text: Option<String>,
}

impl Default for MediaState {
    fn default() -> Self {
        Self {
            playing: false,
            volume: 1.0,
            muted: false,
            volume_before_mute: 1.0,
            time: String::new(),
            progress: 0.0,
            metadata: String::new(),
            has_video: false,
            show_controls: false,
            position_secs: 0.0,
            duration_secs: 0.0,
            video_ended: false,
            error: None,
            subtitles_enabled: false,
            subtitle_tracks: Vec::new(),
            active_subtitle_track: None,
            current_subtitle_text: None,
        }
    }
}
