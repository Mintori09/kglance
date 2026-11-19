use iced::widget::{button, column, container, pick_list, row, text, text_input};
use iced::{Alignment, Element, Font, Theme};

use crate::app::messages::Message;
use crate::core::config::{TtsBackend, TtsConfig};
use crate::features::tts::TtsMsg;
use crate::features::tts::piper::PIPER_MODEL_CATALOG;
use crate::features::tts::types::{TtsLanguage, TtsState};
use crate::ui::theme::color::BaseColors;
use crate::ui::theme::tokens::{spacing, typography};
use crate::ui::theme::{
    default_button, default_button_primary, default_button_secondary, default_card,
    default_pick_list, default_text_input,
};

use super::build_setting_row;

pub fn tts_tab<'a>(
    theme: &Theme,
    tts_config: &'a TtsConfig,
    tts_state: &'a TtsState,
    main_font: Font,
) -> Element<'a, Message> {
    let base_colors = BaseColors::palette(theme);
    let text_color = base_colors.text;
    let dim_color = base_colors.text_dim;

    let backend_picker = pick_list(&TtsBackend::ALL[..], Some(tts_config.backend), |backend| {
        Message::Tts(TtsMsg::SetBackend(backend))
    })
    .text_size(typography::BODY)
    .font(main_font)
    .style({
        let theme_clone = theme.clone();
        move |_, status| default_pick_list(&theme_clone, status)
    });

    let engine_card = container(
        column![
            text("Local TTS engine")
                .size(typography::BODY_MD)
                .font(main_font)
                .style(move |_| iced::widget::text::Style {
                    color: Some(text_color)
                }),
            build_setting_row(
                "TTS backend",
                "Auto tries VieNeu -> Piper -> eSpeak (100% local)",
                backend_picker,
                main_font,
                text_color,
                dim_color,
            ),
        ]
        .spacing(spacing::M),
    )
    .padding(spacing::M)
    .style(default_card);

    let endpoint_input = text_input(
        "http://127.0.0.1:8000/v1/audio/speech",
        &tts_config.vieneu_endpoint,
    )
    .on_input(|s| Message::Tts(TtsMsg::SetVieNeuEndpoint(s)))
    .size(typography::BODY)
    .font(main_font)
    .padding([spacing::XS, spacing::S])
    .style({
        let theme_clone = theme.clone();
        move |_, status| default_text_input(&theme_clone, status)
    });

    let voice_input = text_input("Default server voice (optional)", &tts_config.vieneu_voice)
        .on_input(|s| Message::Tts(TtsMsg::SetVieNeuVoice(s)))
        .size(typography::BODY)
        .font(main_font)
        .padding([spacing::XS, spacing::S])
        .style({
            let theme_clone = theme.clone();
            move |_, status| default_text_input(&theme_clone, status)
        });

    let vieneu_card = container(
        column![
            text("VieNeu-TTS (local server)")
                .size(typography::BODY_MD)
                .font(main_font)
                .style(move |_| iced::widget::text::Style {
                    color: Some(text_color)
                }),
            build_setting_row(
                "Endpoint URL",
                "OpenAI-compatible /v1/audio/speech URL",
                endpoint_input,
                main_font,
                text_color,
                dim_color,
            ),
            build_setting_row(
                "Voice ID",
                "Optional preset or cloned speaker ID",
                voice_input,
                main_font,
                text_color,
                dim_color,
            ),
        ]
        .spacing(spacing::M),
    )
    .padding(spacing::M)
    .style(default_card);

    let mut model_rows = column![
        text("Piper local models (piper-rs)")
            .size(typography::BODY_MD)
            .font(main_font)
            .style(move |_| iced::widget::text::Style {
                color: Some(text_color)
            }),
        text("Models are downloaded on demand and stored outside the Kglance binary.")
            .size(typography::CAPTION)
            .font(main_font)
            .style(move |_| iced::widget::text::Style {
                color: Some(dim_color)
            }),
    ]
    .spacing(spacing::S);

    if let Some(err) = &tts_state.model_download_error {
        let err_text = err.clone();
        model_rows = model_rows.push(
            text(format!("Error: {err_text}"))
                .size(typography::CAPTION)
                .font(main_font),
        );
    }

    for spec in PIPER_MODEL_CATALOG {
        let is_installed = tts_state.installed_piper_models.contains(spec.id);
        let is_downloading = tts_state.downloading_models.contains(spec.id);
        let is_selected = match spec.lang {
            TtsLanguage::Vi => tts_config.piper_model_vi == spec.id,
            TtsLanguage::En => tts_config.piper_model_en == spec.id,
            TtsLanguage::Ja => false,
        };

        let meta = format!(
            "{} · {} · {}",
            spec.lang.short_label(),
            spec.quality,
            spec.approx_size
        );

        let info_col = column![
            text(spec.display_name)
                .size(typography::BODY_MD)
                .font(main_font)
                .style(move |_| iced::widget::text::Style {
                    color: Some(text_color)
                }),
            text(meta)
                .size(typography::CAPTION)
                .font(main_font)
                .style(move |_| iced::widget::text::Style {
                    color: Some(dim_color)
                }),
        ]
        .spacing(spacing::XXS)
        .width(iced::Length::Fill);

        let mut actions = row![].spacing(spacing::XS).align_y(Alignment::Center);

        if is_downloading {
            actions = actions.push(
                button(
                    text("Downloading...")
                        .size(typography::CAPTION)
                        .font(main_font),
                )
                .style(default_button_secondary)
                .padding([spacing::XS, spacing::S]),
            );
        } else if is_installed {
            let use_label = if is_selected { "Active" } else { "Use" };
            let mut use_btn = button(text(use_label).size(typography::CAPTION).font(main_font))
                .style(if is_selected {
                    default_button_primary
                } else {
                    default_button
                })
                .padding([spacing::XS, spacing::S]);
            if !is_selected {
                use_btn = use_btn.on_press(Message::Tts(TtsMsg::SelectPiperModel {
                    lang: spec.lang,
                    model_id: spec.id.to_string(),
                }));
            }
            actions = actions.push(use_btn).push(
                button(text("Delete").size(typography::CAPTION).font(main_font))
                    .style(default_button_secondary)
                    .padding([spacing::XS, spacing::S])
                    .on_press(Message::Tts(TtsMsg::DeletePiperModel(spec.id.to_string()))),
            );
        } else {
            actions = actions.push(
                button(text("Download").size(typography::CAPTION).font(main_font))
                    .style(default_button_primary)
                    .padding([spacing::XS, spacing::S])
                    .on_press(Message::Tts(TtsMsg::DownloadPiperModel(
                        spec.id.to_string(),
                    ))),
            );
        }

        model_rows = model_rows.push(
            row![info_col, actions]
                .align_y(Alignment::Center)
                .spacing(spacing::M),
        );
    }

    let piper_card = container(model_rows)
        .padding(spacing::M)
        .style(default_card);

    column![engine_card, vieneu_card, piper_card]
        .spacing(spacing::M)
        .into()
}
