use crate::app::Message;
use crate::core::PdfState;
use crate::ui::theme::{breeze_button, glass_scrollable};
use iced::widget::{button, column, container, image, row, scrollable, text};
use iced::{Element, Length};

pub fn view_pdf<'a>(
    state: &'a PdfState,
    page_rgba: &'a [u8],
    width: u32,
    height: u32,
) -> Element<'a, Message> {
    let nav_bar = row![
        button(text("Previous Page"))
            .on_press(Message::PrevPageClicked)
            .style(breeze_button),
        text(format!(
            " Page {} of {} ",
            state.current_page + 1,
            state.page_count
        ))
        .size(16),
        button(text("Next Page"))
            .on_press(Message::NextPageClicked)
            .style(breeze_button),
    ]
    .spacing(15)
    .padding(10)
    .align_y(iced::Alignment::Center);

    let handle = image::Handle::from_rgba(width, height, page_rgba.to_vec());
    let page_image = image(handle).width(Length::Shrink).height(Length::Shrink);

    let main_layout = column![
        nav_bar,
        scrollable(
            container(page_image)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
        )
        .style(glass_scrollable)
        .height(Length::Fill)
    ]
    .spacing(5);

    main_layout.into()
}
