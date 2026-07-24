use crate::app::Message;
use crate::core::preview::{ContentType, PreviewContent};
use crate::core::types::{KglanceState, TableRowState};
use crate::core::utils::{human_size, icon_for_entry};
use crate::features::document::types::DirEntry;
use crate::features::document::view::table_view;
use iced::Element;

pub struct FolderContent {
    pub entries: Vec<DirEntry>,
}

impl PreviewContent<Message> for FolderContent {
    fn populate_state(&self, state: &mut KglanceState) {
        let total_size = self.entries.iter().map(|e| e.size).sum();
        state.table.rows = self
            .entries
            .iter()
            .map(|entry| {
                let icon = icon_for_entry(&entry.name, entry.is_dir);
                TableRowState {
                    name: entry.name.clone(),
                    kind: if entry.is_dir {
                        "Directory".to_string()
                    } else {
                        "File".to_string()
                    },
                    size: human_size(entry.size),
                    raw_size: entry.size,
                    modified: entry.modified.clone(),
                    raw_modified: entry.raw_modified,
                    path: entry.name.clone(),
                    is_dir: entry.is_dir,
                    icon,
                }
            })
            .collect();
        state.table.total_size = total_size;
        state.table.folder_path = state.file_name.clone();
        state.table.selected_index = None;
        state.file_type_text = "Folder".to_string();
        state.file_size_text.clear();
    }

    fn view<'a>(&'a self, state: &'a KglanceState) -> Element<'a, Message> {
        table_view::view_table(&state.table, state.theme_dark)
    }

    fn content_type(&self) -> ContentType {
        ContentType::Folder
    }

    fn is_folder_view(&self) -> bool {
        true
    }

    fn supports_text_operations(&self) -> bool {
        false
    }
}
