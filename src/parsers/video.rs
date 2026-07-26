use crate::parsers::{ParseError, ParsedContent, PreviewParser};
use std::path::Path;

pub struct VideoParser;

impl PreviewParser for VideoParser {
    fn supported_extensions(&self) -> &[&str] {
        &["mp4", "mkv", "avi", "mov", "wmv", "webm", "flv", "m4v"]
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let path_str = path.to_string_lossy().to_string();
        let duration = probe_duration(path);

        Ok(ParsedContent::Video {
            path: path_str,
            duration,
            thumbnail: Vec::new(),
        })
    }
}

fn probe_duration(path: &Path) -> f64 {
    use std::process::Command;
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-show_entries",
            "format=duration",
            "-of",
            "csv=p=0",
            path.to_string_lossy().as_ref(),
        ])
        .output();
    match output {
        Ok(out) => {
            let s = String::from_utf8_lossy(&out.stdout);
            s.trim().parse::<f64>().unwrap_or(0.0)
        }
        Err(_) => 0.0,
    }
}

pub fn extract_video_thumbnail(path: &Path) -> Option<Vec<u8>> {
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

    if output.status.success() {
        Some(output.stdout)
    } else {
        None
    }
}
