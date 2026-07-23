use crate::core::config::{AppConfig, ConfigManager};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::Duration;

pub struct ConfigWatcher {
    _watcher: RecommendedWatcher,
    pub rx: Receiver<AppConfig>,
}

impl ConfigWatcher {
    pub fn new() -> Result<Self, String> {
        let (tx_event, rx_event) = channel();
        let (tx_config, rx_config) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx_event.send(event);
                }
            },
            Config::default(),
        )
        .map_err(|e| e.to_string())?;

        let config_dir = ConfigManager::get_config_dir();
        if config_dir.exists() {
            let _ = watcher.watch(&config_dir, RecursiveMode::NonRecursive);
        }

        thread::spawn(move || {
            let mut last_config = ConfigManager::load_or_create();
            while let Ok(event) = rx_event.recv() {
                if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                    thread::sleep(Duration::from_millis(100));
                    let new_config = ConfigManager::load_or_create();
                    if new_config != last_config {
                        last_config = new_config.clone();
                        let _ = tx_config.send(new_config);
                    }
                }
            }
        });

        Ok(Self {
            _watcher: watcher,
            rx: rx_config,
        })
    }
}
