use crate::ui::types::{ImageState, Message};
use iced::widget::{button, column, container, image, row, scrollable, svg, text};
use iced::{Element, Length};

pub fn view_image<'a>(
    state: &'a ImageState,
    image_bytes: &'a [u8],
    is_svg: bool,
) -> Element<'a, Message> {
    // Toolbar for zoom/rotation
    let toolbar = row![
        button(text("Zoom In"))
            .on_press(Message::ImageZoomIn)
            .style(crate::ui::theme::breeze_button),
        button(text("Zoom Out"))
            .on_press(Message::ImageZoomOut)
            .style(crate::ui::theme::breeze_button),
        button(text("Rotate L"))
            .on_press(Message::ImageRotateLeft)
            .style(crate::ui::theme::breeze_button),
        button(text("Rotate R"))
            .on_press(Message::ImageRotateRight)
            .style(crate::ui::theme::breeze_button),
        button(text("Reset"))
            .on_press(Message::ImageReset)
            .style(crate::ui::theme::breeze_button),
        button(text("Exif Info"))
            .on_press(Message::ToggleExifSidebar)
            .style(crate::ui::theme::breeze_button),
    ]
    .spacing(10)
    .padding(5);

    // Display image or SVG
    let img_element: Element<'a, Message> = if is_svg {
        let handle = svg::Handle::from_memory(image_bytes.to_vec());
        svg(handle).width(Length::Fill).height(Length::Fill).into()
    } else {
        let handle = image::Handle::from_bytes(image_bytes.to_vec());
        image(handle)
            .width(Length::Shrink)
            .height(Length::Shrink)
            .into()
    };

    // Zoom and pan container
    let img_container = container(img_element)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|theme: &iced::Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(palette.background.strong.color.into()),
                ..Default::default()
            }
        });

    let main_view = if state.show_exif {
        row![
            column![toolbar, img_container].width(Length::FillPortion(4)),
            container(scrollable(text(&state.exif_content).size(14)))
                .width(Length::FillPortion(1))
                .padding(10)
                .style(crate::ui::theme::breeze_header_container)
        ]
        .spacing(5)
    } else {
        row![column![toolbar, img_container].width(Length::Fill)]
    };

    main_view.into()
}
