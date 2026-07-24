use crate::app::Message;
use crate::core::preview::{ContentType, PreviewContent};
use crate::core::types::{KglanceState, SheetInfo};
use crate::features::document::types::SheetData;
use crate::features::document::view::spreadsheet_view;
use iced::Element;

pub struct SpreadsheetContent {
    pub sheets: Vec<SheetData>,
}

impl PreviewContent<Message> for SpreadsheetContent {
    fn populate_state(&self, state: &mut KglanceState) {
        state.spreadsheet.sheets = self
            .sheets
            .iter()
            .map(|s| SheetInfo {
                name: s.name.clone(),
                headers: s.headers.clone(),
                rows: s.rows.clone(),
            })
            .collect();
        state.spreadsheet.active_sheet = 0;
        state.file_type_text = "Spreadsheet".to_string();
    }

    fn view<'a>(&'a self, state: &'a KglanceState) -> Element<'a, Message> {
        spreadsheet_view::view_spreadsheet(&state.spreadsheet)
    }

    fn content_type(&self) -> ContentType {
        ContentType::Spreadsheet
    }
}
