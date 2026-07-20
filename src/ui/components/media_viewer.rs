use iced::{Element, Length};
use iced::widget::{column, row, text, container, button, image};
use crate::ui::types::{MediaState, Message};

pub fn view_media<'a>(
    state: &'a MediaState,
    waveform_rgba: &'a [u8],
    wf_width: u32,
    wf_height: u32,
) -> Element<'a, Message> {
    let mut main_content = column![].spacing(15).padding(10);

    // Audio Waveform if available
    if wf_width > 0 && wf_height > 0 && !waveform_rgba.is_empty() {
        let handle = image::Handle::from_rgba(wf_width, wf_height, waveform_rgba.to_vec());
        let waveform_image = image(handle)
            .width(Length::Fill)
            .height(Length::Fixed(150.0));
        main_content = main_content.push(waveform_image);
    }

    // Metadata text
    if !state.metadata.is_empty() {
        main_content = main_content.push(text(&state.metadata).size(16));
    }

    // Controls
    let play_button_text = if state.playing { "Pause" } else { "Play" };
    let controls = row![
        button(text(play_button_text)).on_press(Message::PlayPauseClicked),
        button(text("Seek -10s")).on_press(Message::SeekRelativeClicked(-10.0)),
        button(text("Seek +10s")).on_press(Message::SeekRelativeClicked(10.0)),
        text(format!("Time: {}", state.time)).size(14),
    ]
    .spacing(15)
    .align_y(iced::Alignment::Center);

    main_content = main_content.push(controls);

    container(main_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
