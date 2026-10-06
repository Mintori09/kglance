use crate::app::Message;
use crate::core::MediaState;
use crate::ui::theme::{video_controls_pill, video_pill_button, video_slider};
use iced::widget::{
    Space, button, column, container, image, mouse_area, row, slider, stack, svg, text,
};
use iced::{Alignment, Color, Element, Length, Padding};
use iced_video_player::VideoPlayer;

const SEEK_STEP: f32 = 0.001;
const SEEK_SKIP_SECONDS: f32 = 10.0;

static SVG_PLAY: &[u8] =
    br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>"#;
static SVG_PAUSE: &[u8] =
    br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor"><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/></svg>"#;
static SVG_REWIND: &[u8] =
    br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor"><path d="M11 18V6l-8.5 6 8.5 6zm.5-6l8.5 6V6l-8.5 6z"/></svg>"#;
static SVG_FAST_FORWARD: &[u8] =
    br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor"><path d="M4 18l8.5-6L4 6v12zm9-12v12l8.5-6L13 6z"/></svg>"#;

const STATUS_TEXT_SIZE: f32 = 14.0;
const TIME_TEXT_SIZE: f32 = 12.0;

const ICON_COLOR: Color = Color::from_rgba(0.92, 0.92, 0.95, 1.0);
const TIME_COLOR: Color = Color::from_rgba(0.82, 0.82, 0.86, 1.0);

const PILL_MAX_WIDTH: f32 = 620.0;
const PILL_PADDING: Padding = Padding {
    top: 6.0,
    right: 14.0,
    bottom: 6.0,
    left: 14.0,
};
const PILL_BOTTOM_MARGIN: f32 = 18.0;
const PILL_SIDE_MARGIN: f32 = 24.0;

const BOTTOM_TRIGGER_ZONE_HEIGHT: f32 = 110.0;

const CONTROLS_SPACING: f32 = 4.0;
const ACTION_ROW_SPACING: f32 = 4.0;

pub fn view_media<'a>(
    state: &'a MediaState,
    data: &'a [u8],
    video: Option<&'a iced_video_player::Video>,
    _wf_width: u32,
    _wf_height: u32,
) -> Element<'a, Message> {
    let media_display = render_media_display(state, data, video);
    let subtitles_layer = render_subtitles_overlay(state);
    let controls_layer = render_bottom_controls_layer(state);

    let content = stack(vec![media_display, subtitles_layer, controls_layer]);

    container(content)
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
        render_video_player(video, state)
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

fn render_video_player<'a>(
    video: Option<&'a iced_video_player::Video>,
    _state: &'a MediaState,
) -> Element<'a, Message> {
    let Some(video) = video else {
        return text("Loading video...").size(STATUS_TEXT_SIZE).into();
    };

    mouse_area(
        VideoPlayer::new(video)
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(iced::ContentFit::Contain)
            .on_end_of_stream(crate::app::messages::MediaMsg::VideoEndOfStream.into())
            .on_new_frame(crate::app::messages::MediaMsg::VideoNewFrame.into()),
    )
    .on_press(crate::app::messages::MediaMsg::PlayPauseClicked.into())
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

fn render_subtitles_overlay<'a>(state: &'a MediaState) -> Element<'a, Message> {
    if !state.subtitles_enabled {
        return Space::new()
            .width(Length::Shrink)
            .height(Length::Shrink)
            .into();
    }

    let Some(ref sub_text) = state.current_subtitle_text else {
        return Space::new()
            .width(Length::Shrink)
            .height(Length::Shrink)
            .into();
    };

    if sub_text.is_empty() {
        return Space::new()
            .width(Length::Shrink)
            .height(Length::Shrink)
            .into();
    }

    // Place subtitle slightly higher if controls are showing to avoid overlap
    let bottom_margin = if state.show_controls { 85.0 } else { 35.0 };

    let sub_box = container(
        text(sub_text.as_str())
            .size(16.0)
            .align_x(iced::alignment::Horizontal::Center)
            .style(|_theme| iced::widget::text::Style {
                color: Some(Color::WHITE),
            }),
    )
    .style(|_theme| container::Style {
        background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.75).into()),
        border: iced::Border {
            radius: 6.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: iced::Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
            offset: iced::Vector::new(0.0, 2.0),
            blur_radius: 6.0,
        },
        snap: false,
        ..Default::default()
    })
    .padding(Padding {
        top: 4.0,
        right: 12.0,
        bottom: 4.0,
        left: 12.0,
    })
    .max_width(720.0);

    container(
        column![Space::new().height(Length::Fill), sub_box]
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(Padding {
        top: 0.0,
        right: 24.0,
        bottom: bottom_margin,
        left: 24.0,
    })
    .center_x(Length::Fill)
    .into()
}

fn render_bottom_controls_layer<'a>(state: &'a MediaState) -> Element<'a, Message> {
    let bottom_content: Element<'a, Message> = if state.show_controls {
        render_controls_overlay(state)
    } else {
        Space::new()
            .width(Length::Fill)
            .height(Length::Fixed(BOTTOM_TRIGGER_ZONE_HEIGHT))
            .into()
    };

    let bottom_hover_area = mouse_area(
        container(bottom_content)
            .width(Length::Fill)
            .height(Length::Fixed(BOTTOM_TRIGGER_ZONE_HEIGHT)),
    )
    .on_enter(crate::app::messages::MediaMsg::MouseEnter.into())
    .on_exit(crate::app::messages::MediaMsg::MouseLeave.into());

    column![Space::new().height(Length::Fill), bottom_hover_area,]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn render_controls_overlay<'a>(state: &'a MediaState) -> Element<'a, Message> {
    let play_pause_svg = if state.playing { SVG_PAUSE } else { SVG_PLAY };

    let seek_bar = slider(0.0..=1.0, state.progress, |val| {
        crate::app::messages::MediaMsg::SeekClicked(val).into()
    })
    .step(SEEK_STEP)
    .style(video_slider)
    .width(Length::Fill);

    let rewind_icon = svg(svg::Handle::from_memory(SVG_REWIND))
        .width(Length::Fixed(14.0))
        .height(Length::Fixed(14.0))
        .style(|_theme: &iced::Theme, _status| svg::Style {
            color: Some(ICON_COLOR),
        });

    let play_pause_icon = svg(svg::Handle::from_memory(play_pause_svg))
        .width(Length::Fixed(16.0))
        .height(Length::Fixed(16.0))
        .style(|_theme: &iced::Theme, _status| svg::Style {
            color: Some(ICON_COLOR),
        });

    let fast_forward_icon = svg(svg::Handle::from_memory(SVG_FAST_FORWARD))
        .width(Length::Fixed(14.0))
        .height(Length::Fixed(14.0))
        .style(|_theme: &iced::Theme, _status| svg::Style {
            color: Some(ICON_COLOR),
        });

    let rewind_btn = button(rewind_icon)
        .on_press(crate::app::messages::MediaMsg::SeekRelativeClicked(-SEEK_SKIP_SECONDS).into())
        .style(video_pill_button)
        .padding([4, 8]);

    let play_pause_btn = button(play_pause_icon)
        .on_press(crate::app::messages::MediaMsg::PlayPauseClicked.into())
        .style(video_pill_button)
        .padding([4, 10]);

    let fast_forward_btn = button(fast_forward_icon)
        .on_press(crate::app::messages::MediaMsg::SeekRelativeClicked(SEEK_SKIP_SECONDS).into())
        .style(video_pill_button)
        .padding([4, 8]);

    let time_label = if state.time.is_empty() {
        "0:00 / 0:00"
    } else {
        state.time.as_str()
    };

    let mut action_items: Vec<Element<'a, Message>> = vec![
        rewind_btn.into(),
        play_pause_btn.into(),
        fast_forward_btn.into(),
        Space::new().width(Length::Fill).into(),
    ];

    // Subtitle button if tracks exist
    if !state.subtitle_tracks.is_empty() {
        let is_on = state.subtitles_enabled;
        let label = if let (true, Some(idx)) = (is_on, state.active_subtitle_track) {
            if let Some(track) = state.subtitle_tracks.get(idx) {
                let track_label = &track.label;
                format!("CC • {track_label}")
            } else {
                "CC".to_string()
            }
        } else {
            "CC".to_string()
        };

        let cc_color = if is_on {
            Color::WHITE
        } else {
            Color::from_rgba(0.55, 0.55, 0.60, 1.0)
        };

        let cc_btn =
            button(
                text(label)
                    .size(11.0)
                    .style(move |_theme| iced::widget::text::Style {
                        color: Some(cc_color),
                    }),
            )
            .on_press(crate::app::messages::MediaMsg::CycleSubtitleTrack.into())
            .style(video_pill_button)
            .padding([3, 7]);

        action_items.push(cc_btn.into());
    }

    action_items.push(
        text(time_label)
            .size(TIME_TEXT_SIZE)
            .style(|_theme| iced::widget::text::Style {
                color: Some(TIME_COLOR),
            })
            .into(),
    );

    let action_row = row(action_items)
        .spacing(ACTION_ROW_SPACING)
        .align_y(Alignment::Center);

    let pill = container(column![seek_bar, action_row].spacing(CONTROLS_SPACING))
        .style(video_controls_pill)
        .padding(PILL_PADDING)
        .max_width(PILL_MAX_WIDTH)
        .width(Length::Fill);

    container(pill)
        .width(Length::Fill)
        .height(Length::Fixed(BOTTOM_TRIGGER_ZONE_HEIGHT))
        .align_x(Alignment::Center)
        .align_y(iced::alignment::Vertical::Bottom)
        .padding(Padding {
            top: 0.0,
            right: PILL_SIDE_MARGIN,
            bottom: PILL_BOTTOM_MARGIN,
            left: PILL_SIDE_MARGIN,
        })
        .into()
}
