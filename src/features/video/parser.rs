use crate::features::common::parser::traits::{ParseError, PreviewParser};
use crate::features::common::parser::types::ParsedContent;
use std::path::Path;

pub struct VideoParser;

impl PreviewParser for VideoParser {
    fn supported_extensions(&self) -> &[&str] {
        &["mp4", "mkv", "avi", "mov", "wmv", "webm", "flv", "m4v"]
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let path_str = path.to_string_lossy().to_string();
        let (duration, width, height) = probe_video_info(path);

        Ok(ParsedContent::Video {
            path: path_str,
            duration,
            thumbnail: Vec::new(),
            width,
            height,
        })
    }
}

pub fn probe_video_info(path: &Path) -> (f64, u32, u32) {
    use std::process::Command;
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height:format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            path.to_string_lossy().as_ref(),
        ])
        .output();

    let Ok(out) = output else {
        return (0.0, 1280, 720);
    };

    let text = String::from_utf8_lossy(&out.stdout);
    let lines = text.lines().map(str::trim).filter(|s| !s.is_empty());

    // With stream=width,height and format=duration, ffprobe prints:
    // width
    // height
    // duration
    // If there is no video stream (e.g. audio only), only duration may appear.
    let mut width = 0u32;
    let mut height = 0u32;
    let mut duration = 0.0f64;

    let mut vals = Vec::new();
    for line in lines {
        vals.push(line);
    }

    match vals.as_slice() {
        [w_str, h_str, d_str, ..] => {
            width = w_str.parse::<u32>().unwrap_or(0);
            height = h_str.parse::<u32>().unwrap_or(0);
            duration = d_str.parse::<f64>().unwrap_or(0.0);
        }
        [w_str, h_str] => {
            width = w_str.parse::<u32>().unwrap_or(0);
            height = h_str.parse::<u32>().unwrap_or(0);
        }
        [d_str] => {
            duration = d_str.parse::<f64>().unwrap_or(0.0);
        }
        _ => {}
    }

    let width = if width == 0 { 1280 } else { width };
    let height = if height == 0 { 720 } else { height };

    (duration, width, height)
}

pub fn extract_video_thumbnail(path: &Path) -> Option<Vec<u8>> {
    if let Some(cached_path) = crate::core::disk_cache::get_cached_path("media", path, "jpg")
        && let Ok(bytes) = std::fs::read(&cached_path)
        && !bytes.is_empty()
    {
        return Some(bytes);
    }

    let bytes = extract_video_thumbnail_ffmpeg(path)?;
    let _ = crate::core::disk_cache::put_bytes("media", path, "jpg", &bytes);
    Some(bytes)
}

fn extract_video_thumbnail_ffmpeg(path: &Path) -> Option<Vec<u8>> {
    use std::process::Command;

    let output = Command::new("ffmpeg")
        .args([
            "-ss",
            "0.1",
            "-i",
            path.to_string_lossy().as_ref(),
            "-vf",
            "scale=512:-1",
            "-vframes",
            "1",
            "-q:v",
            "2",
            "-f",
            "image2pipe",
            "-vcodec",
            "mjpeg",
            "pipe:1",
        ])
        .output()
        .ok()?;

    if output.status.success() && !output.stdout.is_empty() {
        Some(output.stdout)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_video_parser_supported_extensions() {
        let parser = VideoParser;
        assert!(parser.supported_extensions().contains(&"mp4"));
        assert!(parser.supported_extensions().contains(&"mkv"));
        assert!(parser.supported_extensions().contains(&"webm"));
    }

    #[test]
    fn test_extract_video_thumbnail_returns_cached_bytes() {
        let temp_dir = tempdir().unwrap();

        let video_path = temp_dir.path().join("clip.mp4");
        std::fs::write(&video_path, b"dummy video file").unwrap();

        let fake_thumb = b"\xFF\xD8\xFF\xE0 cached thumbnail jpeg bytes";
        crate::core::disk_cache::put_bytes("media", &video_path, "jpg", fake_thumb).unwrap();

        let result = extract_video_thumbnail(&video_path).expect("should hit cache");
        assert_eq!(result, fake_thumb);
    }
}
