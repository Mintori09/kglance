use crate::app::Message;
use crate::core::MediaState;
use crate::ui::theme::{glass_button, glass_card, glass_slider};
use iced::widget::{Space, button, column, container, image, mouse_area, row, slider, stack, text};
use iced::{Alignment, Element, Length};

pub fn view_media<'a>(
    state: &'a MediaState,
    data: &'a [u8],
    wf_width: u32,
    wf_height: u32,
) -> Element<'a, Message> {
    let display: Element<'a, Message> = if state.has_video {
        if !state.frame_data.is_empty() {
            let handle = image::Handle::from_rgba(
                state.frame_width,
                state.frame_height,
                state.frame_data.clone(),
            );
            image(handle)
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(iced::ContentFit::Contain)
                .into()
        } else if !data.is_empty() {
            let handle = image::Handle::from_bytes(data.to_vec());
            image(handle)
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(iced::ContentFit::Contain)
                .into()
        } else {
            text(if state.metadata.is_empty() {
                "No preview"
            } else {
                ""
            })
            .size(14)
            .into()
        }
    } else if !data.is_empty() && wf_width > 0 && wf_height > 0 {
        let handle = image::Handle::from_rgba(wf_width, wf_height, data.to_vec());
        image(handle)
            .width(Length::Fill)
            .height(Length::Fixed(150.0))
            .content_fit(iced::ContentFit::Contain)
            .into()
    } else if !data.is_empty() {
        let handle = image::Handle::from_bytes(data.to_vec());
        image(handle)
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(iced::ContentFit::Contain)
            .into()
    } else {
        text(if state.metadata.is_empty() {
            "No preview"
        } else {
            ""
        })
        .size(14)
        .into()
    };

    let play_text = if state.playing {
        "\u{23F8}"
    } else {
        "\u{25B6}"
    };

    let controls = container(
        column![
            slider(0.0..=1.0, state.progress, Message::SeekClicked)
                .step(0.001)
                .style(glass_slider)
                .width(Length::Fill),
            row![
                text(&state.metadata).size(12).width(Length::Fill),
                text(&state.time).size(12),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            row![
                button(text("\u{23EA}"))
                    .on_press(Message::SeekRelativeClicked(-10.0))
                    .style(glass_button)
                    .padding([4, 8]),
                button(text(play_text))
                    .on_press(Message::PlayPauseClicked)
                    .style(glass_button)
                    .padding([4, 12]),
                button(text("\u{23E9}"))
                    .on_press(Message::SeekRelativeClicked(10.0))
                    .style(glass_button)
                    .padding([4, 8]),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(6),
    )
    .style(glass_card)
    .padding([8, 12])
    .width(Length::Fill);

    let bottom_layer = column![Space::new().height(Length::Fill), controls,]
        .width(Length::Fill)
        .height(Length::Fill);

    let mut layers: Vec<Element<'a, Message>> = vec![
        container(display)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into(),
    ];

    if state.show_controls {
        layers.push(bottom_layer.into());
    }

    let content = mouse_area(stack(layers))
        .on_enter(Message::MediaMouseEnter)
        .on_exit(Message::MediaMouseLeave);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(0)
        .into()
}
