use gst::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct TtsAudioPlayer {
    pipeline: gst::Pipeline,
    bus: gst::Bus,
}

impl TtsAudioPlayer {
    pub fn play_bytes(bytes: Vec<u8>) -> Result<Self, String> {
        gst::init().map_err(|e| format!("Failed to initialize GStreamer: {e}"))?;

        let pipeline = gst::Pipeline::new();
        let appsrc = gst_app::AppSrc::builder().name("tts-src").build();
        appsrc.set_stream_type(gst_app::AppStreamType::Stream);
        appsrc.set_format(gst::Format::Bytes);

        let decodebin = gst::ElementFactory::make("decodebin")
            .build()
            .map_err(|e| format!("Failed to create decodebin: {e}"))?;
        let convert = gst::ElementFactory::make("audioconvert")
            .build()
            .map_err(|e| format!("Failed to create audioconvert: {e}"))?;
        let resample = gst::ElementFactory::make("audioresample")
            .build()
            .map_err(|e| format!("Failed to create audioresample: {e}"))?;
        let sink = gst::ElementFactory::make("autoaudiosink")
            .build()
            .map_err(|e| format!("Failed to create autoaudiosink: {e}"))?;

        pipeline
            .add_many([appsrc.upcast_ref(), &decodebin, &convert, &resample, &sink])
            .map_err(|e| format!("Failed to assemble TTS pipeline: {e}"))?;
        gst::Element::link_many([appsrc.upcast_ref(), &decodebin])
            .map_err(|e| format!("Failed to link appsrc -> decodebin: {e}"))?;
        gst::Element::link_many([&convert, &resample, &sink])
            .map_err(|e| format!("Failed to link audio output chain: {e}"))?;

        let convert_weak = convert.downgrade();
        decodebin.connect_pad_added(move |_dbin, src_pad| {
            let Some(convert) = convert_weak.upgrade() else {
                return;
            };
            let caps = src_pad
                .current_caps()
                .unwrap_or_else(|| src_pad.query_caps(None));
            let is_audio = caps
                .structure(0)
                .is_some_and(|s| s.name().as_str().starts_with("audio/"));
            if !is_audio {
                return;
            }
            if let Some(sink_pad) = convert.static_pad("sink")
                && !sink_pad.is_linked()
            {
                let _ = src_pad.link(&sink_pad);
            }
        });

        let bus = pipeline
            .bus()
            .ok_or_else(|| "TTS GStreamer pipeline missing bus".to_string())?;

        pipeline
            .set_state(gst::State::Playing)
            .map_err(|e| format!("Failed to start TTS playback: {e:?}"))?;

        let buffer = gst::Buffer::from_slice(bytes);
        appsrc
            .push_buffer(buffer)
            .map_err(|e| format!("Failed to push audio buffer: {e:?}"))?;
        appsrc
            .end_of_stream()
            .map_err(|e| format!("Failed to signal end-of-stream: {e:?}"))?;

        Ok(Self { pipeline, bus })
    }

    pub fn pause(&mut self) -> Result<(), String> {
        self.pipeline
            .set_state(gst::State::Paused)
            .map_err(|e| format!("Failed to pause TTS: {e:?}"))?;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), String> {
        self.pipeline
            .set_state(gst::State::Playing)
            .map_err(|e| format!("Failed to resume TTS: {e:?}"))?;
        Ok(())
    }

    pub fn bus(&self) -> gst::Bus {
        self.bus.clone()
    }
}

impl Drop for TtsAudioPlayer {
    fn drop(&mut self) {
        let pipeline = self.pipeline.clone();
        let _ = std::thread::Builder::new()
            .name("kglance-tts-teardown".to_string())
            .spawn(move || {
                let _ = pipeline.set_state(gst::State::Null);
            });
    }
}

/// Wait on the GStreamer bus in a blocking worker until EOS, Error, or `cancel_flag`.
pub async fn wait_for_playback_end(
    bus: gst::Bus,
    cancel_flag: Arc<AtomicBool>,
) -> Result<bool, String> {
    tokio::task::spawn_blocking(move || {
        let timeout = gst::ClockTime::from_mseconds(100);
        loop {
            if cancel_flag.load(Ordering::Relaxed) {
                return Ok(false);
            }
            if let Some(msg) = bus.timed_pop_filtered(
                Some(timeout),
                &[gst::MessageType::Eos, gst::MessageType::Error],
            ) {
                match msg.view() {
                    gst::MessageView::Eos(_) => return Ok(true),
                    gst::MessageView::Error(err) => {
                        return Err(format!("TTS audio error: {}", err.error()));
                    }
                    _ => {}
                }
            }
        }
    })
    .await
    .map_err(|e| format!("TTS bus watcher join error: {e}"))?
}
