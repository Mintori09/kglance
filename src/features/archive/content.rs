use crate::app::Message;
use crate::core::preview::{ContentType, PreviewContent};
use crate::core::types::{KglanceState, TableRowState};
use crate::core::utils::{human_size, icon_for_entry};
use crate::features::archive::types::ArchiveEntry;
use iced::Element;

pub struct ArchiveContent {
    pub entries: Vec<ArchiveEntry>,
    pub total_files: usize,
}

impl PreviewContent<Message> for ArchiveContent {
    fn populate_state(&self, state: &mut KglanceState) {
        let total_size = self.entries.iter().map(|e| e.size).sum();
        state.table.rows = self
            .entries
            .iter()
            .map(|entry| {
                let icon = icon_for_entry(&entry.path, entry.is_dir);
                TableRowState {
                    name: entry.path.clone(),
                    kind: if entry.is_dir {
                        "Directory".to_string()
                    } else {
                        "File".to_string()
                    },
                    size: human_size(entry.size),
                    raw_size: entry.size,
                    modified: entry.modified.clone(),
                    raw_modified: 0,
                    path: entry.path.clone(),
                    is_dir: entry.is_dir,
                    icon,
                }
            })
            .collect();
        state.table.total_size = total_size;
        state.table.folder_path = state.file_name.clone();
        state.table.selected_index = None;
        state.file_type_text = "Archive".to_string();
        state.file_size_text.clear();
    }

    fn view<'a>(&'a self, state: &'a KglanceState) -> Element<'a, Message> {
        crate::features::document::view_table(&state.table, state.theme_dark)
    }

    fn content_type(&self) -> ContentType {
        ContentType::Archive
    }
    fn is_folder_view(&self) -> bool {
        true
    }
    fn supports_text_operations(&self) -> bool {
        false
    }
}
