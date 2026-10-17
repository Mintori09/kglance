use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrashedItem {
    pub original_path: PathBuf,
    pub original_index: usize,
}

#[derive(Debug, Clone, Default)]
pub struct TrashHistory {
    items: Vec<TrashedItem>,
}

impl TrashHistory {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn push(&mut self, item: TrashedItem) {
        self.items.push(item);
    }

    pub fn pop(&mut self) -> Option<TrashedItem> {
        self.items.pop()
    }

    pub fn last(&self) -> Option<&TrashedItem> {
        self.items.last()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}

pub fn move_to_trash(path: &Path, original_index: usize) -> Result<TrashedItem, String> {
    if !path.exists() {
        return Err(format!("File does not exist: {}", path.display()));
    }

    trash::delete(path).map_err(|e| format!("Failed to move to trash: {e}"))?;

    Ok(TrashedItem {
        original_path: path.to_path_buf(),
        original_index,
    })
}

pub fn restore_trashed_item(item: &TrashedItem) -> Result<(), String> {
    let trash_items =
        trash::os_limited::list().map_err(|e| format!("Failed to list trash items: {e}"))?;

    let matching = trash_items
        .into_iter()
        .find(|ti| ti.original_path() == item.original_path);

    if let Some(target) = matching {
        trash::os_limited::restore_all([target])
            .map_err(|e| format!("Failed to restore file: {e}"))?;
        Ok(())
    } else {
        Err(format!(
            "Item not found in trash: {}",
            item.original_path.display()
        ))
    }
}

#[cfg(test)]
#[path = "trash_tests.rs"]
mod tests;
