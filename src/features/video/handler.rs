use iced_video_player::Video;
use std::time::Duration;

pub fn load_video(path: &str) -> Result<Video, String> {
    let url = url::Url::from_file_path(path).map_err(|_| format!("invalid file path: {path}"))?;
    Video::new(&url).map_err(|e| format!("iced_video_player failed to load video: {e:?}"))
}

pub fn toggle_play_pause(video: &mut Video) {
    video.set_paused(!video.paused());
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
