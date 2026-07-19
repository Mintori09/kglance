use std::io::Write;
use std::path::Path;
use std::time::UNIX_EPOCH;

pub type FileHandler = Box<dyn Fn(String)>;
pub type ThumbnailData = Vec<(Vec<u8>, u32, u32)>;

pub const FILE_TYPES: &[(&[&str], &str)] = &[
    (
        &[
            "rs", "py", "js", "ts", "jsx", "tsx", "go", "java", "c", "cpp", "h", "hpp", "rb",
            "php", "swift", "kt", "scala",
        ],
        "Source Code",
    ),
    (
        &[
            "html", "css", "json", "xml", "yaml", "yml", "toml", "ini", "cfg", "conf", "md", "rst",
            "tex",
        ],
        "Document",
    ),
    (&["sh", "bash", "zsh", "fish", "bat", "ps1"], "Script"),
    (
        &["png", "jpg", "jpeg", "gif", "bmp", "webp", "svg", "ico"],
        "Image",
    ),
    (&["pdf"], "PDF Document"),
    (&["zip", "tar", "gz", "bz2", "xz", "7z", "rar"], "Archive"),
    (&["mp3", "wav", "flac", "ogg", "aac", "m4a"], "Audio"),
    (&["mp4", "mkv", "avi", "mov", "webm"], "Video"),
    (
        &[
            "doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt", "ods", "odp",
        ],
        "Office Document",
    ),
    (&["ttf", "otf", "woff", "woff2"], "Font"),
];

pub fn human_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    if unit_idx == 0 {
        format!("{} B", bytes)
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
}

pub fn breadcrumb(path: &str) -> String {
    let p = Path::new(path);
    let parts: Vec<String> = p
        .components()
        .map(|c| format!("\u{1f4c1} {}", c.as_os_str().to_string_lossy()))
        .collect();
    parts.join(" \u{203a} ")
}

pub fn human_date(secs: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let is_recent = secs + 86400 * 180 > now;
    let days = secs / 86400;
    if is_recent {
        let hours = (secs % 86400) / 3600;
        let mins = (secs % 3600) / 60;
        format!("{:02}:{:02}", hours, mins)
    } else {
        let month = (days as f64 % 365.0) / 30.0 + 1.0;
        let day = (days as f64 % 30.0) + 1.0;
        format!("{}/{}", month as i32, day as i32)
    }
}

pub fn guess_file_type(path: &Path) -> &'static str {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    for (exts, label) in FILE_TYPES {
        if exts.contains(&ext) {
            return label;
        }
    }
    "Unknown"
}

pub fn set_file_info(ui: &super::generated::PreviewWindow, path: &str) {
    let p = Path::new(path);
    ui.set_file_type_text(guess_file_type(p).into());
    match std::fs::metadata(path) {
        Ok(m) => {
            ui.set_file_size_text(human_size(m.len()).into());
            ui.set_file_modified_text(
                m.modified()
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| human_date(d.as_secs()))
                    .unwrap_or_else(|| "-".into())
                    .into(),
            );
        }
        Err(_) => {
            ui.set_file_size_text("-".into());
            ui.set_file_modified_text("-".into());
        }
    }
    ui.set_show_file_info(true);
}

pub fn make_image_from_rgba(data: &[u8], width: u32, height: u32) -> slint::Image {
    let mut buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(width, height);
    let pixel_slice = buffer.make_mut_slice();
    for (i, pixel) in pixel_slice.iter_mut().enumerate() {
        let offset = i * 4;
        pixel.r = data[offset];
        pixel.g = data[offset + 1];
        pixel.b = data[offset + 2];
        pixel.a = data[offset + 3];
    }
    slint::Image::from_rgba8(buffer)
}

pub fn scan_dir_for_files(path: &str) -> Vec<String> {
    let p = Path::new(path);
    let parent = p.parent().unwrap_or(Path::new("/"));
    let _current_name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");

    let preview_exts: &[&str] = &[
        "rs", "py", "js", "ts", "go", "java", "c", "cpp", "html", "css", "json", "xml", "md",
        "txt", "png", "jpg", "jpeg", "gif", "bmp", "webp", "svg", "ico", "pdf", "mp3", "wav",
        "flac", "ogg", "aac", "m4a", "mp4", "mkv", "avi", "mov", "webm", "docx", "xlsx", "pptx",
        "odt", "ods", "odp", "ttf", "otf", "woff", "woff2", "zip", "tar", "gz", "bz2", "xz", "7z",
        "rar",
    ];

    let mut files: Vec<String> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(parent) {
        for entry in rd.flatten() {
            if let Ok(ft) = entry.file_type()
                && ft.is_file()
                && let Some(name) = entry.file_name().to_str()
            {
                let ext = Path::new(name)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                if preview_exts.contains(&ext) {
                    files.push(entry.path().to_string_lossy().to_string());
                }
            }
        }
    }
    files.sort();
    files
}

pub fn find_file_index(files: &[String], target: &str) -> Option<usize> {
    let target = Path::new(target).canonicalize().ok()?;
    files.iter().position(|f| {
        Path::new(f)
            .canonicalize()
            .map(|p| p == target)
            .unwrap_or(false)
    })
}

pub fn copy_to_clipboard(text: &str) {
    if std::process::Command::new("wl-copy")
        .arg(text)
        .spawn()
        .is_err()
        && let Ok(mut child) = std::process::Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(std::process::Stdio::piped())
            .spawn()
        && let Some(stdin) = child.stdin.as_mut()
    {
        let _ = stdin.write_all(text.as_bytes());
    }
}
