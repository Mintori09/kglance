use kglance::core::preview::{FilePreviewer, PreviewData};
use kglance::core::types::KglanceState;
use kglance::engine::build_registry;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

fn create_mock_epub(dir_path: &std::path::Path) -> std::path::PathBuf {
    let epub_path = dir_path.join("complete_book.epub");
    let file = File::create(&epub_path).expect("Failed to create mock epub file");
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default();

    // 1. mimetype
    zip.start_file("mimetype", options).unwrap();
    zip.write_all(b"application/epub+zip").unwrap();

    // 2. META-INF/container.xml
    zip.start_file("META-INF/container.xml", options).unwrap();
    zip.write_all(
        br#"<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#,
    )
    .unwrap();

    // 3. OEBPS/content.opf
    zip.start_file("OEBPS/content.opf", options).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="utf-8"?>
<package version="2.0" unique-identifier="BookId" xmlns="http://www.idpf.org/2007/opf">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Kglance Comprehensive Test Book</dc:title>
    <dc:creator>Kglance Team</dc:creator>
    <dc:language>en</dc:language>
  </metadata>
  <manifest>
    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
    <item id="ch1" href="chapter1.xhtml" media-type="application/xhtml+xml"/>
    <item id="ch2" href="chapter2.xhtml" media-type="application/xhtml+xml"/>
    <item id="img1" href="images/cover.png" media-type="image/png"/>
  </manifest>
  <spine toc="ncx">
    <itemref idref="ch1"/>
    <itemref idref="ch2"/>
  </spine>
</package>"#,
    )
    .unwrap();

    // 4. OEBPS/toc.ncx
    zip.start_file("OEBPS/toc.ncx", options).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <head>
    <meta name="dtb:uid" content="urn:uuid:12345"/>
  </head>
  <docTitle><text>Kglance Test Book</text></docTitle>
  <navMap>
    <navPoint id="navPoint-1" playOrder="1">
      <navLabel><text>Chapter 1: Getting Started</text></navLabel>
      <content src="chapter1.xhtml"/>
    </navPoint>
    <navPoint id="navPoint-2" playOrder="2">
      <navLabel><text>Chapter 2: Advanced Topics</text></navLabel>
      <content src="chapter2.xhtml#section2"/>
    </navPoint>
  </navMap>
</ncx>"#,
    )
    .unwrap();

    // 5. OEBPS/chapter1.xhtml
    zip.start_file("OEBPS/chapter1.xhtml", options).unwrap();
    zip.write_all(
        br#"<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head><title>Chapter 1</title></head>
<body>
  <h1>Getting Started</h1>
  <p>Welcome to <strong>Kglance</strong> file previewer!</p>
  <pre><code class="language-rust">fn main() { println!("Hello from Kglance"); }</code></pre>
  <p><img src="images/cover.png" alt="Cover Image"/></p>
</body>
</html>"#,
    )
    .unwrap();

    // 6. OEBPS/chapter2.xhtml
    zip.start_file("OEBPS/chapter2.xhtml", options).unwrap();
    zip.write_all(
        br#"<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head><title>Chapter 2</title></head>
<body>
  <h1>Advanced Topics</h1>
  <div id="section2">
    <h2>Sub-Section 2.1</h2>
    <p>Details about high performance indexing and rendering.</p>
  </div>
</body>
</html>"#,
    )
    .unwrap();

    // 7. OEBPS/images/cover.png
    zip.start_file("OEBPS/images/cover.png", options).unwrap();
    zip.write_all(&[137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13])
        .unwrap();

    zip.finish().expect("Failed to write zip content");
    epub_path
}

#[test]
fn test_epub_parser_registry_integration() {
    let dir = tempdir().expect("Failed to create temporary directory");
    let epub_path = create_mock_epub(dir.path());

    let registry = build_registry();
    let parsed_data = FilePreviewer::parse(&registry, &epub_path)
        .expect("Epub file must parse successfully through registry");

    if let PreviewData::Epub {
        title,
        chapters,
        images,
        ..
    } = &parsed_data
    {
        assert_eq!(title, "Kglance Comprehensive Test Book");
        assert_eq!(chapters.len(), 2, "Expected 2 parsed chapters");

        assert_eq!(chapters[0].title, "Chapter 1: Getting Started");
        assert_eq!(chapters[1].title, "Chapter 2: Advanced Topics");
        assert_eq!(chapters[1].anchor.as_deref(), Some("section2"));

        assert!(
            images.contains_key("images/cover.png") || images.contains_key("cover.png"),
            "Cover image must be extracted into images map"
        );
        assert!(
            !chapters[0].blocks.is_empty(),
            "Chapter 1 should contain blocks"
        );
        assert!(
            !chapters[1].blocks.is_empty(),
            "Chapter 2 should contain blocks"
        );

        let mut state = KglanceState::default();
        parsed_data.populate_state(&mut state);

        assert_eq!(state.epub.title, "Kglance Comprehensive Test Book");
        assert_eq!(state.epub.chapters.len(), 2);
        assert_eq!(state.epub.active_chapter, 0);

        // Switch active chapter
        state.epub.active_chapter = 1;
        assert_eq!(state.epub.active_chapter, 1);
    } else {
        panic!("Expected PreviewData::Epub, got different variant");
    }
}
