use iced::{Element, Length};
use iced::widget::{column, row, text, scrollable, container, button, image, svg};
use crate::ui::types::{ImageState, Message};

pub fn view_image<'a>(
    state: &'a ImageState,
    image_bytes: &'a [u8],
    is_svg: bool,
) -> Element<'a, Message> {
    // Toolbar for zoom/rotation
    let toolbar = row![
        button(text("Zoom In")).on_press(Message::ImageZoomIn),
        button(text("Zoom Out")).on_press(Message::ImageZoomOut),
        button(text("Rotate L")).on_press(Message::ImageRotateLeft),
        button(text("Rotate R")).on_press(Message::ImageRotateRight),
        button(text("Reset")).on_press(Message::ImageReset),
        button(text("Exif Info")).on_press(Message::ToggleExifSidebar),
    ]
    .spacing(10)
    .padding(5);

    // Display image or SVG
    let img_element: Element<'a, Message> = if is_svg {
        let handle = svg::Handle::from_memory(image_bytes.to_vec());
        svg(handle)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
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
                .style(|theme: &iced::Theme| {
                    let palette = theme.extended_palette();
                    container::Style {
                        background: Some(palette.background.weak.color.into()),
                        ..Default::default()
                    }
                })
        ]
        .spacing(5)
    } else {
        row![column![toolbar, img_container].width(Length::Fill)]
    };

    main_view.into()
}
