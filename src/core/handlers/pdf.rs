use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

use iced::Task;

use crate::app::Message;

const PRELOAD_RADIUS: usize = 15;
const BATCH_SIZE: usize = 4;

pub struct WindowRange {
    pub start: usize,
    pub end: usize,
}

impl WindowRange {
    pub fn new(current: usize, page_count: usize, radius: usize) -> Self {
        if page_count == 0 {
            return Self { start: 0, end: 0 };
        }
        let start = current.saturating_sub(radius);
        let end = (current + radius).min(page_count.saturating_sub(1));
        Self { start, end }
    }

    pub fn contains(&self, page: usize) -> bool {
        page >= self.start && page <= self.end
    }
}

pub fn lazy_load_pages(
    path: String,
    page_count: usize,
    visible_page: Arc<AtomicUsize>,
) -> Task<Message> {
    let stream = iced::stream::channel(
        10,
        move |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
            use iced::futures::SinkExt;

            let mut rendered = vec![false; page_count];
            if page_count > 0 {
                rendered[0] = true;
            }

            loop {
                let curr = visible_page.load(std::sync::atomic::Ordering::Relaxed);
                let range = WindowRange::new(curr, page_count, PRELOAD_RADIUS);

                let mut need_render: Vec<usize> =
                    (0..page_count).filter(|&i| !rendered[i]).collect();

                if need_render.is_empty() {
                    break;
                }

                // Prioritize visible/nearby window range first
                need_render.sort_by_key(|&i| {
                    let distance = (i as isize - curr as isize).abs();
                    let in_range = range.contains(i);
                    (!in_range, distance)
                });

                let batch: Vec<usize> = need_render.into_iter().take(BATCH_SIZE).collect();
                let mut tasks = Vec::new();
                for &i in &batch {
                    rendered[i] = true;
                    let p = path.clone();
                    tasks.push(tokio::task::spawn_blocking(move || {
                        let p_path = Path::new(&p);
                        crate::parsers::pdf::render_pdf_page(p_path, i as u32).ok()
                    }));
                }

                let results = iced::futures::future::join_all(tasks).await;
                for (idx, res) in batch.iter().zip(results) {
                    if let Some(d) = res.ok().flatten() {
                        let _ = output
                            .send(Message::PdfPageReady(*idx, d.data, d.width, d.height))
                            .await;
                    }
                }

                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
            let _ = output.send(Message::PdfPagesLoaded(Vec::new())).await;
        },
    );
    Task::run(stream, |msg| msg)
}
