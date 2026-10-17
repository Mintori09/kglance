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
    crate::log_info!("move_to_trash called for: {}", path.display());
    let abs_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    if !abs_path.exists() {
        crate::log_error!("File does not exist: {}", abs_path.display());
        return Err(format!("File does not exist: {}", abs_path.display()));
    }

    match trash::delete(&abs_path) {
        Ok(()) => {
            crate::log_info!("Successfully moved to trash: {}", abs_path.display());
            Ok(TrashedItem {
                original_path: abs_path,
                original_index,
            })
        }
        Err(e) => {
            crate::log_error!("trash::delete failed for {}: {e}", abs_path.display());
            Err(format!("Failed to move to trash: {e}"))
        }
    }
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
