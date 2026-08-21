use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use iced::Task;
use iced::futures::SinkExt;
use iced::futures::channel::mpsc::Sender;
use iced::futures::future::join_all;

use crate::app::Message;
use crate::features::pdf::lazy_handler::{
    BATCH_SIZE, CHANNEL_BUFFER_SIZE, LOOP_POLL_INTERVAL, PRELOAD_RADIUS, WindowRange,
};

pub fn lazy_load_pages(
    file_path: String,
    total_pages: usize,
    visible_page: Arc<AtomicUsize>,
    generation_id: Arc<AtomicUsize>,
) -> Task<Message> {
    crate::features::pdf::lazy_handler::lazy_load_pages(
        file_path,
        total_pages,
        visible_page,
        generation_id,
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
    visible_page: Arc<AtomicUsize>,
    generation_id: Arc<AtomicUsize>,
) -> Task<Message> {
    let stream = iced::stream::channel(CHANNEL_BUFFER_SIZE, move |output| {
        process_pdf_thumb_loading(output, file_path, total_pages, visible_page, generation_id)
    });

    Task::run(stream, |message| message)
}

async fn process_pdf_thumb_loading(
    mut output: Sender<Message>,
    file_path: String,
    total_pages: usize,
    visible_page: Arc<AtomicUsize>,
    generation_id: Arc<AtomicUsize>,
) {
    let mut rendered_thumbs = vec![false; total_pages];

    let expected_gen = generation_id.load(Ordering::Relaxed);

    loop {
        if generation_id.load(Ordering::Relaxed) != expected_gen {
            return;
        }
        let current_page = visible_page.load(std::sync::atomic::Ordering::Relaxed);
        let window_range = WindowRange::new(current_page, total_pages, PRELOAD_RADIUS);

        let unrendered = find_unrendered_thumbs(&rendered_thumbs);
        if unrendered.is_empty() {
            break;
        }

        let batch = select_prioritized_thumbs(unrendered, current_page, &window_range);

        for &page_index in &batch {
            rendered_thumbs[page_index] = true;
        }

        render_and_send_thumb_batch(&mut output, &file_path, &batch).await;

        if generation_id.load(Ordering::Relaxed) != expected_gen {
            return;
        }

        tokio::time::sleep(LOOP_POLL_INTERVAL).await;
    }
}

fn find_unrendered_thumbs(rendered_thumbs: &[bool]) -> Vec<usize> {
    rendered_thumbs
        .iter()
        .enumerate()
        .filter_map(|(page_index, &is_rendered)| (!is_rendered).then_some(page_index))
        .collect()
}

fn select_prioritized_thumbs(
    mut unrendered: Vec<usize>,
    current_page: usize,
    window_range: &WindowRange,
) -> Vec<usize> {
    unrendered.sort_by_key(|&page_index| {
        let distance = (page_index as isize - current_page as isize).abs();
        let is_out_of_range = !window_range.contains(page_index);
        (is_out_of_range, distance)
    });

    unrendered.into_iter().take(BATCH_SIZE).collect()
}

async fn render_and_send_thumb_batch(
    output: &mut Sender<Message>,
    file_path: &str,
    batch: &[usize],
) {
    let render_tasks: Vec<_> = batch
        .iter()
        .map(|&page_index| {
            let path_buffer = file_path.to_owned();
            tokio::task::spawn_blocking(move || {
                let path = Path::new(&path_buffer);
                if path_buffer.to_lowercase().ends_with(".typ") {
                    if let Ok((temp_pdf, _, _, _)) =
                        crate::parsers::typst::compile_typst_to_pdf(path)
                    {
                        crate::parsers::pdf::render_pdf_page_at_dpi(
                            temp_pdf.path(),
                            page_index as u32,
                            36.0,
                        )
                        .ok()
                    } else {
                        None
                    }
                } else {
                    crate::parsers::pdf::render_pdf_page_at_dpi(path, page_index as u32, 36.0).ok()
                }
            })
        })
        .collect();

    let render_results = join_all(render_tasks).await;

    for (&page_index, task_result) in batch.iter().zip(render_results) {
        if let Ok(Some(page_data)) = task_result {
            let message: Message = crate::app::messages::PdfMsg::ThumbReady(
                page_index,
                page_data.data,
                page_data.width,
                page_data.height,
            )
            .into();
            let _ = output.send(message).await;
        }
    }
}
