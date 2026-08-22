use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

use iced::Task;

use crate::app::Message;

pub fn lazy_load_pages(
    file_path: String,
    total_pages: usize,
    visible_page: Arc<AtomicUsize>,
    generation_id: Arc<AtomicUsize>,
    disk_cache: Option<Arc<crate::features::pdf::PdfDiskCache>>,
) -> Task<Message> {
    crate::features::pdf::lazy_handler::lazy_load_pages(
        file_path,
        total_pages,
        visible_page,
        generation_id,
        disk_cache,
        |page_index, page_data| {
            crate::app::messages::PdfMsg::PageReady(
                page_index,
                page_data.data,
                page_data.width,
                page_data.height,
            )
            .into()
        },
        crate::app::messages::PdfMsg::PagesLoaded(Vec::new()).into(),
    )
}

pub fn lazy_load_thumbnails(
    file_path: String,
    total_pages: usize,
    visible_thumb_page: Arc<AtomicUsize>,
    generation_id: Arc<AtomicUsize>,
) -> Task<Message> {
    crate::features::pdf::lazy_handler::lazy_load_thumbnails(
        file_path,
        total_pages,
        visible_thumb_page,
        generation_id,
        |page_index, page_data| {
            crate::app::messages::PdfMsg::ThumbReady(
                page_index,
                page_data.data,
                page_data.width,
                page_data.height,
            )
            .into()
        },
        crate::app::messages::PdfMsg::PagesLoaded(Vec::new()).into(),
    )
}
