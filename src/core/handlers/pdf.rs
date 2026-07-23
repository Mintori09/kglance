use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

use iced::Task;

use crate::app::Message;

const WINDOW_RADIUS: usize = 10;
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

            let mut full_rendered = vec![false; page_count];
            let mut thumb_rendered = vec![false; page_count];
            if page_count > 0 {
                full_rendered[0] = true;
            }

            loop {
                let curr = visible_page.load(std::sync::atomic::Ordering::Relaxed);
                let inner = WindowRange::new(curr, page_count, WINDOW_RADIUS);
                let outer = WindowRange::new(curr, page_count, PRELOAD_RADIUS);

                let mut need_full: Vec<usize> = (inner.start..=inner.end)
                    .filter(|&i| !full_rendered[i])
                    .collect();
                need_full.sort_by_key(|&i| (i as isize - curr as isize).abs());

                let mut need_thumb: Vec<usize> = (outer.start..=outer.end)
                    .filter(|&i| !thumb_rendered[i] && !inner.contains(i))
                    .collect();
                need_thumb.sort_by_key(|&i| (i as isize - curr as isize).abs());

                if need_full.is_empty() && need_thumb.is_empty() {
                    break;
                }

                let to_render_full: Vec<usize> = need_full.into_iter().take(BATCH_SIZE).collect();
                if !to_render_full.is_empty() {
                    let mut tasks = Vec::new();
                    for &i in &to_render_full {
                        full_rendered[i] = true;
                        let p = path.clone();
                        tasks.push(tokio::task::spawn_blocking(move || {
                            let p_path = Path::new(&p);
                            crate::parsers::pdf::render_pdf_page(p_path, i as u32).ok()
                        }));
                    }
                    let results = iced::futures::future::join_all(tasks).await;
                    for (idx, res) in to_render_full.iter().zip(results) {
                        if let Some(d) = res.ok().flatten() {
                            let _ = output
                                .send(Message::PdfPageReady(*idx, d.data, d.width, d.height))
                                .await;
                        }
                    }
                    continue;
                }

                let to_render_thumb: Vec<usize> = need_thumb.into_iter().take(BATCH_SIZE).collect();
                if !to_render_thumb.is_empty() {
                    let mut tasks = Vec::new();
                    for &i in &to_render_thumb {
                        thumb_rendered[i] = true;
                        let p = path.clone();
                        tasks.push(tokio::task::spawn_blocking(move || {
                            let p_path = Path::new(&p);
                            crate::parsers::pdf::render_pdf_page_at_dpi(p_path, i as u32, 36.0).ok()
                        }));
                    }
                    let results = iced::futures::future::join_all(tasks).await;
                    for (idx, res) in to_render_thumb.iter().zip(results) {
                        if let Some(d) = res.ok().flatten() {
                            let _ = output
                                .send(Message::PdfPageReady(*idx, d.data, d.width, d.height))
                                .await;
                        }
                    }
                    continue;
                }

                break;
            }
            let _ = output.send(Message::PdfPagesLoaded(Vec::new())).await;
        },
    );
    Task::run(stream, |msg| msg)
}
