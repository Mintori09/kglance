use std::borrow::Cow;

/// Collapses multi-line fenced code blocks inside GFM table cells into single-line
/// `<br>`-separated inline code spans so `pulldown_cmark` parses the row intact.
///
/// Returns `Cow::Borrowed(content)` without allocation when no table contains an
/// unclosed multi-line code fence.
pub(super) fn preprocess_multiline_table_fences(content: &str) -> Cow<'_, str> {
    if !content.contains('|') || (!content.contains("```") && !content.contains("~~~")) {
        return Cow::Borrowed(content);
    }

    let lines: Vec<&str> = content.split_inclusive('\n').collect();
    let mut out: Option<String> = None;
    let mut top_fence: Option<(u8, usize)> = None;
    let mut in_table = false;
    let mut i = 0;

    while i < lines.len() {
        let raw_line = lines[i];
        let trimmed_line = strip_line_ending(raw_line);

        if let Some((fence_byte, fence_len)) = top_fence {
            if let Some(buf) = out.as_mut() {
                buf.push_str(raw_line);
            }
            if is_closing_top_fence(trimmed_line, fence_byte, fence_len) {
                top_fence = None;
            }
            i += 1;
            continue;
        }

        if !in_table {
            if let Some(opener) = parse_top_fence_opener(trimmed_line) {
                top_fence = Some(opener);
                if let Some(buf) = out.as_mut() {
                    buf.push_str(raw_line);
                }
                i += 1;
                continue;
            }

            if i + 1 < lines.len()
                && trimmed_line.contains('|')
                && !trimmed_line.trim().is_empty()
                && is_table_delimiter_row(strip_line_ending(lines[i + 1]))
            {
                if let Some(buf) = out.as_mut() {
                    buf.push_str(raw_line);
                    buf.push_str(lines[i + 1]);
                }
                in_table = true;
                i += 2;
                continue;
            }

            if let Some(buf) = out.as_mut() {
                buf.push_str(raw_line);
            }
            i += 1;
            continue;
        }

        // Inside a GFM table
        if trimmed_line.trim().is_empty() || !trimmed_line.contains('|') {
            in_table = false;
            if let Some(opener) = parse_top_fence_opener(trimmed_line) {
                top_fence = Some(opener);
            }
            if let Some(buf) = out.as_mut() {
                buf.push_str(raw_line);
            }
            i += 1;
            continue;
        }

        if let Some((rewritten, next_idx)) = try_collapse_table_row_fences(&lines, i) {
            let buf = out.get_or_insert_with(|| {
                let prefix_len: usize = lines[..i].iter().map(|s| s.len()).sum();
                let mut s = String::with_capacity(content.len());
                s.push_str(&content[..prefix_len]);
                s
            });
            buf.push_str(&rewritten);
            i = next_idx;
        } else {
            if let Some(buf) = out.as_mut() {
                buf.push_str(raw_line);
            }
            i += 1;
        }
    }

    match out {
        Some(s) => Cow::Owned(s),
        None => Cow::Borrowed(content),
    }
}

fn strip_line_ending(line: &str) -> &str {
    let without_lf = line.strip_suffix('\n').unwrap_or(line);
    without_lf.strip_suffix('\r').unwrap_or(without_lf)
}

fn line_ending_of(line: &str) -> &str {
    if line.ends_with("\r\n") {
        "\r\n"
    } else if line.ends_with('\n') {
        "\n"
    } else {
        ""
    }
}

fn leading_spaces_up_to_3(s: &str) -> Option<&str> {
    let bytes = s.as_bytes();
    let mut spaces = 0;
    while spaces < bytes.len() && bytes[spaces] == b' ' {
        spaces += 1;
        if spaces > 3 {
            return None;
        }
    }
    Some(&s[..spaces])
}

fn parse_top_fence_opener(line: &str) -> Option<(u8, usize)> {
    let rest = &line[leading_spaces_up_to_3(line)?.len()..];
    let first = *rest.as_bytes().first()?;
    if first != b'`' && first != b'~' {
        return None;
    }
    let fence_len = rest.bytes().take_while(|&b| b == first).count();
    if fence_len < 3 {
        return None;
    }
    let info = &rest[fence_len..];
    if first == b'`' && info.contains('`') {
        return None;
    }
    Some((first, fence_len))
}

fn is_closing_top_fence(line: &str, fence_byte: u8, min_len: usize) -> bool {
    let Some(prefix) = leading_spaces_up_to_3(line) else {
        return false;
    };
    let rest = &line[prefix.len()..];
    let fence_len = rest.bytes().take_while(|&b| b == fence_byte).count();
    fence_len >= min_len && rest[fence_len..].trim().is_empty()
}

fn is_table_delimiter_row(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.contains('|') {
        return false;
    }
    let inner = trimmed.strip_prefix('|').unwrap_or(trimmed);
    let inner = inner.strip_suffix('|').unwrap_or(inner);
    let mut cell_count = 0;
    for cell in inner.split('|') {
        let c = cell.trim();
        if c.is_empty() {
            return false;
        }
        let c = c.strip_prefix(':').unwrap_or(c);
        let c = c.strip_suffix(':').unwrap_or(c);
        if c.is_empty() || !c.bytes().all(|b| b == b'-') {
            return false;
        }
        cell_count += 1;
    }
    cell_count > 0
}

#[derive(Debug, Clone, Copy)]
struct FenceOpener {
    start_byte: usize,
    fence_byte: u8,
    fence_len: usize,
}

/// Finds the first unclosed fenced code opener (` ``` ` or `~~~`) on a table row line,
/// skipping escaped characters and balanced inline code spans.
fn find_unclosed_fence_opener(line: &str, start_from: usize) -> Option<FenceOpener> {
    let bytes = line.as_bytes();
    let mut i = start_from;

    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\\' {
            i += 2;
            continue;
        }

        if b == b'`' || b == b'~' {
            let run_len = bytes[i..].iter().take_while(|&&c| c == b).count();
            if let Some(close_end) = find_closing_inline_run(bytes, i + run_len, b, run_len) {
                i = close_end;
                continue;
            }
            if run_len >= 3 {
                return Some(FenceOpener {
                    start_byte: i,
                    fence_byte: b,
                    fence_len: run_len,
                });
            }
            i += run_len;
            continue;
        }

        i += 1;
    }

    None
}

fn find_closing_inline_run(
    bytes: &[u8],
    mut pos: usize,
    delimiter: u8,
    target_len: usize,
) -> Option<usize> {
    while pos < bytes.len() {
        if bytes[pos] == delimiter {
            let run_len = bytes[pos..].iter().take_while(|&&c| c == delimiter).count();
            if run_len == target_len {
                return Some(pos + run_len);
            }
            pos += run_len;
        } else {
            pos += 1;
        }
    }
    None
}

/// Looks for a closing fence matching `(fence_byte, min_len)` at the start of `line`
/// (after optional leading whitespace), returning the suffix after the closing fence.
fn find_closing_fence_in_row(line: &str, fence_byte: u8, min_len: usize) -> Option<&str> {
    let trimmed_start = line.trim_start_matches([' ', '\t']);
    let fence_len = trimmed_start
        .bytes()
        .take_while(|&b| b == fence_byte)
        .count();
    if fence_len < min_len {
        return None;
    }
    let after = &trimmed_start[fence_len..];
    // Closing fence in a table cell may be followed by EOL, whitespace, `|`, or inline HTML like `<br>`.
    match after.as_bytes().first() {
        None | Some(b' ' | b'\t' | b'|' | b'<') => Some(after),
        Some(_) => None,
    }
}

fn format_code_lines_for_table_cell(code_lines: &[&str]) -> String {
    let mut out = String::new();
    for (idx, &raw) in code_lines.iter().enumerate() {
        if idx > 0 {
            out.push_str("<br>");
        }
        if raw.is_empty() {
            continue;
        }
        let escaped = if raw.contains('|') {
            Cow::Owned(raw.replace('|', "\\|"))
        } else {
            Cow::Borrowed(raw)
        };
        let max_bt = max_consecutive_backticks(&escaped);
        let delim = "`".repeat(max_bt + 1);
        out.push_str(&delim);
        if escaped.starts_with('`') || escaped.ends_with('`') {
            out.push(' ');
            out.push_str(&escaped);
            out.push(' ');
        } else {
            out.push_str(&escaped);
        }
        out.push_str(&delim);
    }
    out
}

fn max_consecutive_backticks(s: &str) -> usize {
    let mut max_run = 0;
    let mut current = 0;
    for b in s.bytes() {
        if b == b'`' {
            current += 1;
            max_run = max_run.max(current);
        } else {
            current = 0;
        }
    }
    max_run
}

fn try_collapse_table_row_fences(lines: &[&str], start_idx: usize) -> Option<(String, usize)> {
    let first_trimmed = strip_line_ending(lines[start_idx]);
    let mut opener = find_unclosed_fence_opener(first_trimmed, 0)?;

    let mut row = String::new();
    row.push_str(&first_trimmed[..opener.start_byte]);
    let mut scan_from = row.len();
    let mut idx = start_idx + 1;
    let mut last_ending = line_ending_of(lines[start_idx]);

    loop {
        let mut code_lines = Vec::new();
        let mut closed = false;

        while idx < lines.len() {
            let candidate = strip_line_ending(lines[idx]);
            if let Some(after_fence) =
                find_closing_fence_in_row(candidate, opener.fence_byte, opener.fence_len)
            {
                let formatted = format_code_lines_for_table_cell(&code_lines);
                row.push_str(&formatted);
                scan_from = row.len();
                row.push_str(after_fence);
                last_ending = line_ending_of(lines[idx]);
                idx += 1;
                closed = true;
                break;
            }
            code_lines.push(candidate);
            idx += 1;
        }

        if !closed {
            return None;
        }

        if let Some(next_opener) = find_unclosed_fence_opener(&row, scan_from) {
            row.truncate(next_opener.start_byte);
            opener = next_opener;
        } else {
            break;
        }
    }

    row.push_str(last_ending);
    Some((row, idx))
}
