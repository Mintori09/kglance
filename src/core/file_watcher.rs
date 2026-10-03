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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchedEvent {
    ActiveFileModified(PathBuf),
    ActiveFileDeleted(PathBuf),
    DirectoryChanged(PathBuf),
}

pub struct FileWatcher {
    pub cmd_tx: Sender<WatchCommand>,
    pub events: Arc<Mutex<Option<Receiver<WatchedEvent>>>>,
}

fn is_ignored_path(path: &std::path::Path) -> bool {
    for component in path.components() {
        if let std::path::Component::Normal(name) = component {
            let s = name.to_string_lossy();
            if s == ".git"
                || s == ".svn"
                || s == ".hg"
                || s == "node_modules"
                || s == "target"
                || s == "build"
                || s == "dist"
                || s == "__pycache__"
                || s == ".venv"
                || s == "venv"
            {
                return true;
            }
        }
    }
    false
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
            let mut last_file_change: Option<Instant> = None;
            let mut last_dir_change: Option<Instant> = None;

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
                            last_file_change = None;
                            last_dir_change = None;
                        }
                        WatchCommand::Unwatch => {
                            if let Some(ref old) = current_path
                                && let Some(parent) = old.parent()
                            {
                                let _ = watcher.unwatch(parent);
                            }
                            current_path = None;
                            last_file_change = None;
                            last_dir_change = None;
                        }
                    }
                }

                match rx_notify.recv_timeout(Duration::from_millis(50)) {
                    Ok(event) => {
                        if let Some(ref watched) = current_path
                            && let Some(parent) = watched.parent()
                        {
                            let is_active_remove = matches!(event.kind, EventKind::Remove(_))
                                && event.paths.iter().any(|p| p == watched);

                            if is_active_remove {
                                let _ = tx_result
                                    .send(WatchedEvent::ActiveFileDeleted(watched.clone()));
                                last_file_change = None;
                                last_dir_change = Some(Instant::now());
                                continue;
                            }

                            let is_active_modified =
                                matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_))
                                    && event.paths.iter().any(|p| p == watched);

                            if is_active_modified {
                                last_file_change = Some(Instant::now());
                            }

                            let is_dir_membership_change = matches!(
                                event.kind,
                                EventKind::Create(_)
                                    | EventKind::Remove(_)
                                    | EventKind::Modify(notify::event::ModifyKind::Name(_))
                            ) && event
                                .paths
                                .iter()
                                .any(|p| p.parent() == Some(parent) && !is_ignored_path(p));

                            if is_dir_membership_change {
                                last_dir_change = Some(Instant::now());
                            }
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }

                // Debounce active file modified (150ms)
                if let Some(ref t) = last_file_change
                    && t.elapsed() >= Duration::from_millis(150)
                {
                    if let Some(ref path) = current_path {
                        let _ = tx_result.send(WatchedEvent::ActiveFileModified(path.clone()));
                    }
                    last_file_change = None;
                }

                // Debounce directory changes (250ms)
                if let Some(ref t) = last_dir_change
                    && t.elapsed() >= Duration::from_millis(250)
                {
                    if let Some(ref path) = current_path
                        && let Some(parent) = path.parent()
                    {
                        let _ =
                            tx_result.send(WatchedEvent::DirectoryChanged(parent.to_path_buf()));
                    }
                    last_dir_change = None;
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
    events: Arc<Mutex<Option<Receiver<WatchedEvent>>>>,
}

impl FileWatcherRecipe {
    pub fn new(events: Arc<Mutex<Option<Receiver<WatchedEvent>>>>) -> Self {
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
                            Ok(event) => {
                                let msg = match event {
                                    WatchedEvent::ActiveFileModified(path) => {
                                        crate::app::messages::SystemMsg::ActiveFileModified(
                                            path.to_string_lossy().to_string(),
                                        )
                                    }
                                    WatchedEvent::ActiveFileDeleted(path) => {
                                        crate::app::messages::SystemMsg::ActiveFileDeleted(
                                            path.to_string_lossy().to_string(),
                                        )
                                    }
                                    WatchedEvent::DirectoryChanged(dir) => {
                                        crate::app::messages::SystemMsg::DirectoryChanged(dir)
                                    }
                                };
                                let _ = output.send(msg.into()).await;
                            }
                            Err(mpsc::TryRecvError::Empty) => {
                                tokio::time::sleep(Duration::from_millis(50)).await;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_is_ignored_path() {
        assert!(is_ignored_path(Path::new("/home/user/project/.git/HEAD")));
        assert!(is_ignored_path(Path::new(
            "/home/user/project/node_modules/pkg/index.js"
        )));
        assert!(is_ignored_path(Path::new(
            "/home/user/project/target/debug/app"
        )));
        assert!(is_ignored_path(Path::new(
            "/home/user/project/__pycache__/mod.pyc"
        )));
        assert!(is_ignored_path(Path::new(
            "/home/user/project/.venv/bin/python"
        )));

        assert!(!is_ignored_path(Path::new("/tmp/test.md")));
        assert!(!is_ignored_path(Path::new("/home/user/docs/readme.txt")));
    }

    #[test]
    fn test_watched_event_equality() {
        let p1 = PathBuf::from("/tmp/test.md");
        let p2 = PathBuf::from("/tmp/test.md");
        assert_eq!(
            WatchedEvent::ActiveFileModified(p1.clone()),
            WatchedEvent::ActiveFileModified(p2.clone())
        );
        assert_ne!(
            WatchedEvent::ActiveFileModified(p1.clone()),
            WatchedEvent::ActiveFileDeleted(p2)
        );
    }
}
