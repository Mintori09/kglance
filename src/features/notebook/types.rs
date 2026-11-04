use crate::parsers::markdown::Block;

#[derive(Debug, Clone)]
pub enum NotebookCell {
    Markdown {
        source: String,
        blocks: Vec<Block>,
    },
    Code {
        source: String,
        execution_count: Option<i64>,
        outputs: Vec<CellOutput>,
        language: Option<String>,
    },
    Raw {
        source: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum CellOutput {
    Text {
        text: String,
        is_error: bool,
        truncated: bool,
    },
    Image {
        handle: iced::widget::image::Handle,
        data: Vec<u8>,
        format_hint: String,
    },
}

#[derive(Debug, Clone)]
pub struct NotebookData {
    pub cells: Vec<NotebookCell>,
    pub kernel_language: String,
    pub total_cells: usize,
    pub fallback_notice: Option<String>,
}
