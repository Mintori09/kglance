use crate::app::Message;
use crate::features::audio::types::AudioState;
use crate::ui::theme::color::{BaseColors, roles};
use crate::ui::theme::{default_button, default_button_primary, default_card, default_slider};
use iced::widget::{Space, button, column, container, image, row, slider, text};
use iced::{Alignment, Border, Color, ContentFit, Element, Length, Shadow, Vector};

const SEEK_STEP: f32 = 0.001;
const SEEK_SKIP_SECONDS: f32 = 10.0;

const ICON_PAUSE: &str = "⏸";
const ICON_PLAY: &str = "▶";
const ICON_REWIND: &str = "⏪";
const ICON_FAST_FORWARD: &str = "⏩";
const ICON_MUSIC: &str = "🎵";

const COVER_SIZE: f32 = 190.0;
const TITLE_TEXT_SIZE: f32 = 20.0;
const SUBTITLE_TEXT_SIZE: f32 = 13.0;
const BADGE_TEXT_SIZE: f32 = 12.0;

const CARD_PADDING: [u16; 2] = [20, 24];
const BUTTON_CONTROL_PADDING: [u16; 2] = [7, 14];
const BUTTON_PLAY_PADDING: [u16; 2] = [8, 22];

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
                let role = roles::palette(theme);
                container::Style {
                    border: Border {
                        color: role.accent,
                        width: 1.5,
                        radius: 12.0.into(),
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
                        offset: Vector::new(0.0, 4.0),
                        blur_radius: 12.0,
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

    let status_badge = if state.playing {
        row![
            text("●").size(10.0).style(|theme: &iced::Theme| {
                let role = roles::palette(theme);
                iced::widget::text::Style {
                    color: Some(role.success),
                }
            }),
            text("Playing")
                .size(BADGE_TEXT_SIZE)
                .style(|theme: &iced::Theme| {
                    let role = roles::palette(theme);
                    iced::widget::text::Style {
                        color: Some(role.success),
                    }
                }),
        ]
        .spacing(4)
        .align_y(Alignment::Center)
    } else {
        row![
            text("⏸").size(10.0).style(|theme: &iced::Theme| {
                let p = BaseColors::palette(theme);
                iced::widget::text::Style {
                    color: Some(p.text_dim),
                }
            }),
            text("Paused")
                .size(BADGE_TEXT_SIZE)
                .style(|theme: &iced::Theme| {
                    let p = BaseColors::palette(theme);
                    iced::widget::text::Style {
                        color: Some(p.text_dim),
                    }
                }),
        ]
        .spacing(4)
        .align_y(Alignment::Center)
    };

    let mut status_row = row![time_badge, status_badge]
        .spacing(12)
        .align_y(Alignment::Center);

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

    let seek_bar = slider(0.0..=1.0, progress, |val| {
        crate::app::messages::AudioMsg::SeekClicked(val).into()
    })
    .step(SEEK_STEP)
    .style(default_slider)
    .width(Length::Fill);

    let rewind_button = button(text(ICON_REWIND))
        .on_press(crate::app::messages::AudioMsg::SeekRelativeClicked(-SEEK_SKIP_SECONDS).into())
        .style(default_button)
        .padding(BUTTON_CONTROL_PADDING);

    let play_pause_icon = if state.playing { ICON_PAUSE } else { ICON_PLAY };
    let play_pause_button = button(text(play_pause_icon).size(16.0))
        .on_press(crate::app::messages::AudioMsg::PlayPauseClicked.into())
        .style(default_button_primary)
        .padding(BUTTON_PLAY_PADDING);

    let fast_forward_button = button(text(ICON_FAST_FORWARD))
        .on_press(crate::app::messages::AudioMsg::SeekRelativeClicked(SEEK_SKIP_SECONDS).into())
        .style(default_button)
        .padding(BUTTON_CONTROL_PADDING);

    let controls_row = row![rewind_button, play_pause_button, fast_forward_button]
        .spacing(14)
        .align_y(Alignment::Center);

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

    right_col = right_col.push(Space::new().height(8));
    right_col = right_col.push(status_row);
    right_col = right_col.push(seek_bar);
    right_col = right_col.push(Space::new().height(4));
    right_col = right_col.push(container(controls_row).center_x(Length::Fill));

    let card_content = row![cover_view, right_col]
        .spacing(24)
        .align_y(Alignment::Center);

    let player_card = container(card_content)
        .width(Length::Fixed(620.0))
        .height(Length::Fixed(240.0))
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
    let vinyl_center = container(text(ICON_MUSIC).size(22.0))
        .width(Length::Fixed(50.0))
        .height(Length::Fixed(50.0))
        .center_x(Length::Fixed(50.0))
        .center_y(Length::Fixed(50.0))
        .style(|theme: &iced::Theme| {
            let role = roles::palette(theme);
            container::Style {
                background: Some(role.accent.into()),
                border: Border {
                    color: Color::from_rgba(1.0, 1.0, 1.0, 0.25),
                    width: 2.0,
                    radius: 25.0.into(),
                },
                ..Default::default()
            }
        });

    let vinyl_disc = container(vinyl_center)
        .width(Length::Fixed(146.0))
        .height(Length::Fixed(146.0))
        .center_x(Length::Fixed(146.0))
        .center_y(Length::Fixed(146.0))
        .style(|_theme: &iced::Theme| container::Style {
            background: Some(Color::from_rgb8(24, 25, 28).into()),
            border: Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                width: 2.0,
                radius: 73.0.into(),
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
                    radius: 12.0.into(),
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
