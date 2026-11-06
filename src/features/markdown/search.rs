use crate::parsers::markdown::Block;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMatchItem {
    pub block_index: usize,
    pub snippet: String,
    pub match_char_offset: usize,
}

pub const MAX_SEARCH_RESULTS: usize = 200;

/// Extracts a clean searchable plain-text representation from a Block.
pub fn block_to_searchable_text(block: &Block) -> String {
    match block {
        Block::Heading { content, .. } | Block::Paragraph(content) => {
            crate::parsers::markdown::flatten_inlines_visual(content)
        }
        Block::CodeBlock { code, .. } | Block::Math(code) => code.clone(),
        Block::Quote(inner) => inner
            .iter()
            .map(block_to_searchable_text)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
        Block::List { items, .. } => items
            .iter()
            .flat_map(|item| {
                let own = crate::parsers::markdown::flatten_inlines_visual(&item.content);
                let sub = item
                    .sub_blocks
                    .iter()
                    .map(block_to_searchable_text)
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ");
                vec![own, sub]
            })
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
        Block::Table(tbl) => tbl
            .rows
            .iter()
            .flat_map(|r| r.iter())
            .map(|cell| crate::parsers::markdown::flatten_inlines_visual(&cell.content))
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
        _ => String::new(),
    }
}

/// Creates a 1-2 line context snippet around a match index in text.
pub fn create_search_snippet(text: &str, match_start: usize, match_len: usize) -> (String, usize) {
    const CONTEXT_CHARS: usize = 36;

    let char_indices: Vec<(usize, char)> = text.char_indices().collect();
    if char_indices.is_empty() {
        return (String::new(), 0);
    }

    let match_char_idx = char_indices
        .iter()
        .position(|&(byte_offset, _)| byte_offset >= match_start)
        .unwrap_or(0);

    let start_char_idx = match_char_idx.saturating_sub(CONTEXT_CHARS);
    let end_char_idx = (match_char_idx + match_len + CONTEXT_CHARS).min(char_indices.len());

    let prefix_ellipsis = if start_char_idx > 0 { "..." } else { "" };
    let suffix_ellipsis = if end_char_idx < char_indices.len() {
        "..."
    } else {
        ""
    };

    let start_byte = char_indices[start_char_idx].0;
    let end_byte = if end_char_idx < char_indices.len() {
        char_indices[end_char_idx].0
    } else {
        text.len()
    };

    let raw_slice = &text[start_byte..end_byte];
    let cleaned: String = raw_slice.split_whitespace().collect::<Vec<_>>().join(" ");

    let snippet = format!("{prefix_ellipsis}{cleaned}{suffix_ellipsis}");
    let relative_offset = prefix_ellipsis.len() + (match_start.saturating_sub(start_byte));
    (snippet, relative_offset)
}

/// Searches blocks and produces matching snippets.
pub fn search_blocks(blocks: &[Block], query: &str) -> Vec<SearchMatchItem> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Vec::new();
    }

    let mut matches = Vec::new();

    for (b_idx, block) in blocks.iter().enumerate() {
        let text = block_to_searchable_text(block);
        if text.is_empty() {
            continue;
        }

        let text_lower = text.to_lowercase();
        let mut search_from = 0;

        while let Some(found_idx) = text_lower[search_from..].find(&q) {
            let abs_match_byte = search_from + found_idx;
            let (snippet, relative_offset) = create_search_snippet(&text, abs_match_byte, q.len());

            matches.push(SearchMatchItem {
                block_index: b_idx,
                snippet,
                match_char_offset: relative_offset,
            });

            if matches.len() >= MAX_SEARCH_RESULTS {
                return matches;
            }

            search_from = abs_match_byte + q.len();
            if search_from >= text_lower.len() {
                break;
            }
        }
    }

    matches
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::markdown::{Block, Inline};

    #[test]
    fn test_create_search_snippet() {
        let long_text = "Before keyword something something here is the crucial keyword that we want to match and see context after.";
        let match_start = long_text.find("crucial keyword").unwrap();
        let (snippet, _) = create_search_snippet(long_text, match_start, "crucial keyword".len());
        assert!(snippet.contains("crucial keyword"));
        assert!(snippet.starts_with("..."));
        assert!(snippet.ends_with("..."));
    }

    #[test]
    fn test_search_blocks_case_insensitive_and_types() {
        let blocks = vec![
            Block::Heading {
                level: 1,
                content: vec![Inline::Text("Introduction to Rust".into())],
            },
            Block::Paragraph(vec![Inline::Text(
                "Rust is a systems programming language focused on safety.".into(),
            )]),
            Block::CodeBlock {
                lang: Some("rust".into()),
                title: None,
                code: "fn main() { println!(\"Hello rust\"); }".into(),
            },
        ];

        let results = search_blocks(&blocks, "rust");
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].block_index, 0);
        assert_eq!(results[1].block_index, 1);
        assert_eq!(results[2].block_index, 2);

        // Empty query returns nothing
        let empty_results = search_blocks(&blocks, "   ");
        assert!(empty_results.is_empty());

        // Non-matching query returns nothing
        let none_results = search_blocks(&blocks, "nonexistent_query_string");
        assert!(none_results.is_empty());
    }

    #[test]
    fn test_search_snippet_utf8() {
        let text =
            "Một đoạn văn bản tiếng Việt chứa từ khóa quan trọng cần tìm kiếm trong tài liệu.";
        let match_start = text.find("từ khóa").unwrap();
        let (snippet, _) = create_search_snippet(text, match_start, "từ khóa".len());
        assert!(snippet.contains("từ khóa"));
    }
}
