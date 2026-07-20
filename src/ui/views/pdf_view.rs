use crate::app::Message;
use crate::core::PdfState;
use crate::ui::theme::glass_scrollable;
use iced::widget::{column, container, image, scrollable, text};
use iced::{Element, Length};

pub fn view_pdf<'a>(state: &'a PdfState) -> Element<'a, Message> {
    let mut col = column![].spacing(8).padding(10);

    for (i, handle_opt) in state.cached_handles.iter().enumerate() {
        if let Some(handle) = handle_opt {
            // Use pre-built handle — no clone of raw bytes, no rebuild
            let page_img = image(handle.clone())
                .width(Length::Shrink)
                .height(Length::Shrink);
            col = col.push(
                container(page_img)
                    .width(Length::Fill)
                    .center_x(Length::Fill)
                    .padding(4),
            );
        } else {
            // Placeholder for pages still loading
            let label = format!("Page {} loading…", i + 1);
            col = col.push(
                container(text(label).size(13).width(Length::Fill).center())
                    .width(Length::Fill)
                    .padding(8),
            );
        }
    }

    if state.loading {
        col = col.push(text("Loading…").size(14).width(Length::Fill).center());
    }

    scrollable(container(col).width(Length::Fill).height(Length::Fill))
        .id("content_scroll")
        .style(glass_scrollable)
        .on_scroll(Message::PdfScrolled)
        .height(Length::Fill)
        .into()
}
