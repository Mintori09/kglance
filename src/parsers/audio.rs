use std::path::Path;

use symphonia::core::formats::FormatOptions;
use symphonia::core::formats::probe::Hint;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

use crate::parsers::{ParseError, ParsedContent, PreviewParser};

pub struct AudioParser;

impl PreviewParser for AudioParser {
    fn supported_extensions(&self) -> &[&str] {
        &["mp3", "wav", "flac", "ogg", "aac", "m4a", "opus"]
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
        let file = std::fs::File::open(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        let probe = symphonia::default::get_probe();
        let mut reader = probe
            .probe(
                &hint,
                mss,
                FormatOptions::default(),
                MetadataOptions::default(),
            )
            .map_err(|e| ParseError::ParseFailed(format!("probe: {e}")))?;

        let mut title = String::new();
        let mut artist = String::new();
        let mut album = String::new();

        if let Some(meta) = reader.metadata().current() {
            for tag in &meta.media.tags {
                let key = tag.raw.key.to_lowercase();
                let val = tag.raw.value.to_string();
                if key == "title" {
                    title = val;
                } else if key == "artist" {
                    artist = val;
                } else if key == "album" {
                    album = val;
                }
            }
        }

        let total_ns = reader
            .tracks()
            .iter()
            .filter_map(|t| {
                t.num_frames.zip(t.codec_params.as_ref().and_then(|c| {
                    if let symphonia::core::codecs::CodecParameters::Audio(a) = c {
                        a.sample_rate
                    } else {
                        None
                    }
                }))
            })
            .map(|(nf, sr)| {
                if sr > 0 {
                    (nf as u128 * 1_000_000_000) / sr as u128
                } else {
                    0
                }
            })
            .max()
            .unwrap_or(0);

        let total_secs = (total_ns / 1_000_000_000) as u64;
        let duration_str = if total_secs > 0 {
            format!("{}:{:02}", total_secs / 60, total_secs % 60)
        } else {
            String::new()
        };

        let mut meta_parts = Vec::new();
        if !title.is_empty() {
            let part = if !artist.is_empty() {
                format!("{title} — {artist}")
            } else {
                title.clone()
            };
            meta_parts.push(part);
        }
        if !album.is_empty() {
            meta_parts.push(album);
        }
        if !duration_str.is_empty() {
            meta_parts.push(duration_str);
        }

        let metadata_str = if meta_parts.is_empty() {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Audio")
                .to_string()
        } else {
            meta_parts.join("  |  ")
        };

        Ok(ParsedContent::Audio {
            metadata: metadata_str,
            waveform: Vec::new(),
            waveform_width: 0,
            waveform_height: 0,
        })
    }
}
