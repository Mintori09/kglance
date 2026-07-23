use kglance::core::navigation::{is_supported_extension, scan_sibling_files};
use std::fs::File;

#[test]
fn test_supported_extensions() {
    assert!(is_supported_extension("test.png"));
    assert!(is_supported_extension("doc.md"));
    assert!(is_supported_extension("code.rs"));
    assert!(!is_supported_extension("app.exe"));
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
