use iced_video_player::Video;
use std::path::Path;
use std::time::Duration;

pub fn load_video(path: &str) -> Result<Video, String> {
    let abs_path = std::fs::canonicalize(path).unwrap_or_else(|_| Path::new(path).to_path_buf());
    let url =
        url::Url::from_file_path(&abs_path).map_err(|_| format!("invalid file path: {path}"))?;
    Video::new(&url).map_err(|e| format!("iced_video_player failed to load video: {e:?}"))
}

pub fn toggle_play_pause(video: &mut Video) {
    video.set_paused(!video.paused());
}

pub fn set_volume(video: &mut Video, volume: f64) {
    video.set_volume(volume);
}

pub fn set_muted(video: &mut Video, muted: bool) {
    video.set_muted(muted);
}

pub fn seek_to_ratio(video: &mut Video, ratio: f64) {
    let dur = video.duration().as_secs_f64();
    let target = (ratio * dur).clamp(0.0, dur);
    let _ = video.seek(Duration::from_secs_f64(target), true);
}

pub fn seek_relative(video: &mut Video, secs: f64) {
    let cur = video.position().as_secs_f64();
    let dur = video.duration().as_secs_f64();
    let target = (cur + secs).clamp(0.0, dur);
    let _ = video.seek(Duration::from_secs_f64(target), true);
}
