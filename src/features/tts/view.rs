use iced::widget::{button, container, pick_list, row, text};
use iced::{Alignment, Border, Element, Length, Padding, Shadow, Vector};

use crate::app::messages::Message;
use crate::core::config::TtsBackend;
use crate::core::types::KglanceState;
use crate::features::tts::engine::{active_local_voice_id, local_voice_options};
use crate::features::tts::types::{TtsLangSelection, TtsStatus};
use crate::features::tts::update::TtsMsg;
use crate::ui::theme::{
    AppTheme, default_button_primary, default_button_secondary, default_pick_list,
};

pub fn tts_active_block_style(theme: &AppTheme) -> container::Style {
    let accent = theme.palette().roles.accent;
    container::Style {
        background: Some(iced::Color::from_rgba(accent.r, accent.g, accent.b, 0.10).into()),
        border: Border {
            color: iced::Color::from_rgba(accent.r, accent.g, accent.b, 0.35),
            width: 0.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

pub fn render_tts_mini_player(state: &KglanceState) -> Option<Element<'_, Message>> {
    if !state.tts.visible {
        return None;
    }

    let palette = state.app_theme.palette();
    let total = state.tts.segments.len();
    let current = if total == 0 {
        0
    } else {
        (state.tts.current_index + 1).min(total)
    };

    let active_lang = state.tts.effective_language();
    let active_voice_id = active_local_voice_id(active_lang, &state.tts_config);
    let voice_options = local_voice_options(
        active_lang,
        &state.tts_config,
        &state.tts.installed_piper_models,
    );
    let selected_voice = voice_options
        .iter()
        .find(|opt| opt.id == active_voice_id)
        .cloned()
        .or_else(|| voice_options.first().cloned());
    let iced_theme = iced::Theme::from(state.app_theme);

    let rate = if state.tts.rate <= 0.0 {
        state.tts_config.rate
    } else {
        state.tts.rate
    };

    let play_label = match &state.tts.status {
        TtsStatus::Playing => "Pause",
        TtsStatus::Synthesizing => "...",
        TtsStatus::Paused | TtsStatus::Idle | TtsStatus::Error(_) => "Play",
    };

    let status_text = match &state.tts.status {
        TtsStatus::Error(err) => {
            let short: String = err.chars().take(22).collect();
            format!("Err: {short}")
        }
        TtsStatus::Idle | TtsStatus::Synthesizing | TtsStatus::Playing | TtsStatus::Paused => {
            let backend = if state.tts.active_backend_label.is_empty() {
                "TTS"
            } else {
                state.tts.active_backend_label.as_str()
            };
            format!("{backend} {current}/{total}")
        }
    };

    let prev_btn = button(text("<").size(12))
        .padding([4, 8])
        .style(default_button_secondary)
        .on_press(Message::Tts(TtsMsg::Prev));

    let play_btn = button(text(play_label).size(12))
        .padding([4, 10])
        .style(default_button_primary)
        .on_press(Message::Tts(TtsMsg::TogglePlayPause));

    let next_btn = button(text(">").size(12))
        .padding([4, 8])
        .style(default_button_secondary)
        .on_press(Message::Tts(TtsMsg::Next));

    let slower_btn = button(text("-").size(12))
        .padding([4, 7])
        .style(default_button_secondary)
        .on_press(Message::Tts(TtsMsg::AdjustRate(-0.1)));

    let speed_lbl = text(format!("{rate:.1}x"))
        .size(12)
        .color(palette.base.text);

    let faster_btn = button(text("+").size(12))
        .padding([4, 7])
        .style(default_button_secondary)
        .on_press(Message::Tts(TtsMsg::AdjustRate(0.1)));

    let backend_theme = iced_theme.clone();
    let backend_picker = pick_list(
        &TtsBackend::ALL[..],
        Some(state.tts_config.backend),
        |backend| Message::Tts(TtsMsg::SetBackend(backend)),
    )
    .text_size(11)
    .padding([4, 8])
    .style(move |_, status| default_pick_list(&backend_theme, status));

    let lang_theme = iced_theme.clone();
    let lang_picker = pick_list(
        &TtsLangSelection::ALL[..],
        Some(state.tts.lang_selection),
        |selection| Message::Tts(TtsMsg::SelectLanguage(selection)),
    )
    .text_size(11)
    .padding([4, 8])
    .style(move |_, status| default_pick_list(&lang_theme, status));

    let voice_picker = pick_list(voice_options, selected_voice, |option| {
        Message::Tts(TtsMsg::SelectVoice(option))
    })
    .text_size(11)
    .padding([4, 8])
    .style(move |_, status| default_pick_list(&iced_theme, status));

    let stop_btn = button(text("Stop").size(12))
        .padding([4, 8])
        .style(default_button_secondary)
        .on_press(Message::Tts(TtsMsg::Stop));

    let status_lbl = text(status_text).size(11).color(palette.base.text_dim);

    let bar = container(
        row![
            status_lbl,
            prev_btn,
            play_btn,
            next_btn,
            slower_btn,
            speed_lbl,
            faster_btn,
            backend_picker,
            lang_picker,
            voice_picker,
            stop_btn,
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([6, 10]))
    .style(move |_| container::Style {
        background: Some(palette.base.surface.into()),
        border: Border {
            color: palette.base.border,
            width: 1.0,
            radius: 10.0.into(),
        },
        shadow: Shadow {
            color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.25),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },
        ..Default::default()
    });

    let layer = container(bar)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Right)
        .align_y(iced::alignment::Vertical::Bottom)
        .padding(Padding {
            top: 0.0,
            right: 16.0,
            bottom: 44.0,
            left: 16.0,
        });

    Some(layer.into())
}
