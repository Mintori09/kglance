use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use iced::Task;
use iced::futures::SinkExt;
use iced::futures::channel::mpsc::Sender;

use crate::app::Message;
use crate::features::pdf::types::PageData;

pub const PRELOAD_RADIUS: usize = 3;
pub const BATCH_SIZE: usize = 2;
pub const CHANNEL_BUFFER_SIZE: usize = 8;
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

#[allow(clippy::too_many_arguments)]
pub fn lazy_load_pages<F>(
    file_path: String,
    total_pages: usize,
    visible_page: Arc<AtomicUsize>,
    generation_id: Arc<AtomicUsize>,
    disk_cache: Option<Arc<crate::features::pdf::PdfDiskCache>>,
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
            disk_cache,
            make_message,
            done,
        )
    });

    Task::run(stream, |message| message)
}

#[allow(clippy::too_many_arguments)]
pub async fn process_page_loading<F>(
    output: Sender<Message>,
    file_path: String,
    total_pages: usize,
    visible_page: Arc<AtomicUsize>,
    generation_id: Arc<AtomicUsize>,
    disk_cache: Option<Arc<crate::features::pdf::PdfDiskCache>>,
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

        for (idx, is_rendered) in rendered_pages.iter_mut().enumerate() {
            if idx > 0 && !window_range.contains(idx) {
                *is_rendered = false;
            }
        }

        let unrendered_pages =
            find_unrendered_window_pages(&rendered_pages, &window_range, current_page);
        if unrendered_pages.is_empty() {
            tokio::time::sleep(Duration::from_millis(50)).await;
            continue;
        }

        let batch: Vec<usize> = unrendered_pages.into_iter().take(BATCH_SIZE).collect();

        for &page_index in &batch {
            rendered_pages[page_index] = true;
        }

        render_and_send_batch(
            &mut output,
            &pdf_path,
            &batch,
            &make_message,
            disk_cache.clone(),
        )
        .await;

        tokio::time::sleep(LOOP_POLL_INTERVAL).await;
    }

    if generation_id.load(std::sync::atomic::Ordering::Relaxed) == expected_gen {
        let _ = output.send(done).await;
    }
}

fn find_unrendered_window_pages(
    rendered_pages: &[bool],
    window_range: &WindowRange,
    current_page: usize,
) -> Vec<usize> {
    let mut pages: Vec<usize> = (window_range.start..=window_range.end)
        .filter(|&page_index| page_index < rendered_pages.len() && !rendered_pages[page_index])
        .collect();

    pages.sort_by_key(|&page_index| (page_index as isize - current_page as isize).abs());
    pages
}

async fn render_and_send_batch<F>(
    output: &mut Sender<Message>,
    pdf_path: &Path,
    batch: &[usize],
    make_message: &F,
    disk_cache: Option<Arc<crate::features::pdf::PdfDiskCache>>,
) where
    F: Fn(usize, PageData) -> Message + Send + Sync,
{
    let path_buffer = pdf_path.to_owned();
    let batch_vec = batch.to_vec();

    let render_results = tokio::task::spawn_blocking(move || {
        let path = Path::new(&path_buffer);
        let mut results = Vec::with_capacity(batch_vec.len());
        let mut to_render = Vec::new();

        for &page_index in &batch_vec {
            if let Some(ref cache) = disk_cache
                && let Ok(cached) = cache.load_page_with_meta(page_index)
            {
                results.push((
                    page_index,
                    Ok(PageData {
                        width: cached.width,
                        height: cached.height,
                        data: cached.png_bytes,
                    }),
                ));
                continue;
            }
            to_render.push(page_index);
        }

        if !to_render.is_empty() {
            let rendered = crate::features::pdf::parser::render_pdf_pages_batch(path, &to_render);
            for (idx, res) in rendered {
                let compressed = res.map(|p| {
                    let compressed_data = crate::features::pdf::compress::compress_rgba_to_png(
                        &p.data, p.width, p.height,
                    );
                    let data = compressed_data.unwrap_or(p.data);
                    if let Some(ref cache) = disk_cache {
                        let _ = cache.save_page_with_meta(idx, &data, p.width, p.height);
                    }
                    PageData {
                        width: p.width,
                        height: p.height,
                        data,
                    }
                });
                results.push((idx, compressed));
            }
        }
        results
    })
    .await
    .unwrap_or_default();

    for (page_index, task_result) in render_results {
        if let Ok(page_data) = task_result {
            let message = make_message(page_index, page_data);
            let _ = output.send(message).await;
        }
    }
}

#[test]
fn test_window_range_filtering() {
    let range = WindowRange::new(10, 100, 2); // 8..=12
    assert!(range.contains(8));
    assert!(range.contains(12));
    assert!(!range.contains(7));
    assert!(!range.contains(13));
}
