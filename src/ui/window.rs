use iced::{
    Alignment, Element, Length, Padding,
    widget::{Stack, button, column, container, row, text},
};

use crate::app::Message;
use crate::core::KglanceState;

fn metadata_text(state: &KglanceState) -> String {
    let mut parts = Vec::new();

    if !state.file_type_text.is_empty() {
        parts.push(state.file_type_text.as_str());
    }

    if !state.file_size_text.is_empty() {
        parts.push(state.file_size_text.as_str());
    }

    if !state.file_modified_text.is_empty() {
        parts.push(state.file_modified_text.as_str());
    }

    parts.join(" • ")
}

fn file_info<'a>(state: &'a KglanceState) -> Element<'a, Message> {
    let metadata = metadata_text(state);

    column![
        text(&state.file_name).size(15).line_height(1.1),
        text(metadata).size(11).style(|theme: &iced::Theme| {
            let palette = theme.extended_palette();

            iced::widget::text::Style {
                color: Some(palette.background.weak.text),
            }
        }),
    ]
    .spacing(2)
    .width(Length::Fill)
    .into()
}

use crate::ui::components::button as component_button;

pub fn header_actions<'a>() -> Element<'a, Message> {
    row![
        button("Copy")
            .on_press(Message::CopyPathClicked)
            .padding([6, 12])
            .style(component_button::glass_button),
        button("Open")
            .on_press(Message::OpenClicked)
            .padding([6, 12])
            .style(component_button::glass_button),
        button("×")
            .on_press(Message::CloseRequested)
            .padding([6, 10])
            .style(component_button::close_button),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn header<'a>(state: &'a KglanceState) -> Element<'a, Message> {
    container(
        row![file_info(state), header_actions(),]
            .align_y(Alignment::Center)
            .padding([8, 12]),
    )
    .width(Length::Fill)
    .style(crate::ui::theme::breeze_header_container)
    .into()
}

fn content<'a>(preview_body: Element<'a, Message>, edge_to_edge: bool) -> Element<'a, Message> {
    let padding = if edge_to_edge { 0 } else { 10 };
    container(preview_body)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(padding)
        .into()
}

fn toasts<'a>(state: &'a KglanceState) -> Element<'a, Message> {
    if state.toasts.is_empty() {
        return Element::from(container(text("")).padding(0));
    }

    let items: Vec<Element<'a, Message>> = state
        .toasts
        .iter()
        .map(|t| {
            container(text(&t.message).size(13))
                .padding(Padding {
                    top: 6.0,
                    right: 16.0,
                    bottom: 6.0,
                    left: 16.0,
                })
                .style(|theme: &iced::Theme| {
                    use iced::widget::container;
                    let palette = theme.extended_palette();
                    container::Style {
                        background: Some(palette.background.base.color.into()),
                        text_color: Some(palette.background.base.text),
                        border: iced::Border {
                            radius: 6.0.into(),
                            width: 0.0,
                            color: iced::Color::TRANSPARENT,
                        },
                        shadow: iced::Shadow {
                            offset: iced::Vector::new(0.0, 2.0),
                            blur_radius: 8.0,
                            color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                        },
                        ..Default::default()
                    }
                })
                .into()
        })
        .collect();

    column(items)
        .spacing(6)
        .padding(Padding {
            top: 0.0,
            right: 0.0,
            bottom: 24.0,
            left: 0.0,
        })
        .width(Length::Shrink)
        .into()
}

pub fn view_window<'a>(
    state: &'a KglanceState,
    preview_body: Element<'a, Message>,
    edge_to_edge: bool,
) -> Element<'a, Message> {
    let layout = column![header(state), content(preview_body, edge_to_edge),]
        .width(Length::Fill)
        .height(Length::Fill);

    let base = container(layout)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(crate::ui::theme::breeze_container);

    let toast_layer = container(toasts(state))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(Alignment::End)
        .align_x(iced::alignment::Horizontal::Center);

    Stack::new().push(base).push(toast_layer).into()
}
