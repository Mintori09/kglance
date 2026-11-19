use std::process::Stdio;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use tokio::io::AsyncWriteExt;

const VIENEU_OFFLINE_COOLDOWN: Duration = Duration::from_secs(30);

fn offline_cooldown() -> &'static Mutex<Option<Instant>> {
    static COOLDOWN: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();
    COOLDOWN.get_or_init(|| Mutex::new(None))
}

fn is_in_cooldown() -> bool {
    offline_cooldown()
        .lock()
        .ok()
        .and_then(|g| *g)
        .is_some_and(|until| Instant::now() < until)
}

fn mark_offline_cooldown() {
    if let Ok(mut guard) = offline_cooldown().lock() {
        *guard = Some(Instant::now() + VIENEU_OFFLINE_COOLDOWN);
    }
}

pub fn clear_offline_cooldown() {
    if let Ok(mut guard) = offline_cooldown().lock() {
        *guard = None;
    }
}

/// Build the OpenAI-compatible `/v1/audio/speech` JSON payload for VieNeu-TTS.
pub fn build_speech_payload(text: &str, voice: &str, rate: f32) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "model": "vieneu-tts",
        "input": text,
        "speed": rate.clamp(0.5, 2.0),
        "response_format": "wav",
    });
    let trimmed_voice = voice.trim();
    if !trimmed_voice.is_empty()
        && let Some(obj) = payload.as_object_mut()
    {
        obj.insert(
            "voice".to_string(),
            serde_json::Value::String(trimmed_voice.to_string()),
        );
    }
    payload
}

/// Synthesize speech via a local VieNeu-TTS OpenAI-compatible HTTP endpoint.
///
/// When `respect_cooldown` is true (Auto fallback mode), connection failures
/// trigger a 30s cooldown so subsequent segments fall through immediately.
pub async fn synthesize_vieneu(
    text: &str,
    endpoint: &str,
    voice: &str,
    rate: f32,
    respect_cooldown: bool,
) -> Result<Vec<u8>, String> {
    let endpoint = endpoint.trim();
    if !(endpoint.starts_with("http://") || endpoint.starts_with("https://")) {
        return Err("VieNeu-TTS endpoint must start with http:// or https://".to_string());
    }
    if respect_cooldown && is_in_cooldown() {
        return Err("VieNeu-TTS local server in offline cooldown".to_string());
    }

    let payload = build_speech_payload(text, voice, rate);
    let body = serde_json::to_vec(&payload)
        .map_err(|e| format!("Failed to encode VieNeu-TTS payload: {e}"))?;

    let connect_timeout = if respect_cooldown { "1" } else { "3" };
    let mut child = tokio::process::Command::new("curl")
        .args([
            "-sS",
            "--fail-with-body",
            "--connect-timeout",
            connect_timeout,
            "--max-time",
            "25",
            "-X",
            "POST",
            "-H",
            "Content-Type: application/json",
            "--data-binary",
            "@-",
            endpoint,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("Failed to invoke curl for VieNeu-TTS: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(&body)
            .await
            .map_err(|e| format!("Failed writing VieNeu-TTS request body: {e}"))?;
        drop(stdin);
    }

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| format!("VieNeu-TTS request error: {e}"))?;

    if !output.status.success() {
        if respect_cooldown && matches!(output.status.code(), Some(7) | Some(28)) {
            mark_offline_cooldown();
        }
        let err_msg = if output.stderr.is_empty() {
            String::from_utf8_lossy(&output.stdout).into_owned()
        } else {
            String::from_utf8_lossy(&output.stderr).into_owned()
        };
        return Err(format!("VieNeu-TTS request failed: {}", err_msg.trim()));
    }

    clear_offline_cooldown();
    if output.stdout.len() <= 44 {
        return Err("VieNeu-TTS returned empty audio".to_string());
    }
    Ok(output.stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_openai_compatible_payload_with_optional_voice() {
        let without_voice = build_speech_payload("Xin chào", "", 1.2);
        assert_eq!(without_voice["model"], "vieneu-tts");
        assert_eq!(without_voice["input"], "Xin chào");
        assert_eq!(without_voice["response_format"], "wav");
        assert!(without_voice.get("voice").is_none());

        let with_voice = build_speech_payload("Hello", "Binh", 1.0);
        assert_eq!(with_voice["voice"], "Binh");
    }
}
