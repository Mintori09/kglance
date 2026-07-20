use crate::app::Message;
use crate::core::PdfState;
use crate::ui::theme::glass_scrollable;
use iced::widget::{column, container, image, scrollable, text};
use iced::{Element, Length};

pub fn view_pdf<'a>(state: &'a PdfState) -> Element<'a, Message> {
    let mut col = column![].spacing(8).padding(10);

    for page_data in state.pages.iter().flatten() {
        let (data, width, height) = page_data;
        let handle = image::Handle::from_rgba(*width, *height, data.clone());
        let page_img = image(handle).width(Length::Shrink).height(Length::Shrink);
        col = col.push(
            container(page_img)
                .width(Length::Fill)
                .center_x(Length::Fill)
                .padding(4),
        );
    }

    if state.loading {
        col = col.push(
            text("Loading pages...")
                .size(14)
                .width(Length::Fill)
                .center(),
        );
    }

    scrollable(container(col).width(Length::Fill).height(Length::Fill))
        .id("content_scroll")
        .style(glass_scrollable)
        .height(Length::Fill)
        .into()
}
