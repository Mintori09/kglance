use std::path::Path;

pub fn icon_for_entry(name: &str, is_dir: bool) -> &'static str {
    if is_dir {
        return "inode-directory";
    }

    // Match special filenames first
    let lower_name = name.to_ascii_lowercase();

    match lower_name.as_str() {
        "cargo.toml" => return "text-x-toml",
        "cargo.lock" => return "text-x-generic",
        "cmakelists.txt" => return "text-x-cmake",
        "dockerfile" => return "text-x-dockerfile",
        "containerfile" => return "text-x-dockerfile",
        "makefile" => return "text-x-makefile",
        "license" | "copying" => return "text-x-copying",
        "readme" | "readme.md" => return "text-x-readme",
        ".gitignore" => return "text-x-generic",
        ".gitattributes" => return "text-x-generic",
        ".editorconfig" => return "text-x-generic",
        ".env" => return "text-x-generic",
        _ => {}
    }

    let ext = Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match ext.as_str() {
        // =========================
        // Rust
        // =========================
        "rs" => "text-x-rust-source",

        // =========================
        // C / C++
        // =========================
        "c" | "h" => "text-x-c-source",
        "cpp" | "cxx" | "cc" | "hpp" | "hh" | "hxx" => "text-x-c++source",

        // =========================
        // Programming Languages
        // =========================
        "py" => "text-x-python",
        "pyi" => "text-x-python",
        "pyw" => "text-x-python",

        "go" => "text-x-go",

        "java" => "text-x-java-source",
        "class" => "application-java",

        "kt" | "kts" => "text-x-kotlin",

        "swift" => "text-x-swift",

        "cs" => "text-x-csharp",

        "php" => "application-x-php",

        "rb" => "application-x-ruby",

        "lua" => "text-x-lua",

        "zig" => "text-x-zig",

        "dart" => "text-x-dart",

        "scala" => "text-x-scala",

        "hs" => "text-x-haskell",

        "ml" | "mli" => "text-x-ocaml",

        "fs" | "fsi" | "fsx" => "text-x-fsharp",

        "r" => "text-x-r-source",

        "jl" => "text-x-julia",

        "nim" => "text-x-nim",

        "clj" | "cljs" | "cljc" => "text-x-clojure",

        "erl" => "text-x-erlang",

        "ex" | "exs" => "text-x-elixir",

        "sql" => "text-x-sql",

        // =========================
        // Web
        // =========================
        "html" | "htm" => "text-html",
        "css" => "text-css",
        "scss" | "sass" => "text-x-scss",
        "less" => "text-x-less",

        "js" | "mjs" | "cjs" => "text-x-javascript",
        "jsx" => "text-jsx",

        "ts" => "text-x-typescript",
        "tsx" => "text-tsx",

        "vue" => "text-x-vue",
        "svelte" => "text-x-svelte",
        "astro" => "text-x-astro",

        // =========================
        // Config
        // =========================
        "json" => "application-json",
        "jsonc" => "application-json",
        "toml" => "text-x-toml",
        "yaml" | "yml" => "text-x-yaml",
        "xml" => "application-xml",
        "ini" | "cfg" | "conf" => "text-x-config",
        "properties" => "text-x-java-properties",

        // =========================
        // Shell
        // =========================
        "sh" => "text-x-shellscript",
        "bash" => "text-x-shellscript",
        "zsh" => "text-x-shellscript",
        "fish" => "text-x-shellscript",
        "ps1" | "psm1" => "text-x-powershell",
        "bat" | "cmd" => "application-x-ms-dos-executable",

        // =========================
        // Documents
        // =========================
        "txt" => "text-plain",
        "md" | "markdown" => "text-x-markdown",
        "pdf" => "application-pdf",
        "csv" => "text-csv",
        "rtf" => "application-rtf",

        "doc" | "docx" => "application-msword",
        "xls" | "xlsx" => "application-vnd-ms-excel",
        "ppt" | "pptx" => "application-vnd-ms-powerpoint",

        "odt" => "application-vnd.oasis.opendocument.text",
        "ods" => "application-vnd.oasis.opendocument.spreadsheet",
        "odp" => "application-vnd.oasis.opendocument.presentation",

        "epub" => "application-epub+zip",
        "mobi" => "application-x-mobipocket-ebook",
        "azw" | "azw3" => "application-x-mobipocket-ebook",
        "fb2" => "application-x-fictionbook+xml",
        "djvu" | "djv" => "image-vnd.djvu",

        // =========================
        // Images
        // =========================
        "png" => "image-png",
        "jpg" | "jpeg" => "image-jpeg",
        "gif" => "image-gif",
        "bmp" => "image-bmp",
        "webp" => "image-webp",
        "svg" => "image-svg+xml",
        "ico" => "image-x-icon",
        "tif" | "tiff" => "image-tiff",
        "avif" => "image-avif",
        "heic" | "heif" => "image-heif",
        "psd" => "image-vnd.adobe.photoshop",
        "xcf" => "image-x-xcf",
        "kra" => "application-x-krita",

        // =========================
        // Video
        // =========================
        "mp4" => "video-mp4",
        "mkv" => "video-x-matroska",
        "webm" => "video-webm",
        "avi" => "video-x-msvideo",
        "mov" => "video-quicktime",
        "wmv" => "video-x-ms-wmv",
        "mpeg" | "mpg" => "video-mpeg",
        "m4v" => "video-mp4",
        "flv" => "video-x-flv",
        "3gp" => "video-3gpp",
        "ogv" => "video-ogg",

        // =========================
        // Audio
        // =========================
        "mp3" => "audio-mpeg",
        "wav" => "audio-wav",
        "flac" => "audio-flac",
        "ogg" | "oga" => "audio-vorbis",
        "opus" => "audio-opus",
        "aac" => "audio-aac",
        "m4a" => "audio-mp4",
        "mid" | "midi" => "audio-midi",
        "aiff" | "aif" => "audio-x-aiff",
        "ape" => "audio-x-ape",
        "amr" => "audio-amr",
        "wma" => "audio-x-ms-wma",

        // =========================
        // Archives
        // =========================
        "zip" => "application-zip",
        "tar" => "application-x-tar",
        "gz" | "tgz" => "application-gzip",
        "bz2" => "application-x-bzip",
        "xz" => "application-x-xz",
        "zst" => "application-zstd",
        "7z" => "application-x-7z-compressed",
        "rar" => "application-vnd.rar",
        "deb" => "application-vnd.debian.binary-package",
        "rpm" => "application-x-rpm",
        "iso" => "application-x-cd-image",

        // =========================
        // Fonts
        // =========================
        "ttf" => "font-ttf",
        "ttc" => "font-ttf",
        "otf" => "font-otf",
        "woff" | "woff2" => "font-woff",

        // =========================
        // Database
        // =========================
        "db" | "sqlite" | "sqlite3" => "application-x-sqlite3",

        // =========================
        // Certificates
        // =========================
        "pem" | "crt" | "cer" | "key" | "csr" | "p12" | "pfx" => "application-x-x509-ca-cert",

        // =========================
        // Logs
        // =========================
        "log" => "text-x-log",

        // =========================
        // Executables
        // =========================
        "exe" => "application-x-ms-dos-executable",
        "dll" => "application-x-ms-dos-executable",
        "msi" => "application-x-ms-dos-executable",
        "so" => "application-x-sharedlib",
        "appimage" => "application-x-executable",

        // =========================
        // Default
        // =========================
        _ => "text-x-generic",
    }
}
