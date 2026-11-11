use crate::core::types::KglanceState;
use crate::features::markdown::types::TocEntry;
use crate::features::notebook::types::{CellOutput, NotebookCell, NotebookData};
use crate::parsers::markdown::{Block, flatten_inlines_toc};

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;

pub const CONTENT_PADDING: f32 = 24.0;
pub const CELL_SPACING: f32 = 16.0;

pub fn estimate_cell_height(cell: &NotebookCell) -> f32 {
    match cell {
        NotebookCell::Code {
            source, outputs, ..
        } => {
            let src_lines = source.lines().count().max(1) as f32;
            let output_h = outputs.iter().fold(0.0f32, |o_acc, out| match out {
                CellOutput::Text { text, .. } => o_acc + text.lines().count().max(1) as f32 * 20.0,
                CellOutput::Image { .. } => o_acc + 300.0,
            });
            40.0 + src_lines * 22.0 + output_h + 24.0
        }
        NotebookCell::Markdown { source, .. } => {
            let lines = source.lines().count().max(1) as f32;
            lines * 24.0 + 32.0
        }
        NotebookCell::Raw { source } => {
            let lines = source.lines().count().max(1) as f32;
            lines * 22.0 + 24.0
        }
    }
}

pub fn extract_notebook_toc(cells: &[NotebookCell]) -> Vec<TocEntry> {
    let mut toc = Vec::new();
    let mut current_y = CONTENT_PADDING;
    let mut next_block_index = 0;

    for cell in cells {
        let cell_h = estimate_cell_height(cell);

        if let NotebookCell::Markdown { blocks, .. } = cell {
            let mut inner_y = 0.0f32;
            for block in blocks {
                if let Block::Heading { level, content } = block {
                    let text = flatten_inlines_toc(content);
                    toc.push(TocEntry {
                        level: *level,
                        text,
                        block_index: next_block_index,
                        y_offset: current_y + inner_y,
                    });
                }
                inner_y += 32.0;
                next_block_index += 1;
            }
        }

        current_y += cell_h + CELL_SPACING;
    }

    toc
}

pub fn populate_state(state: &mut KglanceState, data: &NotebookData) {
    let mut words = 0;
    let mut chars = 0;

    for cell in &data.cells {
        match cell {
            NotebookCell::Markdown { source, .. }
            | NotebookCell::Code { source, .. }
            | NotebookCell::Raw { source } => {
                words += source.split_whitespace().count();
                chars += source.chars().count();
            }
        }
    }

    let estimated_height = data
        .cells
        .iter()
        .fold(0.0f32, |acc, cell| acc + estimate_cell_height(cell));

    let toc = extract_notebook_toc(&data.cells);

    state.markdown.word_count = words;
    state.markdown.char_count = chars;
    state.markdown.reading_time_mins = (words as f32 / 200.0).ceil() as usize;

    state.markdown.scroll_y = 0.0;
    state.markdown.block_y_offsets.clear();
    state.markdown.block_layouts.clear();
    state.markdown.toc = toc;
    state.markdown.toc_visible = false;
    state.markdown.search_visible = false;
    state.markdown.search_query.clear();
    state.markdown.smooth_scroll.stop(0.0);
    state.markdown.scroll_controller.stop(0.0);
    state.markdown.total_content_height = estimated_height.max(800.0);
    state.markdown.viewport_height = if state.window_height > 0.0 {
        state.window_height
    } else if state.markdown.viewport_height > 0.0 {
        state.markdown.viewport_height
    } else {
        800.0
    };
}
