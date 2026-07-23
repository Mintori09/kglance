use crate::app::Message;
use crate::core::PdfState;
use crate::ui::theme::glass_scrollable;
use iced::widget::{column, container, image, scrollable, text};
use iced::{Element, Length};

const PAGE_GAP: f32 = 8.0;
const PLACEHOLDER_HEIGHT: f32 = 200.0;

pub fn view_pdf<'a>(state: &'a PdfState) -> Element<'a, Message> {
    let mut col = column![].spacing(PAGE_GAP).padding(10);

    if state.page_count == 0 {
        return scrollable(container(text("No pages").size(14)))
            .height(Length::Fill)
            .into();
    }

    for i in 0..state.page_count {
        if let Some(entry) = state.pages[i].as_ref() {
            let page_img = image(entry.handle.clone())
                .width(Length::Shrink)
                .height(Length::Shrink);
            col = col.push(
                container(page_img)
                    .width(Length::Fill)
                    .center_x(Length::Fill)
                    .padding(4),
            );
        } else if let Some(thumb) = state.thumbnails[i].as_ref() {
            let thumb_img = image(thumb.handle.clone())
                .width(Length::Shrink)
                .height(Length::Shrink);
            col = col.push(
                container(thumb_img)
                    .width(Length::Fill)
                    .center_x(Length::Fill)
                    .padding(4),
            );
        } else {
            col = col.push(
                container(text(format!("Page {}…", i + 1)).size(13).center())
                    .width(Length::Fill)
                    .height(Length::Fixed(PLACEHOLDER_HEIGHT))
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
