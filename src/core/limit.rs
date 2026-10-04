pub const KB: u64 = 1024;
pub const MB: u64 = KB * 1024;
pub const GB: u64 = MB * 1024;

pub fn preview_size_limit(ext: &str) -> u64 {
    match ext {
        // Video & Audio: 10 GB
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "webm" | "flv" | "m4v" | "ogv" | "ts" | "3gp"
        | "mp3" | "wav" | "flac" | "ogg" | "aac" | "m4a" | "opus" | "alac" | "aiff" | "wma"
        | "mid" | "midi" => 10 * GB,

        // Archives & Disk Images: 2 GB
        "zip" | "tar" | "gz" | "tgz" | "bz2" | "tbz2" | "xz" | "txz" | "7z" | "rar" | "zst"
        | "lz4" | "iso" | "cab" | "deb" | "rpm" => 2 * GB,

        // PDF, Office & Ebooks: 500 MB
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "odt" | "ods" | "odp"
        | "rtf" | "epub" | "djvu" | "mobi" | "azw" | "azw3" | "cbr" | "cbz" => 500 * MB,

        // Images, Fonts & Design files: 100 MB
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "ico" | "avif" | "heic"
        | "heif" | "tiff" | "tif" | "jxl" | "apng" | "kra" | "ora" | "psd" | "xcf" | "ai"
        | "eps" | "ttf" | "otf" | "woff" | "woff2" | "eot" => 100 * MB,

        // Default (text, code, structured data, other): 20 MB
        _ => 20 * MB,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(KB, 1024);
        assert_eq!(MB, 1024 * 1024);
        assert_eq!(GB, 1024 * 1024 * 1024);
    }

    #[test]
    fn test_video_and_audio_limits() {
        for ext in [
            "mp4", "mkv", "avi", "mov", "wmv", "webm", "flv", "m4v", "ogv", "ts", "3gp", "mp3",
            "wav", "flac", "ogg", "aac", "m4a", "opus", "alac", "aiff", "wma", "mid", "midi",
        ] {
            assert_eq!(preview_size_limit(ext), 10 * GB);
        }
    }

    #[test]
    fn test_archive_limits() {
        for ext in [
            "zip", "tar", "gz", "tgz", "bz2", "tbz2", "xz", "txz", "7z", "rar", "zst", "lz4",
            "iso", "cab", "deb", "rpm",
        ] {
            assert_eq!(preview_size_limit(ext), 2 * GB);
        }
    }

    #[test]
    fn test_document_and_ebook_limits() {
        for ext in [
            "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt", "ods", "odp", "rtf", "epub",
            "djvu", "mobi", "azw", "azw3", "cbr", "cbz",
        ] {
            assert_eq!(preview_size_limit(ext), 500 * MB);
        }
    }

    #[test]
    fn test_image_font_and_design_limits() {
        for ext in [
            "png", "jpg", "jpeg", "gif", "bmp", "webp", "svg", "ico", "avif", "heic", "heif",
            "tiff", "tif", "jxl", "apng", "kra", "ora", "psd", "xcf", "ai", "eps", "ttf", "otf",
            "woff", "woff2", "eot",
        ] {
            assert_eq!(preview_size_limit(ext), 100 * MB);
        }
    }

    #[test]
    fn test_default_limits() {
        for ext in ["txt", "rs", "py", "json", "md", "csv", "unknown", ""] {
            assert_eq!(preview_size_limit(ext), 20 * MB);
        }
    }
}
