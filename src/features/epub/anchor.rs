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

/// Synthetic block counts inserted by [`crate::features::epub::state::build_continuous_blocks`]
/// when a chapter does not begin with an existing heading:
/// - Chapter 0: 1 block (`Heading`)
/// - Subsequent chapters: 2 blocks (`HorizontalRule` + `Heading`) to visually separate chapters.
pub const SYNTHETIC_HEADER_BLOCKS_FIRST_CHAPTER: usize = 1;
pub const SYNTHETIC_HEADER_BLOCKS_SUBSEQUENT_CHAPTER: usize = 2;

/// Counts synthetic heading blocks injected when concatenating chapters in Continuous mode.
pub fn chapter_header_blocks_count(ch: &EpubChapterInfo, ch_idx: usize) -> usize {
    if ch.blocks.is_empty() {
        return 0;
    }
    let first_is_heading = matches!(ch.blocks.first(), Some(Block::Heading { .. }));
    if !first_is_heading && !ch.title.is_empty() {
        if ch_idx > 0 {
            SYNTHETIC_HEADER_BLOCKS_SUBSEQUENT_CHAPTER
        } else {
            SYNTHETIC_HEADER_BLOCKS_FIRST_CHAPTER
        }
    } else {
        0
    }
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

    let header_count = chapters
        .get(first_seen_idx)
        .map_or(0, |ch| chapter_header_blocks_count(ch, first_seen_idx));

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
