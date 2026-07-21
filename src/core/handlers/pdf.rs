use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

use iced::Task;

use crate::app::Message;

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

                let mut unrendered: Vec<usize> =
                    (1..page_count).filter(|&i| !rendered[i]).collect();
                if unrendered.is_empty() {
                    break;
                }

                unrendered.sort_by_key(|&i| (i as isize - curr as isize).abs());

                let to_render: Vec<usize> = unrendered.into_iter().take(4).collect();
                let mut tasks = Vec::new();

                for &i in &to_render {
                    rendered[i] = true;
                    let p = path.clone();
                    tasks.push(tokio::task::spawn_blocking(move || {
                        let p_path = Path::new(&p);
                        crate::parsers::pdf::render_pdf_page(p_path, i as u32).ok()
                    }));
                }

                let results = iced::futures::future::join_all(tasks).await;
                for (idx, res) in to_render.iter().zip(results) {
                    if let Some(d) = res.ok().flatten() {
                        let _ = output
                            .send(Message::PdfPageReady(*idx, d.data, d.width, d.height))
                            .await;
                    }
                }
            }
            let _ = output.send(Message::PdfPagesLoaded(vec![])).await;
        },
    );
    Task::run(stream, |msg| msg)
}
