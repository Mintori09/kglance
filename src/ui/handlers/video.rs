use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use url::Url;

use crate::app::Message;
use crate::log_error;
use iced_futures::subscription::{self, Recipe};
use iced_video_player::Video;

#[derive(Debug, Clone)]
pub enum PlayerCommand {
    Load(String),
    Play,
    Pause,
    Seek(f64),
    SeekRelative(f64),
    Stop,
}

#[derive(Debug, Clone)]
pub enum VideoEvent {
    Progress {
        position: f64,
        duration: f64,
        is_playing: bool,
    },
}

pub struct VideoController {
    pub video: Option<Video>,
    pub is_playing: bool,
}

impl Default for VideoController {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoController {
    pub fn new() -> Self {
        Self {
            video: None,
            is_playing: false,
        }
    }

    pub fn load(&mut self, path: &str, _event_tx: mpsc::Sender<VideoEvent>) -> Result<(), String> {
        self.stop();

        let url = Url::from_file_path(path).map_err(|_| format!("invalid file path: {path}"))?;

        match Video::new(&url) {
            Ok(v) => {
                self.video = Some(v);
                self.is_playing = true;
                Ok(())
            }
            Err(e) => Err(format!("iced_video_player failed to load video: {e:?}")),
        }
    }

    pub fn play(&mut self) {
        if let Some(ref mut video) = self.video {
            video.set_paused(false);
            self.is_playing = true;
        }
    }

    pub fn pause(&mut self) {
        if let Some(ref mut video) = self.video {
            video.set_paused(true);
            self.is_playing = false;
        }
    }

    pub fn seek(&mut self, ratio: f64) {
        if let Some(ref mut video) = self.video {
            let dur = video.duration().as_secs_f64();
            let target = (ratio * dur).clamp(0.0, dur);
            let _ = video.seek(Duration::from_secs_f64(target), true);
        }
    }

    pub fn seek_relative(&mut self, secs: f64) {
        if let Some(ref mut video) = self.video {
            let cur = video.position().as_secs_f64();
            let dur = video.duration().as_secs_f64();
            let target = (cur + secs).clamp(0.0, dur);
            let _ = video.seek(Duration::from_secs_f64(target), true);
        }
    }

    pub fn stop(&mut self) {
        if let Some(mut video) = self.video.take() {
            video.set_paused(true);
        }
        self.is_playing = false;
    }
}

impl Drop for VideoController {
    fn drop(&mut self) {
        self.stop();
    }
}

pub fn spawn_video_player(
    mut cmd_rx: mpsc::Receiver<PlayerCommand>,
    event_tx: mpsc::Sender<VideoEvent>,
) -> Arc<Mutex<VideoController>> {
    let controller = Arc::new(Mutex::new(VideoController::new()));
    let ctrl = controller.clone();

    thread::spawn(move || {
        let mut last_progress = Instant::now();

        loop {
            while let Ok(cmd) = cmd_rx.try_recv() {
                if let Ok(mut c) = ctrl.lock() {
                    match cmd {
                        PlayerCommand::Load(path) => {
                            if let Err(e) = c.load(&path, event_tx.clone()) {
                                log_error!("VideoController::load failed: {e}");
                            }
                        }
                        PlayerCommand::Play => c.play(),
                        PlayerCommand::Pause => c.pause(),
                        PlayerCommand::Seek(ratio) => c.seek(ratio),
                        PlayerCommand::SeekRelative(secs) => c.seek_relative(secs),
                        PlayerCommand::Stop => c.stop(),
                    }
                }
            }

            let now = Instant::now();
            if now - last_progress >= Duration::from_millis(33) {
                last_progress = now;
                if let Ok(c) = ctrl.lock()
                    && c.is_playing
                    && let Some(ref v) = c.video
                {
                    let pos = v.position().as_secs_f64();
                    let dur = v.duration().as_secs_f64();
                    let _ = event_tx.try_send(VideoEvent::Progress {
                        position: pos,
                        duration: dur,
                        is_playing: true,
                    });
                }
            }

            thread::sleep(Duration::from_millis(10));
        }
    });

    controller
}

pub struct VideoRecipe {
    rx: Arc<Mutex<Option<mpsc::Receiver<VideoEvent>>>>,
}

impl VideoRecipe {
    pub fn new(rx: Arc<Mutex<Option<mpsc::Receiver<VideoEvent>>>>) -> Self {
        Self { rx }
    }
}

impl Recipe for VideoRecipe {
    type Output = Message;

    fn hash(&self, state: &mut subscription::Hasher) {
        "video_subscription".hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: subscription::EventStream,
    ) -> iced_futures::BoxStream<Self::Output> {
        let rx = self.rx.lock().unwrap().take();
        match rx {
            Some(mut rx) => iced_futures::boxed_stream(iced::stream::channel(
                100,
                move |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
                    use iced::futures::SinkExt;
                    while let Some(event) = rx.recv().await {
                        if output
                            .send(Message::VideoEventReceived(event))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                },
            )),
            None => iced_futures::boxed_stream(iced::futures::stream::empty()),
        }
    }
}
