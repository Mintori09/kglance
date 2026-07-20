use crate::parsers::{ParseError, ParsedContent, PreviewParser};
use std::path::Path;

pub struct VideoParser;

impl PreviewParser for VideoParser {
    fn supported_extensions(&self) -> &[&str] {
        &["mp4", "mkv", "avi", "mov", "wmv", "webm", "flv", "m4v"]
    }

    fn is_supported(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| {
                self.supported_extensions()
                    .contains(&e.to_lowercase().as_str())
            })
            .unwrap_or(false)
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let path_str = path.to_string_lossy().to_string();
        let duration = probe_duration(path);

        let thumbnail_bytes = extract_thumbnail(path).unwrap_or_default();

        Ok(ParsedContent::Video {
            path: path_str,
            duration,
            thumbnail: thumbnail_bytes,
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

fn extract_thumbnail(path: &Path) -> Option<Vec<u8>> {
    use std::process::Command;

    let output = Command::new("ffmpeg")
        .args([
            "-ss",
            "00:00:01",
            "-i",
            path.to_string_lossy().as_ref(),
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
