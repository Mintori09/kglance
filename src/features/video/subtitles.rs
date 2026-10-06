use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct SubtitleEntry {
    pub start_secs: f64,
    pub end_secs: f64,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubtitleTrack {
    pub label: String,
    pub entries: Vec<SubtitleEntry>,
}

/// Discover sidecar subtitle files (.srt, .vtt) in the same directory as the video,
/// or attempt to extract embedded subtitles via ffmpeg.
pub fn discover_and_load_subtitles(video_path: &Path) -> Vec<SubtitleTrack> {
    let mut tracks = Vec::new();

    // 1. Search for sidecar files in the same directory
    if let (Some(parent), Some(file_stem)) = (video_path.parent(), video_path.file_stem()) {
        let stem_str = file_stem.to_string_lossy().to_lowercase();
        if let Ok(entries) = std::fs::read_dir(parent) {
            let mut matching_files = Vec::new();
            for entry in entries.flatten() {
                let path = entry.path();
                if path == video_path || !path.is_file() {
                    continue;
                }
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if ext != "srt" && ext != "vtt" {
                    continue;
                }

                let sub_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                if sub_name == stem_str
                    || sub_name.starts_with(&format!("{stem_str}."))
                    || sub_name.starts_with(&format!("{stem_str}_"))
                    || sub_name.starts_with(&format!("{stem_str}-"))
                {
                    matching_files.push(path);
                }
            }

            // Sort files so default / primary comes first
            matching_files.sort();

            for path in matching_files {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let label = derive_track_label(&path, &stem_str);
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    let entries = if ext.eq_ignore_ascii_case("vtt") {
                        parse_vtt(&content)
                    } else {
                        parse_srt(&content)
                    };
                    if !entries.is_empty() {
                        tracks.push(SubtitleTrack { label, entries });
                    }
                }
            }
        }
    }

    // 2. If no sidecar subtitle found, try extracting embedded subtitle track via ffmpeg
    if tracks.is_empty() {
        tracks.extend(
            extract_embedded_subtitles(video_path).map(|entries| SubtitleTrack {
                label: "Embedded Subtitles".to_string(),
                entries,
            }),
        );
    }

    tracks
}

/// Derive a user-friendly label from subtitle filename (e.g. "vi", "en", "Full").
fn derive_track_label(path: &Path, video_stem: &str) -> String {
    let sub_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    if sub_stem == video_stem {
        "Subtitles".to_string()
    } else if let Some(suffix) = sub_stem.strip_prefix(video_stem) {
        let tag = suffix.trim_matches(|c| c == '.' || c == '_' || c == '-');
        if tag.is_empty() {
            "Subtitles".to_string()
        } else {
            tag.to_uppercase()
        }
    } else {
        sub_stem
    }
}

/// Extract embedded subtitle track from video container via ffmpeg.
fn extract_embedded_subtitles(video_path: &Path) -> Option<Vec<SubtitleEntry>> {
    use std::process::Command;

    let output = Command::new("ffmpeg")
        .args([
            "-v",
            "quiet",
            "-i",
            video_path.to_string_lossy().as_ref(),
            "-map",
            "0:s:0",
            "-f",
            "webvtt",
            "pipe:1",
        ])
        .output()
        .ok()?;

    if output.status.success() && !output.stdout.is_empty() {
        let content = String::from_utf8_lossy(&output.stdout);
        let entries = parse_vtt(&content);
        if !entries.is_empty() {
            return Some(entries);
        }
    }
    None
}

/// Parse standard SubRip (.srt) subtitle format.
pub fn parse_srt(content: &str) -> Vec<SubtitleEntry> {
    let mut entries = Vec::new();
    let normalized = content.replace("\r\n", "\n").replace('\r', "\n");
    let blocks = normalized.split("\n\n");

    for block in blocks {
        let lines: Vec<&str> = block
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();
        if lines.is_empty() {
            continue;
        }

        // Find the line with "-->"
        let timing_line_idx = lines.iter().position(|l| l.contains("-->"));
        let Some(idx) = timing_line_idx else {
            continue;
        };

        let timing_line = lines[idx];
        if let Some((start, end)) = parse_time_range(timing_line) {
            let text_lines: Vec<&str> = lines[idx + 1..].to_vec();
            let raw_text = text_lines.join("\n");
            let cleaned_text = clean_subtitle_tags(&raw_text);
            if !cleaned_text.is_empty() {
                entries.push(SubtitleEntry {
                    start_secs: start,
                    end_secs: end,
                    text: cleaned_text,
                });
            }
        }
    }

    entries
}

/// Parse WebVTT (.vtt) subtitle format.
pub fn parse_vtt(content: &str) -> Vec<SubtitleEntry> {
    let mut entries = Vec::new();
    let normalized = content.replace("\r\n", "\n").replace('\r', "\n");
    let blocks = normalized.split("\n\n");

    for block in blocks {
        let lines: Vec<&str> = block
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();
        if lines.is_empty() {
            continue;
        }

        // Skip WEBVTT header block
        if lines[0].starts_with("WEBVTT") || lines[0].starts_with("NOTE") {
            continue;
        }

        // Find the line with "-->"
        let timing_line_idx = lines.iter().position(|l| l.contains("-->"));
        let Some(idx) = timing_line_idx else {
            continue;
        };

        let timing_line = lines[idx];
        if let Some((start, end)) = parse_time_range(timing_line) {
            let text_lines: Vec<&str> = lines[idx + 1..].to_vec();
            let raw_text = text_lines.join("\n");
            let cleaned_text = clean_subtitle_tags(&raw_text);
            if !cleaned_text.is_empty() {
                entries.push(SubtitleEntry {
                    start_secs: start,
                    end_secs: end,
                    text: cleaned_text,
                });
            }
        }
    }

    entries
}

/// Parse "00:01:20.000 --> 00:01:23.500" into start and end seconds.
fn parse_time_range(line: &str) -> Option<(f64, f64)> {
    let parts: Vec<&str> = line.split("-->").collect();
    if parts.len() != 2 {
        return None;
    }

    let start_str = parts[0].trim();
    // VTT lines can contain positioning settings after the end time, e.g. "00:01:23.500 align:start"
    let end_str = parts[1].split_whitespace().next()?;

    let start = parse_timestamp(start_str)?;
    let end = parse_timestamp(end_str)?;
    Some((start, end))
}

/// Parse timestamp formatted as HH:MM:SS.mmm, HH:MM:SS,mmm, MM:SS.mmm, or MM:SS,mmm
fn parse_timestamp(s: &str) -> Option<f64> {
    let s = s.trim();
    let parts: Vec<&str> = s.split(':').collect();

    match parts.len() {
        3 => {
            let hours: f64 = parts[0].parse().ok()?;
            let mins: f64 = parts[1].parse().ok()?;
            let sec_part = parts[2].replace(',', ".");
            let secs: f64 = sec_part.parse().ok()?;
            Some(hours * 3600.0 + mins * 60.0 + secs)
        }
        2 => {
            let mins: f64 = parts[0].parse().ok()?;
            let sec_part = parts[1].replace(',', ".");
            let secs: f64 = sec_part.parse().ok()?;
            Some(mins * 60.0 + secs)
        }
        _ => None,
    }
}

/// Strip formatting HTML/VTT tags like `<i>`, `<b>`, `<font ...>`, `<c.yellow>`, etc.
fn clean_subtitle_tags(raw: &str) -> String {
    let mut result = String::with_capacity(raw.len());
    let mut in_tag = false;

    for ch in raw.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(ch);
        }
    }

    result.trim().to_string()
}

/// Find active subtitle entry matching the current position timestamp.
pub fn find_active_subtitle(entries: &[SubtitleEntry], position: f64) -> Option<&str> {
    for entry in entries {
        if position >= entry.start_secs && position <= entry.end_secs {
            return Some(&entry.text);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_srt() {
        let sample = r#"
1
00:00:01,000 --> 00:00:04,500
Hello <i>world</i>!

2
00:00:05,200 --> 00:00:08,000
Second subtitle line 1
Second subtitle line 2
"#;
        let entries = parse_srt(sample);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].start_secs, 1.0);
        assert_eq!(entries[0].end_secs, 4.5);
        assert_eq!(entries[0].text, "Hello world!");
        assert_eq!(entries[1].start_secs, 5.2);
        assert_eq!(entries[1].end_secs, 8.0);
        assert_eq!(
            entries[1].text,
            "Second subtitle line 1\nSecond subtitle line 2"
        );
    }

    #[test]
    fn test_parse_vtt() {
        let sample = r#"WEBVTT

00:01.000 --> 00:04.500 position:50%
Welcome to the stream!

00:05.000 --> 00:09.000
Enjoy watching.
"#;
        let entries = parse_vtt(sample);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].start_secs, 1.0);
        assert_eq!(entries[0].end_secs, 4.5);
        assert_eq!(entries[0].text, "Welcome to the stream!");
        assert_eq!(entries[1].start_secs, 5.0);
        assert_eq!(entries[1].end_secs, 9.0);
        assert_eq!(entries[1].text, "Enjoy watching.");
    }

    #[test]
    fn test_find_active_subtitle() {
        let entries = vec![
            SubtitleEntry {
                start_secs: 1.0,
                end_secs: 4.0,
                text: "First".to_string(),
            },
            SubtitleEntry {
                start_secs: 5.0,
                end_secs: 9.0,
                text: "Second".to_string(),
            },
        ];

        assert_eq!(find_active_subtitle(&entries, 0.5), None);
        assert_eq!(find_active_subtitle(&entries, 2.5), Some("First"));
        assert_eq!(find_active_subtitle(&entries, 4.5), None);
        assert_eq!(find_active_subtitle(&entries, 7.0), Some("Second"));
        assert_eq!(find_active_subtitle(&entries, 10.0), None);
    }
}
