use gst::prelude::*;
use std::path::Path;

pub struct AudioPlayer {
    pipeline: gst::Element,
    duration_secs: f64,
    is_paused: bool,
}

impl AudioPlayer {
    pub fn new(path: &str, duration_secs: f64) -> Result<Self, String> {
        gst::init().map_err(|e| format!("Failed to initialize GStreamer: {e}"))?;
        let abs_path =
            std::fs::canonicalize(path).unwrap_or_else(|_| Path::new(path).to_path_buf());
        let url = url::Url::from_file_path(&abs_path)
            .map_err(|_| format!("Invalid file path: {path}"))?;

        let pipeline = gst::ElementFactory::make("playbin")
            .build()
            .map_err(|e| format!("Failed to create playbin element: {e}"))?;

        pipeline.set_property("uri", url.as_str());

        let res = pipeline.set_state(gst::State::Playing);
        if res.is_err() {
            let _ = pipeline.set_state(gst::State::Null);
            return Err("Failed to start audio playback".to_string());
        }

        Ok(Self {
            pipeline,
            duration_secs,
            is_paused: false,
        })
    }

    pub fn toggle_play_pause(&mut self) -> Result<bool, String> {
        if self.is_playing() {
            self.pause()?;
            Ok(false)
        } else {
            self.play()?;
            Ok(true)
        }
    }

    pub fn play(&mut self) -> Result<(), String> {
        let pos = self.position_secs();
        let dur = self.duration_secs();
        if dur > 0.0 && pos >= dur - 0.2 {
            self.seek_to_secs(0.0);
        }
        self.is_paused = false;
        self.pipeline
            .set_state(gst::State::Playing)
            .map_err(|e| format!("Failed to play: {e:?}"))?;
        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), String> {
        self.is_paused = true;
        self.pipeline
            .set_state(gst::State::Paused)
            .map_err(|e| format!("Failed to pause: {e:?}"))?;
        Ok(())
    }

    pub fn is_playing(&self) -> bool {
        !self.is_paused
    }

    pub fn query_position_secs(&self) -> Option<f64> {
        self.pipeline
            .query_position::<gst::ClockTime>()
            .map(|t| t.nseconds() as f64 / 1_000_000_000.0)
    }

    pub fn position_secs(&self) -> f64 {
        self.query_position_secs().unwrap_or(0.0)
    }

    pub fn duration_secs(&self) -> f64 {
        let gst_dur = self
            .pipeline
            .query_duration::<gst::ClockTime>()
            .map(|t| t.nseconds() as f64 / 1_000_000_000.0)
            .unwrap_or(0.0);
        if gst_dur > 0.0 {
            gst_dur
        } else {
            self.duration_secs
        }
    }

    pub fn seek_to_ratio(&self, ratio: f64) {
        let dur = self.duration_secs();
        if dur > 0.0 {
            let target = (ratio * dur).clamp(0.0, dur);
            self.seek_to_secs(target);
        }
    }

    pub fn seek_relative(&self, delta_secs: f64) {
        let cur = self.position_secs();
        let dur = self.duration_secs();
        let target = if dur > 0.0 {
            (cur + delta_secs).clamp(0.0, dur)
        } else {
            (cur + delta_secs).max(0.0)
        };
        self.seek_to_secs(target);
    }

    pub fn seek_to_secs(&self, secs: f64) {
        let dur = self.duration_secs();
        let clamped = if dur > 0.0 {
            secs.clamp(0.0, dur)
        } else {
            secs.max(0.0)
        };
        let ns = (clamped * 1_000_000_000.0) as u64;
        let clock_time = gst::ClockTime::from_nseconds(ns);
        let res = self
            .pipeline
            .seek_simple(gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT, clock_time);
        if res.is_err() {
            let _ = self
                .pipeline
                .seek_simple(gst::SeekFlags::FLUSH | gst::SeekFlags::ACCURATE, clock_time);
        }
        if !self.is_paused {
            let _ = self.pipeline.set_state(gst::State::Playing);
        }
    }

    pub fn set_volume(&self, vol: f64) {
        self.pipeline.set_property("volume", vol.clamp(0.0, 1.5));
    }

    pub fn volume(&self) -> f64 {
        self.pipeline.property::<f64>("volume")
    }

    pub fn poll_eos(&mut self) -> bool {
        if let Some(bus) = self.pipeline.bus() {
            while let Some(msg) = bus.pop() {
                match msg.view() {
                    gst::MessageView::Eos(..) => {
                        let _ = self.pipeline.set_state(gst::State::Paused);
                        self.is_paused = true;
                        return true;
                    }
                    gst::MessageView::Error(err) => {
                        crate::log_error!(
                            "GStreamer audio error: {} ({:?})",
                            err.error(),
                            err.debug()
                        );
                        if !self.is_paused {
                            let _ = self.pipeline.set_state(gst::State::Playing);
                        }
                    }
                    gst::MessageView::Buffering(b) if b.percent() >= 100 && !self.is_paused => {
                        let _ = self.pipeline.set_state(gst::State::Playing);
                    }
                    _ => {}
                }
            }
        }
        false
    }
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        let pipeline = self.pipeline.clone();
        pipeline.set_property("volume", 0.0f64);
        if std::thread::Builder::new()
            .name("kglance-audio-teardown".into())
            .spawn(move || {
                let _ = pipeline.set_state(gst::State::Null);
            })
            .is_err()
        {
            let _ = self.pipeline.set_state(gst::State::Null);
        }
    }
}
