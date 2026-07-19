use std::path::Path;

use crate::parser::{ParseError, ParsedContent, PreviewParser};

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
        Ok(ParsedContent::Video {
            path: path_str,
            duration,
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
