pub mod types;
pub use types::{
    DirState, HistoryState, ImageState, KglanceState, MediaState, Message, PdfState, SortField,
    SortState, TableRowState, TableState, TextState,
};

pub struct PreviewWindow;

impl PreviewWindow {
    pub fn new(_hidden: bool) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self)
    }

    pub fn set_file_selected_handler<F>(&self, _handler: F)
    where
        F: Fn(String) + 'static,
    {
    }

    pub fn show(&self, _path: &str, _content: &crate::parser::ParsedContent) {}
}
