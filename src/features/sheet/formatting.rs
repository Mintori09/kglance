use std::borrow::Cow;

/// Formats numeric strings with thousands commas (e.g., 12500.5 -> 12,500.5, -1200000 -> -1,200,000)
/// while preserving non-numeric strings or short numbers without allocation.
pub fn format_number_with_commas(s: &str) -> Cow<'_, str> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Cow::Borrowed(s);
    }

    let (is_negative, num_str) = if let Some(stripped) = trimmed.strip_prefix('-') {
        (true, stripped)
    } else {
        (false, trimmed)
    };

    let mut parts = num_str.splitn(2, '.');
    let int_part = match parts.next() {
        Some(p) => p,
        None => return Cow::Borrowed(s),
    };

    if int_part.len() <= 3 || !int_part.bytes().all(|b| b.is_ascii_digit()) {
        return Cow::Borrowed(s);
    }

    let frac_part = parts.next();
    if let Some(frac) = frac_part
        && !frac.bytes().all(|b| b.is_ascii_digit())
    {
        return Cow::Borrowed(s);
    }

    // Format with commas
    let num_commas = (int_part.len() - 1) / 3;
    let mut out =
        String::with_capacity(trimmed.len() + num_commas + if is_negative { 1 } else { 0 });
    if is_negative {
        out.push('-');
    }

    let first_chunk_len = int_part.len() % 3;
    let (head, tail) = if first_chunk_len == 0 {
        ("", int_part)
    } else {
        int_part.split_at(first_chunk_len)
    };

    if !head.is_empty() {
        out.push_str(head);
        if !tail.is_empty() {
            out.push(',');
        }
    }

    for (i, chunk) in tail.as_bytes().chunks(3).enumerate() {
        if i > 0 {
            out.push(',');
        }
        if let Ok(c) = std::str::from_utf8(chunk) {
            out.push_str(c);
        }
    }

    if let Some(frac) = frac_part {
        out.push('.');
        out.push_str(frac);
    }

    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number_with_commas() {
        assert_eq!(format_number_with_commas("12500.5"), "12,500.5");
        assert_eq!(format_number_with_commas("35600.75"), "35,600.75");
        assert_eq!(format_number_with_commas("450"), "450");
        assert_eq!(format_number_with_commas("1200"), "1,200");
        assert_eq!(format_number_with_commas("1000000"), "1,000,000");
        assert_eq!(format_number_with_commas("-9876543.21"), "-9,876,543.21");
        assert_eq!(format_number_with_commas("KB-001"), "KB-001");
        assert_eq!(format_number_with_commas("hello"), "hello");
        assert_eq!(format_number_with_commas("12.34"), "12.34");
    }
}
