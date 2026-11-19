use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TtsLanguage {
    #[default]
    En,
    Vi,
    Ja,
}

impl TtsLanguage {
    pub fn short_label(self) -> &'static str {
        match self {
            Self::En => "EN",
            Self::Vi => "VI",
            Self::Ja => "JA",
        }
    }
}

/// User-facing language selector in the TTS mini-player (`Auto` or forced language).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TtsLangSelection {
    #[default]
    Auto,
    En,
    Vi,
    Ja,
}

impl TtsLangSelection {
    pub const ALL: [Self; 4] = [Self::Auto, Self::En, Self::Vi, Self::Ja];

    pub fn resolve(self, detected: TtsLanguage) -> TtsLanguage {
        match self {
            Self::Auto => detected,
            Self::En => TtsLanguage::En,
            Self::Vi => TtsLanguage::Vi,
            Self::Ja => TtsLanguage::Ja,
        }
    }
}

impl fmt::Display for TtsLangSelection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Auto => "Auto",
            Self::En => "EN",
            Self::Vi => "VI",
            Self::Ja => "JA",
        };
        f.write_str(label)
    }
}

/// Selectable TTS voice entry displayed in the mini-player voice dropdown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TtsVoiceOption {
    pub id: String,
    pub label: String,
}

impl fmt::Display for TtsVoiceOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TtsSegment {
    pub block_index: usize,
    pub text: String,
    pub lang: TtsLanguage,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum TtsStatus {
    #[default]
    Idle,
    Synthesizing,
    Playing,
    Paused,
    Error(String),
}

#[derive(Debug, Clone, Default)]
pub struct TtsState {
    pub visible: bool,
    pub status: TtsStatus,
    /// Incremented when session, voice, rate, or document/chapter changes (invalidates prefetch cache).
    pub generation_id: u64,
    /// Incremented on every segment playback start (invalidates stale Synthesized/PlaybackEnded).
    pub playback_id: u64,
    pub segments: Vec<TtsSegment>,
    pub current_index: usize,
    pub active_block: Option<usize>,
    pub rate: f32,
    pub lang_selection: TtsLangSelection,
    pub active_backend_label: String,
    pub prefetched: HashMap<usize, (Vec<u8>, String)>,
    pub prefetch_in_flight: HashSet<usize>,
    pub eos_cancel_flag: Option<Arc<AtomicBool>>,
    pub installed_piper_models: HashSet<String>,
    pub downloading_models: HashSet<String>,
    pub model_download_error: Option<String>,
}

impl TtsState {
    pub fn is_active(&self) -> bool {
        self.visible || !matches!(self.status, TtsStatus::Idle) || self.active_block.is_some()
    }

    pub fn current_segment(&self) -> Option<&TtsSegment> {
        self.segments.get(self.current_index)
    }

    pub fn effective_language(&self) -> TtsLanguage {
        let detected = self
            .current_segment()
            .map(|s| s.lang)
            .unwrap_or(TtsLanguage::En);
        self.lang_selection.resolve(detected)
    }

    pub fn cancel_eos_watcher(&mut self) {
        if let Some(flag) = self.eos_cancel_flag.take() {
            flag.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }

    pub fn invalidate_prefetch(&mut self) {
        self.generation_id = self.generation_id.wrapping_add(1);
        self.prefetched.clear();
        self.prefetch_in_flight.clear();
    }

    pub fn reset_playback(&mut self) {
        self.cancel_eos_watcher();
        self.invalidate_prefetch();
        self.playback_id = self.playback_id.wrapping_add(1);
        self.status = TtsStatus::Idle;
        self.active_block = None;
        self.active_backend_label.clear();
    }
}
