use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;

const MAX_ENTRIES: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReadPosition {
    pub scroll_y: f32,
    pub chapter: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ReadPositionsFile {
    positions: HashMap<String, ReadPosition>,
}

#[derive(Debug, Default)]
pub struct ReadPositions {
    map: HashMap<String, ReadPosition>,
    order: VecDeque<String>,
}

impl ReadPositions {
    pub fn load() -> Self {
        let mut cache = ReadPositions::default();
        if let Ok(json) = std::fs::read_to_string(Self::file_path())
            && let Ok(file) = serde_json::from_str::<ReadPositionsFile>(&json)
        {
            for (path, pos) in file.positions {
                if pos.scroll_y > 0.0 || pos.chapter > 0 {
                    cache.map.insert(path.clone(), pos);
                    cache.order.push_back(path);
                }
            }
        }
        cache
    }

    pub fn save(&self) -> Result<(), String> {
        let dir = crate::core::config::ConfigManager::get_config_dir();
        if !dir.exists() {
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(&ReadPositionsFile {
            positions: self.map.clone(),
        })
        .map_err(|e| e.to_string())?;
        std::fs::write(Self::file_path(), json).map_err(|e| e.to_string())
    }

    fn file_path() -> PathBuf {
        crate::core::config::ConfigManager::get_config_dir().join("read_positions.json")
    }

    pub fn get(&self, path: &str) -> Option<ReadPosition> {
        self.map.get(path).copied()
    }

    pub fn insert(&mut self, path: String, pos: ReadPosition) {
        if pos.scroll_y <= 0.0 && pos.chapter == 0 {
            return;
        }
        if self.map.insert(path.clone(), pos).is_none() {
            self.order.push_back(path.clone());
        } else {
            if let Some(idx) = self.order.iter().position(|p| *p == path) {
                self.order.remove(idx);
            }
            self.order.push_back(path);
        }
        if let Some(evicted) = (self.map.len() > MAX_ENTRIES)
            .then(|| self.order.pop_front())
            .flatten()
        {
            self.map.remove(&evicted);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> std::path::PathBuf {
        let d = std::env::temp_dir().join("kglance-rp-test");
        let _ = std::fs::create_dir_all(&d);
        d
    }

    #[test]
    fn insert_get_roundtrip() {
        let mut rp = ReadPositions::default();
        rp.insert(
            "/a.md".into(),
            ReadPosition {
                scroll_y: 120.0,
                chapter: 0,
            },
        );
        assert_eq!(
            rp.get("/a.md"),
            Some(ReadPosition {
                scroll_y: 120.0,
                chapter: 0
            })
        );
        assert_eq!(rp.get("/missing"), None);
    }

    #[test]
    fn empty_position_is_ignored() {
        let mut rp = ReadPositions::default();
        rp.insert(
            "/t.txt".into(),
            ReadPosition {
                scroll_y: 0.0,
                chapter: 0,
            },
        );
        assert!(rp.get("/t.txt").is_none());
    }

    #[test]
    fn save_load_roundtrip() {
        let dir = temp_dir();
        // SAFETY: tests run single-threaded; env var is scoped to the test.
        unsafe {
            std::env::set_var("KGLANCE_CONFIG_DIR", &dir);
        }
        let mut rp = ReadPositions::default();
        rp.insert(
            "/a.md".into(),
            ReadPosition {
                scroll_y: 5.0,
                chapter: 1,
            },
        );
        rp.save().unwrap();
        let loaded = ReadPositions::load();
        assert_eq!(
            loaded.get("/a.md"),
            Some(ReadPosition {
                scroll_y: 5.0,
                chapter: 1
            })
        );
        let _ = std::fs::remove_dir_all(&dir);
        unsafe {
            std::env::remove_var("KGLANCE_CONFIG_DIR");
        }
    }

    #[test]
    fn eviction_beyond_max() {
        let mut rp = ReadPositions::default();
        for i in 0..(MAX_ENTRIES + 5) {
            rp.insert(
                format!("/f{i}.md"),
                ReadPosition {
                    scroll_y: 1.0,
                    chapter: 0,
                },
            );
        }
        assert_eq!(rp.map.len(), MAX_ENTRIES);
        assert!(rp.get("/f0.md").is_none());
        assert!(rp.get(&format!("/f{}.md", MAX_ENTRIES + 4)).is_some());
    }
}
