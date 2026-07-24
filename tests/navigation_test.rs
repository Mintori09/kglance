use kglance::parsers::ParserRegistry;
use std::fs::File;

fn test_registry() -> ParserRegistry {
    let mut r = ParserRegistry::new();
    r.register(Box::new(kglance::parsers::markdown::MarkdownParser::new()));
    r.register(Box::new(kglance::parsers::text::TextParser::new()));
    r.register(Box::new(kglance::parsers::image::ImageParser));
    r
}

#[test]
fn test_supported_extensions() {
    let registry = test_registry();
    let exts = registry.all_extensions(false);
    assert!(exts.iter().any(|e| e == "png"));
    assert!(exts.iter().any(|e| e == "md"));
    assert!(exts.iter().any(|e| e == "rs"));
    assert!(!exts.iter().any(|e| e == "exe"));
}

#[test]
fn test_sibling_files_scanner() {
    let registry = test_registry();
    let temp = std::env::temp_dir().join("kglance_scan_test");
    let _ = std::fs::create_dir_all(&temp);
    File::create(temp.join("a.png")).unwrap();
    File::create(temp.join("b.txt")).unwrap();
    File::create(temp.join("c.exe")).unwrap();

    let siblings = registry.scan_sibling_files(&temp.join("a.png").to_string_lossy(), false);
    assert_eq!(siblings.len(), 2);
    assert!(siblings.iter().any(|p| p.ends_with("a.png")));
    assert!(siblings.iter().any(|p| p.ends_with("b.txt")));

    let _ = std::fs::remove_dir_all(&temp);
}
