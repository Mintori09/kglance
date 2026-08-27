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
fn test_normalize_nfc_vietnamese_decomposed() {
    let nfd_html = "<p>să´p thâ´y Kiê´m tiê´n bô´i bâ´t đă´c dĩ</p>";
    let md = convert_html_to_markdown(nfd_html);
    assert_eq!(md, "sắp thấy Kiếm tiến bối bất đắc dĩ");

    let nfd_html_2 = "<p>să´p thâ´y Kiê´m tiê`n bô´i bâ´t đă´c dĩ</p>";
    let md_2 = convert_html_to_markdown(nfd_html_2);
    assert_eq!(md_2, "sắp thấy Kiếm tiền bối bất đắc dĩ");
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

#[test]
fn test_convert_html_table_to_markdown() {
    let html = r#"
        <table>
            <thead>
                <tr><th>Col 1</th><th>Col 2</th></tr>
            </thead>
            <tbody>
                <tr><td>Val 1 | special</td><td>Val 2</td></tr>
                <tr><td>Val 3</td><td>Val 4</td></tr>
            </tbody>
        </table>
    "#;
    let md = convert_html_to_markdown(html);
    assert!(md.contains("| Col 1 | Col 2 |"));
    assert!(md.contains("| --- | --- |"));
    assert!(md.contains(r"| Val 1 \| special | Val 2 |"));
    assert!(md.contains("| Val 3 | Val 4 |"));
}

#[test]
fn test_convert_html_codeblock_to_markdown() {
    let html = r#"
        <pre><code class="language-rust">fn main() {
    println!("Hello");
}</code></pre>
    "#;
    let md = convert_html_to_markdown(html);
    assert!(md.contains("```rust"));
    assert!(md.contains("fn main() {"));
    assert!(md.contains("    println!(\"Hello\");"));
    assert!(md.contains("}"));
    assert!(md.contains("```"));
}

#[test]
fn test_convert_html_math_to_markdown() {
    let html_tex = r#"
        <math xmlns="http://www.w3.org/1998/Math/MathML" display="block">
            <semantics>
                <mrow><mi>E</mi><mo>=</mo><mi>m</mi><msup><mi>c</mi><mn>2</mn></msup></mrow>
                <annotation encoding="application/x-tex">E = mc^2</annotation>
            </semantics>
        </math>
    "#;
    let md_tex = convert_html_to_markdown(html_tex);
    assert!(md_tex.contains("$$E = mc^2$$"));

    let html_inline =
        r#"<p>Inline formula <span class="math inline">x + y = z</span> in text.</p>"#;
    let md_inline = convert_html_to_markdown(html_inline);
    assert!(md_inline.contains("$x + y = z$"));
}

#[test]
fn test_extract_epub3_navpoints() {
    let nav_html = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
        <body>
            <nav epub:type="toc" id="toc">
                <h1>Table of Contents</h1>
                <ol>
                    <li><a href="chapter1.xhtml">Chapter 1 - Introduction</a></li>
                    <li>
                        <a href="chapter2.xhtml">Chapter 2 - Methods</a>
                        <ol>
                            <li><a href="chapter2.xhtml#sec1">2.1 Sampling</a></li>
                            <li><a href="chapter2.xhtml#sec2">2.2 Analysis</a></li>
                        </ol>
                    </li>
                </ol>
            </nav>
        </body>
        </html>
    "#;
    let navpoints = crate::features::epub::parser::ncx::extract_epub3_navpoints(nav_html);
    assert_eq!(navpoints.len(), 4);
    assert_eq!(navpoints[0].0, "Chapter 1 - Introduction");
    assert_eq!(navpoints[0].1, 1);
    assert_eq!(navpoints[0].2, "chapter1.xhtml");
    assert_eq!(navpoints[1].0, "Chapter 2 - Methods");
    assert_eq!(navpoints[1].1, 1);
    assert_eq!(navpoints[2].0, "2.1 Sampling");
    assert_eq!(navpoints[2].1, 2);
    assert_eq!(navpoints[2].3.as_deref(), Some("sec1"));
    assert_eq!(navpoints[3].0, "2.2 Analysis");
    assert_eq!(navpoints[3].1, 2);
    assert_eq!(navpoints[3].3.as_deref(), Some("sec2"));
}

#[test]
fn test_extract_headings_from_html() {
    let html = r#"
        <html xmlns="http://www.w3.org/1999/xhtml">
        <body>
            <h1 id="intro">1. Giới thiệu</h1>
            <p>Nội dung giới thiệu...</p>
            <h2 id="background">1.1 Bối cảnh</h2>
            <p>Nội dung bối cảnh...</p>
            <h3 id="details">1.1.1 Chi tiết</h3>
            <p>Chi tiết...</p>
            <h2>1.2 Kết luận sơ bộ</h2>
        </body>
        </html>
    "#;
    let headings = super::html::extract_headings_from_html(html);
    assert_eq!(headings.len(), 4);
    assert_eq!(headings[0].title, "1. Giới thiệu");
    assert_eq!(headings[0].level, 1);
    assert_eq!(headings[0].id.as_deref(), Some("intro"));

    assert_eq!(headings[1].title, "1.1 Bối cảnh");
    assert_eq!(headings[1].level, 2);
    assert_eq!(headings[1].id.as_deref(), Some("background"));

    assert_eq!(headings[2].title, "1.1.1 Chi tiết");
    assert_eq!(headings[2].level, 3);
    assert_eq!(headings[2].id.as_deref(), Some("details"));

    assert_eq!(headings[3].title, "1.2 Kết luận sơ bộ");
    assert_eq!(headings[3].level, 2);
    assert_eq!(headings[3].id, None);
}

#[test]
fn test_convert_html_pre_span_highlight_codeblock() {
    let html = r#"
<pre><code><span class="syntax-all syntax-comment"><span class="bookspan">// Secure user authentication with result types
</span></span><span class="syntax-all syntax-keyword"><span class="bookspan">class</span></span> <span class="syntax-all syntax-entity"><span class="bookspan">AuthenticationService</span></span><span class="bookspan"> {
  </span><span class="syntax-all syntax-keyword"><span class="bookspan">async</span></span><span class="bookspan"> authenticateUser(</span><span class="syntax-all syntax-variable"><span class="bookspan">credentials</span></span><span class="syntax-all syntax-keyword"><span class="bookspan">:</span></span> <span class="syntax-all syntax-entity"><span class="bookspan">LoginCredentials</span></span><span class="bookspan">)</span><span class="syntax-all syntax-keyword"><span class="bookspan">:</span></span> <span class="syntax-all syntax-entity"><span class="bookspan">Promise</span></span><span class="bookspan">&lt;</span><span class="syntax-all syntax-entity"><span class="bookspan">Result</span></span><span class="bookspan">&lt;</span><span class="syntax-all syntax-entity"><span class="bookspan">AuthenticatedUser</span></span><span class="bookspan">, </span><span class="syntax-all syntax-entity"><span class="bookspan">AuthenticationError</span></span><span class="bookspan">&gt;&gt; {
    </span><span class="syntax-all syntax-keyword"><span class="bookspan">try</span></span><span class="bookspan"> {
      </span><span class="syntax-all syntax-keyword"><span class="bookspan">const</span></span><span class="bookspan"> validationResult </span><span class="syntax-all syntax-keyword"><span class="bookspan">=</span></span> <span class="syntax-all syntax-constant"><span class="bookspan">this</span></span><span class="bookspan">.</span><span class="syntax-all syntax-entity"><span class="bookspan">validateCredentials</span></span><span class="bookspan">(</span><span class="syntax-all syntax-parameter"><span class="bookspan">credentials</span></span><span class="bookspan">);
    }
  }
}</span></code></pre>
    "#;

    let md = convert_html_to_markdown(html);
    assert!(md.contains("```ts"));
    assert!(md.contains("// Secure user authentication with result types"));
    assert!(md.contains("class AuthenticationService {"));
    assert!(md.contains("async authenticateUser(credentials: LoginCredentials): Promise<Result<AuthenticatedUser, AuthenticationError>> {"));
}
