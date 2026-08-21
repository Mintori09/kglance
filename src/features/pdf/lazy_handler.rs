use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use iced::Task;
use iced::futures::SinkExt;
use iced::futures::channel::mpsc::Sender;
use iced::futures::future::join_all;

use crate::app::Message;
use crate::features::pdf::types::PageData;
use crate::parsers::pdf::render_pdf_page;

pub const PRELOAD_RADIUS: usize = 5;
pub const BATCH_SIZE: usize = 2;
pub const CHANNEL_BUFFER_SIZE: usize = 10;
pub const LOOP_POLL_INTERVAL: Duration = Duration::from_millis(10);

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

pub fn lazy_load_pages<F>(
    file_path: String,
    total_pages: usize,
    visible_page: Arc<AtomicUsize>,
    generation_id: Arc<AtomicUsize>,
    make_message: F,
    done: Message,
) -> Task<Message>
where
    F: Fn(usize, PageData) -> Message + Send + Sync + 'static,
{
    let stream = iced::stream::channel(CHANNEL_BUFFER_SIZE, move |output| {
        process_page_loading(
            output,
            file_path,
            total_pages,
            visible_page,
            generation_id,
            make_message,
            done,
        )
    });

    Task::run(stream, |message| message)
}

pub async fn process_page_loading<F>(
    output: Sender<Message>,
    file_path: String,
    total_pages: usize,
    visible_page: Arc<AtomicUsize>,
    generation_id: Arc<AtomicUsize>,
    make_message: F,
    done: Message,
) where
    F: Fn(usize, PageData) -> Message + Send + Sync,
{
    let mut output = output;
    let pdf_path = PathBuf::from(file_path);

    let mut rendered_pages = vec![false; total_pages];
    if total_pages > 0 {
        rendered_pages[0] = true;
    }

    let expected_gen = generation_id.load(std::sync::atomic::Ordering::Relaxed);

    loop {
        if generation_id.load(std::sync::atomic::Ordering::Relaxed) != expected_gen {
            break;
        }

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

        render_and_send_batch(&mut output, &pdf_path, &batch, &make_message).await;

        tokio::time::sleep(LOOP_POLL_INTERVAL).await;
    }

    if generation_id.load(std::sync::atomic::Ordering::Relaxed) == expected_gen {
        let _ = output.send(done).await;
    }
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

async fn render_and_send_batch<F>(
    output: &mut Sender<Message>,
    pdf_path: &Path,
    batch: &[usize],
    make_message: &F,
) where
    F: Fn(usize, PageData) -> Message + Send + Sync,
{
    let render_tasks: Vec<_> = batch
        .iter()
        .map(|&page_index| {
            let path_buffer = pdf_path.to_owned();
            tokio::task::spawn_blocking(move || {
                let path = Path::new(&path_buffer);
                render_pdf_page(path, page_index as u32).ok()
            })
        })
        .collect();

    let render_results = join_all(render_tasks).await;

    for (&page_index, task_result) in batch.iter().zip(render_results) {
        if let Ok(Some(page_data)) = task_result {
            let message = make_message(page_index, page_data);
            let _ = output.send(message).await;
        }
    }
}
