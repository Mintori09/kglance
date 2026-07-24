use crate::app::Message;
use crate::core::preview::{ContentType, PreviewContent};
use crate::core::types::KglanceState;
use crate::features::pdf::types::PageData;
use iced::{Element, Task};

pub struct PdfContent {
    pub page_count: u32,
    pub first_page: PageData,
}

impl PreviewContent<Message> for PdfContent {
    fn populate_state(&self, state: &mut KglanceState) {
        state.pdf = crate::core::PdfState::default();
        state.pdf.page_count = self.page_count as usize;
        state.pdf.pages = vec![None; self.page_count as usize];
        state.pdf.thumbnails = vec![None; self.page_count as usize];
        state.file_type_text = "PDF Document".to_string();
    }

    fn view<'a>(&'a self, state: &'a KglanceState) -> Element<'a, Message> {
        crate::features::pdf::view_pdf(&state.pdf)
    }

    fn content_type(&self) -> ContentType {
        ContentType::Pdf
    }

    fn on_loaded(&self, state: &KglanceState, path: &str) -> Task<Message> {
        let page_count = self.page_count as usize;
        if page_count > 1 {
            let pdf_path = path.to_string();
            let visible_page = state.pdf.visible_page.clone();
            Some(crate::features::pdf::lazy_load_pages(
                pdf_path,
                page_count,
                visible_page,
            ))
        } else {
            None
        }
        .unwrap_or(Task::none())
    }
}
