use iced::{Element, Length};
use iced::widget::{column, row, text, scrollable, container, button, image};
use crate::ui::types::{PdfState, Message};

pub fn view_pdf<'a>(
    state: &'a PdfState,
    page_rgba: &'a [u8],
    width: u32,
    height: u32,
) -> Element<'a, Message> {
    // Navigation bar for PDF pages
    let nav_bar = row![
        button(text("Previous Page")).on_press(Message::PrevPageClicked),
        text(format!(" Page {} of {} ", state.current_page + 1, state.page_count)).size(16),
        button(text("Next Page")).on_press(Message::NextPageClicked),
    ]
    .spacing(15)
    .padding(10)
    .align_y(iced::Alignment::Center);

    // Render the page image
    let handle = image::Handle::from_rgba(width, height, page_rgba.to_vec());
    let page_image = image(handle)
        .width(Length::Shrink)
        .height(Length::Shrink);

    let main_layout = column![
        nav_bar,
        scrollable(
            container(page_image)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
        )
        .height(Length::Fill)
    ]
    .spacing(5);

    main_layout.into()
}
