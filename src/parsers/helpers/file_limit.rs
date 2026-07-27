pub const KB: u64 = 1024;
pub const MB: u64 = KB * 1024;
pub const GB: u64 = MB * 1024;

pub fn preview_size_limit(ext: &str) -> u64 {
    match ext {
        // Video & Audio: 10 GB
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "webm" | "mp3" | "wav" | "flac" | "ogg" | "aac"
        | "m4a" => 10 * GB,

        // Archives: 2 GB
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => 2 * GB,

        // PDF / Office: 500 MB
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "odt" | "ods" | "odp"
        | "epub" => 500 * MB,

        // Images / Fonts: 100 MB
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "ico" | "ttf" | "otf"
        | "woff" | "woff2" => 100 * MB,

        // Default (text/code)
        _ => 20 * MB,
    }
}
