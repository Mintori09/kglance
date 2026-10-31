use unicode_normalization::UnicodeNormalization;

/// Decodes HTML entities (e.g. `&quot;`, `&#x201C;`) without running NFC accent normalization.
pub fn decode_html_entities_only(text: &str) -> String {
    if !text.contains('&') {
        return text.to_string();
    }

    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '&' {
            let mut entity_buf = String::new();
            let mut found_semi = false;

            while let Some(&next_ch) = chars.peek() {
                if next_ch == ';' {
                    chars.next();
                    found_semi = true;
                    break;
                } else if next_ch.is_alphanumeric() || next_ch == '#' {
                    if let Some(ch) = chars.next() {
                        entity_buf.push(ch);
                    }
                    if entity_buf.len() > 16 {
                        break;
                    }
                } else {
                    break;
                }
            }

            if found_semi {
                if let Some(decoded) = decode_named_or_numeric_entity(&entity_buf) {
                    result.push(decoded);
                    continue;
                }
                result.push('&');
                result.push_str(&entity_buf);
                result.push(';');
            } else {
                result.push('&');
                result.push_str(&entity_buf);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

pub fn decode_html_entities(text: &str) -> String {
    let decoded = decode_html_entities_only(text);
    normalize_nfc(&decoded)
}

pub fn is_vietnamese_vowel(c: char) -> bool {
    matches!(
        c,
        'a' | 'ă'
            | 'â'
            | 'e'
            | 'ê'
            | 'i'
            | 'o'
            | 'ô'
            | 'ơ'
            | 'u'
            | 'ư'
            | 'y'
            | 'A'
            | 'Ă'
            | 'Â'
            | 'E'
            | 'Ê'
            | 'I'
            | 'O'
            | 'Ô'
            | 'Ơ'
            | 'U'
            | 'Ư'
            | 'Y'
    )
}

pub fn normalize_nfc(text: &str) -> String {
    if !text.contains('´') && !text.contains('`') && !text.contains('ộ') {
        return text.to_string();
    }
    let preprocessed = text.replace("ộ\u{0301}", "ối").replace("ộ´", "ối");

    let mut transformed = String::with_capacity(preprocessed.len());
    let mut prev_char: Option<char> = None;
    for c in preprocessed.chars() {
        match c {
            '´' if prev_char.is_some_and(is_vietnamese_vowel) => {
                transformed.push('\u{0301}');
            }
            '`' if prev_char.is_some_and(is_vietnamese_vowel) => {
                transformed.push('\u{0300}');
            }
            other => transformed.push(other),
        }
        prev_char = Some(c);
    }

    transformed.nfc().collect()
}

fn decode_named_or_numeric_entity(entity: &str) -> Option<char> {
    if let Some(hex) = entity
        .strip_prefix("#x")
        .or_else(|| entity.strip_prefix("#X"))
    {
        u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
    } else if let Some(dec) = entity.strip_prefix('#') {
        dec.parse::<u32>().ok().and_then(char::from_u32)
    } else {
        match entity {
            "quot" => Some('"'),
            "apos" => Some('\''),
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "nbsp" => Some(' '),
            "copy" => Some('©'),
            "reg" => Some('®'),
            "trade" => Some('™'),
            "hellip" => Some('…'),
            "mdash" => Some('—'),
            "ndash" => Some('–'),
            "ldquo" => Some('“'),
            "rdquo" => Some('”'),
            "lsquo" => Some('‘'),
            "rsquo" => Some('’'),
            "bull" => Some('•'),
            "prime" => Some('′'),
            "Prime" => Some('″'),
            _ => None,
        }
    }
}
