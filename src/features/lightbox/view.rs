use iced::widget::{button, container, row, text};
use iced::{Alignment, Border, Color, Element, Length, Shadow};

use super::types::LightboxState;
use crate::app::Message;
use crate::app::messages::{LightboxMsg, SystemMsg};

pub fn render_lightbox<'a>(lightbox: &'a LightboxState) -> Element<'a, Message> {
    if lightbox.items.is_empty() {
        return container(text("")).width(0).height(0).into();
    }

    let total = lightbox.items.len().max(1);
    let current = lightbox.current_index + 1;
    let item = &lightbox.items[lightbox.current_index.min(total - 1)];

    let index_text = format!("Image {current} / {total}");
    let index_label = text(index_text).size(13).style(|_| text::Style {
        color: Some(Color::from_rgb(0.9, 0.92, 0.95)),
    });

    let subtitle_label = text(&item.subtitle).size(11).style(|_| text::Style {
        color: Some(Color::from_rgba(0.7, 0.75, 0.8, 0.7)),
    });

    let mut info_row = row![index_label].spacing(10).align_y(Alignment::Center);

    if let Some(ref title) = item.title {
        let title_label = text(title).size(12).style(|_| text::Style {
            color: Some(Color::from_rgb(0.85, 0.9, 0.98)),
        });
        info_row = info_row.push(title_label);
    }
    info_row = info_row.push(subtitle_label);

    let link_btn: Option<Element<'a, Message>> = item.link_url.as_ref().map(|url| {
        let url_clone = url.clone();
        button(
            row![text("Open Link").size(11), text("↗").size(11)]
                .spacing(4)
                .align_y(Alignment::Center),
        )
        .on_press(SystemMsg::OpenLink(url_clone).into())
        .padding([4, 8])
        .style(|_theme: &iced::Theme, status: button::Status| {
            let bg = match status {
                button::Status::Hovered => Some(Color::from_rgba(0.4, 0.7, 1.0, 0.3).into()),
                button::Status::Pressed => Some(Color::from_rgba(0.4, 0.7, 1.0, 0.5).into()),
                _ => Some(Color::from_rgba(0.4, 0.7, 1.0, 0.15).into()),
            };
            button::Style {
                background: bg,
                text_color: Color::from_rgb(0.85, 0.92, 1.0),
                border: Border {
                    radius: 4.0.into(),
                    width: 1.0,
                    color: Color::from_rgba(0.4, 0.7, 1.0, 0.3),
                },
                shadow: Shadow::default(),
                snap: false,
            }
        })
        .into()
    });

    let close_btn = button(text("✕").size(14))
        .on_press(LightboxMsg::Close.into())
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

    let mut right_actions = row![].spacing(8).align_y(Alignment::Center);
    if let Some(link) = link_btn {
        right_actions = right_actions.push(link);
    }
    right_actions = right_actions.push(close_btn);

    let top_bar = container(
        row![
            info_row,
            iced::widget::Space::new().width(Length::Fill),
            right_actions,
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
    let has_multiple = lightbox.items.len() > 1;
    let prev_btn: Element<'a, Message> = if has_multiple {
        button(text("‹").size(28))
            .on_press(LightboxMsg::Prev.into())
            .padding([8, 12])
            .style(nav_button_style)
            .into()
    } else {
        iced::widget::Space::new().width(0).height(0).into()
    };

    let next_btn: Element<'a, Message> = if has_multiple {
        button(text("›").size(28))
            .on_press(LightboxMsg::Next.into())
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
            background: Some(Color::from_rgb(0.04, 0.05, 0.07).into()),
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
