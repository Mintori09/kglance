use std::path::Path;

use symphonia::core::formats::FormatOptions;
use symphonia::core::formats::probe::Hint;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;

use crate::features::common::parser::traits::{ParseError, PreviewParser};
use crate::features::common::parser::types::ParsedContent;

pub struct AudioParser;

impl PreviewParser for AudioParser {
    fn supported_extensions(&self) -> &[&str] {
        &["mp3", "wav", "flac", "ogg", "aac", "m4a", "opus"]
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let file = std::fs::File::open(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        let probe = symphonia::default::get_probe();
        let mut meta_opts = MetadataOptions::default();
        meta_opts.limit_visual_bytes = symphonia::core::common::Limit::None;

        let mut reader = probe
            .probe(&hint, mss, FormatOptions::default(), meta_opts)
            .map_err(|e| ParseError::ParseFailed(format!("probe: {e}")))?;

        let mut title = String::new();
        let mut artist = String::new();
        let mut album = String::new();
        let mut cover_art: Option<Vec<u8>> = None;

        let mut read_meta = |meta: &symphonia::core::meta::MetadataRevision| {
            if let Some(visual) = meta
                .media
                .visuals
                .iter()
                .find(|v| v.usage == Some(symphonia::core::meta::StandardVisualKey::FrontCover))
                .or_else(|| meta.media.visuals.first())
            {
                cover_art = Some(visual.data.to_vec());
            }

            for tag in &meta.media.tags {
                if let Some(std_tag) = &tag.std {
                    use symphonia::core::meta::StandardTag;
                    match std_tag {
                        StandardTag::TrackTitle(t) => {
                            let clean = fix_mojikake(t);
                            if !clean.is_empty() {
                                title = clean;
                            }
                        }
                        StandardTag::Artist(a)
                        | StandardTag::AlbumArtist(a)
                        | StandardTag::Author(a) => {
                            let clean = fix_mojikake(a);
                            if !clean.is_empty() {
                                artist = clean;
                            }
                        }
                        StandardTag::Album(al) => {
                            let clean = fix_mojikake(al);
                            if !clean.is_empty() {
                                album = clean;
                            }
                        }
                        _ => {}
                    }
                }
                let key = tag.raw.key.to_lowercase();
                let raw_val = tag.raw.value.to_string();
                let clean = fix_mojikake(&raw_val);
                if !clean.is_empty() {
                    match key.as_str() {
                        "title" | "tit2" if title.is_empty() || key == "tit2" => {
                            title = clean;
                        }
                        "artist" | "tpe1" | "tpe2" | "albumartist" | "author"
                            if artist.is_empty() || key == "tpe1" =>
                        {
                            artist = clean;
                        }
                        "album" | "talb" if album.is_empty() || key == "talb" => {
                            album = clean;
                        }
                        _ => {}
                    }
                }
            }
        };

        let mut meta_queue = reader.metadata();
        loop {
            if let Some(meta) = meta_queue.current() {
                read_meta(meta);
            }
            if meta_queue.pop().is_none() {
                break;
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

        let duration_secs = (total_ns / 1_000_000_000) as u64;

        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Audio Track")
            .to_string();

        let display_title = if !title.is_empty() { title } else { file_stem };

        if let Some(parent) = path.parent().filter(|_| cover_art.is_none()) {
            for name in &[
                "cover.jpg",
                "cover.png",
                "folder.jpg",
                "front.jpg",
                "album.jpg",
            ] {
                let candidate = parent.join(name);
                if let Ok(bytes) = std::fs::read(&candidate) {
                    cover_art = Some(bytes);
                    break;
                }
            }
        }

        let mut tech_specs = Vec::new();
        let format_ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_uppercase())
            .unwrap_or_else(|| "AUDIO".to_string());
        tech_specs.push(format_ext);

        if let Some(symphonia::core::codecs::CodecParameters::Audio(a)) = reader
            .tracks()
            .first()
            .and_then(|t| t.codec_params.as_ref())
        {
            if let Some(sr) = a.sample_rate {
                if sr >= 1000 {
                    let khz = sr as f32 / 1000.0;
                    if (khz.fract()).abs() < 0.05 {
                        tech_specs.push(format!("{khz:.0} kHz"));
                    } else {
                        tech_specs.push(format!("{khz:.1} kHz"));
                    }
                } else {
                    tech_specs.push(format!("{sr} Hz"));
                }
            }
            if let Some(bps) = a.bits_per_sample {
                tech_specs.push(format!("{bps}-bit"));
            }
        }
        let metadata_str = tech_specs.join(" • ");

        Ok(ParsedContent::Audio {
            path: path.to_string_lossy().to_string(),
            title: display_title,
            artist,
            album,
            duration_secs,
            metadata: metadata_str,
            cover_art,
            waveform: Vec::new(),
            waveform_width: 0,
            waveform_height: 0,
        })
    }
}

fn fix_mojikake(s: &str) -> String {
    let trimmed = s.trim_matches('\0').trim();
    if trimmed.is_empty() {
        return String::new();
    }
    // If all chars are within ISO-8859-1 (0..=255),
    // they might be raw UTF-8 bytes that were decoded as Latin-1.
    if trimmed.chars().all(|c| (c as u32) <= 0xFF) {
        let bytes: Vec<u8> = trimmed.chars().map(|c| c as u8).collect();
        if let Ok(utf8_str) = std::str::from_utf8(&bytes)
            && utf8_str != trimmed
        {
            return utf8_str.trim_matches('\0').trim().to_string();
        }
    }
    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_extensions() {
        let parser = AudioParser;
        assert!(parser.supported_extensions().contains(&"mp3"));
        assert!(parser.supported_extensions().contains(&"flac"));
        assert!(parser.supported_extensions().contains(&"wav"));
    }

    #[test]
    fn test_fix_mojikake() {
        assert_eq!(
            fix_mojikake("Thanh Trá»« | Lá»\u{9d}i Viá»\u{87}t | H"),
            "Thanh Trừ | Lời Việt | H"
        );
        assert_eq!(
            fix_mojikake("HÃ\u{a0}n Dung Official ð\u{9f}\u{8e}µ"),
            "Hàn Dung Official 🎵"
        );
        assert_eq!(
            fix_mojikake("Thanh Trừ | Lời Việt | Hàn Dung Cover"),
            "Thanh Trừ | Lời Việt | Hàn Dung Cover"
        );
        assert_eq!(fix_mojikake("Normal Text"), "Normal Text");
        assert_eq!(fix_mojikake(""), "");
    }

    #[test]
    fn test_parse_mp3_with_cover() {
        let parser = AudioParser;
        let test_path =
            Path::new("/home/mintori/Desktop/Youtube/Thanh Trừ ｜ Lời Việt ｜ Hàn Dung Cover.mp3");
        if test_path.exists() {
            let res = parser.parse(test_path).expect("parse failed");
            if let ParsedContent::Audio {
                title,
                artist,
                cover_art,
                duration_secs,
                ..
            } = res
            {
                assert_eq!(title, "Thanh Trừ | Lời Việt | Hàn Dung Cover");
                assert_eq!(artist, "Hàn Dung Official 🎵");
                assert!(cover_art.is_some(), "Expected cover art to be extracted");
                assert_eq!(duration_secs, 178);
            } else {
                panic!("Expected ParsedContent::Audio");
            }
        }
    }
}
