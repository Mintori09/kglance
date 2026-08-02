#[derive(Debug, Clone)]
pub struct ArchiveEntry {
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub modified: String,
    pub archive_path: String,
}
