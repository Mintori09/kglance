use crate::parser::{ArchiveEntry, DirEntry};
use crate::ui::generated::TableRow;
use crate::ui::helpers::human_size;
use slint::{SharedString, VecModel};
use std::cmp::Ordering;
use std::path::Path;

#[derive(Clone, Copy, PartialEq)]
pub enum SortField {
    Name,
    Kind,
    Modified,
    Size,
}

pub struct SortState {
    pub field: SortField,
    pub ascending: bool,
}

pub fn build_archive_rows(entries: &[ArchiveEntry]) -> VecModel<TableRow> {
    let rows: Vec<TableRow> = entries
        .iter()
        .map(|e| {
            let icon = if e.is_dir { "\u{1f4c1} " } else { "\u{1f4c4} " };
            let kind = if e.is_dir { "Dir" } else { "File" };
            let path = if e.is_dir {
                String::new()
            } else {
                e.path.clone()
            };
            TableRow {
                name: SharedString::from(format!("{}{}", icon, e.path)),
                kind: SharedString::from(kind),
                modified: SharedString::from(if e.modified.is_empty() {
                    "-"
                } else {
                    &e.modified
                }),
                size: SharedString::from(human_size(e.size)),
                path: SharedString::from(path),
            }
        })
        .collect();
    VecModel::from(rows)
}

pub fn build_folder_rows(
    entries: &[DirEntry],
    parent_path: &str,
    sort: &SortState,
) -> VecModel<TableRow> {
    let mut sorted: Vec<&DirEntry> = entries.iter().collect();
    sorted.sort_by(|a, b| {
        let dir_cmp = b.is_dir.cmp(&a.is_dir);
        if dir_cmp != Ordering::Equal {
            return if sort.ascending {
                dir_cmp
            } else {
                dir_cmp.reverse()
            };
        }
        let cmp = match sort.field {
            SortField::Name => a.name.cmp(&b.name),
            SortField::Kind => a.is_dir.cmp(&b.is_dir),
            SortField::Modified => a.modified.cmp(&b.modified),
            SortField::Size => a.size.cmp(&b.size),
        };
        if sort.ascending { cmp } else { cmp.reverse() }
    });
    let rows: Vec<TableRow> = sorted
        .iter()
        .map(|e| {
            let icon = if e.is_dir { "\u{1f4c1} " } else { "\u{1f4c4} " };
            let kind = if e.is_dir { "Dir" } else { "File" };
            let full_path = Path::new(parent_path)
                .join(&e.name)
                .to_string_lossy()
                .to_string();
            TableRow {
                name: SharedString::from(format!("{}{}", icon, e.name)),
                kind: SharedString::from(kind),
                modified: SharedString::from(&e.modified),
                size: SharedString::from(human_size(e.size)),
                path: SharedString::from(full_path),
            }
        })
        .collect();
    VecModel::from(rows)
}
