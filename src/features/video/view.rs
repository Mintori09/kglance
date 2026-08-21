use crate::app::Message;
use crate::core::MediaState;
use crate::ui::theme::{default_button, default_card, default_slider};
use iced::widget::{Space, button, column, container, image, mouse_area, row, slider, stack, text};
use iced::{Alignment, Element, Length};
use iced_video_player::VideoPlayer;

const SEEK_STEP: f32 = 0.001;
const SEEK_SKIP_SECONDS: f32 = 10.0;

const ICON_PAUSE: &str = "⏸";
const ICON_PLAY: &str = "▶";
const ICON_REWIND: &str = "⏩";
const ICON_FAST_FORWARD: &str = "⏪";

const STATUS_TEXT_SIZE: f32 = 14.0;
const METADATA_TEXT_SIZE: f32 = 12.0;

const CONTROLS_SPACING: f32 = 6.0;
const METADATA_ROW_SPACING: f32 = 8.0;
const ACTION_ROW_SPACING: f32 = 8.0;

const BUTTON_SMALL_PADDING: [u16; 2] = [4, 8];
const BUTTON_LARGE_PADDING: [u16; 2] = [4, 12];
const CARD_PADDING: [u16; 2] = [8, 12];

pub fn view_media<'a>(
    state: &'a MediaState,
    data: &'a [u8],
    video: Option<&'a iced_video_player::Video>,
    _wf_width: u32,
    _wf_height: u32,
) -> Element<'a, Message> {
    let media_display = render_media_display(state, data, video);
    let mut layers: Vec<Element<'a, Message>> = vec![media_display];

    if state.show_controls {
        layers.push(render_controls_overlay(state));
    }

    let interactive_content = mouse_area(stack(layers))
        .on_enter(crate::app::messages::MediaMsg::MouseEnter.into())
        .on_exit(crate::app::messages::MediaMsg::MouseLeave.into());

    container(interactive_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(0)
        .into()
}

fn render_media_display<'a>(
    state: &'a MediaState,
    data: &'a [u8],
    video: Option<&'a iced_video_player::Video>,
) -> Element<'a, Message> {
    let content: Element<'a, Message> = if let Some(err) = &state.error {
        text(err.as_str()).size(STATUS_TEXT_SIZE).into()
    } else if state.has_video {
        render_video_player(video)
    } else if !data.is_empty() {
        render_image_preview(data)
    } else {
        render_placeholder_text(&state.metadata)
    };

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

fn render_video_player<'a>(video: Option<&'a iced_video_player::Video>) -> Element<'a, Message> {
    let Some(video) = video else {
        return text("Loading video...").size(STATUS_TEXT_SIZE).into();
    };

    VideoPlayer::new(video)
        .width(Length::Fill)
        .height(Length::Fill)
        .content_fit(iced::ContentFit::Contain)
        .on_end_of_stream(crate::app::messages::MediaMsg::VideoEndOfStream.into())
        .on_new_frame(crate::app::messages::MediaMsg::VideoNewFrame.into())
        .into()
}

fn render_image_preview<'a>(data: &[u8]) -> Element<'a, Message> {
    let image_handle = image::Handle::from_bytes(data.to_vec());
    image(image_handle)
        .width(Length::Fill)
        .height(Length::Fill)
        .content_fit(iced::ContentFit::Contain)
        .into()
}

fn render_placeholder_text<'a>(metadata: &str) -> Element<'a, Message> {
    let label = if metadata.is_empty() {
        "No preview"
    } else {
        ""
    };

    text(label).size(STATUS_TEXT_SIZE).into()
}

fn render_controls_overlay<'a>(state: &'a MediaState) -> Element<'a, Message> {
    let play_pause_icon = if state.playing { ICON_PAUSE } else { ICON_PLAY };

    let seek_bar = slider(0.0..=1.0, state.progress, |val| {
        crate::app::messages::MediaMsg::SeekClicked(val).into()
    })
    .step(SEEK_STEP)
    .style(default_slider)
    .width(Length::Fill);

    let metadata_row = row![
        text(&state.metadata)
            .size(METADATA_TEXT_SIZE)
            .width(Length::Fill),
        text(&state.time).size(METADATA_TEXT_SIZE),
    ]
    .spacing(METADATA_ROW_SPACING)
    .align_y(Alignment::Center);

    let rewind_button = button(text(ICON_FAST_FORWARD))
        .on_press(crate::app::messages::MediaMsg::SeekRelativeClicked(-SEEK_SKIP_SECONDS).into())
        .style(default_button)
        .padding(BUTTON_SMALL_PADDING);

    let play_pause_button = button(text(play_pause_icon))
        .on_press(crate::app::messages::MediaMsg::PlayPauseClicked.into())
        .style(default_button)
        .padding(BUTTON_LARGE_PADDING);

    let fast_forward_button = button(text(ICON_REWIND))
        .on_press(crate::app::messages::MediaMsg::SeekRelativeClicked(SEEK_SKIP_SECONDS).into())
        .style(default_button)
        .padding(BUTTON_SMALL_PADDING);

    let action_row = row![rewind_button, play_pause_button, fast_forward_button]
        .spacing(ACTION_ROW_SPACING)
        .align_y(Alignment::Center);

    let controls_card =
        container(column![seek_bar, metadata_row, action_row].spacing(CONTROLS_SPACING))
            .style(default_card)
            .padding(CARD_PADDING)
            .width(Length::Fill);

    column![Space::new().height(Length::Fill), controls_card]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
