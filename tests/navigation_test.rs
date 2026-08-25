use kglance::core::navigation::{is_supported_path, scan_sibling_files};
use std::fs::File;

#[test]
fn test_is_supported_path_linux() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    use std::path::Path;
    assert!(is_supported_path(Path::new("/var/log/custom.txt")));
    assert!(is_supported_path(Path::new("/home/user/workspace/app.rs")));
    assert!(is_supported_path(Path::new("/tmp/render.png")));
    assert!(is_supported_path(Path::new("./nested/docs/spec.md")));

    assert!(is_supported_path(Path::new("/home/user/IMAGE.PNG")));
    assert!(is_supported_path(Path::new("/tmp/CONFIG.JSON")));

    assert!(!is_supported_path(Path::new("/usr/bin/bash")));
    assert!(!is_supported_path(Path::new(
        "/lib/x86_64-linux-gnu/libc.so"
    )));
    assert!(!is_supported_path(Path::new("/home/user/script.sh")));
    assert!(!is_supported_path(Path::new("/home/user/package.deb")));
    assert!(!is_supported_path(Path::new("/home/user/archive.tar.gz")));

    assert!(!is_supported_path(Path::new("/home/user/.bashrc")));
    assert!(!is_supported_path(Path::new("/home/user/.config")));
    assert!(is_supported_path(Path::new(
        "/home/user/.config/settings.json"
    )));

    let invalid_utf8_ext = OsStr::from_bytes(b"/tmp/file.\xFF\xFE");
    assert!(!is_supported_path(Path::new(invalid_utf8_ext)));
}

#[test]
fn test_sibling_files_scanner() {
    let temp = std::env::temp_dir().join("kglance_scan_test");
    let _ = std::fs::create_dir_all(&temp);
    File::create(temp.join("a.png")).unwrap();
    File::create(temp.join("b.txt")).unwrap();
    File::create(temp.join("c.exe")).unwrap();

    let siblings = scan_sibling_files(&temp.join("a.png").to_string_lossy());
    assert_eq!(siblings.len(), 2);
    assert!(siblings.iter().any(|p| p.ends_with("a.png")));
    assert!(siblings.iter().any(|p| p.ends_with("b.txt")));

    let _ = std::fs::remove_dir_all(&temp);
}
