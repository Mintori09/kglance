use crate::core::preview::PreviewContent;
use crate::features::video::content::MediaContent;
use crate::parsers::{ParseError, PreviewParser};
use std::path::Path;

pub struct VideoParser;

impl PreviewParser<crate::app::Message> for VideoParser {
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

    fn parse(
        &self,
        path: &Path,
    ) -> Result<Box<dyn PreviewContent<crate::app::Message>>, ParseError> {
        let path_str = path.to_string_lossy().to_string();
        let duration = probe_duration(path);

        Ok(Box::new(MediaContent {
            path: path_str,
            duration,
            thumbnail: Vec::new(),
            metadata: format!("Video Duration: {:.2}s", duration),
            waveform: Vec::new(),
            waveform_width: 320,
            waveform_height: 240,
            is_video: true,
        }))
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
