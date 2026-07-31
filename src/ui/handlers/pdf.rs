use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use iced::Task;
use iced::futures::SinkExt;
use iced::futures::channel::mpsc::Sender;
use iced::futures::future::join_all;

use crate::app::Message;
use crate::parsers::pdf::render_pdf_page;

const PRELOAD_RADIUS: usize = 15;
const BATCH_SIZE: usize = 4;
const CHANNEL_BUFFER_SIZE: usize = 10;
const LOOP_POLL_INTERVAL: Duration = Duration::from_millis(10);

pub struct WindowRange {
    pub start: usize,
    pub end: usize,
}

impl WindowRange {
    pub fn new(current_page: usize, total_pages: usize, radius: usize) -> Self {
        if total_pages == 0 {
            return Self { start: 0, end: 0 };
        }

        let start = current_page.saturating_sub(radius);
        let end = (current_page + radius).min(total_pages.saturating_sub(1));

        Self { start, end }
    }

    pub fn contains(&self, page: usize) -> bool {
        (self.start..=self.end).contains(&page)
    }
}

pub fn lazy_load_pages(
    file_path: String,
    total_pages: usize,
    visible_page: Arc<AtomicUsize>,
) -> Task<Message> {
    let stream = iced::stream::channel(CHANNEL_BUFFER_SIZE, move |output| {
        process_pdf_page_loading(output, file_path, total_pages, visible_page)
    });

    Task::run(stream, |message| message)
}

async fn process_pdf_page_loading(
    mut output: Sender<Message>,
    file_path: String,
    total_pages: usize,
    visible_page: Arc<AtomicUsize>,
) {
    let mut rendered_pages = vec![false; total_pages];
    if total_pages > 0 {
        rendered_pages[0] = true;
    }

    loop {
        let current_page = visible_page.load(Ordering::Relaxed);
        let window_range = WindowRange::new(current_page, total_pages, PRELOAD_RADIUS);

        let unrendered_pages = find_unrendered_pages(&rendered_pages);
        if unrendered_pages.is_empty() {
            break;
        }

        let batch = select_prioritized_batch(unrendered_pages, current_page, &window_range);

        for &page_index in &batch {
            rendered_pages[page_index] = true;
        }

        render_and_send_batch(&mut output, &file_path, &batch).await;

        tokio::time::sleep(LOOP_POLL_INTERVAL).await;
    }

    let _ = output.send(Message::PdfPagesLoaded(Vec::new())).await;
}

fn find_unrendered_pages(rendered_pages: &[bool]) -> Vec<usize> {
    rendered_pages
        .iter()
        .enumerate()
        .filter_map(|(page_index, &is_rendered)| (!is_rendered).then_some(page_index))
        .collect()
}

fn select_prioritized_batch(
    mut unrendered_pages: Vec<usize>,
    current_page: usize,
    window_range: &WindowRange,
) -> Vec<usize> {
    unrendered_pages.sort_by_key(|&page_index| {
        let distance = (page_index as isize - current_page as isize).abs();
        let is_out_of_range = !window_range.contains(page_index);
        (is_out_of_range, distance)
    });

    unrendered_pages.into_iter().take(BATCH_SIZE).collect()
}

async fn render_and_send_batch(output: &mut Sender<Message>, file_path: &str, batch: &[usize]) {
    let render_tasks: Vec<_> = batch
        .iter()
        .map(|&page_index| {
            let path_buffer = file_path.to_owned();
            tokio::task::spawn_blocking(move || {
                let path = Path::new(&path_buffer);
                render_pdf_page(path, page_index as u32).ok()
            })
        })
        .collect();

    let render_results = join_all(render_tasks).await;

    for (&page_index, task_result) in batch.iter().zip(render_results) {
        if let Ok(Some(page_data)) = task_result {
            let message = Message::PdfPageReady(
                page_index,
                page_data.data,
                page_data.width,
                page_data.height,
            );
            let _ = output.send(message).await;
        }
    }
}
