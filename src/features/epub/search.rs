use crate::parsers::markdown::Block;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpubSearchMatch {
    pub chapter_index: usize,
    pub block_index: usize,
    pub snippet: String,
    pub match_char_offset: usize,
}

#[derive(Debug, Clone, Default)]
pub struct EpubSearchState {
    pub query: String,
    pub active_match_index: usize,
    pub matches: Vec<EpubSearchMatch>,
    pub is_searching: bool,
}

impl EpubSearchState {
    pub fn is_empty(&self) -> bool {
        self.query.trim().is_empty()
    }

    pub fn total_matches(&self) -> usize {
        self.matches.len()
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.active_match_index = 0;
        self.matches.clear();
        self.is_searching = false;
    }
}

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

    // Find char index corresponding to byte offset
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
    // Clean up excessive whitespace/newlines in snippet
    let cleaned: String = raw_slice.split_whitespace().collect::<Vec<_>>().join(" ");

    let snippet = format!("{prefix_ellipsis}{cleaned}{suffix_ellipsis}");
    let relative_offset = prefix_ellipsis.len() + (match_start.saturating_sub(start_byte));
    (snippet, relative_offset)
}

pub const MAX_SEARCH_RESULTS: usize = 200;

/// Searches all chapters in parallel (or single thread if small) and produces matching snippets.
pub fn search_epub_chapters(
    chapters: &[crate::core::types::EpubChapterInfo],
    query: &str,
) -> Vec<EpubSearchMatch> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Vec::new();
    }

    let mut all_matches = Vec::new();

    for (ch_idx, chapter) in chapters.iter().enumerate() {
        for (b_idx, block) in chapter.blocks.iter().enumerate() {
            let text = block_to_searchable_text(block);
            if text.is_empty() {
                continue;
            }

            let text_lower = text.to_lowercase();
            let mut search_from = 0;

            while let Some(found_idx) = text_lower[search_from..].find(&q) {
                let abs_match_byte = search_from + found_idx;
                let (snippet, relative_offset) =
                    create_search_snippet(&text, abs_match_byte, q.len());

                all_matches.push(EpubSearchMatch {
                    chapter_index: ch_idx,
                    block_index: b_idx,
                    snippet,
                    match_char_offset: relative_offset,
                });

                if all_matches.len() >= MAX_SEARCH_RESULTS {
                    return all_matches;
                }

                search_from = abs_match_byte + q.len();
                if search_from >= text_lower.len() {
                    break;
                }
            }
        }
    }

    all_matches
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::EpubChapterInfo;
    use crate::parsers::markdown::{Block, Inline};

    #[test]
    fn test_block_to_searchable_text() {
        let block = Block::Paragraph(vec![
            Inline::Text("Hello ".into()),
            Inline::Bold(vec![Inline::Text("World".into())]),
            Inline::Text("!".into()),
        ]);
        assert_eq!(block_to_searchable_text(&block), "Hello World!");

        let heading = Block::Heading {
            level: 2,
            content: vec![Inline::Text("Introduction".into())],
        };
        assert_eq!(block_to_searchable_text(&heading), "Introduction");
    }

    #[test]
    fn test_create_search_snippet() {
        let long_text = "Before context here. This is a crucial keyword inside a long paragraph with after context.";
        let match_start = long_text.find("crucial keyword").unwrap();
        let (snippet, offset) =
            create_search_snippet(long_text, match_start, "crucial keyword".len());

        assert!(snippet.contains("crucial keyword"));
        assert!(offset > 0);
    }

    #[test]
    fn test_search_epub_chapters_finds_matches() {
        let chapters = vec![
            EpubChapterInfo {
                title: "Chapter 1".into(),
                level: 1,
                anchor: None,
                file_href: "ch1.xhtml".into(),
                blocks: vec![
                    Block::Paragraph(vec![Inline::Text("TypeScript is wonderful.".into())]),
                    Block::Paragraph(vec![Inline::Text(
                        "Generics are powerful in TypeScript.".into(),
                    )]),
                ],
            },
            EpubChapterInfo {
                title: "Chapter 2".into(),
                level: 1,
                anchor: None,
                file_href: "ch2.xhtml".into(),
                blocks: vec![Block::Paragraph(vec![Inline::Text(
                    "Advanced type systems.".into(),
                )])],
            },
        ];

        let matches = search_epub_chapters(&chapters, "typescript");
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].chapter_index, 0);
        assert_eq!(matches[0].block_index, 0);
        assert_eq!(matches[1].chapter_index, 0);
        assert_eq!(matches[1].block_index, 1);

        let generic_matches = search_epub_chapters(&chapters, "generics");
        assert_eq!(generic_matches.len(), 1);
        assert_eq!(generic_matches[0].block_index, 1);

        let none_matches = search_epub_chapters(&chapters, "nonexistent");
        assert!(none_matches.is_empty());
    }
}
