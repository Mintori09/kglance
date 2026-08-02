use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

use iced::Task;
use iced::futures::SinkExt;
use iced::futures::channel::mpsc::Sender;

use crate::app::Message;
use crate::parsers::typst::compile_typst_to_pdf;

pub fn lazy_load_typst_pages(
    file_path: String,
    total_pages: usize,
    visible_page: Arc<AtomicUsize>,
) -> Task<Message> {
    let stream = iced::stream::channel(
        crate::ui::handlers::lazy_pdf::CHANNEL_BUFFER_SIZE,
        move |output| process_typst_page_loading(output, file_path, total_pages, visible_page),
    );

    Task::run(stream, |message| message)
}

async fn process_typst_page_loading(
    mut output: Sender<Message>,
    file_path: String,
    total_pages: usize,
    visible_page: Arc<AtomicUsize>,
) {
    let compiled = tokio::task::spawn_blocking(move || {
        let path = Path::new(&file_path);
        compile_typst_to_pdf(path).ok()
    })
    .await
    .ok()
    .flatten();

    let Some((temp_pdf, _, _, _)) = compiled else {
        let _ = output
            .send(crate::app::messages::TypstMsg::CompileError.into())
            .await;
        return;
    };

    let pdf_path = temp_pdf.path().to_path_buf();

    crate::ui::handlers::lazy_pdf::process_page_loading(
        output,
        pdf_path.to_string_lossy().into_owned(),
        total_pages,
        visible_page,
        |page_index, page_data| {
            crate::app::messages::TypstMsg::PageReady(
                page_index,
                page_data.data,
                page_data.width,
                page_data.height,
            )
            .into()
        },
        crate::app::messages::TypstMsg::PagesLoaded.into(),
    )
    .await;
}
