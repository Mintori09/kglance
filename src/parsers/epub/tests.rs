use super::html::{convert_html_to_markdown, decode_html_entities};
use super::ncx::extract_ncx_navpoints;
use super::*;
use std::fs::File;
use std::io::Write;

#[test]
fn test_decode_html_entities() {
    let text = "&#x201C;Cough. Let&#x2019;s talk.&#x201D; \\-Haha";
    assert_eq!(decode_html_entities(text), "“Cough. Let’s talk.” \\-Haha");
}

#[test]
fn test_convert_html_to_markdown() {
    let html = "<h1>Header 1</h1><p>&#x201C;Hello&#x201D; \\-World</p>";
    let md = convert_html_to_markdown(html);
    assert_eq!(md, "# Header 1\n\n“Hello” -World");
}

#[test]
fn test_extract_ncx_navpoints() {
    let ncx_xml = r#"
        <ncx xmlns="http://www.daisy.org/z3986/2005/ncx/">
          <navMap>
            <navPoint id="n1">
              <navLabel><text>LỜI NÓI ĐẦU</text></navLabel>
              <content src="index_split_001.html"/>
            </navPoint>
            <navPoint id="n2">
              <navLabel><text>CHƯƠNG 1 - NHỮNG NGUYÊN TẮC CƠ BẢN</text></navLabel>
              <content src="index_split_001.html#filepos32808"/>
            </navPoint>
          </navMap>
        </ncx>
    "#;
    let navpoints = extract_ncx_navpoints(ncx_xml);
    assert_eq!(navpoints.len(), 2);
    assert_eq!(navpoints[0].0, "LỜI NÓI ĐẦU");
    assert_eq!(navpoints[1].0, "CHƯƠNG 1 - NHỮNG NGUYÊN TẮC CƠ BẢN");
    assert_eq!(navpoints[1].3.as_deref(), Some("filepos32808"));
}

#[test]
fn test_convert_html_img_to_markdown() {
    let html =
        r#"<p>Paragraph text</p><img src="images/00001.jpg" alt="Habit Loop" class="calibre_59"/>"#;
    let md = convert_html_to_markdown(html);
    assert_eq!(md, "Paragraph text\n\n![Habit Loop](images/00001.jpg)");
}

#[test]
fn test_parse_epub_with_embedded_image() {
    let temp_dir = std::env::temp_dir();
    let test_epub_path = temp_dir.join("test_kglance_image.epub");

    let file = File::create(&test_epub_path).unwrap();
    let mut zip = ::zip::ZipWriter::new(file);

    let options = ::zip::write::SimpleFileOptions::default();

    zip.start_file("META-INF/container.xml", options).unwrap();
    zip.write_all(r#"<?xml version="1.0"?><container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#.as_bytes()).unwrap();

    zip.start_file("content.opf", options).unwrap();
    zip.write_all(r#"<?xml version="1.0"?><package><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>Test EPUB</dc:title></metadata><manifest><item id="item1" href="page.html" media-type="application/xhtml+xml"/></manifest><spine><itemref idref="item1"/></spine></package>"#.as_bytes()).unwrap();

    zip.start_file("page.html", options).unwrap();
    zip.write_all(r#"<html><body><h1>Chapter 1</h1><p><img src="images/sample.png" alt="Sample"/></p></body></html>"#.as_bytes()).unwrap();

    zip.start_file("images/sample.png", options).unwrap();
    zip.write_all(&[137, 80, 78, 71, 13, 10, 26, 10]).unwrap();

    zip.finish().unwrap();

    let parser = EpubParser;
    let result = parser.parse(&test_epub_path).unwrap();

    if let ParsedContent::Epub {
        chapters, images, ..
    } = result
    {
        assert_eq!(chapters.len(), 1);
        assert!(images.contains_key("images/sample.png"));
        assert!(images.contains_key("sample.png"));
        assert_eq!(
            images.get("sample.png").unwrap(),
            &[137, 80, 78, 71, 13, 10, 26, 10]
        );
    } else {
        panic!("Expected ParsedContent::Epub variant");
    }

    let _ = std::fs::remove_file(test_epub_path);
}
