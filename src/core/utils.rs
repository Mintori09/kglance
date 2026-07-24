pub(crate) fn human_size(bytes: u64) -> String {
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

pub fn icon_for_entry(name: &str, is_dir: bool) -> &'static str {
    if is_dir {
        return "inode-directory";
    }
    let ext = std::path::Path::new(name)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "rs" => "text-x-rust-source",
        "md" => "text-x-markdown",
        "txt" => "text-plain",
        "pdf" => "application-pdf",
        "png" => "image-png",
        "jpg" | "jpeg" => "image-jpeg",
        "gif" => "image-gif",
        "bmp" => "image-bmp",
        "webp" => "image-webp",
        "svg" => "image-svg-xml",
        "ico" => "image-x-ico",
        "zip" => "application-zip",
        "tar" => "application-x-tar",
        "gz" | "tgz" => "application-gzip",
        "bz2" => "application-x-bzip",
        "xz" => "application-x-xz",
        "7z" => "application-x-7z-compressed",
        "rar" => "application-vnd.rar",
        "mp4" => "video-mp4",
        "mkv" => "video-x-matroska",
        "webm" => "video-webm",
        "avi" => "video-x-msvideo",
        "mov" => "video-quicktime",
        "wmv" => "video-x-ms-wmv",
        "mp3" => "audio-mpeg",
        "wav" => "audio-wav",
        "flac" => "audio-flac",
        "ogg" | "oga" => "audio-vorbis",
        "aac" => "audio-aac",
        "m4a" => "audio-mp4",
        "opus" => "audio-opus",
        "c" | "h" => "text-x-c-source",
        "cpp" | "hpp" | "cc" | "hh" => "text-x-c++source",
        "py" => "text-x-python",
        "js" => "text-x-javascript",
        "ts" | "tsx" => "text-x-typescript",
        "html" | "htm" => "text-html",
        "css" => "text-css",
        "json" => "application-json",
        "xml" => "application-xml",
        "toml" => "text-x-toml",
        "yaml" | "yml" => "text-x-yaml",
        "sh" => "text-x-shellscript",
        "conf" | "cfg" | "ini" => "text-x-config",
        "ttf" => "font-ttf",
        "otf" => "font-otf",
        "woff" | "woff2" => "font-woff",
        "csv" => "text-csv",
        "doc" | "docx" => "application-msword",
        "xls" | "xlsx" => "application-vnd-ms-excel",
        "ppt" | "pptx" => "application-vnd-ms-powerpoint",
        "odt" => "application-vnd-oasis-opendocument-text",
        "ods" => "application-vnd-oasis-opendocument-spreadsheet",
        "odp" => "application-vnd-oasis-opendocument-presentation",
        "epub" => "application-epub+zip",
        _ => "text-x-generic",
    }
}

pub fn human_time(datetime: std::time::SystemTime) -> String {
    let now = chrono::Local::now();
    let dt = chrono::DateTime::<chrono::Local>::from(datetime);
    let duration = now.signed_duration_since(dt);

    if duration.num_seconds() < 0 {
        return dt.format("%b %d").to_string();
    }

    if duration.num_minutes() < 1 {
        return "Just now".to_string();
    } else if duration.num_hours() < 1 {
        return format!("{}m ago", duration.num_minutes());
    } else if duration.num_days() < 1 {
        return format!("{}h ago", duration.num_hours());
    } else if duration.num_days() == 1 {
        return "Yesterday".to_string();
    } else if duration.num_days() < 7 {
        return format!("{}d ago", duration.num_days());
    }

    dt.format("%b %d, %Y").to_string()
}
