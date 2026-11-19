use std::collections::HashSet;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::core::config::{TtsBackend, TtsConfig};
use crate::features::tts::piper::{
    PIPER_MODEL_CATALOG, is_model_installed, synthesize_with_piper_rs,
};
use crate::features::tts::types::{TtsLanguage, TtsVoiceOption};
use crate::features::tts::vieneu_client::synthesize_vieneu;

pub struct SynthesisOutput {
    pub audio_bytes: Vec<u8>,
    pub backend_label: &'static str,
}

/// Return selectable voice/model options for the mini-player dropdown based on backend & language.
pub fn local_voice_options(
    lang: TtsLanguage,
    config: &TtsConfig,
    installed_piper: &HashSet<String>,
) -> Vec<TtsVoiceOption> {
    match config.backend {
        TtsBackend::VieNeu => {
            if lang == TtsLanguage::Ja {
                return vec![TtsVoiceOption {
                    id: "espeak".to_string(),
                    label: "Unsupported (JA)".to_string(),
                }];
            }
            let mut opts = vec![
                TtsVoiceOption {
                    id: String::new(),
                    label: "Server default".to_string(),
                },
                TtsVoiceOption {
                    id: "Mai Anh".to_string(),
                    label: "Mai Anh".to_string(),
                },
                TtsVoiceOption {
                    id: "Binh".to_string(),
                    label: "Binh".to_string(),
                },
                TtsVoiceOption {
                    id: "Ngoc".to_string(),
                    label: "Ngoc".to_string(),
                },
                TtsVoiceOption {
                    id: "Tuyen".to_string(),
                    label: "Tuyen".to_string(),
                },
            ];
            let custom = config.vieneu_voice.trim();
            if !custom.is_empty() && !opts.iter().any(|o| o.id == custom) {
                opts.insert(
                    1,
                    TtsVoiceOption {
                        id: custom.to_string(),
                        label: custom.to_string(),
                    },
                );
            }
            opts
        }
        TtsBackend::Auto | TtsBackend::Piper => {
            let mut opts: Vec<TtsVoiceOption> = PIPER_MODEL_CATALOG
                .iter()
                .filter(|spec| spec.lang == lang)
                .map(|spec| {
                    let base_label = spec
                        .display_name
                        .split('—')
                        .nth(1)
                        .map(str::trim)
                        .unwrap_or(spec.id);
                    let label = if installed_piper.contains(spec.id) {
                        base_label.to_string()
                    } else {
                        format!("{base_label} (not installed)")
                    };
                    TtsVoiceOption {
                        id: spec.id.to_string(),
                        label,
                    }
                })
                .collect();
            if opts.is_empty() {
                opts.push(TtsVoiceOption {
                    id: "espeak".to_string(),
                    label: "eSpeak".to_string(),
                });
            }
            opts
        }
        TtsBackend::EspeakNg => vec![TtsVoiceOption {
            id: "espeak".to_string(),
            label: "eSpeak".to_string(),
        }],
    }
}

pub fn active_local_voice_id(lang: TtsLanguage, config: &TtsConfig) -> &str {
    match config.backend {
        TtsBackend::VieNeu => config.vieneu_voice.as_str(),
        TtsBackend::Auto | TtsBackend::Piper => match lang {
            TtsLanguage::En => config.piper_model_en.as_str(),
            TtsLanguage::Vi => config.piper_model_vi.as_str(),
            TtsLanguage::Ja => "espeak",
        },
        TtsBackend::EspeakNg => "espeak",
    }
}

fn piper_model_for_language(lang: TtsLanguage, config: &TtsConfig) -> Result<&str, String> {
    match lang {
        TtsLanguage::En => Ok(config.piper_model_en.as_str()),
        TtsLanguage::Vi => Ok(config.piper_model_vi.as_str()),
        TtsLanguage::Ja => {
            Err("Piper does not support Japanese; switch to Auto or eSpeak".to_string())
        }
    }
}

pub async fn synthesize_segment(
    text: &str,
    lang: TtsLanguage,
    rate: f32,
    config: TtsConfig,
) -> Result<SynthesisOutput, String> {
    match config.backend {
        TtsBackend::VieNeu => {
            if !matches!(lang, TtsLanguage::En | TtsLanguage::Vi) {
                return Err("VieNeu-TTS only supports English and Vietnamese".to_string());
            }
            let audio_bytes = synthesize_vieneu(
                text,
                &config.vieneu_endpoint,
                &config.vieneu_voice,
                rate,
                false,
            )
            .await?;
            Ok(SynthesisOutput {
                audio_bytes,
                backend_label: "VieNeu",
            })
        }
        TtsBackend::Piper => {
            let model_id = piper_model_for_language(lang, &config)?;
            let audio_bytes = synthesize_with_piper_rs(text, model_id, rate).await?;
            Ok(SynthesisOutput {
                audio_bytes,
                backend_label: "Piper",
            })
        }
        TtsBackend::EspeakNg => {
            let audio_bytes = synthesize_espeak(text, lang, rate).await?;
            Ok(SynthesisOutput {
                audio_bytes,
                backend_label: "eSpeak",
            })
        }
        TtsBackend::Auto => {
            if matches!(lang, TtsLanguage::En | TtsLanguage::Vi) {
                if let Ok(audio_bytes) = synthesize_vieneu(
                    text,
                    &config.vieneu_endpoint,
                    &config.vieneu_voice,
                    rate,
                    true,
                )
                .await
                {
                    return Ok(SynthesisOutput {
                        audio_bytes,
                        backend_label: "VieNeu",
                    });
                }

                if let Ok(model_id) = piper_model_for_language(lang, &config)
                    && is_model_installed(model_id)
                    && let Ok(audio_bytes) = synthesize_with_piper_rs(text, model_id, rate).await
                {
                    return Ok(SynthesisOutput {
                        audio_bytes,
                        backend_label: "Piper",
                    });
                }
            }

            let audio_bytes = synthesize_espeak(text, lang, rate).await?;
            Ok(SynthesisOutput {
                audio_bytes,
                backend_label: "eSpeak",
            })
        }
    }
}

async fn synthesize_espeak(text: &str, lang: TtsLanguage, rate: f32) -> Result<Vec<u8>, String> {
    let voice = match lang {
        TtsLanguage::En => "en",
        TtsLanguage::Vi => "vi",
        TtsLanguage::Ja => "ja",
    };
    let wpm = ((165.0 * rate).round() as u32).clamp(80, 400);
    let wpm_str = wpm.to_string();

    let mut child = Command::new("espeak-ng")
        .args(["--stdout", "--stdin", "-s", &wpm_str, "-v", voice])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("Failed to launch espeak-ng: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(text.as_bytes())
            .await
            .map_err(|e| format!("Failed writing to espeak-ng stdin: {e}"))?;
        drop(stdin);
    }

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| format!("espeak-ng execution error: {e}"))?;
    if !output.status.success() || output.stdout.len() <= 44 {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("espeak-ng failed or returned empty audio: {err}"));
    }
    Ok(output.stdout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::tts::types::TtsLangSelection;

    #[test]
    fn lang_selection_resolve_works() {
        assert_eq!(
            TtsLangSelection::Auto.resolve(TtsLanguage::Vi),
            TtsLanguage::Vi
        );
        assert_eq!(
            TtsLangSelection::En.resolve(TtsLanguage::Vi),
            TtsLanguage::En
        );
        assert_eq!(
            TtsLangSelection::Vi.resolve(TtsLanguage::En),
            TtsLanguage::Vi
        );
        assert_eq!(
            TtsLangSelection::Ja.resolve(TtsLanguage::En),
            TtsLanguage::Ja
        );
    }

    #[test]
    fn local_voice_options_match_backend() {
        let mut cfg = TtsConfig::default();
        let mut installed = HashSet::new();
        installed.insert("vi_VN-vais1000-medium".to_string());

        let piper_vi = local_voice_options(TtsLanguage::Vi, &cfg, &installed);
        assert_eq!(piper_vi.len(), 2);
        assert_eq!(piper_vi[0].id, "vi_VN-vais1000-medium");
        assert_eq!(piper_vi[0].label, "VAIS1000");
        assert_eq!(piper_vi[1].label, "25h Single (not installed)");
        assert!(piper_vi.iter().all(|o| !o.id.starts_with("en_US-")));

        cfg.backend = TtsBackend::VieNeu;
        cfg.vieneu_voice = "CustomVoice".to_string();
        let vieneu_opts = local_voice_options(TtsLanguage::Vi, &cfg, &installed);
        assert!(vieneu_opts.iter().any(|o| o.id == "CustomVoice"));

        let vieneu_ja = local_voice_options(TtsLanguage::Ja, &cfg, &installed);
        assert_eq!(vieneu_ja.len(), 1);
        assert_eq!(vieneu_ja[0].label, "Unsupported (JA)");
    }
}
