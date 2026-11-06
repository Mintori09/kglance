use crate::parsers::markdown::{AlertKind, Block, Inline};

pub(crate) fn detect_and_convert_alert(mut blocks: Vec<Block>) -> Option<(AlertKind, Vec<Block>)> {
    if blocks.is_empty() {
        return None;
    }

    if let Block::Paragraph(ref mut inlines) = blocks[0] {
        if inlines.is_empty() {
            return None;
        }

        let first_inline = &inlines[0];
        if let Inline::Text(t) = first_inline {
            let trimmed = t.trim_start();
            let kind = if trimmed.starts_with("[!NOTE]") {
                AlertKind::Note
            } else if trimmed.starts_with("[!TIP]") {
                AlertKind::Tip
            } else if trimmed.starts_with("[!IMPORTANT]") {
                AlertKind::Important
            } else if trimmed.starts_with("[!WARNING]") {
                AlertKind::Warning
            } else if trimmed.starts_with("[!CAUTION]") {
                AlertKind::Caution
            } else {
                return None;
            };

            let prefix_tag = match kind {
                AlertKind::Note => "[!NOTE]",
                AlertKind::Tip => "[!TIP]",
                AlertKind::Important => "[!IMPORTANT]",
                AlertKind::Warning => "[!WARNING]",
                AlertKind::Caution => "[!CAUTION]",
            };

            let idx = t.find(prefix_tag).unwrap_or(0);
            let rem = t[idx + prefix_tag.len()..].trim_start().to_string();

            if rem.is_empty() {
                inlines.remove(0);
                if inlines
                    .first()
                    .is_some_and(|i| matches!(i, Inline::SoftBreak))
                {
                    inlines.remove(0);
                }
            } else {
                inlines[0] = Inline::Text(rem);
            }

            if inlines.is_empty() {
                blocks.remove(0);
            }

            return Some((kind, blocks));
        }
    }

    None
}

pub(crate) fn extract_frontmatter(content: &str) -> (Option<Vec<(String, String)>>, &str) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return (None, content);
    }

    let after_start = &trimmed[3..];
    if !after_start.starts_with('\n') && !after_start.starts_with("\r\n") {
        return (None, content);
    }

    let body_start = if let Some(rest) = after_start.strip_prefix("\r\n") {
        rest
    } else if let Some(rest) = after_start.strip_prefix('\n') {
        rest
    } else {
        return (None, content);
    };

    if let Some(end_pos) = body_start.find("\n---") {
        let frontmatter_text = &body_start[..end_pos];
        let rest_after = &body_start[end_pos + 4..];
        let remaining_markdown = if let Some(rest) = rest_after.strip_prefix("\r\n") {
            rest
        } else if let Some(rest) = rest_after.strip_prefix('\n') {
            rest
        } else {
            rest_after
        };

        let mut entries = Vec::new();
        for line in frontmatter_text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once(':') {
                let key = k.trim().to_string();
                let mut val = v.trim().to_string();
                let is_quoted = (val.starts_with('"') && val.ends_with('"'))
                    || (val.starts_with('\'') && val.ends_with('\''));
                if is_quoted && val.len() >= 2 {
                    val = val[1..val.len() - 1].to_string();
                }
                entries.push((key, val));
            }
        }

        if entries.is_empty() {
            (None, content)
        } else {
            (Some(entries), remaining_markdown)
        }
    } else {
        (None, content)
    }
}
