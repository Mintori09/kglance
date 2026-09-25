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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PagedDocKind {
    Pdf,
    Typst,
}

pub fn prepare_paged_preview_task(
    pdf_state: &mut crate::core::PdfState,
    path: &str,
    page_0_data: Option<(&[u8], u32, u32)>,
    kind: PagedDocKind,
) -> Option<Task<Message>> {
    let page_count = pdf_state.page_count;
    if page_count == 0 {
        return None;
    }

    let session_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as usize)
        .unwrap_or(0);
    let disk_cache = crate::features::pdf::PdfDiskCache::new(session_id)
        .ok()
        .map(Arc::new);
    pdf_state.disk_cache = disk_cache.clone();

    if let Some((data, width, height)) = page_0_data
        && !data.is_empty()
        && !pdf_state.pages.is_empty()
    {
        if let Some(ref dc) = disk_cache {
            let _ = dc.save_page_with_meta(0, data, width, height);
        }
        let handle = iced::widget::image::Handle::from_bytes(data.to_vec());
        pdf_state.pages.insert(
            0,
            crate::core::PageCacheEntry {
                width,
                height,
                handle,
            },
            0,
        );
    }

    let doc_path = path.to_string();
    let visible_thumb_page = pdf_state.visible_thumb_page.clone();
    let thumb_generation_id = pdf_state.thumb_generation_id.clone();
    let thumb_task = if pdf_state.sidebar_visible
        && pdf_state.sidebar_mode == crate::core::types::PdfSidebarMode::Thumbnails
    {
        Some(lazy_load_thumbnails(
            doc_path.clone(),
            page_count,
            visible_thumb_page,
            thumb_generation_id,
        ))
    } else {
        None
    };

    if page_count > 1 {
        pdf_state.active_page_tasks = pdf_state.active_page_tasks.saturating_add(1);
        let visible_page = pdf_state.visible_page.clone();
        let generation_id = pdf_state.generation_id.clone();

        let page_task = match kind {
            PagedDocKind::Pdf => lazy_load_pages(
                doc_path,
                page_count,
                visible_page,
                generation_id,
                disk_cache,
            ),
            PagedDocKind::Typst => crate::features::typst::handler::lazy_load_typst_pages(
                doc_path,
                page_count,
                visible_page,
                generation_id,
                disk_cache,
            ),
        };

        if let Some(tt) = thumb_task {
            Some(Task::batch([page_task, tt]))
        } else {
            Some(page_task)
        }
    } else {
        thumb_task
    }
}
