use iced_futures::subscription;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::{app::Message, log_error};

#[derive(Debug, Clone)]
pub enum WatchCommand {
    Watch(PathBuf),
    Unwatch,
}

pub struct FileWatcher {
    pub cmd_tx: Sender<WatchCommand>,
    pub events: Arc<Mutex<Option<Receiver<PathBuf>>>>,
}

impl FileWatcher {
    pub fn new() -> Result<Self, String> {
        let (tx_notify, rx_notify) = mpsc::channel();
        let (tx_result, rx_result) = mpsc::channel();
        let (cmd_tx, cmd_rx) = mpsc::channel();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx_notify.send(event);
                }
            },
            Config::default(),
        )
        .map_err(|e| e.to_string())?;

        thread::spawn(move || {
            let mut current_path: Option<PathBuf> = None;
            let mut last_change: Option<Instant> = None;

            loop {
                if let Ok(cmd) = cmd_rx.try_recv() {
                    match cmd {
                        WatchCommand::Watch(path) => {
                            if current_path.as_ref() == Some(&path) {
                                continue;
                            }
                            if let Some(ref old) = current_path
                                && let Some(parent) = old.parent()
                            {
                                let _ = watcher.unwatch(parent);
                            }
                            if let Some(parent) = path.parent()
                                && let Err(e) = watcher.watch(parent, RecursiveMode::NonRecursive)
                            {
                                log_error!(
                                    "FileWatcher: failed to watch {}: {}",
                                    parent.display(),
                                    e
                                );
                            }
                            current_path = Some(path);
                            last_change = None;
                        }
                        WatchCommand::Unwatch => {
                            if let Some(ref old) = current_path
                                && let Some(parent) = old.parent()
                            {
                                let _ = watcher.unwatch(parent);
                            }
                            current_path = None;
                            last_change = None;
                        }
                    }
                }

                match rx_notify.recv_timeout(Duration::from_millis(200)) {
                    Ok(event) => {
                        if let Some(ref watched) = current_path {
                            let is_relevant =
                                matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_))
                                    && event.paths.iter().any(|p| p == watched);

                            if is_relevant {
                                last_change = Some(Instant::now());
                            }
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        if let Some(ref t) = last_change
                            && t.elapsed() >= Duration::from_millis(300)
                            && let Some(ref path) = current_path
                        {
                            let _ = tx_result.send(path.clone());
                            last_change = None;
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        });

        Ok(Self {
            cmd_tx,
            events: Arc::new(Mutex::new(Some(rx_result))),
        })
    }
}

pub struct FileWatcherRecipe {
    events: Arc<Mutex<Option<Receiver<PathBuf>>>>,
}

impl FileWatcherRecipe {
    pub fn new(events: Arc<Mutex<Option<Receiver<PathBuf>>>>) -> Self {
        Self { events }
    }
}

impl subscription::Recipe for FileWatcherRecipe {
    type Output = Message;

    fn hash(&self, state: &mut subscription::Hasher) {
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: subscription::EventStream,
    ) -> iced_futures::BoxStream<Self::Output> {
        let rx = self.events.lock().unwrap().take();
        match rx {
            Some(rx) => iced_futures::boxed_stream(iced::stream::channel(
                100,
                move |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
                    use iced::futures::SinkExt;
                    loop {
                        match rx.try_recv() {
                            Ok(path) => {
                                let _ = output
                                    .send(Message::FileChanged(path.to_string_lossy().to_string()))
                                    .await;
                            }
                            Err(mpsc::TryRecvError::Empty) => {
                                tokio::time::sleep(Duration::from_millis(100)).await;
                            }
                            Err(mpsc::TryRecvError::Disconnected) => break,
                        }
                    }
                },
            )),
            None => iced_futures::boxed_stream(iced::futures::stream::empty()),
        }
    }
}
