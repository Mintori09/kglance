use crate::app::Message;
use crate::core::MediaState;
use crate::ui::theme::{breeze_button, glass_card};
use iced::widget::{button, column, container, image, row, text};
use iced::{Element, Length};

pub fn view_media<'a>(
    state: &'a MediaState,
    data: &'a [u8],
    wf_width: u32,
    wf_height: u32,
) -> Element<'a, Message> {
    let mut main_content = column![].spacing(15).padding(10);

    if !data.is_empty() {
        if wf_width > 0 && wf_height > 0 {
            let handle = image::Handle::from_rgba(wf_width, wf_height, data.to_vec());
            let waveform_image = image(handle)
                .width(Length::Fill)
                .height(Length::Fixed(150.0));
            main_content = main_content.push(waveform_image);
        } else {
            let handle = image::Handle::from_bytes(data.to_vec());
            let img = image(handle).width(Length::Shrink).height(Length::Shrink);
            main_content = main_content.push(img);
        }
    }

    if !state.metadata.is_empty() {
        main_content = main_content.push(text(&state.metadata).size(16));
    }

    let play_button_text = if state.playing { "Pause" } else { "Play" };
    let controls = container(
        row![
            button(text(play_button_text))
                .on_press(Message::PlayPauseClicked)
                .style(breeze_button),
            button(text("Seek -10s"))
                .on_press(Message::SeekRelativeClicked(-10.0))
                .style(breeze_button),
            button(text("Seek +10s"))
                .on_press(Message::SeekRelativeClicked(10.0))
                .style(breeze_button),
            text(format!("Time: {}", state.time)).size(14),
        ]
        .spacing(15)
        .align_y(iced::Alignment::Center),
    )
    .style(glass_card)
    .padding([8, 12]);

    main_content = main_content.push(controls);

    container(main_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
