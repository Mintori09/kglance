#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: String,
    pub raw_modified: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    Name,
    Kind,
    Modified,
    Size,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SortState {
    pub field: SortField,
    pub ascending: bool,
    pub active: bool,
}

impl Default for SortState {
    fn default() -> Self {
        Self {
            field: SortField::Name,
            ascending: true,
            active: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FolderRowState {
    pub name: String,
    pub kind: String,
    pub size: String,
    pub raw_size: u64,
    pub modified: String,
    pub raw_modified: i64,
    pub path: String,
    pub is_dir: bool,
    pub icon: &'static str,
}

#[derive(Debug, Clone, Default)]
pub struct FolderState {
    pub rows: Vec<FolderRowState>,
    pub sort_state: SortState,
    pub selected_index: Option<usize>,
    pub total_size: u64,
    pub folder_path: String,
    pub scroll_y: f32,
    pub viewport_height: f32,
}

pub fn sort_folder_rows(rows: &mut [FolderRowState], sort: &SortState) {
    if !sort.active {
        return;
    }
    match sort.field {
        SortField::Name => {
            rows.sort_by(|a, b| a.name.cmp(&b.name));
        }
        SortField::Kind => {
            rows.sort_by(|a, b| a.kind.cmp(&b.kind));
        }
        SortField::Size => {
            rows.sort_by_key(|a| a.raw_size);
        }
        SortField::Modified => {
            rows.sort_by_key(|a| a.raw_modified);
        }
    }
    if !sort.ascending {
        rows.reverse();
    }
}
