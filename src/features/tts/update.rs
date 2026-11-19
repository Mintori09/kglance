use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use iced::Task;
use iced::widget::operation;

use crate::app::KglanceApp;
use crate::app::messages::Message;
use crate::core::PreviewData;
use crate::core::config::{EpubReadingMode, TtsBackend};
use crate::features::markdown::types::MarkdownState;
use crate::features::tts::engine::synthesize_segment;
use crate::features::tts::extract::{extract_segments, find_start_segment_from_scroll};
use crate::features::tts::piper::{delete_model, download_model, installed_models_set};
use crate::features::tts::player::{TtsAudioPlayer, wait_for_playback_end};
use crate::features::tts::types::{TtsLangSelection, TtsLanguage, TtsStatus, TtsVoiceOption};
use crate::features::tts::vieneu_client::clear_offline_cooldown;

#[derive(Debug, Clone)]
pub enum TtsMsg {
    Toggle,
    Play,
    TogglePlayPause,
    Stop,
    Next,
    Prev,
    AdjustRate(f32),
    SelectLanguage(TtsLangSelection),
    SelectVoice(TtsVoiceOption),
    RefreshPiperModels,
    DownloadPiperModel(String),
    PiperModelDownloaded {
        model_id: String,
        result: Result<(), String>,
    },
    DeletePiperModel(String),
    PiperModelDeleted {
        model_id: String,
        result: Result<(), String>,
    },
    SelectPiperModel {
        lang: TtsLanguage,
        model_id: String,
    },
    SetBackend(TtsBackend),
    SetVieNeuEndpoint(String),
    SetVieNeuVoice(String),
    Synthesized {
        playback_id: u64,
        seg_idx: usize,
        audio: Vec<u8>,
        backend_label: String,
    },
    Prefetched {
        gen_id: u64,
        seg_idx: usize,
        audio: Vec<u8>,
        backend_label: String,
    },
    PrefetchFailed {
        gen_id: u64,
        seg_idx: usize,
    },
    PlaybackEnded(u64),
    EngineError(u64, String),
}

pub fn handle_tts_message(app: &mut KglanceApp, msg: TtsMsg) -> Task<Message> {
    match msg {
        TtsMsg::Toggle => {
            if app.state.tts.visible {
                stop_tts(app);
                Task::none()
            } else {
                start_tts_from_current_view(app)
            }
        }
        TtsMsg::Play => start_tts_from_current_view(app),
        TtsMsg::TogglePlayPause => match app.state.tts.status {
            TtsStatus::Playing => {
                if let Some(player) = &mut app.tts_player
                    && let Err(e) = player.pause()
                {
                    app.state.tts.status = TtsStatus::Error(e);
                    return Task::none();
                }
                app.state.tts.status = TtsStatus::Paused;
                Task::none()
            }
            TtsStatus::Paused => {
                if let Some(player) = &mut app.tts_player
                    && let Err(e) = player.resume()
                {
                    app.state.tts.status = TtsStatus::Error(e);
                    return Task::none();
                }
                app.state.tts.status = TtsStatus::Playing;
                Task::none()
            }
            TtsStatus::Idle | TtsStatus::Error(_) => start_tts_from_current_view(app),
            TtsStatus::Synthesizing => Task::none(),
        },
        TtsMsg::Stop => {
            stop_tts(app);
            Task::none()
        }
        TtsMsg::Next => {
            if app.state.tts.current_index + 1 < app.state.tts.segments.len() {
                app.state.tts.current_index += 1;
                play_current_segment(app)
            } else {
                advance_epub_chapter_or_stop(app)
            }
        }
        TtsMsg::Prev => {
            if app.state.tts.current_index > 0 {
                app.state.tts.current_index -= 1;
                play_current_segment(app)
            } else {
                Task::none()
            }
        }
        TtsMsg::AdjustRate(delta) => {
            let base = if app.state.tts.rate <= 0.0 {
                app.state.tts_config.rate
            } else {
                app.state.tts.rate
            };
            app.state.tts.rate = ((base + delta) * 100.0).round() / 100.0;
            app.state.tts.rate = app.state.tts.rate.clamp(0.5, 2.0);
            app.state.tts.invalidate_prefetch();
            if matches!(app.state.tts.status, TtsStatus::Playing | TtsStatus::Paused) {
                play_current_segment(app)
            } else {
                Task::none()
            }
        }
        TtsMsg::SelectLanguage(selection) => {
            app.state.tts.lang_selection = selection;
            app.state.tts.invalidate_prefetch();
            if matches!(
                app.state.tts.status,
                TtsStatus::Playing | TtsStatus::Paused | TtsStatus::Synthesizing
            ) {
                play_current_segment(app)
            } else {
                Task::none()
            }
        }
        TtsMsg::SelectVoice(option) => {
            let lang = app.state.tts.effective_language();
            match app.state.tts_config.backend {
                TtsBackend::VieNeu => {
                    app.state.tts_config.vieneu_voice = option.id;
                }
                TtsBackend::Auto | TtsBackend::Piper => match lang {
                    TtsLanguage::En => app.state.tts_config.piper_model_en = option.id,
                    TtsLanguage::Vi => app.state.tts_config.piper_model_vi = option.id,
                    TtsLanguage::Ja => {}
                },
                TtsBackend::EspeakNg => {}
            }
            persist_tts_config(app);
            app.state.tts.invalidate_prefetch();
            if matches!(
                app.state.tts.status,
                TtsStatus::Playing | TtsStatus::Paused | TtsStatus::Synthesizing
            ) {
                play_current_segment(app)
            } else {
                Task::none()
            }
        }
        TtsMsg::RefreshPiperModels => {
            app.state.tts.installed_piper_models = installed_models_set();
            Task::none()
        }
        TtsMsg::DownloadPiperModel(model_id) => {
            if !app.state.tts.downloading_models.insert(model_id.clone()) {
                return Task::none();
            }
            app.state.tts.model_download_error = None;
            let id_for_task = model_id.clone();
            Task::perform(
                async move { download_model(&id_for_task).await },
                move |result| Message::Tts(TtsMsg::PiperModelDownloaded { model_id, result }),
            )
        }
        TtsMsg::PiperModelDownloaded { model_id, result } => {
            app.state.tts.downloading_models.remove(&model_id);
            match result {
                Ok(()) => {
                    app.state.tts.model_download_error = None;
                    app.state.tts.installed_piper_models = installed_models_set();
                }
                Err(err) => {
                    app.state.tts.model_download_error = Some(err);
                }
            }
            Task::none()
        }
        TtsMsg::DeletePiperModel(model_id) => {
            let id_for_task = model_id.clone();
            Task::perform(
                async move { delete_model(&id_for_task).await },
                move |result| Message::Tts(TtsMsg::PiperModelDeleted { model_id, result }),
            )
        }
        TtsMsg::PiperModelDeleted { model_id, result } => {
            match result {
                Ok(()) => {
                    app.state.tts.model_download_error = None;
                    app.state.tts.installed_piper_models = installed_models_set();
                }
                Err(err) => {
                    app.state.tts.model_download_error = Some(format!("Delete {model_id}: {err}"));
                }
            }
            Task::none()
        }
        TtsMsg::SelectPiperModel { lang, model_id } => {
            match lang {
                TtsLanguage::En => app.state.tts_config.piper_model_en = model_id,
                TtsLanguage::Vi => app.state.tts_config.piper_model_vi = model_id,
                TtsLanguage::Ja => {}
            }
            persist_tts_config(app);
            app.state.tts.invalidate_prefetch();
            Task::none()
        }
        TtsMsg::SetBackend(backend) => {
            app.state.tts_config.backend = backend;
            clear_offline_cooldown();
            persist_tts_config(app);
            app.state.tts.invalidate_prefetch();
            if matches!(
                app.state.tts.status,
                TtsStatus::Playing | TtsStatus::Paused | TtsStatus::Synthesizing
            ) {
                play_current_segment(app)
            } else {
                Task::none()
            }
        }
        TtsMsg::SetVieNeuEndpoint(endpoint) => {
            app.state.tts_config.vieneu_endpoint = endpoint;
            clear_offline_cooldown();
            persist_tts_config(app);
            Task::none()
        }
        TtsMsg::SetVieNeuVoice(voice) => {
            app.state.tts_config.vieneu_voice = voice;
            persist_tts_config(app);
            app.state.tts.invalidate_prefetch();
            Task::none()
        }
        TtsMsg::Synthesized {
            playback_id,
            seg_idx,
            audio,
            backend_label,
        } => {
            if playback_id != app.state.tts.playback_id || seg_idx != app.state.tts.current_index {
                return Task::none();
            }
            app.state.tts.active_backend_label = backend_label;
            start_audio_and_prefetch(app, audio)
        }
        TtsMsg::Prefetched {
            gen_id,
            seg_idx,
            audio,
            backend_label,
        } => {
            if gen_id == app.state.tts.generation_id {
                app.state.tts.prefetch_in_flight.remove(&seg_idx);
                let cur = app.state.tts.current_index;
                if seg_idx > cur && seg_idx <= cur + 2 {
                    app.state
                        .tts
                        .prefetched
                        .insert(seg_idx, (audio, backend_label));
                }
            }
            Task::none()
        }
        TtsMsg::PrefetchFailed { gen_id, seg_idx } => {
            if gen_id == app.state.tts.generation_id {
                app.state.tts.prefetch_in_flight.remove(&seg_idx);
            }
            Task::none()
        }
        TtsMsg::PlaybackEnded(playback_id) => {
            if playback_id != app.state.tts.playback_id {
                return Task::none();
            }
            if app.state.tts.current_index + 1 < app.state.tts.segments.len() {
                app.state.tts.current_index += 1;
                play_current_segment(app)
            } else {
                advance_epub_chapter_or_stop(app)
            }
        }
        TtsMsg::EngineError(playback_id, err) => {
            if playback_id != app.state.tts.playback_id {
                return Task::none();
            }
            app.tts_player = None;
            app.state.tts.cancel_eos_watcher();
            app.state.tts.status = TtsStatus::Error(err);
            Task::none()
        }
    }
}

pub fn stop_tts(app: &mut KglanceApp) {
    app.tts_player = None;
    app.state.tts.reset_playback();
    app.state.tts.segments.clear();
    app.state.tts.current_index = 0;
    app.state.tts.visible = false;
    sync_active_tts_block(app, None);
}

fn sync_active_tts_block(app: &mut KglanceApp, block_idx: Option<usize>) {
    app.state.tts.active_block = block_idx;
    app.state.markdown.active_tts_block = block_idx;
    app.state.epub.markdown_state.active_tts_block = block_idx;
}

fn start_tts_from_current_view(app: &mut KglanceApp) -> Task<Message> {
    let skip_code = app.state.tts_config.skip_code_blocks;
    let (segments, start_idx) = match &app.current_content {
        Some(PreviewData::Markdown { blocks, .. }) => {
            let segs = extract_segments(blocks, skip_code, None);
            let start = find_start_segment_from_scroll(
                &segs,
                &app.state.markdown.block_y_offsets,
                app.state.markdown.scroll_y,
            );
            (segs, start)
        }
        Some(PreviewData::Epub { .. }) => {
            let blocks = app.state.epub.active_blocks();
            let segs = extract_segments(blocks, skip_code, None);
            let start = find_start_segment_from_scroll(
                &segs,
                &app.state.epub.markdown_state.block_y_offsets,
                app.state.epub.markdown_state.scroll_y,
            );
            (segs, start)
        }
        _ => return Task::none(),
    };

    if segments.is_empty() {
        app.state.tts.visible = true;
        app.state.tts.status = TtsStatus::Error("No readable text found".to_string());
        return Task::none();
    }

    if app.state.tts.rate <= 0.0 {
        app.state.tts.rate = app.state.tts_config.rate;
    }
    app.state.tts.invalidate_prefetch();
    app.state.tts.installed_piper_models = installed_models_set();
    app.state.tts.visible = true;
    app.state.tts.segments = segments;
    app.state.tts.current_index = start_idx;
    play_current_segment(app)
}

fn advance_epub_chapter_or_stop(app: &mut KglanceApp) -> Task<Message> {
    let is_epub_chapter = matches!(app.current_content, Some(PreviewData::Epub { .. }))
        && app.state.epub.reading_mode == EpubReadingMode::SingleChapter
        && app.state.tts_config.auto_advance_chapter;

    if !is_epub_chapter {
        app.tts_player = None;
        app.state.tts.reset_playback();
        app.state.tts.current_index = 0;
        sync_active_tts_block(app, None);
        return Task::none();
    }

    let next_ch = app.state.epub.active_chapter + 1;
    if next_ch >= app.state.epub.chapters.len() {
        app.tts_player = None;
        app.state.tts.reset_playback();
        app.state.tts.current_index = 0;
        sync_active_tts_block(app, None);
        return Task::none();
    }

    crate::features::epub::update::ensure_chapter_loaded(app, next_ch);
    let skip_code = app.state.tts_config.skip_code_blocks;
    let segs = extract_segments(app.state.epub.active_blocks(), skip_code, None);
    if segs.is_empty() {
        app.tts_player = None;
        app.state.tts.reset_playback();
        sync_active_tts_block(app, None);
        return Task::none();
    }
    app.state.tts.invalidate_prefetch();
    app.state.tts.segments = segs;
    app.state.tts.current_index = 0;
    let scroll_reset = operation::scroll_to(
        "content_scroll",
        operation::AbsoluteOffset { x: 0.0, y: 0.0 },
    );
    Task::batch([scroll_reset, play_current_segment(app)])
}

fn play_current_segment(app: &mut KglanceApp) -> Task<Message> {
    app.tts_player = None;
    app.state.tts.cancel_eos_watcher();
    app.state.tts.playback_id = app.state.tts.playback_id.wrapping_add(1);
    let playback_id = app.state.tts.playback_id;
    let seg_idx = app.state.tts.current_index;

    let Some(seg) = app.state.tts.segments.get(seg_idx).cloned() else {
        app.state.tts.reset_playback();
        sync_active_tts_block(app, None);
        return Task::none();
    };

    sync_active_tts_block(app, Some(seg.block_index));
    app.state
        .tts
        .prefetched
        .retain(|&idx, _| idx >= seg_idx && idx <= seg_idx + 2);

    if let Some((audio, backend_label)) = app.state.tts.prefetched.remove(&seg_idx) {
        app.state.tts.active_backend_label = backend_label;
        return start_audio_and_prefetch(app, audio);
    }

    app.state.tts.status = TtsStatus::Synthesizing;
    let scroll_task = maybe_scroll_to_active_block(app, seg.block_index);
    let rate = if app.state.tts.rate <= 0.0 {
        app.state.tts_config.rate
    } else {
        app.state.tts.rate
    };
    let cfg = app.state.tts_config.clone();
    let effective_lang = app.state.tts.lang_selection.resolve(seg.lang);

    let synth_task = Task::perform(
        async move { synthesize_segment(&seg.text, effective_lang, rate, cfg).await },
        move |res| match res {
            Ok(out) => Message::Tts(TtsMsg::Synthesized {
                playback_id,
                seg_idx,
                audio: out.audio_bytes,
                backend_label: out.backend_label.to_string(),
            }),
            Err(err) => Message::Tts(TtsMsg::EngineError(playback_id, err)),
        },
    );

    Task::batch([scroll_task, synth_task])
}

fn start_audio_and_prefetch(app: &mut KglanceApp, audio: Vec<u8>) -> Task<Message> {
    let playback_id = app.state.tts.playback_id;
    let gen_id = app.state.tts.generation_id;
    let seg_idx = app.state.tts.current_index;
    let block_index = app.state.tts.segments.get(seg_idx).map(|s| s.block_index);

    match TtsAudioPlayer::play_bytes(audio) {
        Ok(player) => {
            let bus = player.bus();
            app.tts_player = Some(player);
            app.state.tts.status = TtsStatus::Playing;
            sync_active_tts_block(app, block_index);

            let cancel_flag = Arc::new(AtomicBool::new(false));
            app.state.tts.eos_cancel_flag = Some(cancel_flag.clone());

            let eos_task =
                Task::perform(
                    wait_for_playback_end(bus, cancel_flag),
                    move |res| match res {
                        Ok(true) => Message::Tts(TtsMsg::PlaybackEnded(playback_id)),
                        Ok(false) => Message::None,
                        Err(err) => Message::Tts(TtsMsg::EngineError(playback_id, err)),
                    },
                );

            let scroll_task = block_index
                .map(|b| maybe_scroll_to_active_block(app, b))
                .unwrap_or_else(Task::none);
            let prefetch_task = spawn_prefetch_tasks(app, gen_id, seg_idx);

            Task::batch([eos_task, scroll_task, prefetch_task])
        }
        Err(err) => {
            app.tts_player = None;
            app.state.tts.status = TtsStatus::Error(err);
            Task::none()
        }
    }
}

fn spawn_prefetch_tasks(app: &mut KglanceApp, gen_id: u64, current_idx: usize) -> Task<Message> {
    let mut tasks = Vec::new();
    let rate = if app.state.tts.rate <= 0.0 {
        app.state.tts_config.rate
    } else {
        app.state.tts.rate
    };
    let cfg = app.state.tts_config.clone();

    for offset in 1..=2 {
        let target_idx = current_idx + offset;
        if target_idx >= app.state.tts.segments.len() {
            break;
        }
        if app.state.tts.prefetched.contains_key(&target_idx)
            || app.state.tts.prefetch_in_flight.contains(&target_idx)
        {
            continue;
        }
        let Some(seg) = app.state.tts.segments.get(target_idx).cloned() else {
            continue;
        };
        app.state.tts.prefetch_in_flight.insert(target_idx);
        let c = cfg.clone();
        let effective_lang = app.state.tts.lang_selection.resolve(seg.lang);
        tasks.push(Task::perform(
            async move { synthesize_segment(&seg.text, effective_lang, rate, c).await },
            move |res| match res {
                Ok(out) => Message::Tts(TtsMsg::Prefetched {
                    gen_id,
                    seg_idx: target_idx,
                    audio: out.audio_bytes,
                    backend_label: out.backend_label.to_string(),
                }),
                Err(_) => Message::Tts(TtsMsg::PrefetchFailed {
                    gen_id,
                    seg_idx: target_idx,
                }),
            },
        ));
    }
    Task::batch(tasks)
}

fn maybe_scroll_to_active_block(app: &mut KglanceApp, block_idx: usize) -> Task<Message> {
    if !app.state.tts_config.auto_scroll {
        return Task::none();
    }

    let (md_state, threshold) = match &app.current_content {
        Some(PreviewData::Markdown { .. }) => (
            &mut app.state.markdown,
            crate::features::markdown::view::VIRTUAL_THRESHOLD,
        ),
        Some(PreviewData::Epub { .. }) => (
            &mut app.state.epub.markdown_state,
            crate::features::epub::view::content::VIRTUAL_THRESHOLD,
        ),
        _ => return Task::none(),
    };

    scroll_markdown_state_to_block(md_state, block_idx, threshold)
}

fn scroll_markdown_state_to_block(
    s: &mut MarkdownState,
    block_idx: usize,
    virtual_threshold: usize,
) -> Task<Message> {
    let Some(&raw_block_y) = s.block_y_offsets.get(block_idx) else {
        return Task::none();
    };
    let vh = s.viewport_height.max(100.0);
    // Keep block comfortably in upper-third if not already visible
    if raw_block_y >= s.scroll_y + 16.0 && raw_block_y <= s.scroll_y + vh * 0.72 {
        return Task::none();
    }

    let offset_padding = (vh * 0.22).max(36.0);
    let target_y = (raw_block_y - offset_padding).max(0.0);
    let max_y = crate::core::scroll::max_scroll_y(s.total_content_height, vh);
    let clamped_y = target_y.min(max_y);

    s.scroll_controller.stop(clamped_y);
    s.smooth_scroll
        .start_navigation(s.scroll_y, clamped_y, max_y);
    s.scroll_y = clamped_y;

    crate::features::markdown::view::window::sync_window(
        &mut s.virtual_window,
        &s.block_y_offsets,
        clamped_y,
        vh,
        virtual_threshold,
    );

    operation::scroll_to(
        "content_scroll",
        operation::AbsoluteOffset {
            x: 0.0,
            y: clamped_y,
        },
    )
}

fn persist_tts_config(app: &KglanceApp) {
    let mut cfg = app.load_config();
    cfg.tts = app.state.tts_config.clone();
    let _ = app.save_config(&cfg);
}
