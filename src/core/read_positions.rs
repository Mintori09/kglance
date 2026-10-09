use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;

const MAX_ENTRIES: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReadPosition {
    pub scroll_y: f32,
    pub chapter: usize,
    #[serde(default)]
    pub reading_mode: Option<crate::core::config::EpubReadingMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ReadPositionsFile {
    positions: HashMap<String, ReadPosition>,
}

#[derive(Debug, Default, Clone)]
pub struct ReadPositions {
    map: HashMap<String, ReadPosition>,
    order: VecDeque<String>,
}

impl ReadPositions {
    pub fn load() -> Self {
        Self::load_from_path(&Self::file_path())
    }

    pub fn load_from_path(path: &std::path::Path) -> Self {
        let mut cache = ReadPositions::default();
        if let Ok(json) = std::fs::read_to_string(path)
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
        self.save_to_path(&Self::file_path())
    }

    pub fn save_to_path(&self, path: &std::path::Path) -> Result<(), String> {
        if let Some(dir) = path.parent()
            && !dir.exists()
        {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(&ReadPositionsFile {
            positions: self.map.clone(),
        })
        .map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }

    pub fn file_path() -> PathBuf {
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

    #[test]
    fn insert_get_roundtrip() {
        let mut rp = ReadPositions::default();
        rp.insert(
            "/a.md".into(),
            ReadPosition {
                scroll_y: 120.0,
                chapter: 0,
                reading_mode: None,
            },
        );
        assert_eq!(
            rp.get("/a.md"),
            Some(ReadPosition {
                scroll_y: 120.0,
                chapter: 0,
                reading_mode: None,
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
                reading_mode: None,
            },
        );
        assert!(rp.get("/t.txt").is_none());
    }

    #[test]
    fn save_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("read_positions.json");
        let mut rp = ReadPositions::default();
        rp.insert(
            "/a.md".into(),
            ReadPosition {
                scroll_y: 5.0,
                chapter: 1,
                reading_mode: Some(crate::core::config::EpubReadingMode::Continuous),
            },
        );
        rp.save_to_path(&path).unwrap();
        let loaded = ReadPositions::load_from_path(&path);
        assert_eq!(
            loaded.get("/a.md"),
            Some(ReadPosition {
                scroll_y: 5.0,
                chapter: 1,
                reading_mode: Some(crate::core::config::EpubReadingMode::Continuous),
            })
        );
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
                    reading_mode: None,
                },
            );
        }
        assert_eq!(rp.map.len(), MAX_ENTRIES);
        assert!(rp.get("/f0.md").is_none());
        assert!(rp.get(&format!("/f{}.md", MAX_ENTRIES + 4)).is_some());
    }
}
