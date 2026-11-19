use crate::features::tts::types::{TtsLanguage, TtsSegment};
use crate::parsers::markdown::{Block, ListItem, flatten_inlines_plain};

/// Detect language of a text segment using Unicode script and Vietnamese diacritic heuristics.
pub fn detect_language(text: &str, doc_lang_hint: Option<TtsLanguage>) -> TtsLanguage {
    let mut ja_chars = 0usize;
    let mut vi_chars = 0usize;
    let mut latin_chars = 0usize;

    for ch in text.chars() {
        if is_japanese_char(ch) {
            ja_chars += 1;
        } else if is_vietnamese_diacritic(ch) {
            vi_chars += 1;
        } else if ch.is_ascii_alphabetic() {
            latin_chars += 1;
        }
    }

    if ja_chars >= 2 || (ja_chars > 0 && ja_chars >= latin_chars) {
        return TtsLanguage::Ja;
    }
    if vi_chars >= 2 || (vi_chars > 0 && vi_chars * 8 >= latin_chars.max(1)) {
        return TtsLanguage::Vi;
    }
    if vi_chars == 0 && ja_chars == 0 {
        return doc_lang_hint.unwrap_or(TtsLanguage::En);
    }
    doc_lang_hint.unwrap_or(TtsLanguage::En)
}

pub fn parse_doc_lang_hint(raw: &str) -> Option<TtsLanguage> {
    let lower = raw.trim().to_ascii_lowercase();
    if lower.starts_with("vi") {
        Some(TtsLanguage::Vi)
    } else if lower.starts_with("ja") || lower.starts_with("jp") {
        Some(TtsLanguage::Ja)
    } else if lower.starts_with("en") {
        Some(TtsLanguage::En)
    } else {
        None
    }
}

fn is_japanese_char(ch: char) -> bool {
    matches!(
        ch as u32,
        0x3040..=0x309F // Hiragana
            | 0x30A0..=0x30FF // Katakana
            | 0x31F0..=0x31FF // Katakana Phonetic Extensions
            | 0x4E00..=0x9FFF // CJK Unified Ideographs
            | 0x3400..=0x4DBF // CJK Extension A
    )
}

fn is_vietnamese_diacritic(ch: char) -> bool {
    matches!(
        ch,
        'ă' | 'Ă'
            | 'â'
            | 'Â'
            | 'đ'
            | 'Đ'
            | 'ê'
            | 'Ê'
            | 'ô'
            | 'Ô'
            | 'ơ'
            | 'Ơ'
            | 'ư'
            | 'Ư'
            | 'à'
            | 'À'
            | 'ả'
            | 'Ả'
            | 'ã'
            | 'Ã'
            | 'á'
            | 'Á'
            | 'ạ'
            | 'Ạ'
            | 'ằ'
            | 'Ằ'
            | 'ẳ'
            | 'Ẳ'
            | 'ẵ'
            | 'Ẵ'
            | 'ắ'
            | 'Ắ'
            | 'ặ'
            | 'Ặ'
            | 'ầ'
            | 'Ầ'
            | 'ẩ'
            | 'Ẩ'
            | 'ẫ'
            | 'Ẫ'
            | 'ấ'
            | 'Ấ'
            | 'ậ'
            | 'Ậ'
            | 'è'
            | 'È'
            | 'ẻ'
            | 'Ẻ'
            | 'ẽ'
            | 'Ẽ'
            | 'é'
            | 'É'
            | 'ẹ'
            | 'Ẹ'
            | 'ề'
            | 'Ề'
            | 'ể'
            | 'Ể'
            | 'ễ'
            | 'Ễ'
            | 'ế'
            | 'Ế'
            | 'ệ'
            | 'Ệ'
            | 'ì'
            | 'Ì'
            | 'ỉ'
            | 'Ỉ'
            | 'ĩ'
            | 'Ĩ'
            | 'í'
            | 'Í'
            | 'ị'
            | 'Ị'
            | 'ò'
            | 'Ò'
            | 'ỏ'
            | 'Ỏ'
            | 'õ'
            | 'Õ'
            | 'ó'
            | 'Ó'
            | 'ọ'
            | 'Ọ'
            | 'ồ'
            | 'Ồ'
            | 'ổ'
            | 'Ổ'
            | 'ỗ'
            | 'Ỗ'
            | 'ố'
            | 'Ố'
            | 'ộ'
            | 'Ộ'
            | 'ờ'
            | 'Ờ'
            | 'ở'
            | 'Ở'
            | 'ỡ'
            | 'Ỡ'
            | 'ớ'
            | 'Ớ'
            | 'ợ'
            | 'Ợ'
            | 'ù'
            | 'Ù'
            | 'ủ'
            | 'Ủ'
            | 'ũ'
            | 'Ũ'
            | 'ú'
            | 'Ú'
            | 'ụ'
            | 'Ụ'
            | 'ừ'
            | 'Ừ'
            | 'ử'
            | 'Ử'
            | 'ữ'
            | 'Ữ'
            | 'ứ'
            | 'Ứ'
            | 'ự'
            | 'Ự'
            | 'ỳ'
            | 'Ỳ'
            | 'ỷ'
            | 'Ỷ'
            | 'ỹ'
            | 'Ỹ'
            | 'ý'
            | 'Ý'
            | 'ỵ'
            | 'Ỵ'
    )
}

/// Infer a document-wide fallback language from all extracted text chunks.
fn infer_document_language(chunks: &[(usize, String)]) -> Option<TtsLanguage> {
    let mut ja_chars = 0usize;
    let mut vi_chars = 0usize;
    let mut latin_chars = 0usize;

    for (_, chunk) in chunks {
        for ch in chunk.chars() {
            if is_japanese_char(ch) {
                ja_chars += 1;
            } else if is_vietnamese_diacritic(ch) {
                vi_chars += 1;
            } else if ch.is_ascii_alphabetic() {
                latin_chars += 1;
            }
        }
    }

    if ja_chars >= 4 || (ja_chars >= 2 && ja_chars >= latin_chars) {
        Some(TtsLanguage::Ja)
    } else if vi_chars >= 4 && vi_chars * 12 >= latin_chars.max(1) {
        Some(TtsLanguage::Vi)
    } else {
        None
    }
}

pub fn extract_segments(
    blocks: &[Block],
    skip_code_blocks: bool,
    doc_lang_hint: Option<TtsLanguage>,
) -> Vec<TtsSegment> {
    let mut raw_chunks = Vec::new();
    for (block_index, block) in blocks.iter().enumerate() {
        let mut buf = String::new();
        collect_block_text(block, skip_code_blocks, &mut buf);
        let cleaned = normalize_whitespace(&buf);
        if cleaned.is_empty() {
            continue;
        }
        // Split very long blocks into ~1500-char sentence chunks so Edge TTS stays well under its 4KB SSML limit
        for chunk in split_long_text(&cleaned, 1500) {
            if chunk.chars().any(char::is_alphanumeric) {
                raw_chunks.push((block_index, chunk));
            }
        }
    }

    let effective_doc_hint = doc_lang_hint.or_else(|| infer_document_language(&raw_chunks));
    raw_chunks
        .into_iter()
        .map(|(block_index, text)| {
            let lang = detect_language(&text, effective_doc_hint);
            TtsSegment {
                block_index,
                text,
                lang,
            }
        })
        .collect()
}

fn collect_block_text(block: &Block, skip_code_blocks: bool, out: &mut String) {
    match block {
        Block::Heading { content, .. } | Block::Paragraph(content) => {
            push_sentence(out, &flatten_inlines_plain(content));
        }
        Block::List { items, .. } => {
            for item in items {
                collect_list_item(item, skip_code_blocks, out);
            }
        }
        Block::Quote(children)
        | Block::Alert {
            content: children, ..
        }
        | Block::FootnoteDefinition {
            content: children, ..
        } => {
            for child in children {
                collect_block_text(child, skip_code_blocks, out);
            }
        }
        Block::Table(table) => {
            let headers: Vec<String> = table
                .headers
                .iter()
                .map(|c| flatten_inlines_plain(&c.content))
                .filter(|s| !s.trim().is_empty())
                .collect();
            if !headers.is_empty() {
                push_sentence(out, &headers.join(", "));
            }
            for row in &table.rows {
                let cells: Vec<String> = row
                    .iter()
                    .map(|c| flatten_inlines_plain(&c.content))
                    .filter(|s| !s.trim().is_empty())
                    .collect();
                if !cells.is_empty() {
                    push_sentence(out, &cells.join(", "));
                }
            }
        }
        Block::Math(expr) => {
            push_sentence(out, expr);
        }
        Block::CodeBlock { code, .. } => {
            if !skip_code_blocks {
                push_sentence(out, code);
            }
        }
        Block::Frontmatter(_)
        | Block::Mermaid { .. }
        | Block::Image { .. }
        | Block::HorizontalRule
        | Block::Html(_) => {}
    }
}

fn collect_list_item(item: &ListItem, skip_code_blocks: bool, out: &mut String) {
    push_sentence(out, &flatten_inlines_plain(&item.content));
    for sub in &item.sub_blocks {
        collect_block_text(sub, skip_code_blocks, out);
    }
}

fn push_sentence(out: &mut String, text: &str) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return;
    }
    if !out.is_empty() {
        if !out.ends_with(['.', '!', '?', '。', '！', '？', ':', ';']) {
            out.push('.');
        }
        out.push(' ');
    }
    out.push_str(trimmed);
}

fn normalize_whitespace(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut prev_space = true;
    for ch in input.chars() {
        let code = ch as u32;
        if ch.is_whitespace() || matches!(code, 0..=8 | 11..=12 | 14..=31) {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    if out.ends_with(' ') {
        out.pop();
    }
    out
}

fn split_long_text(text: &str, max_chars: usize) -> Vec<String> {
    if text.chars().count() <= max_chars {
        return vec![text.to_string()];
    }
    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut current_len = 0usize;

    for word in text.split_whitespace() {
        let word_len = word.chars().count();
        if current_len > 0 && current_len + 1 + word_len > max_chars {
            chunks.push(std::mem::take(&mut current));
            current_len = 0;
        }
        if !current.is_empty() {
            current.push(' ');
            current_len += 1;
        }
        current.push_str(word);
        current_len += word_len;
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

/// Find the first segment index whose block is at or below the current `scroll_y`.
pub fn find_start_segment_from_scroll(
    segments: &[TtsSegment],
    block_y_offsets: &[f32],
    scroll_y: f32,
) -> usize {
    if segments.is_empty() || block_y_offsets.is_empty() || scroll_y <= 8.0 {
        return 0;
    }
    let target_y = scroll_y + 12.0;
    let visible_block = match block_y_offsets.binary_search_by(|probe| {
        probe
            .partial_cmp(&target_y)
            .unwrap_or(std::cmp::Ordering::Equal)
    }) {
        Ok(idx) => idx,
        Err(idx) => idx.saturating_sub(1),
    };
    segments
        .iter()
        .position(|seg| seg.block_index >= visible_block)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::markdown::Inline;

    #[test]
    fn detects_en_vi_ja_scripts() {
        assert_eq!(
            detect_language("Hello world from Rust", None),
            TtsLanguage::En
        );
        assert_eq!(
            detect_language("Xin chào các bạn đang đọc sách", None),
            TtsLanguage::Vi
        );
        assert_eq!(
            detect_language("これは日本語のテストです", None),
            TtsLanguage::Ja
        );
    }

    #[test]
    fn extracts_segments_skips_code_when_enabled() {
        let blocks = vec![
            Block::Heading {
                level: 1,
                content: vec![Inline::Text("Title".to_string())],
            },
            Block::CodeBlock {
                lang: Some("rust".to_string()),
                title: None,
                code: "fn main() {}".to_string(),
            },
            Block::Paragraph(vec![Inline::Text("Nội dung tiếng Việt".to_string())]),
        ];

        let segs = extract_segments(&blocks, true, None);
        assert_eq!(
            segs,
            vec![
                TtsSegment {
                    block_index: 0,
                    text: "Title".to_string(),
                    lang: TtsLanguage::En,
                },
                TtsSegment {
                    block_index: 2,
                    text: "Nội dung tiếng Việt".to_string(),
                    lang: TtsLanguage::Vi,
                },
            ]
        );
    }

    #[test]
    fn finds_start_segment_from_scroll_offset() {
        let segs = vec![
            TtsSegment {
                block_index: 0,
                text: "a".into(),
                lang: TtsLanguage::En,
            },
            TtsSegment {
                block_index: 2,
                text: "b".into(),
                lang: TtsLanguage::En,
            },
            TtsSegment {
                block_index: 5,
                text: "c".into(),
                lang: TtsLanguage::En,
            },
        ];
        let offsets = vec![0.0, 50.0, 120.0, 200.0, 280.0, 350.0];
        assert_eq!(find_start_segment_from_scroll(&segs, &offsets, 0.0), 0);
        assert_eq!(find_start_segment_from_scroll(&segs, &offsets, 140.0), 1);
        assert_eq!(find_start_segment_from_scroll(&segs, &offsets, 360.0), 2);
    }

    #[test]
    fn multiline_vietnamese_document_keeps_vi_for_ascii_headings() {
        let blocks = vec![
            Block::Heading {
                level: 1,
                content: vec![Inline::Text("Giới thiệu".to_string())],
            },
            Block::Paragraph(vec![Inline::Text("Kglance v0.5.3".to_string())]),
            Block::Paragraph(vec![Inline::Text(
                "Đây là trình xem trước tệp nhanh.".to_string(),
            )]),
            Block::Heading {
                level: 2,
                content: vec![Inline::Text("Installation".to_string())],
            },
            Block::Paragraph(vec![Inline::Text(
                "Cài đặt dễ dàng trên KDE Plasma 6.".to_string(),
            )]),
        ];

        let segs = extract_segments(&blocks, true, None);
        assert_eq!(segs.len(), 5);
        assert!(segs.iter().all(|seg| seg.lang == TtsLanguage::Vi));
    }

    #[test]
    fn skips_punctuation_and_symbol_only_blocks() {
        let blocks = vec![
            Block::Paragraph(vec![Inline::Text("---".to_string())]),
            Block::Paragraph(vec![Inline::Text("...".to_string())]),
            Block::Paragraph(vec![Inline::Text("***".to_string())]),
            Block::Paragraph(vec![Inline::Text("Xin chào các bạn.".to_string())]),
        ];

        let segs = extract_segments(&blocks, true, None);
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].text, "Xin chào các bạn.");
        assert_eq!(segs[0].lang, TtsLanguage::Vi);
    }
}
