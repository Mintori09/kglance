use iced::widget::{button, container, row, text};
use iced::{Alignment, Border, Color, Element, Length, Shadow};

use crate::app::Message;
use crate::app::messages::EpubMsg;
use crate::core::types::EpubState;

pub fn render_lightbox<'a>(state: &'a EpubState) -> Element<'a, Message> {
    let Some(lightbox) = &state.lightbox else {
        return container(text("")).width(0).height(0).into();
    };

    let total = lightbox.all_image_paths.len().max(1);
    let current = lightbox.current_index + 1;
    let title_text = format!("Image {current} / {total}");

    // Header bar
    let title_label = text(title_text).size(13).style(|_| text::Style {
        color: Some(Color::from_rgb(0.9, 0.92, 0.95)),
    });

    let filename = std::path::Path::new(&lightbox.active_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&lightbox.active_path);

    let filename_label = text(filename).size(11).style(|_| text::Style {
        color: Some(Color::from_rgba(0.7, 0.75, 0.8, 0.7)),
    });

    let close_btn = button(text("✕").size(14))
        .on_press(EpubMsg::CloseImageLightbox.into())
        .padding([4, 10])
        .style(|_theme: &iced::Theme, status: button::Status| {
            let bg = match status {
                button::Status::Hovered => Some(Color::from_rgba(1.0, 0.2, 0.2, 0.3).into()),
                button::Status::Pressed => Some(Color::from_rgba(1.0, 0.2, 0.2, 0.5).into()),
                _ => Some(Color::from_rgba(1.0, 1.0, 1.0, 0.05).into()),
            };
            button::Style {
                background: bg,
                text_color: Color::from_rgb(0.95, 0.95, 0.95),
                border: Border {
                    radius: 6.0.into(),
                    width: 1.0,
                    color: Color::from_rgba(1.0, 1.0, 1.0, 0.1),
                },
                shadow: Shadow::default(),
                snap: false,
            }
        });

    let top_bar = container(
        row![
            row![title_label, filename_label]
                .spacing(12)
                .align_y(Alignment::Center),
            iced::widget::Space::new().width(Length::Fill),
            close_btn,
        ]
        .align_y(Alignment::Center)
        .padding([8, 16])
        .width(Length::Fill),
    )
    .style(|_| container::Style {
        background: Some(Color::from_rgba(0.05, 0.06, 0.08, 0.85).into()),
        border: Border {
            width: 0.0,
            color: Color::TRANSPARENT,
            radius: 0.0.into(),
        },
        ..Default::default()
    });

    // Image view canvas
    let image_canvas = crate::features::image::view::render_image(&lightbox.image_state);

    // Prev / Next buttons
    let has_multiple = lightbox.all_image_paths.len() > 1;
    let prev_btn: Element<'a, Message> = if has_multiple {
        button(text("‹").size(28))
            .on_press(EpubMsg::PrevLightboxImage.into())
            .padding([8, 12])
            .style(nav_button_style)
            .into()
    } else {
        iced::widget::Space::new().width(0).height(0).into()
    };

    let next_btn: Element<'a, Message> = if has_multiple {
        button(text("›").size(28))
            .on_press(EpubMsg::NextLightboxImage.into())
            .padding([8, 12])
            .style(nav_button_style)
            .into()
    } else {
        iced::widget::Space::new().width(0).height(0).into()
    };

    let main_view = row![
        container(prev_btn)
            .width(Length::Shrink)
            .height(Length::Fill)
            .align_y(Alignment::Center)
            .padding([0, 12]),
        container(image_canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
        container(next_btn)
            .width(Length::Shrink)
            .height(Length::Fill)
            .align_y(Alignment::Center)
            .padding([0, 12]),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .align_y(Alignment::Center);

    let full_content = iced::widget::column![top_bar, main_view]
        .width(Length::Fill)
        .height(Length::Fill);

    container(full_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Color::from_rgba(0.04, 0.05, 0.07, 0.92).into()),
            ..Default::default()
        })
        .into()
}

fn nav_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => Some(Color::from_rgba(1.0, 1.0, 1.0, 0.15).into()),
        button::Status::Pressed => Some(Color::from_rgba(1.0, 1.0, 1.0, 0.25).into()),
        _ => Some(Color::from_rgba(0.1, 0.12, 0.15, 0.6).into()),
    };
    button::Style {
        background: bg,
        text_color: Color::from_rgb(0.95, 0.95, 0.95),
        border: Border {
            radius: 20.0.into(),
            width: 1.0,
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.15),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}
