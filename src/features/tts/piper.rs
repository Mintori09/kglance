use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Mutex, OnceLock};

use ort::session::Session;
use ort::value::Tensor;
use serde::Deserialize;
use unicode_normalization::UnicodeNormalization;

use crate::features::tts::types::TtsLanguage;

const BOS: char = '^';
const EOS: char = '$';
const PAD: char = '_';

#[derive(Deserialize)]
struct AudioConfig {
    sample_rate: u32,
}

#[derive(Deserialize)]
struct ESpeakConfig {
    voice: String,
}

#[derive(Deserialize, Clone)]
struct InferenceConfig {
    noise_scale: f32,
    length_scale: f32,
    noise_w: f32,
}

#[derive(Deserialize)]
struct ModelConfig {
    audio: AudioConfig,
    espeak: ESpeakConfig,
    inference: InferenceConfig,
    num_speakers: u32,
    phoneme_id_map: HashMap<char, Vec<i64>>,
}

struct LoadedPiperModel {
    config: ModelConfig,
    session: Session,
}

const HF_PIPER_VOICES_BASE: &str = "https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0";

/// Metadata for a downloadable Piper ONNX voice model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PiperModelSpec {
    pub id: &'static str,
    pub display_name: &'static str,
    pub lang: TtsLanguage,
    pub quality: &'static str,
    pub approx_size: &'static str,
    pub hf_subpath: &'static str,
}

pub const PIPER_MODEL_CATALOG: &[PiperModelSpec] = &[
    PiperModelSpec {
        id: "vi_VN-vais1000-medium",
        display_name: "Vietnamese — VAIS1000",
        lang: TtsLanguage::Vi,
        quality: "Medium",
        approx_size: "~63 MB",
        hf_subpath: "vi/vi_VN/vais1000/medium/vi_VN-vais1000-medium",
    },
    PiperModelSpec {
        id: "vi_VN-25hours_single-low",
        display_name: "Vietnamese — 25h Single",
        lang: TtsLanguage::Vi,
        quality: "Low",
        approx_size: "~30 MB",
        hf_subpath: "vi/vi_VN/25hours_single/low/vi_VN-25hours_single-low",
    },
    PiperModelSpec {
        id: "en_US-amy-medium",
        display_name: "English (US) — Amy",
        lang: TtsLanguage::En,
        quality: "Medium",
        approx_size: "~63 MB",
        hf_subpath: "en/en_US/amy/medium/en_US-amy-medium",
    },
    PiperModelSpec {
        id: "en_US-lessac-medium",
        display_name: "English (US) — Lessac",
        lang: TtsLanguage::En,
        quality: "Medium",
        approx_size: "~63 MB",
        hf_subpath: "en/en_US/lessac/medium/en_US-lessac-medium",
    },
    PiperModelSpec {
        id: "en_US-ryan-medium",
        display_name: "English (US) — Ryan",
        lang: TtsLanguage::En,
        quality: "Medium",
        approx_size: "~63 MB",
        hf_subpath: "en/en_US/ryan/medium/en_US-ryan-medium",
    },
];

pub fn find_model_spec(model_id: &str) -> Option<&'static PiperModelSpec> {
    PIPER_MODEL_CATALOG.iter().find(|spec| spec.id == model_id)
}

pub fn models_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("kglance")
        .join("models")
        .join("piper")
}

pub fn model_paths_in(dir: &Path, model_id: &str) -> (PathBuf, PathBuf) {
    let onnx = dir.join(format!("{model_id}.onnx"));
    let json = dir.join(format!("{model_id}.onnx.json"));
    (onnx, json)
}

pub fn is_model_installed(model_id: &str) -> bool {
    let (onnx, json) = model_paths_in(&models_dir(), model_id);
    file_non_empty(&onnx) && file_non_empty(&json)
}

pub fn installed_models_set() -> HashSet<String> {
    PIPER_MODEL_CATALOG
        .iter()
        .filter(|spec| is_model_installed(spec.id))
        .map(|spec| spec.id.to_string())
        .collect()
}

fn file_non_empty(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|m| m.is_file() && m.len() > 0)
        .unwrap_or(false)
}

fn download_lock() -> &'static tokio::sync::Mutex<()> {
    static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
}

fn piper_session_cache() -> &'static Mutex<HashMap<String, LoadedPiperModel>> {
    static CACHE: OnceLock<Mutex<HashMap<String, LoadedPiperModel>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Download a Piper `.onnx` model and `.onnx.json` config atomically into `models_dir()`.
pub async fn download_model(model_id: &str) -> Result<(), String> {
    let spec =
        find_model_spec(model_id).ok_or_else(|| format!("Unknown Piper model: {model_id}"))?;

    let _guard = download_lock().lock().await;
    let dir = models_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create Piper model directory: {e}"))?;

    let (onnx_path, json_path) = model_paths_in(&dir, spec.id);
    if file_non_empty(&onnx_path) && file_non_empty(&json_path) {
        return Ok(());
    }

    let onnx_url = format!("{HF_PIPER_VOICES_BASE}/{}.onnx", spec.hf_subpath);
    let json_url = format!("{HF_PIPER_VOICES_BASE}/{}.onnx.json", spec.hf_subpath);

    download_file_atomic(&json_url, &json_path).await?;
    if let Err(err) = download_file_atomic(&onnx_url, &onnx_path).await {
        let _ = std::fs::remove_file(&json_path);
        return Err(err);
    }

    Ok(())
}

async fn download_file_atomic(url: &str, dest: &Path) -> Result<(), String> {
    let part_path = dest.with_extension("part");
    let _ = std::fs::remove_file(&part_path);

    let output = tokio::process::Command::new("curl")
        .args([
            "-sSL",
            "--fail",
            "--connect-timeout",
            "10",
            "--max-time",
            "300",
            "-o",
        ])
        .arg(&part_path)
        .arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|e| format!("Failed to invoke curl: {e}"))?;

    if !output.status.success() {
        let _ = std::fs::remove_file(&part_path);
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Download failed ({url}): {}", stderr.trim()));
    }

    let meta =
        std::fs::metadata(&part_path).map_err(|e| format!("Downloaded file missing: {e}"))?;
    if meta.len() == 0 {
        let _ = std::fs::remove_file(&part_path);
        return Err(format!("Downloaded file is empty: {url}"));
    }

    std::fs::rename(&part_path, dest).map_err(|e| format!("Failed to finalize model file: {e}"))?;
    Ok(())
}

/// Remove a downloaded Piper model and evict any cached ONNX session.
pub async fn delete_model(model_id: &str) -> Result<(), String> {
    if let Ok(mut cache) = piper_session_cache().lock() {
        cache.remove(model_id);
    }
    let (onnx_path, json_path) = model_paths_in(&models_dir(), model_id);
    for path in [onnx_path, json_path] {
        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| format!("Failed to remove {}: {e}", path.display()))?;
        }
    }
    Ok(())
}

fn phonemes_to_ids(config: &ModelConfig, phonemes: &str) -> Vec<i64> {
    let map = &config.phoneme_id_map;
    let pad_id = *map.get(&PAD).and_then(|v| v.first()).unwrap_or(&0);
    let bos_id = *map.get(&BOS).and_then(|v| v.first()).unwrap_or(&0);
    let eos_id = *map.get(&EOS).and_then(|v| v.first()).unwrap_or(&0);

    let mut ids = Vec::with_capacity((phonemes.len() + 1) * 2);
    ids.push(bos_id);
    for ch in phonemes.nfd() {
        if let Some(id) = map.get(&ch).and_then(|v| v.first()) {
            ids.push(*id);
            ids.push(pad_id);
        }
    }
    ids.push(eos_id);
    ids
}

fn text_to_ipa(text: &str, espeak_voice: &str) -> Result<String, String> {
    use std::io::Write;
    let mut child = std::process::Command::new("espeak-ng")
        .args(["--ipa", "-q", "-v", espeak_voice, "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to launch espeak-ng for phonemization: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(text.as_bytes())
            .map_err(|e| format!("Failed writing text to espeak-ng: {e}"))?;
        drop(stdin);
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("espeak-ng phonemization failed: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("espeak-ng --ipa failed: {}", stderr.trim()));
    }
    let ipa = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    Ok(ipa)
}

fn run_piper_inference(
    model: &mut LoadedPiperModel,
    phonemes: &str,
    rate: f32,
) -> Result<Vec<f32>, String> {
    let ids = phonemes_to_ids(&model.config, phonemes);
    let input_len = ids.len();
    let inf = &model.config.inference;
    let length_scale = inf.length_scale / rate.clamp(0.5, 2.5);

    let input_t = Tensor::<i64>::from_array(([1, input_len], ids.into_boxed_slice()))
        .map_err(|e| format!("Failed to build input tensor: {e}"))?;
    let lengths_t = Tensor::<i64>::from_array(([1], vec![input_len as i64].into_boxed_slice()))
        .map_err(|e| format!("Failed to build lengths tensor: {e}"))?;
    let scales_t = Tensor::<f32>::from_array((
        [3],
        vec![inf.noise_scale, length_scale, inf.noise_w].into_boxed_slice(),
    ))
    .map_err(|e| format!("Failed to build scales tensor: {e}"))?;

    let outputs = if model.config.num_speakers > 1 {
        let sid_t = Tensor::<i64>::from_array(([1], vec![0i64].into_boxed_slice()))
            .map_err(|e| format!("Failed to build speaker tensor: {e}"))?;
        model
            .session
            .run(ort::inputs![input_t, lengths_t, scales_t, sid_t])
    } else {
        model
            .session
            .run(ort::inputs![input_t, lengths_t, scales_t])
    }
    .map_err(|e| format!("Piper ONNX inference failed: {e}"))?;

    let (_, audio) = outputs[0]
        .try_extract_tensor::<f32>()
        .map_err(|e| format!("Failed to extract Piper audio tensor: {e}"))?;
    Ok(audio.to_vec())
}

/// Synthesize speech into WAV bytes using `ort` and a cached Piper ONNX session.
pub async fn synthesize_with_piper_rs(
    text: &str,
    model_id: &str,
    rate: f32,
) -> Result<Vec<u8>, String> {
    let model_id = model_id.trim().to_string();
    if model_id.is_empty() {
        return Err("No Piper model configured".to_string());
    }

    let (onnx_path, json_path) = model_paths_in(&models_dir(), &model_id);
    if !file_non_empty(&onnx_path) || !file_non_empty(&json_path) {
        return Err(format!(
            "Piper model '{model_id}' is not downloaded (install in Settings > TTS)"
        ));
    }

    let text = text.to_string();

    tokio::task::spawn_blocking(move || {
        let mut cache = piper_session_cache()
            .lock()
            .map_err(|_| "Piper session cache lock poisoned".to_string())?;

        if !cache.contains_key(&model_id) {
            let file = File::open(&json_path)
                .map_err(|e| format!("Failed to open {}: {e}", json_path.display()))?;
            let config: ModelConfig = serde_json::from_reader(file)
                .map_err(|e| format!("Failed to parse {}: {e}", json_path.display()))?;
            let session = Session::builder()
                .map_err(|e| format!("Failed to create ONNX session builder: {e}"))?
                .commit_from_file(&onnx_path)
                .map_err(|e| format!("Failed to load ONNX model '{}': {e}", onnx_path.display()))?;
            cache.insert(model_id.clone(), LoadedPiperModel { config, session });
        }

        let model = cache
            .get_mut(&model_id)
            .ok_or_else(|| format!("Piper model '{model_id}' missing from cache"))?;

        let ipa = text_to_ipa(&text, &model.config.espeak.voice)?;
        let samples = run_piper_inference(model, &ipa, rate)?;

        if samples.is_empty() {
            return Err("Piper returned empty audio samples".to_string());
        }

        Ok(encode_pcm_f32_to_wav(
            &samples,
            model.config.audio.sample_rate,
        ))
    })
    .await
    .map_err(|e| format!("Piper worker join error: {e}"))?
}

/// Encode mono `f32` PCM samples in `[-1.0, 1.0]` to a 16-bit PCM RIFF/WAVE byte buffer.
pub fn encode_pcm_f32_to_wav(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let data_bytes = (samples.len() * 2) as u32;
    let riff_size = 36 + data_bytes;
    let byte_rate = sample_rate * 2;
    let mut wav = Vec::with_capacity(44 + data_bytes as usize);

    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&riff_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // PCM chunk size
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM format = 1
    wav.extend_from_slice(&1u16.to_le_bytes()); // Mono
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes()); // Block align
    wav.extend_from_slice(&16u16.to_le_bytes()); // Bits per sample
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_bytes.to_le_bytes());

    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let pcm = (clamped * i16::MAX as f32).round() as i16;
        wav.extend_from_slice(&pcm.to_le_bytes());
    }
    wav
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_pcm_f32_samples_into_valid_wav_header() {
        let wav = encode_pcm_f32_to_wav(&[0.0, 0.5, -0.5, 1.5], 22050);
        assert_eq!(wav.len(), 44 + 8);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        assert_eq!(&wav[36..40], b"data");
        let data_size = u32::from_le_bytes([wav[40], wav[41], wav[42], wav[43]]);
        assert_eq!(data_size, 8);
        let sample_rate = u32::from_le_bytes([wav[24], wav[25], wav[26], wav[27]]);
        assert_eq!(sample_rate, 22050);
    }

    #[test]
    fn catalog_contains_expected_vi_and_en_models() {
        assert!(find_model_spec("vi_VN-vais1000-medium").is_some());
        assert!(find_model_spec("en_US-amy-medium").is_some());
        assert!(find_model_spec("missing-model").is_none());
    }
}
