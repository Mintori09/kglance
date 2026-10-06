use crate::app::Message;
use crate::app::messages::AudioMsg;
use crate::features::audio::types::AudioState;
use crate::ui::theme::color::{BaseColors, roles};
use crate::ui::theme::{default_button, default_button_primary, default_card, default_slider};
use iced::widget::{Space, button, column, container, image, row, slider, svg, text};
use iced::{Alignment, Border, Color, ContentFit, Element, Length, Shadow, Vector};

const SEEK_STEP: f32 = 0.001;
const VOLUME_STEP: f32 = 0.01;
const SEEK_SKIP_SECONDS: f32 = 10.0;
const COVER_SIZE: f32 = 176.0;

const TITLE_TEXT_SIZE: f32 = 18.0;
const SUBTITLE_TEXT_SIZE: f32 = 13.0;
const BADGE_TEXT_SIZE: f32 = 12.0;

const CARD_PADDING: [u16; 2] = [20, 24];

static SVG_PLAY: &[u8] =
    br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>"#;
static SVG_PAUSE: &[u8] =
    br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor"><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/></svg>"#;
static SVG_REWIND: &[u8] =
    br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor"><path d="M11 18V6l-8.5 6 8.5 6zm.5-6l8.5 6V6l-8.5 6z"/></svg>"#;
static SVG_FAST_FORWARD: &[u8] =
    br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor"><path d="M4 18l8.5-6L4 6v12zm9-12v12l8.5-6L13 6z"/></svg>"#;
static SVG_VOLUME_HIGH: &[u8] =
    br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor"><path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z"/></svg>"#;
static SVG_VOLUME_MUTE: &[u8] =
    br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor"><path d="M16.5 12c0-1.77-1.02-3.29-2.5-4.03v2.21l2.45 2.45c.03-.2.05-.41.05-.63zm2.5 0c0 .94-.2 1.82-.54 2.64l1.51 1.51C20.63 14.91 21 13.5 21 12c0-4.28-2.99-7.86-7-8.77v2.06c2.89.86 5 3.54 5 6.71zM4.27 3L3 4.27l4.05 4.05L7 9H3v6h4l5 5v-6.73l4.25 4.25c-.67.52-1.42.93-2.25 1.18v2.06c1.38-.31 2.63-.95 3.69-1.81L19.73 21 21 19.73l-9-9L4.27 3zM12 4L9.91 6.09 12 8.18V4z"/></svg>"#;
static SVG_NOTE: &[u8] =
    br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor"><path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"/></svg>"#;

pub fn view_audio<'a>(state: &'a AudioState) -> Element<'a, Message> {
    if let Some(err) = &state.error {
        return container(text(err.as_str()).size(14.0))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
    }

    let cover_view: Element<'a, Message> = if let Some(ref handle) = state.cover_art {
        let img = image(handle.clone())
            .width(Length::Fixed(COVER_SIZE))
            .height(Length::Fixed(COVER_SIZE))
            .content_fit(ContentFit::Cover);

        container(img)
            .width(Length::Fixed(COVER_SIZE))
            .height(Length::Fixed(COVER_SIZE))
            .style(|theme: &iced::Theme| {
                let p = BaseColors::palette(theme);
                container::Style {
                    border: Border {
                        color: p.border,
                        width: 1.0,
                        radius: 10.0.into(),
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                        offset: Vector::new(0.0, 4.0),
                        blur_radius: 10.0,
                    },
                    snap: true,
                    ..Default::default()
                }
            })
            .into()
    } else {
        render_cover_placeholder()
    };

    let title_text = if !state.title.is_empty() {
        state.title.as_str()
    } else {
        "Audio Track"
    };

    let title_widget = text(title_text)
        .size(TITLE_TEXT_SIZE)
        .wrapping(iced::widget::text::Wrapping::Word)
        .style(|theme: &iced::Theme| {
            let p = BaseColors::palette(theme);
            iced::widget::text::Style {
                color: Some(p.text),
            }
        });

    let subtitle = if !state.artist.is_empty() && !state.album.is_empty() {
        format!("{} • {}", state.artist, state.album)
    } else if !state.artist.is_empty() {
        state.artist.clone()
    } else if !state.album.is_empty() {
        state.album.clone()
    } else {
        String::new()
    };

    let progress = if state.progress.is_finite() {
        state.progress.clamp(0.0, 1.0)
    } else {
        0.0
    };

    let pos_formatted = format_seconds(state.position_secs);
    let dur_formatted = format_seconds(state.duration_secs);
    let time_text = format!("{pos_formatted} / {dur_formatted}");

    let time_badge = container(text(time_text).size(BADGE_TEXT_SIZE).style(
        |theme: &iced::Theme| {
            let p = BaseColors::palette(theme);
            iced::widget::text::Style {
                color: Some(p.text),
            }
        },
    ))
    .padding([3, 8])
    .style(|theme: &iced::Theme| {
        let p = BaseColors::palette(theme);
        container::Style {
            background: Some(p.surface_raised.into()),
            border: Border {
                color: p.border,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }
    });

    let mut status_row = row![time_badge].spacing(12).align_y(Alignment::Center);

    if !state.metadata.is_empty() {
        status_row = status_row.push(Space::new().width(Length::Fill));
        let format_badge = container(text(state.metadata.as_str()).size(11.0).style(
            |theme: &iced::Theme| {
                let p = BaseColors::palette(theme);
                iced::widget::text::Style {
                    color: Some(p.text_dim),
                }
            },
        ))
        .padding([2, 7])
        .style(|theme: &iced::Theme| {
            let p = BaseColors::palette(theme);
            container::Style {
                background: Some(Color { a: 0.06, ..p.text }.into()),
                border: Border {
                    color: p.border,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }
        });
        status_row = status_row.push(format_badge);
    }

    let seek_bar = slider(0.0..=1.0, progress, |val| AudioMsg::SeekClicked(val).into())
        .step(SEEK_STEP)
        .style(default_slider)
        .width(Length::Fill);

    let rewind_icon = svg(svg::Handle::from_memory(SVG_REWIND))
        .width(Length::Fixed(14.0))
        .height(Length::Fixed(14.0))
        .style(|theme: &iced::Theme, _status| {
            let p = BaseColors::palette(theme);
            svg::Style {
                color: Some(p.text),
            }
        });
    let rewind_button = button(rewind_icon)
        .on_press(AudioMsg::SeekRelativeClicked(-SEEK_SKIP_SECONDS).into())
        .style(default_button)
        .padding([7, 12]);

    let play_pause_bytes = if state.playing { SVG_PAUSE } else { SVG_PLAY };
    let play_pause_icon = svg(svg::Handle::from_memory(play_pause_bytes))
        .width(Length::Fixed(16.0))
        .height(Length::Fixed(16.0))
        .style(|_theme: &iced::Theme, _status| svg::Style {
            color: Some(Color::WHITE),
        });
    let play_pause_button = button(play_pause_icon)
        .on_press(AudioMsg::PlayPauseClicked.into())
        .style(default_button_primary)
        .padding([8, 18]);

    let ff_icon = svg(svg::Handle::from_memory(SVG_FAST_FORWARD))
        .width(Length::Fixed(14.0))
        .height(Length::Fixed(14.0))
        .style(|theme: &iced::Theme, _status| {
            let p = BaseColors::palette(theme);
            svg::Style {
                color: Some(p.text),
            }
        });
    let fast_forward_button = button(ff_icon)
        .on_press(AudioMsg::SeekRelativeClicked(SEEK_SKIP_SECONDS).into())
        .style(default_button)
        .padding([7, 12]);

    let vol_bytes = if state.muted || state.volume == 0.0 {
        SVG_VOLUME_MUTE
    } else {
        SVG_VOLUME_HIGH
    };
    let vol_icon = svg(svg::Handle::from_memory(vol_bytes))
        .width(Length::Fixed(14.0))
        .height(Length::Fixed(14.0))
        .style(|theme: &iced::Theme, _status| {
            let p = BaseColors::palette(theme);
            svg::Style {
                color: Some(p.text_dim),
            }
        });
    let mute_button = button(vol_icon)
        .on_press(AudioMsg::ToggleMuteClicked.into())
        .style(default_button)
        .padding([6, 8]);

    let vol_val = if state.muted { 0.0 } else { state.volume };
    let volume_slider = slider(0.0..=1.0, vol_val, |val| {
        AudioMsg::VolumeChanged(val).into()
    })
    .step(VOLUME_STEP)
    .width(Length::Fixed(65.0))
    .style(default_slider);

    let volume_row = row![mute_button, volume_slider]
        .spacing(6)
        .align_y(Alignment::Center);

    let center_controls = row![rewind_button, play_pause_button, fast_forward_button]
        .spacing(10)
        .align_y(Alignment::Center);

    let bottom_row = row![
        Space::new().width(Length::Fixed(105.0)),
        container(center_controls)
            .width(Length::Fill)
            .center_x(Length::Fill),
        container(volume_row)
            .width(Length::Fixed(105.0))
            .align_right(Length::Fixed(105.0)),
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill);

    let mut right_col = column![title_widget].spacing(4).width(Length::Fill);

    if !subtitle.is_empty() {
        right_col = right_col.push(
            text(subtitle)
                .size(SUBTITLE_TEXT_SIZE)
                .wrapping(iced::widget::text::Wrapping::Word)
                .style(|theme: &iced::Theme| {
                    let p = BaseColors::palette(theme);
                    iced::widget::text::Style {
                        color: Some(p.text_dim),
                    }
                }),
        );
    }

    right_col = right_col.push(Space::new().height(6));
    right_col = right_col.push(status_row);
    right_col = right_col.push(seek_bar);
    right_col = right_col.push(Space::new().height(4));
    right_col = right_col.push(bottom_row);

    let card_content = row![cover_view, right_col]
        .spacing(24)
        .align_y(Alignment::Center);

    let player_card = container(card_content)
        .width(Length::Fixed(640.0))
        .height(Length::Fixed(230.0))
        .style(default_card)
        .padding(CARD_PADDING);

    container(player_card)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

fn render_cover_placeholder<'a>() -> Element<'a, Message> {
    let note_icon = svg(svg::Handle::from_memory(SVG_NOTE))
        .width(Length::Fixed(20.0))
        .height(Length::Fixed(20.0))
        .style(|_theme: &iced::Theme, _status| svg::Style {
            color: Some(Color::WHITE),
        });

    let vinyl_center = container(note_icon)
        .width(Length::Fixed(44.0))
        .height(Length::Fixed(44.0))
        .center_x(Length::Fixed(44.0))
        .center_y(Length::Fixed(44.0))
        .style(|theme: &iced::Theme| {
            let role = roles::palette(theme);
            container::Style {
                background: Some(role.accent.into()),
                border: Border {
                    color: Color::from_rgba(1.0, 1.0, 1.0, 0.25),
                    width: 2.0,
                    radius: 22.0.into(),
                },
                ..Default::default()
            }
        });

    let vinyl_disc = container(vinyl_center)
        .width(Length::Fixed(136.0))
        .height(Length::Fixed(136.0))
        .center_x(Length::Fixed(136.0))
        .center_y(Length::Fixed(136.0))
        .style(|_theme: &iced::Theme| container::Style {
            background: Some(Color::from_rgb8(24, 25, 28).into()),
            border: Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                width: 2.0,
                radius: 68.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 10.0,
            },
            ..Default::default()
        });

    container(vinyl_disc)
        .width(Length::Fixed(COVER_SIZE))
        .height(Length::Fixed(COVER_SIZE))
        .center_x(Length::Fixed(COVER_SIZE))
        .center_y(Length::Fixed(COVER_SIZE))
        .style(|theme: &iced::Theme| {
            let p = BaseColors::palette(theme);
            container::Style {
                background: Some(p.surface_raised.into()),
                border: Border {
                    color: p.border,
                    width: 1.0,
                    radius: 10.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

fn format_seconds(secs: f64) -> String {
    let total_secs = secs.max(0.0) as u64;
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let rem_secs = total_secs % 60;
    if hours > 0 {
        format!("{hours}:{mins:02}:{rem_secs:02}")
    } else {
        format!("{mins}:{rem_secs:02}")
    }
}
