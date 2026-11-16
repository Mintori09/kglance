use super::*;
use std::fs::{self, File};
use std::io::Write;

#[test]
fn test_is_media_extension_and_file() {
    assert!(is_media_extension("png"));
    assert!(is_media_extension("JPG"));
    assert!(is_media_extension("jpeg"));
    assert!(is_media_extension("svg"));
    assert!(is_media_extension("webp"));
    assert!(is_media_extension("pdf"));
    assert!(!is_media_extension("md"));
    assert!(!is_media_extension("rs"));

    assert!(is_media_file(Path::new("image.png")));
    assert!(is_media_file(Path::new("/path/to/diagram.SVG")));
    assert!(!is_media_file(Path::new("document.md")));
}

#[test]
fn test_decode_percent_str() {
    assert_eq!(decode_percent_str("test%20image.png"), "test image.png");
    assert_eq!(
        decode_percent_str("Ch%C6%B0%C6%A1ng%204.png"),
        "Chương 4.png"
    );
    assert_eq!(decode_percent_str("plain_text.jpg"), "plain_text.jpg");
    assert_eq!(decode_percent_str("broken%2"), "broken%2");
}

#[test]
fn test_parse_wikilink() {
    // 1. Simple image embed
    let w1 = parse_wikilink("![[image.png]]").unwrap();
    assert_eq!(
        w1,
        ParsedWikilink::Image {
            target: "image.png".into(),
            alt: "image.png".into(),
            width: None,
            height: None,
        }
    );

    // 2. Image embed with width
    let w2 = parse_wikilink("![[assets/chart.png|450]]").unwrap();
    assert_eq!(
        w2,
        ParsedWikilink::Image {
            target: "assets/chart.png".into(),
            alt: "assets/chart.png".into(),
            width: Some(450),
            height: None,
        }
    );

    // 3. Image embed with width x height
    let w3 = parse_wikilink("![[photo.jpg|800x600]]").unwrap();
    assert_eq!(
        w3,
        ParsedWikilink::Image {
            target: "photo.jpg".into(),
            alt: "photo.jpg".into(),
            width: Some(800),
            height: Some(600),
        }
    );

    // 4. Image embed with custom alt
    let w4 = parse_wikilink("![[diagram.svg|System Architecture]]").unwrap();
    assert_eq!(
        w4,
        ParsedWikilink::Image {
            target: "diagram.svg".into(),
            alt: "System Architecture".into(),
            width: None,
            height: None,
        }
    );

    // 5. Internal note link without alias
    let l1 = parse_wikilink("[[My Note]]").unwrap();
    assert_eq!(
        l1,
        ParsedWikilink::Link {
            target: "My Note".into(),
            label: "My Note".into(),
        }
    );

    // 6. Internal note link with alias
    let l2 = parse_wikilink("[[My Note|Display Title]]").unwrap();
    assert_eq!(
        l2,
        ParsedWikilink::Link {
            target: "My Note".into(),
            label: "Display Title".into(),
        }
    );
}

#[test]
fn test_vault_detection_and_resolution() {
    let temp_dir = tempfile::tempdir().unwrap();
    let vault_root = temp_dir.path().join("MyVault");
    fs::create_dir_all(vault_root.join(".obsidian")).unwrap();
    fs::create_dir_all(vault_root.join("notes/deep")).unwrap();
    fs::create_dir_all(vault_root.join("_image/archive")).unwrap();

    // Create an image in _image/archive
    let img_path = vault_root.join("_image/archive/sample_chart.jpg");
    File::create(&img_path)
        .unwrap()
        .write_all(b"fake-image")
        .unwrap();

    // Create a note in notes/deep/
    let note_path = vault_root.join("notes/deep/meeting.md");
    File::create(&note_path)
        .unwrap()
        .write_all(b"# Meeting")
        .unwrap();

    // 1. Detect vault
    let detected = find_vault_root(&note_path);
    assert_eq!(detected, Some(vault_root.clone()));

    // 2. Resolve vault-relative path: "_image/archive/sample_chart.jpg"
    let resolved = resolve_vault_path("_image/archive/sample_chart.jpg", &note_path);
    assert_eq!(resolved, Some(img_path.clone()));

    // 3. Resolve path starting with slash: "/_image/archive/sample_chart.jpg"
    let resolved_slash = resolve_vault_path("/_image/archive/sample_chart.jpg", &note_path);
    assert_eq!(resolved_slash, Some(img_path.clone()));

    // 4. Resolve filename only via index: "sample_chart.jpg"
    let resolved_filename = resolve_vault_path("sample_chart.jpg", &note_path);
    assert_eq!(resolved_filename, Some(img_path.clone()));

    // 5. Resolve wikilink embed with dimension: "![[sample_chart.jpg|300]]"
    let resolved_wikilink = resolve_vault_path("![[sample_chart.jpg|300]]", &note_path);
    assert_eq!(resolved_wikilink, Some(img_path.clone()));

    // 6. Resolve extension fallback: note refers to ".png" but file is ".jpg"
    let resolved_ext_mismatch = resolve_vault_path("sample_chart.png", &note_path);
    assert_eq!(resolved_ext_mismatch, Some(img_path.clone()));
}

#[test]
fn test_resolve_against_real_user_vault_if_present() {
    let real_note = Path::new(
        "/home/mintori/Documents/Obsidian/02_University/S03-Database/Chương 6. Phụ thuộc hàm.md",
    );
    if real_note.exists() {
        let resolved = resolve_vault_path(
            "att.iY_fpIo1vupv5U-RgbGiMiF6u8MT3Q8K_7UGs_7Quco.jpg",
            real_note,
        );
        assert!(
            resolved.is_some(),
            "Should find attachment in real user vault"
        );
        let path = resolved.unwrap();
        assert!(path.exists(), "Resolved file must exist on disk");
        assert!(path.ends_with("att.iY_fpIo1vupv5U-RgbGiMiF6u8MT3Q8K_7UGs_7Quco.jpg"));
    }

    let real_note_2 = Path::new(
        "/home/mintori/Documents/Obsidian/01_Document/03_Technology/Bookmarks_Web/Plasma Reddit Feeder.md",
    );
    if real_note_2.exists() {
        let resolved = resolve_vault_path("_image/plasma-reddit-feeder.png", real_note_2);
        assert!(
            resolved.is_some(),
            "Should find plasma reddit feeder image with extension fallback"
        );
        let path = resolved.unwrap();
        assert!(path.exists(), "Resolved file must exist on disk");
        assert!(path.ends_with("plasma-reddit-feeder.jpg"));
    }
}
