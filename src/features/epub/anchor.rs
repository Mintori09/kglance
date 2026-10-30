use crate::core::types::{EpubChapterInfo, EpubState};
use crate::parsers::markdown::Block;

/// Reading position anchor that can be translated bidirectionally between
/// SingleChapter and Continuous reading modes without losing the user's reading line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EpubReadingAnchor {
    pub chapter_idx: usize,
    pub local_block_idx: usize,
    pub intra_block_offset: f32,
}

/// Checks whether a synthetic separator (`HorizontalRule`) is needed before `curr_ch`.
/// If the last non-empty block of previous chapter or first block of current chapter is already
/// a `HorizontalRule`, we avoid inserting an extra synthetic separator to prevent double dividers.
pub fn needs_chapter_separator(prev_blocks: &[Block], curr_blocks: &[Block]) -> bool {
    if prev_blocks.is_empty() || curr_blocks.is_empty() {
        return false;
    }
    let prev_ends_with_hr = matches!(prev_blocks.last(), Some(Block::HorizontalRule));
    let curr_starts_with_hr = matches!(curr_blocks.first(), Some(Block::HorizontalRule));
    !prev_ends_with_hr && !curr_starts_with_hr
}

/// Counts synthetic separator blocks (`HorizontalRule`) injected before a chapter in Continuous mode.
pub fn chapter_header_blocks_count(chapters: &[EpubChapterInfo], ch_idx: usize) -> usize {
    if ch_idx == 0 {
        return 0;
    }
    let Some(curr_ch) = chapters.get(ch_idx) else {
        return 0;
    };
    if curr_ch.blocks.is_empty() {
        return 0;
    }

    // Find previous non-empty chapter
    let prev_blocks = chapters[..ch_idx]
        .iter()
        .rev()
        .find(|c| !c.blocks.is_empty())
        .map(|c| c.blocks.as_slice())
        .unwrap_or(&[]);

    if needs_chapter_separator(prev_blocks, &curr_ch.blocks) {
        1
    } else {
        0
    }
}

pub fn parse_link_target(url: &str) -> (Option<&str>, Option<&str>) {
    if let Some(anchor) = url.strip_prefix('#') {
        return (None, Some(anchor));
    }

    if let Some((file, anchor)) = url.split_once('#') {
        let f = if file.is_empty() { None } else { Some(file) };
        let a = if anchor.is_empty() {
            None
        } else {
            Some(anchor)
        };
        (f, a)
    } else if !url.is_empty() {
        (Some(url), None)
    } else {
        (None, None)
    }
}
pub fn find_chapter_index_by_file(
    chapters: &[EpubChapterInfo],
    target_file: &str,
) -> Option<usize> {
    let target_clean = target_file.trim_start_matches('/');
    if target_clean.is_empty() {
        return None;
    }
    let target_name = target_clean.split('/').next_back().unwrap_or(target_clean);

    // 1. Exact match on full file_href
    for (idx, ch) in chapters.iter().enumerate() {
        if ch.file_href == target_clean {
            return Some(idx);
        }
    }

    // 2. Suffix match (e.g. OEBPS/chapter.xhtml vs chapter.xhtml)
    for (idx, ch) in chapters.iter().enumerate() {
        let ch_clean = ch.file_href.trim_start_matches('/');
        if ch_clean.ends_with(target_clean) || target_clean.ends_with(ch_clean) {
            return Some(idx);
        }
    }

    // 3. Filename match
    for (idx, ch) in chapters.iter().enumerate() {
        let ch_name = ch.file_href.split('/').next_back().unwrap_or(&ch.file_href);
        if ch_name == target_name {
            return Some(idx);
        }
    }

    None
}

/// Returns the global block start index and synthetic header count for a given chapter.
pub fn continuous_chapter_start_and_header(
    chapters: &[EpubChapterInfo],
    target_ch: usize,
    chapter_block_offsets: &[usize],
) -> (usize, usize) {
    let start_block_idx = chapter_block_offsets.get(target_ch).copied().unwrap_or(0);
    let first_seen_idx = if let Some(ch) = chapters.get(target_ch)
        && !ch.file_href.is_empty()
    {
        chapters
            .iter()
            .take(target_ch)
            .position(|c| c.file_href == ch.file_href)
            .unwrap_or(target_ch)
    } else {
        target_ch
    };

    let header_count = chapter_header_blocks_count(chapters, first_seen_idx);

    (start_block_idx, header_count)
}

/// Captures the active chapter, local block index, and intra-block pixel offset from Continuous mode.
pub fn capture_continuous_anchor(epub: &EpubState) -> EpubReadingAnchor {
    let scroll_y = epub.markdown_state.scroll_y;
    let offsets = &epub.markdown_state.block_y_offsets;
    let cont_block_idx = if offsets.is_empty() {
        0
    } else {
        offsets
            .partition_point(|&probe| probe <= scroll_y)
            .saturating_sub(1)
    };

    let target_ch = epub
        .chapter_block_offsets
        .iter()
        .rposition(|&offset| offset <= cont_block_idx)
        .unwrap_or(epub.active_chapter)
        .min(epub.chapters.len().saturating_sub(1));

    let (start_block_idx, header_count) =
        continuous_chapter_start_and_header(&epub.chapters, target_ch, &epub.chapter_block_offsets);

    let (local_block_idx, intra_block_offset) = if cont_block_idx < start_block_idx + header_count {
        (0, 0.0)
    } else {
        let local_idx = cont_block_idx - (start_block_idx + header_count);
        let block_top = offsets.get(cont_block_idx).copied().unwrap_or(0.0);
        let intra = (scroll_y - block_top).max(0.0);
        (local_idx, intra)
    };

    EpubReadingAnchor {
        chapter_idx: target_ch,
        local_block_idx,
        intra_block_offset,
    }
}

/// Captures the active chapter, local block index, and intra-block pixel offset from SingleChapter mode.
pub fn capture_single_chapter_anchor(epub: &EpubState) -> EpubReadingAnchor {
    let scroll_y = epub.markdown_state.scroll_y;
    let offsets = &epub.markdown_state.block_y_offsets;
    let local_block_idx = if offsets.is_empty() {
        0
    } else {
        offsets
            .partition_point(|&probe| probe <= scroll_y)
            .saturating_sub(1)
    };
    let block_top = offsets.get(local_block_idx).copied().unwrap_or(0.0);
    let intra_block_offset = (scroll_y - block_top).max(0.0);

    EpubReadingAnchor {
        chapter_idx: epub
            .active_chapter
            .min(epub.chapters.len().saturating_sub(1)),
        local_block_idx,
        intra_block_offset,
    }
}

/// Resolves the target scroll Y in Continuous mode for a given anchor.
pub fn resolve_continuous_anchor_scroll_y(epub: &EpubState, anchor: &EpubReadingAnchor) -> f32 {
    let (start_block_idx, header_count) = continuous_chapter_start_and_header(
        &epub.chapters,
        anchor.chapter_idx,
        &epub.chapter_block_offsets,
    );

    let target_block_idx = if anchor.local_block_idx == 0 && anchor.intra_block_offset == 0.0 {
        start_block_idx
    } else {
        start_block_idx + header_count + anchor.local_block_idx
    };

    let last_idx = epub.markdown_state.block_y_offsets.len().saturating_sub(1);
    let safe_block_idx = target_block_idx.min(last_idx);
    let target_y = epub
        .markdown_state
        .block_y_offsets
        .get(safe_block_idx)
        .copied()
        .unwrap_or(0.0)
        + anchor.intra_block_offset;

    let max_y = crate::core::scroll::max_scroll_y(
        epub.markdown_state.total_content_height,
        epub.markdown_state.viewport_height,
    );
    if max_y > 0.0 {
        target_y.clamp(0.0, max_y)
    } else {
        target_y.max(0.0)
    }
}

/// Resolves the target scroll Y in SingleChapter mode for a given anchor.
pub fn resolve_single_chapter_anchor_scroll_y(epub: &EpubState, anchor: &EpubReadingAnchor) -> f32 {
    let last_idx = epub.markdown_state.block_y_offsets.len().saturating_sub(1);
    let safe_block_idx = anchor.local_block_idx.min(last_idx);
    let target_y = epub
        .markdown_state
        .block_y_offsets
        .get(safe_block_idx)
        .copied()
        .unwrap_or(0.0)
        + anchor.intra_block_offset;

    let max_y = crate::core::scroll::max_scroll_y(
        epub.markdown_state.total_content_height,
        epub.markdown_state.viewport_height,
    );
    if max_y > 0.0 {
        target_y.clamp(0.0, max_y)
    } else {
        target_y.max(0.0)
    }
}
