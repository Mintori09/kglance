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
    let html = "<h1>Header 1</h1><p>&#x201C;Hello&#x201D; -World</p>";
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
fn test_convert_html_to_markdown_ruby() {
    let html =
        "<p><ruby>不<rt>ふ</rt></ruby><ruby>自<rt>じ</rt></ruby><ruby>由<rt>ゆう</rt></ruby>な</p>";
    let md = convert_html_to_markdown(html);
    assert_eq!(
        md,
        "<ruby>不<rt>ふ</rt></ruby><ruby>自<rt>じ</rt></ruby><ruby>由<rt>ゆう</rt></ruby>な"
    );

    let html_tabular = "<p><ruby>不<rt>ふ</rt>自<rt>じ</rt>由<rt>ゆう</rt></ruby>な</p>";
    let md_tab = convert_html_to_markdown(html_tabular);
    assert_eq!(
        md_tab,
        "<ruby>不<rt>ふ</rt></ruby><ruby>自<rt>じ</rt></ruby><ruby>由<rt>ゆう</rt></ruby>な"
    );

    let html_rp = "<p><ruby><rb>漢字</rb><rp>(</rp><rt>かんじ</rt><rp>)</rp></ruby></p>";
    let md_rp = convert_html_to_markdown(html_rp);
    assert_eq!(md_rp, "<ruby>漢字<rt>かんじ</rt></ruby>");

    let html_table =
        "<table><tr><th>単語</th></tr><tr><td><ruby>漢字<rt>かんじ</rt></ruby></td></tr></table>";
    let md_table = convert_html_to_markdown(html_table);
    assert!(md_table.contains("<ruby>漢字<rt>かんじ</rt></ruby>"));

    let html_unclosed = "<p><ruby>漢字<rt>かんじ</rt></p><p>次の段落</p>";
    let md_unclosed = convert_html_to_markdown(html_unclosed);
    assert!(md_unclosed.contains("<ruby>漢字<rt>かんじ</rt></ruby>"));
    assert!(md_unclosed.contains("次の段落"));
}

#[test]
fn test_convert_html_footnote_and_superscript() {
    let html = "<p>hấp dẫn hơn.<span class=\"text_1\"><sup class=\"calibre7\">2</sup></span></p>\n<p class=\"block_4\"><span class=\"text_1\"><sup class=\"calibre7\">1</sup></span>. Tên một chương trình truyền hình thực tế ở Mỹ...</p>\n<p class=\"block_4\"><span class=\"text_1\"><sup class=\"calibre7\">2</sup></span>. Tiện thể, tôi cần nói ngay: \"Tôi không có thời gian\"...</p>";
    let md = convert_html_to_markdown(html);
    assert_eq!(
        md,
        "hấp dẫn hơn.[^2]\n\n[^1]: Tên một chương trình truyền hình thực tế ở Mỹ...\n\n[^2]: Tiện thể, tôi cần nói ngay: \"Tôi không có thời gian\"..."
    );

    let blocks = parse_to_blocks(&md);
    assert_eq!(blocks.len(), 3);
    match &blocks[0] {
        Block::Paragraph(inlines) => {
            assert!(
                inlines
                    .iter()
                    .any(|i| matches!(i, Inline::FootnoteReference(lbl) if lbl == "2")),
                "Expected footnote reference '2'"
            );
        }
        _ => panic!("Expected Paragraph block"),
    }
    match &blocks[1] {
        Block::FootnoteDefinition { label, .. } => {
            assert_eq!(label, "1");
        }
        _ => panic!("Expected FootnoteDefinition for 1"),
    }
    match &blocks[2] {
        Block::FootnoteDefinition { label, .. } => {
            assert_eq!(label, "2");
        }
        _ => panic!("Expected FootnoteDefinition for 2"),
    }
}

#[test]
fn test_convert_html_custom_bullet_list() {
    let html = r#"<p class="block_4">Trong thời gian đó, tôi cũng làm các việc sau:</p>
<p class="block_4">♦ Loại bỏ hoàn toàn caffeine và rượu khỏi chế độ ăn.</p>
<p class="block_4">♦ Ăn ít đường nhất có thể.</p>
<p class="block_4">♦ Thường xuyên ăn các bữa nhỏ trong ngày để có năng lượng.</p>
<p class="block_4">♦ Dậy và ngủ mà không cần đặt báo thức.</p>
<p class="block_4">♦<span class="text_">&nbsp;2,5 giờ dành cho giải trí</span></p>
<p class="block_4">Ý đồ của thử nghiệm này rất đơn giản</p>"#;
    let md = convert_html_to_markdown(html);
    println!("DEBUG OREILLY MD:\n{md}");
    let blocks = parse_to_blocks(&md);
    for (i, b) in blocks.iter().enumerate() {
        println!("OREILLY BLOCK {i}: {b:?}");
    }
    assert_eq!(blocks.len(), 3);
    match &blocks[1] {
        Block::List { ordered, items, .. } => {
            assert!(!ordered);
            assert_eq!(items.len(), 5);
            assert!(flatten_inlines(&items[0].content).contains("Loại bỏ hoàn toàn caffeine"));
            assert!(
                flatten_inlines(&items[3].content).contains("Dậy và ngủ mà không cần đặt báo thức")
            );
            assert!(flatten_inlines(&items[4].content).contains("2,5 giờ dành cho giải trí"));
        }
        _ => panic!("Expected List block for custom bullets"),
    }
}

#[test]
fn test_quote_attribution_dash_not_converted_to_list() {
    let html = r#"<p class="block_">“Nếu thời gian là thứ đáng giá nhất, phí phạm thời gian hẳn phải là sự lãng phí ngông cuồng nhất.</p>
<p class="block_1">—<span class="text_">&nbsp;Benjamin Franklin</span></p>"#;
    let md = convert_html_to_markdown(html);
    let blocks = parse_to_blocks(&md);
    assert_eq!(blocks.len(), 2, "Expected 2 separate paragraphs");
    match &blocks[0] {
        Block::Paragraph(inlines) => {
            assert!(flatten_inlines(inlines).contains("Nếu thời gian là thứ đáng giá nhất"));
        }
        _ => panic!("Expected Paragraph block for quote"),
    }
    match &blocks[1] {
        Block::Paragraph(inlines) => {
            let text = flatten_inlines(inlines);
            assert!(text.contains("—") || text.contains("Benjamin Franklin"));
            assert!(!text.starts_with("- "));
        }
        _ => panic!("Expected Paragraph block for attribution, not a List"),
    }
}

#[test]
fn test_convert_html_span_classes_to_markdown_formatting() {
    let html = r#"<p>Sách <span class="text_2">Getting Things Done</span> và <span class="text_">Hoàn thành</span>.</p>"#;
    let md = convert_html_to_markdown(html);
    assert!(
        md.contains("*Getting Things Done*"),
        "text_2 should convert to italic markdown, got: {md}"
    );
    assert!(
        md.contains("**Hoàn thành**"),
        "text_ should convert to bold markdown, got: {md}"
    );

    let blocks = parse_to_blocks(&md);
    match &blocks[0] {
        Block::Paragraph(inlines) => {
            assert!(
                inlines.iter().any(|i| matches!(i, Inline::Italic(_))),
                "Expected Inline::Italic for text_2"
            );
            assert!(
                inlines.iter().any(|i| matches!(i, Inline::Bold(_))),
                "Expected Inline::Bold for text_"
            );
        }
        _ => panic!("Expected Paragraph block"),
    }
}

#[test]
fn test_extract_blocks_styles_list_desync_prevention() {
    let html = r#"<p class="block_left">Intro paragraph</p>
<p class="block_left">♦ Item 1</p>
<p class="block_left">♦ Item 2</p>
<p class="block_left">♦ Item 3</p>
<p class="block_center">Should remain center aligned</p>"#;
    let md = convert_html_to_markdown(html);
    println!("DEBUG OREILLY MD:\n{md}");
    let blocks = parse_to_blocks(&md);
    for (i, b) in blocks.iter().enumerate() {
        println!("OREILLY BLOCK {i}: {b:?}");
    }
    assert_eq!(blocks.len(), 3); // Intro, List (3 items), Center paragraph

    let mut stylesheet = HashMap::new();
    stylesheet.insert(
        "block_left".to_string(),
        css::BlockStyle {
            align: css::TextAlign::Left,
            text_indent: 0.0,
            is_bold: false,
            is_italic: false,
            is_uppercase: false,
            is_underline: false,
            is_strikethrough: false,
            font_scale: None,
            ..Default::default()
        },
    );
    stylesheet.insert(
        "block_center".to_string(),
        css::BlockStyle {
            align: css::TextAlign::Center,
            text_indent: 0.0,
            is_bold: false,
            is_italic: false,
            is_uppercase: false,
            is_underline: false,
            is_strikethrough: false,
            font_scale: None,
            ..Default::default()
        },
    );

    let styles = extract_blocks_styles(html, &blocks, &stylesheet);
    // Block 0 is Intro (left)
    // Block 1 is List (left)
    // Block 2 must be Center, NOT left!
    let center_style = styles.get(&2);
    assert!(center_style.is_some(), "Block 2 must have style");
    assert_eq!(center_style.unwrap().align, css::TextAlign::Center);
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
              <navPoint id="n2_1">
                <navLabel><text>Phần 1.1 - Chi tiết</text></navLabel>
                <content src="index_split_001.html#filepos35000"/>
              </navPoint>
            </navPoint>
          </navMap>
        </ncx>
    "#;
    let navpoints = extract_ncx_navpoints(ncx_xml);
    assert_eq!(navpoints.len(), 3);
    assert_eq!(navpoints[0].0, "LỜI NÓI ĐẦU");
    assert_eq!(navpoints[0].1, 1);
    assert_eq!(navpoints[1].0, "CHƯƠNG 1 - NHỮNG NGUYÊN TẮC CƠ BẢN");
    assert_eq!(navpoints[1].1, 1);
    assert_eq!(navpoints[1].3.as_deref(), Some("filepos32808"));
    assert_eq!(navpoints[2].0, "Phần 1.1 - Chi tiết");
    assert_eq!(navpoints[2].1, 2);
    assert_eq!(navpoints[2].3.as_deref(), Some("filepos35000"));
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
fn test_lazy_spine_parsing_and_on_demand_content() {
    let temp_dir = std::env::temp_dir();
    let test_epub_path = temp_dir.join("test_kglance_lazy.epub");

    let file = File::create(&test_epub_path).unwrap();
    let mut zip = ::zip::ZipWriter::new(file);

    let options = ::zip::write::SimpleFileOptions::default();

    zip.start_file("META-INF/container.xml", options).unwrap();
    zip.write_all(r#"<?xml version="1.0"?><container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#.as_bytes()).unwrap();

    zip.start_file("OEBPS/content.opf", options).unwrap();
    zip.write_all(r#"<?xml version="1.0"?><package><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>Lazy Test</dc:title><meta name="cover" content="cov-img"/></metadata><manifest><item id="cov-img" href="images/cover.png" media-type="image/png"/><item id="item1" href="ch1.xhtml" media-type="application/xhtml+xml"/><item id="item2" href="ch2.xhtml" media-type="application/xhtml+xml"/><item id="img2" href="images/ch2_img.png" media-type="image/png"/><item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/></manifest><spine toc="ncx"><itemref idref="item1"/><itemref idref="item2"/></spine></package>"#.as_bytes()).unwrap();

    zip.start_file("OEBPS/toc.ncx", options).unwrap();
    zip.write_all(r#"<?xml version="1.0"?><ncx xmlns="http://www.daisy.org/z3986/2005/ncx/"><navMap><navPoint id="np1"><navLabel><text>First Chapter</text></navLabel><content src="ch1.xhtml"/></navPoint><navPoint id="np2"><navLabel><text>Second Chapter</text></navLabel><content src="ch2.xhtml#part2"/></navPoint></navMap></ncx>"#.as_bytes()).unwrap();

    zip.start_file("OEBPS/ch1.xhtml", options).unwrap();
    zip.write_all(
        r#"<html><body><h1>First Chapter</h1><p>Content of first chapter.</p></body></html>"#
            .as_bytes(),
    )
    .unwrap();

    zip.start_file("OEBPS/ch2.xhtml", options).unwrap();
    zip.write_all(r#"<html><body><h1 id="part2">Second Chapter</h1><p><img src="images/ch2_img.png" alt="Ch2"/></p></body></html>"#.as_bytes()).unwrap();

    zip.start_file("OEBPS/images/cover.png", options).unwrap();
    zip.write_all(&[1, 2, 3, 4]).unwrap();

    zip.start_file("OEBPS/images/ch2_img.png", options).unwrap();
    zip.write_all(&[5, 6, 7, 8]).unwrap();

    zip.finish().unwrap();

    let parser = EpubParser;
    let result = parser.parse(&test_epub_path).unwrap();

    if let ParsedContent::Epub {
        chapters, images, ..
    } = &result
    {
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].0, "First Chapter");
        assert_eq!(chapters[0].3, "ch1.xhtml");
        assert!(!chapters[0].4.is_empty(), "Chapter 0 must be parsed");

        assert_eq!(chapters[1].0, "Second Chapter");
        assert_eq!(chapters[1].3, "ch2.xhtml");
        assert_eq!(chapters[1].2.as_deref(), Some("part2"));
        assert!(
            chapters[1].4.is_empty(),
            "Chapter 1 must be lazy and empty initially"
        );

        // Selective images: cover must be extracted, but ch2_img.png must NOT be extracted
        assert!(images.contains_key("images/cover.png") || images.contains_key("cover.png"));
        assert!(!images.contains_key("images/ch2_img.png") && !images.contains_key("ch2_img.png"));
    } else {
        panic!("Expected ParsedContent::Epub");
    }

    // Now test parse_chapter_content on demand for chapter 2
    let file = File::open(&test_epub_path).unwrap();
    let mut archive = ::zip::ZipArchive::new(file).unwrap();
    let ch2_blocks = parse_chapter_content(
        &mut archive,
        "OEBPS/content.opf",
        "ch2.xhtml",
        Some("part2"),
    )
    .unwrap();

    assert!(!ch2_blocks.is_empty());
    let ch2_imgs = extract_images_from_blocks(&ch2_blocks);
    assert_eq!(ch2_imgs, vec!["images/ch2_img.png"]);

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
fn test_convert_html_table_with_caption_and_p_tags() {
    let html = r#"<table class="Basic-Table">
<caption><p class="TT" id="tab1-2"><span class="Heavy"><span class="SANS_Futura_Std_Heavy_B_11">Table 1-2:</span></span> <span class="SANS_Futura_Std_Book_11">Common Data Types</span></p></caption>
<thead>
<tr>
<th class="TCH" scope="col"><p class="TCH1"><span class="SANS_Futura_Std_Heavy_B_11">Data type</span></p></th>
<th class="TCH" scope="col"><p class="TCH1"><span class="SANS_Futura_Std_Heavy_B_11">Examples</span></p></th>
</tr>
</thead>
<tbody>
<tr>
<td class="TBF"><p class="TB1"><span class="SANS_Futura_Std_Book_11">Integer (int)</span></p></td>
<td class="TBF"><p class="TB1"><span class="SANS_TheSansMonoCd_W5Regular_11">-2</span><span class="SANS_Futura_Std_Book_11">,</span> <span class="SANS_TheSansMonoCd_W5Regular_11">-1</span><span class="SANS_Futura_Std_Book_11">,</span> <span class="SANS_TheSansMonoCd_W5Regular_11">0</span><span class="SANS_Futura_Std_Book_11">,</span> <span class="SANS_TheSansMonoCd_W5Regular_11">1</span><span class="SANS_Futura_Std_Book_11">,</span> <span class="SANS_TheSansMonoCd_W5Regular_11">2</span><span class="SANS_Futura_Std_Book_11">,</span> <span class="SANS_TheSansMonoCd_W5Regular_11">3</span><span class="SANS_Futura_Std_Book_11">,</span> <span class="SANS_TheSansMonoCd_W5Regular_11">4</span><span class="SANS_Futura_Std_Book_11">,</span> <span class="SANS_TheSansMonoCd_W5Regular_11">5</span></p></td>
</tr>
<tr>
<td class="TB"><p class="TB1"><span class="SANS_Futura_Std_Book_11">Floating-point number (float)</span></p></td>
<td class="TB"><p class="TB1"><span class="SANS_TheSansMonoCd_W5Regular_11">-1.25</span><span class="SANS_Futura_Std_Book_11">,</span> <span class="SANS_TheSansMonoCd_W5Regular_11">-1.0</span><span class="SANS_Futura_Std_Book_11">,</span> <span class="SANS_TheSansMonoCd_W5Regular_11">-0.5</span><span class="SANS_Futura_Std_Book_11">,</span> <span class="SANS_TheSansMonoCd_W5Regular_11">0.0</span><span class="SANS_Futura_Std_Book_11">,</span> <span class="SANS_TheSansMonoCd_W5Regular_11">0.5</span><span class="SANS_Futura_Std_Book_11">,</span> <span class="SANS_TheSansMonoCd_W5Regular_11">1.0</span><span class="SANS_Futura_Std_Book_11">,</span> <span class="SANS_TheSansMonoCd_W5Regular_11">1.25</span></p></td>
</tr>
<tr>
<td class="TBL"><p class="TB1"><span class="SANS_Futura_Std_Book_11">String (str)</span></p></td>
<td class="TBL"><p class="TB1"><span class="SANS_TheSansMonoCd_W5Regular_11">'a'</span><span class="SANS_Futura_Std_Book_11">,</span> <span class="SANS_TheSansMonoCd_W5Regular_11">'aa'</span><span class="SANS_Futura_Std_Book_11">,</span> <span class="SANS_TheSansMonoCd_W5Regular_11">'aaa'</span><span class="SANS_Futura_Std_Book_11">,</span> <span class="SANS_TheSansMonoCd_W5Regular_11">'Hello!'</span><span class="SANS_Futura_Std_Book_11">,</span> <span class="SANS_TheSansMonoCd_W5Regular_11">'11 cats'</span><span class="SANS_Futura_Std_Book_11">,</span> <span class="SANS_TheSansMonoCd_W5Regular_11">'5'</span></p></td>
</tr>
</tbody>
</table>"#;
    let md = convert_html_to_markdown(html);
    assert!(md.contains("| Data type | Examples |"));
    assert!(md.contains("| --- | --- |"));
    assert!(md.contains("| Integer (int) | -2, -1, 0, 1, 2, 3, 4, 5 |"));

    let blocks = crate::parsers::markdown::parse_to_blocks(&md);
    let table_block = blocks.iter().find(|b| matches!(b, Block::Table(_)));
    assert!(
        table_block.is_some(),
        "Table must be parsed into Block::Table"
    );
    if let Some(Block::Table(tbl)) = table_block {
        assert_eq!(tbl.headers.len(), 2);
        assert_eq!(tbl.rows.len(), 3);
    }
}

#[test]
fn test_convert_html_table_with_colspan_and_align() {
    let html = r#"
        <table>
            <thead>
                <tr>
                    <th align="left">Col 1</th>
                    <th align="center">Col 2</th>
                    <th align="right">Col 3</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td colspan="2">Span two</td>
                    <td>Col 3 Val</td>
                </tr>
            </tbody>
        </table>
    "#;
    let md = convert_html_to_markdown(html);
    assert!(md.contains("| Col 1 | Col 2 | Col 3 |"));
    assert!(md.contains("| --- | :---: | ---: |"));
    assert!(md.contains("| Span two |  | Col 3 Val |"));
}

#[test]
fn test_convert_html_table_with_br_newlines() {
    let html = r#"
        <table>
            <tr><th>Header</th></tr>
            <tr><td>Line 1<br>Line 2</td></tr>
        </table>
    "#;
    let md = convert_html_to_markdown(html);
    assert!(md.contains("Line 1<br>Line 2") || md.contains("Line 1<br />Line 2"));
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
fn test_extract_epub3_navpoints_with_span_grouping() {
    let nav_html = r#"
        <nav epub:type="toc">
            <ol>
                <li>
                    <span>Part I: Foundations</span>
                    <ol>
                        <li><a href="c1.xhtml">Chapter 1</a></li>
                        <li><a href="c2.xhtml">Chapter 2</a></li>
                    </ol>
                </li>
                <li>
                    <span>Part II: Advanced</span>
                    <ol>
                        <li><a href="c3.xhtml">Chapter 3</a></li>
                    </ol>
                </li>
            </ol>
        </nav>
    "#;
    let navpoints = crate::features::epub::parser::ncx::extract_epub3_navpoints(nav_html);
    assert_eq!(navpoints.len(), 5);
    // Part I span inherits first child's file
    assert_eq!(navpoints[0].0, "Part I: Foundations");
    assert_eq!(navpoints[0].1, 1);
    assert_eq!(navpoints[0].2, "c1.xhtml");

    assert_eq!(navpoints[1].0, "Chapter 1");
    assert_eq!(navpoints[1].1, 2);
    assert_eq!(navpoints[1].2, "c1.xhtml");

    assert_eq!(navpoints[2].0, "Chapter 2");
    assert_eq!(navpoints[2].1, 2);
    assert_eq!(navpoints[2].2, "c2.xhtml");

    // Part II span inherits c3.xhtml
    assert_eq!(navpoints[3].0, "Part II: Advanced");
    assert_eq!(navpoints[3].1, 1);
    assert_eq!(navpoints[3].2, "c3.xhtml");

    assert_eq!(navpoints[4].0, "Chapter 3");
    assert_eq!(navpoints[4].1, 2);
    assert_eq!(navpoints[4].2, "c3.xhtml");
}

#[test]
fn test_extract_nav_and_cover_href_multi_properties() {
    let opf_xml = r#"
        <package>
            <manifest>
                <item id="item-toc" href="nav.xhtml" properties="landmarks nav" media-type="application/xhtml+xml"/>
                <item id="item-cover" href="cover.jpg" properties="cover-image scripted" media-type="image/jpeg"/>
            </manifest>
        </package>
    "#;
    let nav_href = super::opf::extract_nav_href(opf_xml);
    assert_eq!(nav_href.as_deref(), Some("nav.xhtml"));

    let cover_href = super::opf::extract_cover_image_href(opf_xml);
    assert_eq!(cover_href.as_deref(), Some("cover.jpg"));
}

#[test]
fn test_extract_html_blocks_meta_includes_blockquote_and_aside() {
    let html = r#"
        <html xmlns="http://www.w3.org/1999/xhtml">
        <body>
            <blockquote class="famous-quote">Direct quote without p tag</blockquote>
            <p class="chapter-lead">Lead paragraph following quote</p>
            <aside class="sidebar-note">Sidebar aside note</aside>
            <p class="body-text">Normal body text</p>
        </body>
        </html>
    "#;
    let metas = crate::features::common::parser::html::extract_html_blocks_meta(html, None);
    let non_empty: Vec<_> = metas.into_iter().filter(|m| !m.is_empty).collect();
    assert_eq!(non_empty.len(), 4);
    assert_eq!(non_empty[0].tag, "blockquote");
    assert_eq!(non_empty[0].classes, vec!["famous-quote"]);
    assert_eq!(non_empty[1].tag, "p");
    assert_eq!(non_empty[1].classes, vec!["chapter-lead"]);
    assert_eq!(non_empty[2].tag, "aside");
    assert_eq!(non_empty[2].classes, vec!["sidebar-note"]);
    assert_eq!(non_empty[3].tag, "p");
    assert_eq!(non_empty[3].classes, vec!["body-text"]);
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

#[test]
fn test_epub_anchor_find_block_index() {
    let html = r#"
        <html>
        <body>
            <p>Intro paragraph before section</p>
            <h1 id="sec2">Section 2 Title</h1>
            <p>Content of section 2</p>
            <div id="sub_sec">
                <h2>Sub section</h2>
            </div>
        </body>
        </html>
    "#;

    let md = convert_html_to_markdown(html);
    let blocks = parse_to_blocks(&md);

    // Anchor "sec2" should find the block index of the heading or directly after
    let idx = super::find_block_index(&blocks, Some("sec2"), None);
    assert!(idx.is_some(), "Anchor 'sec2' must be found");
    let target_block = &blocks[idx.unwrap()];
    if let Block::Heading { content, .. } = target_block {
        let text = flatten_inlines(content);
        assert_eq!(text, "Section 2 Title");
    } else {
        panic!("Expected Heading block, got {:?}", target_block);
    }

    // Anchor "sub_sec" inside div should be found
    let sub_idx = super::find_block_index(&blocks, Some("sub_sec"), None);
    assert!(sub_idx.is_some(), "Anchor 'sub_sec' must be found");
}

#[test]
fn test_epub_nested_directory_image_resolution() {
    use crate::features::epub::parser::load_chapter_content_and_images_from_epub;

    let temp_dir = std::env::temp_dir();
    let test_epub_path = temp_dir.join(format!("test_nested_img_{}.epub", std::process::id()));

    let file = File::create(&test_epub_path).unwrap();
    let mut zip = ::zip::ZipWriter::new(file);
    let options = ::zip::write::SimpleFileOptions::default();

    zip.start_file("META-INF/container.xml", options).unwrap();
    zip.write_all(br#"<?xml version="1.0"?><container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#).unwrap();

    zip.start_file("OEBPS/content.opf", options).unwrap();
    zip.write_all(br#"<?xml version="1.0"?><package><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>Nested</dc:title></metadata><manifest><item id="ch1" href="text/ch1.xhtml" media-type="application/xhtml+xml"/><item id="fig" href="images/fig1.png" media-type="image/png"/></manifest><spine><itemref idref="ch1"/></spine></package>"#).unwrap();

    // Chapter in OEBPS/text/ch1.xhtml referencing ../images/fig1.png
    zip.start_file("OEBPS/text/ch1.xhtml", options).unwrap();
    zip.write_all(br#"<html><body><h1>Chapter 1</h1><p><img src="../images/fig1.png" alt="Fig 1"/></p></body></html>"#).unwrap();

    let sample_png = vec![137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13];
    zip.start_file("OEBPS/images/fig1.png", options).unwrap();
    zip.write_all(&sample_png).unwrap();

    zip.finish().unwrap();

    let loaded = load_chapter_content_and_images_from_epub(&test_epub_path, "text/ch1.xhtml", None);
    assert!(
        loaded.is_ok(),
        "Chapter and images should load successfully"
    );
    let (blocks, images, _) = loaded.unwrap();
    assert!(!blocks.is_empty());
    assert!(
        images.contains_key("OEBPS/images/fig1.png")
            || images.contains_key("../images/fig1.png")
            || images.contains_key("fig1.png"),
        "Nested relative image ../images/fig1.png must be correctly extracted into images"
    );

    let _ = std::fs::remove_file(test_epub_path);
}

#[test]
fn test_epub_sbo_rt_content_not_displayed() {
    use crate::features::epub::parser::html::convert_html_to_markdown;
    use crate::features::markdown::parser::{
        flatten_inlines_visual, is_standalone_anchor_block, parse_to_blocks, strip_anchor_tags,
    };

    let html = r#"
        <html xmlns="http://www.w3.org/1999/xhtml">
        <body>
            <a id="sbo-rt-content"></a>
            <h1>Chapter 1: Getting Started</h1>
            <p><a id="sec1"></a>Here is the first paragraph with some details.</p>
        </body>
        </html>
    "#;

    let md = convert_html_to_markdown(html);
    let blocks = parse_to_blocks(&md);

    assert!(!blocks.is_empty());
    // Block 0 corresponds to the standalone anchor <a id="sbo-rt-content"></a>
    assert!(
        is_standalone_anchor_block(&blocks[0]),
        "The sbo-rt-content block should be identified as a standalone anchor block"
    );

    // Flattening visual representation should not contain the raw anchor tag
    for block in &blocks {
        if let Block::Heading { content, .. } | Block::Paragraph(content) = block {
            let visual = flatten_inlines_visual(content);
            assert!(
                !visual.contains("<a id="),
                "Visual representation must not contain raw <a id= tags, found: {visual}"
            );
            assert!(
                !visual.contains("</a>"),
                "Visual representation must not contain raw </a> tags, found: {visual}"
            );
        }
    }

    assert_eq!(strip_anchor_tags(r#"<a id="sbo-rt-content"></a>"#), "");
    assert_eq!(strip_anchor_tags(r#"<a id="sbo-rt-content"/>"#), "");
    assert_eq!(
        strip_anchor_tags(r#"<a id="sbo-rt-content"></a>Hello"#),
        "Hello"
    );
}

#[test]
fn test_epub_various_anchor_formats() {
    let html = r#"
        <html>
        <body>
            <h1>Chapter Title</h1>
            <p>Intro text</p>
            <p><a id="empty_a"/>Empty anchor before paragraph</p>
            <h2><span id="span_id">Section with Span ID</span></h2>
            <p>Span section body</p>
            <h3 xml:id="xml_heading">Heading with xml:id</h3>
            <p>XML body</p>
            <div id="encoded space">
                <h4>Section with Space</h4>
            </div>
        </body>
        </html>
    "#;

    let md = convert_html_to_markdown(html);
    let blocks = parse_to_blocks(&md);

    // 1. Empty tag anchor <a id="empty_a"/>
    let idx1 = super::find_block_index(&blocks, Some("empty_a"), None);
    assert!(
        idx1.is_some(),
        "Self-closing <a id='empty_a'/> must be found"
    );

    // 2. <span id="span_id"> inside heading
    let idx2 = super::find_block_index(&blocks, Some("span_id"), None);
    assert!(idx2.is_some(), "<span id='span_id'> must be found");

    // 3. xml:id attribute
    let idx3 = super::find_block_index(&blocks, Some("xml_heading"), None);
    assert!(idx3.is_some(), "xml:id='xml_heading' must be found");

    // 4. URL-encoded anchor matching
    let idx4 = super::find_block_index(&blocks, Some("encoded%20space"), None);
    assert!(
        idx4.is_some(),
        "URL-encoded anchor 'encoded%20space' must match 'encoded space'"
    );
}

#[test]
fn test_epub_toc_heading_resolution() {
    let html = r#"
        <!DOCTYPE html>
        <html>
        <body>
            <p>Introductory remarks before chapter.</p>
            <h1>Chapter 1: The Beginning</h1>
            <p>Body paragraph 1.</p>
            <h2 id="sec1-1">1.1&nbsp;First Steps</h2>
            <p>Section 1 body.</p>
            <h2 id="sec1-2">1.2 Second Steps</h2>
            <p>Section 2 body.</p>
            <h3>1.2.1 Deep Details</h3>
            <p>Deep details body.</p>
        </body>
        </html>
    "#;

    let md = convert_html_to_markdown(html);
    let blocks = parse_to_blocks(&md);

    // 1. With anchor: id="sec1-1"
    let idx_1_1 = super::find_block_index(&blocks, Some("sec1-1"), Some("1.1 First Steps"));
    assert!(idx_1_1.is_some(), "Anchor 'sec1-1' must be resolved");

    // 2. Without anchor, matching title with non-breaking space &nbsp;
    let idx_nbsp = super::find_block_index(&blocks, None, Some("1.1\u{a0}First Steps"));
    assert_eq!(
        idx_nbsp, idx_1_1,
        "Non-breaking space title must resolve to the same heading"
    );

    // 3. Sub-heading without anchor, matching by title
    let idx_deep = super::find_block_index(&blocks, None, Some("1.2.1 Deep Details"));
    assert!(idx_deep.is_some(), "Deep Details must be resolved by title");
    assert!(
        idx_deep.unwrap() > idx_1_1.unwrap(),
        "Sub-heading must be further down in blocks"
    );
}

#[test]
fn test_epub_img_tag_with_dimensions_and_remote_url() {
    use crate::features::markdown::parser::parse_to_blocks;

    let html = r#"
        <html xmlns="http://www.w3.org/1999/xhtml">
        <body>
            <p>Kanji illustration:</p>
            <img src="https://cdn.jsdelivr.net/gh/mistval/kanji_images@master/gifs/6e29.gif" width="90" alt="温" />
            <p>Another image:</p>
            <img src="images/fig1.png" style="width: 150px; height: 100px" alt="Figure 1" />
        </body>
        </html>
    "#;

    let md = convert_html_to_markdown(html);
    let blocks = parse_to_blocks(&md);

    let img_blocks: Vec<&Block> = blocks
        .iter()
        .filter(|b| matches!(b, Block::Image { .. }))
        .collect();

    assert_eq!(img_blocks.len(), 2);

    match img_blocks[0] {
        Block::Image {
            alt,
            path,
            width,
            height,
            ..
        } => {
            assert_eq!(alt, "温");
            assert_eq!(
                path,
                "https://cdn.jsdelivr.net/gh/mistval/kanji_images@master/gifs/6e29.gif"
            );
            assert_eq!(*width, Some(90));
            assert_eq!(*height, None);
        }
        _ => panic!("Expected Block::Image"),
    }

    match img_blocks[1] {
        Block::Image {
            alt,
            path,
            width,
            height,
            ..
        } => {
            assert_eq!(alt, "Figure 1");
            assert_eq!(path, "images/fig1.png");
            assert_eq!(*width, Some(150));
            assert_eq!(*height, Some(100));
        }
        _ => panic!("Expected Block::Image"),
    }
}

#[test]
fn test_epub_parser_preamble_spine_items_before_ncx() {
    let temp_dir = std::env::temp_dir();
    let test_epub_path = temp_dir.join(format!(
        "test_preamble_{}.epub",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    let file = std::fs::File::create(&test_epub_path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default();

    zip.start_file("mimetype", options).unwrap();
    zip.write_all(b"application/epub+zip").unwrap();

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

    zip.start_file("OEBPS/content.opf", options).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="utf-8"?>
<package version="2.0" unique-identifier="BookId" xmlns="http://www.idpf.org/2007/opf">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Preamble Test Book</dc:title>
  </metadata>
  <manifest>
    <item id="cover" href="cover.xhtml" media-type="application/xhtml+xml"/>
    <item id="quotes" href="quotes.xhtml" media-type="application/xhtml+xml"/>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
  </manifest>
  <spine toc="ncx">
    <itemref idref="cover"/>
    <itemref idref="quotes"/>
    <itemref idref="ch1"/>
  </spine>
</package>"#,
    )
    .unwrap();

    zip.start_file("OEBPS/toc.ncx", options).unwrap();
    zip.write_all(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <navMap>
    <navPoint id="nav1" playOrder="1">
      <navLabel><text>Chương 1</text></navLabel>
      <content src="ch1.xhtml"/>
    </navPoint>
  </navMap>
</ncx>"#
            .as_bytes(),
    )
    .unwrap();

    zip.start_file("OEBPS/cover.xhtml", options).unwrap();
    zip.write_all(
        "<html><head><title>Cover</title></head><body><h1>Bia sach</h1></body></html>".as_bytes(),
    )
    .unwrap();

    zip.start_file("OEBPS/quotes.xhtml", options).unwrap();
    zip.write_all(
        "<html><head><title>Danh ngon</title></head><body><p>Thoi gian la vang.</p></body></html>"
            .as_bytes(),
    )
    .unwrap();

    zip.start_file("OEBPS/ch1.xhtml", options).unwrap();
    zip.write_all(
        "<html><head><title>Chương 1</title></head><body><h1>Chương 1</h1><p>Noi dung chinh.</p></body></html>"
            .as_bytes(),
    )
    .unwrap();

    zip.finish().unwrap();

    let parser = EpubParser;
    let res = parser.parse(&test_epub_path).unwrap();

    if let ParsedContent::Epub { chapters, .. } = res {
        assert_eq!(
            chapters.len(),
            3,
            "Phải có 3 chapters: cover, quotes, và ch1"
        );
        assert_eq!(chapters[0].0, "Bia sach");
        assert_eq!(chapters[0].3, "cover.xhtml");
        assert!(
            !chapters[0].4.is_empty(),
            "Chapter 0 ban đầu phải được nạp blocks"
        );

        assert_eq!(chapters[1].0, "Danh ngon");
        assert_eq!(chapters[1].3, "quotes.xhtml");

        assert_eq!(chapters[2].0, "Chương 1");
        assert_eq!(chapters[2].3, "ch1.xhtml");
    } else {
        panic!("Expected ParsedContent::Epub");
    }

    let _ = std::fs::remove_file(test_epub_path);
}

#[test]
fn test_convert_html_oreilly_callout_alerts() {
    use crate::parsers::markdown::AlertKind;

    let html = r#"<div data-type="note" epub:type="note"><h6>Note</h6>
<p>Throughout the book, I use the term <em>Node</em> instead of <em>Node.js</em> for brevity.</p>
</div>
<div data-type="tip"><h6>Tip</h6>
<p>I decided to give Node a spin and learn more about it.</p>
</div>
<div data-type="warning" epub:type="warning"><h6>Warning</h6>
<p>Add-ons are not supported with ES module imports.</p>
</div>"#;

    let md = convert_html_to_markdown(html);
    let blocks = parse_to_blocks(&md);
    assert_eq!(blocks.len(), 3);

    match &blocks[0] {
        Block::Alert { kind, content } => {
            assert_eq!(*kind, AlertKind::Note);
            assert!(format!("{content:?}").contains("Throughout the book, I use the term"));
            assert!(format!("{content:?}").contains("Node"));
        }
        _ => panic!("Expected Alert block for note, got: {:?}", blocks[0]),
    }

    match &blocks[1] {
        Block::Alert { kind, content } => {
            assert_eq!(*kind, AlertKind::Tip);
            assert!(format!("{content:?}").contains("I decided to give Node a spin"));
        }
        _ => panic!("Expected Alert block for tip, got: {:?}", blocks[1]),
    }

    match &blocks[2] {
        Block::Alert { kind, content } => {
            assert_eq!(*kind, AlertKind::Warning);
            assert!(format!("{content:?}").contains("Add-ons are not supported"));
        }
        _ => panic!("Expected Alert block for warning, got: {:?}", blocks[2]),
    }
}

#[test]
fn test_convert_html_oreilly_sidebar() {
    let html = r#"<aside data-type="sidebar" epub:type="sidebar"><div class="sidebar" id="id222">
<h1>Node on Windows</h1>
<p>All the examples are designed for a macOS environment.</p>
</div></aside>"#;

    let md = convert_html_to_markdown(html);
    let blocks = parse_to_blocks(&md);
    assert_eq!(blocks.len(), 1);

    match &blocks[0] {
        Block::Quote(sub_blocks) => {
            assert!(!sub_blocks.is_empty());
            assert!(format!("{sub_blocks:?}").contains("Node on Windows"));
            assert!(format!("{sub_blocks:?}").contains("macOS environment"));
        }
        _ => panic!("Expected Quote block for sidebar, got: {:?}", blocks[0]),
    }
}

#[test]
fn test_convert_html_figure_caption() {
    let html = r#"<figure><div id="fig_101" class="figure">
<img src="assets/enjs_0101.png" alt="enjs 0101" width="1728" height="531"/>
<h6><span class="label">Figure 1-1. </span>Node’s REPL mode</h6>
</div></figure>"#;

    let md = convert_html_to_markdown(html);
    let blocks = parse_to_blocks(&md);
    // Block 0: <a id="fig_101"></a> anchor, Block 1: Image, Block 2: Caption
    assert_eq!(blocks.len(), 3);

    match &blocks[1] {
        Block::Image { path, .. } => {
            assert_eq!(path, "assets/enjs_0101.png");
        }
        _ => panic!("Expected Image block, got: {:?}", blocks[1]),
    }

    match &blocks[2] {
        Block::Paragraph(inlines) => {
            let text = flatten_inlines(inlines);
            assert!(text.contains("Figure 1-1. Node’s REPL mode"));
        }
        _ => panic!(
            "Expected Paragraph block with caption, got: {:?}",
            blocks[2]
        ),
    }
}

#[test]
fn test_convert_html_li_with_newline_and_span() {
    let html = r#"<ul class="stafflist">
    <li>
      <span class="staffrole">Acquisitions Editor:</span> Amanda Quinn
    </li>
    <li>
      <span class="staffrole">Development Editor:</span> Jeff Bleiel
    </li>
    <li>
      <span class="staffrole">Production Editor:</span>
       Gregory Hyman
    </li>
  </ul>"#;

    let md = convert_html_to_markdown(html);
    let blocks = parse_to_blocks(&md);
    assert_eq!(blocks.len(), 1);

    match &blocks[0] {
        Block::List { items, ordered, .. } => {
            assert!(!ordered);
            assert_eq!(items.len(), 3);
            let item0_text = flatten_inlines(&items[0].content);
            assert!(
                item0_text.contains("Acquisitions Editor: Amanda Quinn"),
                "Expected item 0 to contain full text without split, got: {item0_text}"
            );
            let item1_text = flatten_inlines(&items[1].content);
            assert!(
                item1_text.contains("Development Editor: Jeff Bleiel"),
                "Expected item 1 to contain full text without split, got: {item1_text}"
            );
            let item2_text = flatten_inlines(&items[2].content);
            assert!(
                item2_text.contains("Production Editor: Gregory Hyman"),
                "Expected item 2 to contain full text without split, got: {item2_text}"
            );
        }
        _ => panic!("Expected List block, got: {:?}", blocks[0]),
    }
}

#[test]
fn test_code_tags_ending_with_consonant_not_corrupted_by_normalize_nfc() {
    let html = r#"<p>The <code>.editor</code> command is one of many REPL commands that you can see by<a data-type="indexterm" id="id458"></a> typing the <code>.help</code> command:</p>"#;

    let md = convert_html_to_markdown(html);
    assert!(
        md.contains("`.editor`"),
        "`.editor` must preserve its closing backtick, got: {md}"
    );
    assert!(
        md.contains("`.help`"),
        "`.help` must preserve its closing backtick, got: {md}"
    );

    let blocks = parse_to_blocks(&md);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Paragraph(inlines) => {
            // Check that .editor is parsed as Inline::Code
            let has_editor_code = inlines
                .iter()
                .any(|inline| matches!(inline, crate::parsers::markdown::Inline::Code(c) if c == ".editor"));
            assert!(
                has_editor_code,
                "Expected Inline::Code(\".editor\") in inlines: {inlines:?}"
            );

            let has_help_code = inlines
                .iter()
                .any(|inline| matches!(inline, crate::parsers::markdown::Inline::Code(c) if c == ".help"));
            assert!(
                has_help_code,
                "Expected Inline::Code(\".help\") in inlines: {inlines:?}"
            );
        }
        _ => panic!("Expected Paragraph block, got: {:?}", blocks[0]),
    }
}

#[test]
fn test_code_tags_ending_with_vowel_not_corrupted_into_grave_accent() {
    let html = r#"<p>run it with the <code>node</code> command, <code>file</code>, <code>date</code>, and <code>true</code>.</p>"#;

    let md = convert_html_to_markdown(html);
    assert!(
        md.contains("`node`"),
        "`node` must not be corrupted to `nodè, got: {md}"
    );
    assert!(
        md.contains("`file`"),
        "`file` must not be corrupted to `filè, got: {md}"
    );
    assert!(
        md.contains("`date`"),
        "`date` must not be corrupted to `datè, got: {md}"
    );
    assert!(
        md.contains("`true`"),
        "`true` must not be corrupted to `truè, got: {md}"
    );

    let blocks = parse_to_blocks(&md);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Paragraph(inlines) => {
            let has_node_code = inlines.iter().any(
                |inline| matches!(inline, crate::parsers::markdown::Inline::Code(c) if c == "node"),
            );
            assert!(
                has_node_code,
                "Expected Inline::Code(\"node\") in inlines: {inlines:?}"
            );
        }
        _ => panic!("Expected Paragraph block, got: {:?}", blocks[0]),
    }
}

#[test]
fn test_convert_html_definition_list() {
    let html = r#"
<dl>
  <dt>Arrange</dt>
  <dd><p>This section is the setup. It's where we prepare the environment.</p></dd>
  <dt>Act</dt>
  <dd><p>This is where the actual action takes place.</p></dd>
  <dt>Assert</dt>
  <dd><p>This is where we check the result.</p></dd>
</dl>
"#;

    let md = convert_html_to_markdown(html);
    assert!(
        md.contains("- **Arrange**: This section is the setup."),
        "Expected formatted bullet definition, got:\n{md}"
    );
    assert!(
        md.contains("- **Act**: This is where the actual action takes place."),
        "Expected formatted bullet definition, got:\n{md}"
    );
    assert!(
        md.contains("- **Assert**: This is where we check the result."),
        "Expected formatted bullet definition, got:\n{md}"
    );

    let blocks = parse_to_blocks(&md);
    assert_eq!(blocks.len(), 1, "Should parse into a single List block");
    match &blocks[0] {
        Block::List { items, ordered, .. } => {
            assert!(!ordered);
            assert_eq!(items.len(), 3);
        }
        _ => panic!("Expected List block, got: {:?}", blocks[0]),
    }
}

#[test]
fn test_convert_html_callout_list_images() {
    let html = r##"
<dl class="calloutlist">
<dt><a class="co" id="callout_CO1-1" href="#co_CO1-1"><img src="assets/1.png" alt="1" width="12" height="12"/></a></dt>
<dd><p>Updates the package lists.</p></dd>
<dt><a class="co" id="callout_CO1-2" href="#co_CO1-2"><img src="assets/2.png" alt="2" width="12" height="12"/></a></dt>
<dd><p>Installs required tools.</p></dd>
</dl>
"##;

    let md = convert_html_to_markdown(html);
    assert!(
        md.contains("[①](#co_CO1-1)**: Updates the package lists."),
        "Expected callout circle symbol ① in bullet list item, got:\n{md}"
    );
    assert!(
        md.contains("[②](#co_CO1-2)**: Installs required tools."),
        "Expected callout circle symbol ② in bullet list item, got:\n{md}"
    );

    let blocks = parse_to_blocks(&md);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::List { items, .. } => {
            assert_eq!(items.len(), 2);
        }
        _ => panic!("Expected List block, got: {:?}", blocks[0]),
    }
}

#[test]
fn test_epub_find_block_index_image_anchor() {
    let blocks = vec![
        Block::Paragraph(vec![crate::parsers::markdown::Inline::Text(
            "Intro paragraph".to_string(),
        )]),
        Block::Image {
            alt: "Figure 8-1: Node test runner architecture".to_string(),
            path: "images/fig_801.png".to_string(),
            link_url: Some("#fig_801".to_string()),
            width: Some(600),
            height: Some(400),
        },
    ];

    let found = super::find_block_index(&blocks, Some("fig_801"), None);
    assert_eq!(found, Some(1));
}

#[test]
fn test_dialogue_dash_paragraph_not_converted_to_bullet_list() {
    let html = r#"<p class="calibre3">Nhóm dịch: Tiên Huyễn</p>
<p class="calibre3">- Bản đế?</p>
<p class="calibre3">Lý Nhiên và Nhạc Kiếm Ly liếc nhìn nhau, trái tim đột nhiên nhảy dựng..</p>
<p class="calibre3">- Xin hỏi rốt cuộc Ngài là ai?</p>"#;

    let md = convert_html_to_markdown(html);
    let blocks = parse_to_blocks(&md);
    assert_eq!(blocks.len(), 4, "Expected 4 separate paragraphs, not List");

    match &blocks[1] {
        Block::Paragraph(inlines) => {
            let t = flatten_inlines(inlines);
            assert!(
                t.starts_with("- "),
                "Must preserve dash in dialogue, got: {t}"
            );
        }
        other => panic!("Expected Paragraph for dialogue dash, got: {other:?}"),
    }

    match &blocks[3] {
        Block::Paragraph(inlines) => {
            let t = flatten_inlines(inlines);
            assert!(
                t.starts_with("- "),
                "Must preserve dash in dialogue, got: {t}"
            );
        }
        other => panic!("Expected Paragraph for dialogue dash, got: {other:?}"),
    }
}

#[test]
fn test_pre_with_callout_links() {
    let html = r##"<pre translate="no" data-type="programlisting" data-code-language="js"><code class="kr">const</code><code> </code><code class="nx">saveNewPassword</code><code> </code><code class="o">=</code><code> </code><code class="p">(</code><code class="nx">password</code><code class="p">)</code><code> </code><code class="o">=&gt;</code><code> </code><code class="p">{</code><code>
  </code><code class="nx">mockDB</code><code class="p">.</code><code class="nx">hash</code><code> </code><code class="o">=</code><code> </code><code class="nx">bcrypt</code><code class="p">.</code><code class="nx">hashSync</code><code class="p">(</code><code class="nx">password</code><code class="p">,</code><code> </code><code class="mi">10</code><code class="p">)</code><code class="p">;</code><code> </code><a class="co" id="co_build_a_secure_local_password_manager_CO4-1" href="#callout_build_a_secure_local_password_manager_CO4-1"><img src="assets/1.png" alt="1" width="12" height="12"/></a><code>
  </code><code class="nx">console</code><code class="p">.</code><code class="nx">log</code><code class="p">(</code><code class="s2">&quot;Password has been saved!&quot;</code><code class="p">)</code><code class="p">;</code><code> </code><a class="co" id="co_build_a_secure_local_password_manager_CO4-2" href="#callout_build_a_secure_local_password_manager_CO4-2"><img src="assets/2.png" alt="2" width="12" height="12"/></a><code>
</code><code class="p">}</code><code class="p">;</code></pre>"##;
    let md = convert_html_to_markdown(html);
    assert!(!md.starts_with("<a id="));
    assert!(md.contains("mockDB.hash = bcrypt.hashSync(password, 10);  ①"));
    assert!(md.contains("console.log(\"Password has been saved!\");  ②"));
}

#[test]
fn test_calloutlist_spacing() {
    let html = r##"<dl class="calloutlist">
<dt><a class="co" id="callout_1" href="#co_1"><img src="assets/1.png" alt="1" width="12" height="12"/></a></dt>
<dd><p>This is the name of your project.</p></dd>
<dt><a class="co" id="callout_2" href="#co_2"><img src="assets/2.png" alt="2" width="12" height="12"/></a></dt>
<dd><p><em>index.js</em> is where your application will start.</p></dd>
</dl>"##;
    let md = convert_html_to_markdown(html);
    let blocks = parse_to_blocks(&md);
    assert_eq!(blocks.len(), 1, "Expected single List block");
    match &blocks[0] {
        Block::List {
            ordered: false,
            items,
            ..
        } => {
            assert_eq!(items.len(), 2);
            let item1 = flatten_inlines(&items[0].content);
            assert!(item1.contains("①"));
            assert!(item1.contains("This is the name of your project."));
            let item2 = flatten_inlines(&items[1].content);
            assert!(item2.contains("②"));
            assert!(item2.contains("index.js"));
            assert!(item2.contains("where your application will start."));
        }
        other => panic!("Expected List block, got: {other:?}"),
    }
}

#[test]
fn test_ol_with_p_conversion() {
    let html = r#"<ol>
<li>
<p>Download the appropriate installer for your Mac.</p>
</li>
<li>
<p>Open the downloaded file and drag the VS Code icon to the <em>Applications</em> folder.</p>
</li>
<li>
<p>Open VS Code from the <em>Applications</em> folder.</p>
</li>
</ol>"#;
    let md = convert_html_to_markdown(html);
    let blocks = parse_to_blocks(&md);
    assert_eq!(blocks.len(), 1, "Expected single ordered List block");
    match &blocks[0] {
        Block::List {
            ordered: true,
            start_number: 1,
            items,
        } => {
            assert_eq!(items.len(), 3);
            assert!(
                flatten_inlines(&items[0].content).contains("Download the appropriate installer")
            );
            assert!(flatten_inlines(&items[1].content).contains("Open the downloaded file"));
            assert!(flatten_inlines(&items[2].content).contains("Open VS Code"));
        }
        other => panic!("Expected List block, got: {other:?}"),
    }
}

#[test]
fn test_resolve_relative_path_leading_slash_preserves_parent() {
    let base = "OEBPS/content.opf";
    let relative = "/images/cover.jpg";
    let resolved = opf::resolve_relative_path(base, relative);
    assert_eq!(resolved, "OEBPS/images/cover.jpg");

    let relative_no_slash = "images/cover.jpg";
    let resolved_no_slash = opf::resolve_relative_path(base, relative_no_slash);
    assert_eq!(resolved_no_slash, "OEBPS/images/cover.jpg");
}

#[test]
fn test_extract_first_paragraph_snippet_skips_non_p_tags() {
    use crate::features::common::parser::html::extract_first_paragraph_snippet;

    let html = r#"
        <picture>
            <source srcset="image.webp" type="image/webp">
            <img src="image.jpg" alt="test">
        </picture>
        <pre><code>some code block</code></pre>
        <p>This is the actual first paragraph of the chapter.</p>
    "#;
    let snippet = extract_first_paragraph_snippet(html);
    assert_eq!(
        snippet,
        Some("This is the actual first paragraph of th...".to_string())
    );
}

#[test]
fn test_epub3_navpoints_intra_document_anchor_inheritance() {
    let nav_html = r##"
        <nav epub:type="toc">
            <ol>
                <li><a href="text/part1.xhtml">Part 1</a></li>
                <li><a href="#section1">Section 1 (intra-document)</a></li>
                <li><a href="#section2">Section 2 (intra-document)</a></li>
            </ol>
        </nav>
    "##;
    let navpoints = crate::features::epub::parser::ncx::extract_epub3_navpoints(nav_html);
    assert_eq!(navpoints.len(), 3);
    assert_eq!(navpoints[0].2, "text/part1.xhtml");
    assert_eq!(navpoints[0].3, None);

    assert_eq!(navpoints[1].2, "text/part1.xhtml");
    assert_eq!(navpoints[1].3.as_deref(), Some("section1"));

    assert_eq!(navpoints[2].2, "text/part1.xhtml");
    assert_eq!(navpoints[2].3.as_deref(), Some("section2"));
}

#[test]
fn test_extract_blocks_styles_with_hr_table_and_list() {
    let html = r#"
        <h1 class="chapter_title">Chapter Title</h1>
        <hr class="separator"/>
        <p class="styled_para">Styled paragraph</p>
        <ul class="styled_list">
            <li>Item 1</li>
            <li>Item 2</li>
        </ul>
        <p class="final_para">Final paragraph</p>
    "#;
    let md = convert_html_to_markdown(html);
    let blocks = parse_to_blocks(&md);

    let mut stylesheet = HashMap::new();
    stylesheet.insert(
        "chapter_title".to_string(),
        css::BlockStyle {
            align: css::TextAlign::Center,
            text_indent: 0.0,
            is_bold: true,
            is_italic: false,
            is_uppercase: false,
            is_underline: false,
            is_strikethrough: false,
            font_scale: None,
            ..Default::default()
        },
    );
    stylesheet.insert(
        "styled_para".to_string(),
        css::BlockStyle {
            align: css::TextAlign::Justify,
            text_indent: 20.0,
            is_bold: false,
            is_italic: false,
            is_uppercase: false,
            is_underline: false,
            is_strikethrough: false,
            font_scale: None,
            ..Default::default()
        },
    );
    stylesheet.insert(
        "final_para".to_string(),
        css::BlockStyle {
            align: css::TextAlign::Right,
            text_indent: 0.0,
            is_bold: false,
            is_italic: true,
            is_uppercase: false,
            is_underline: false,
            is_strikethrough: false,
            font_scale: None,
            ..Default::default()
        },
    );

    let styles = extract_blocks_styles(html, &blocks, &stylesheet);

    // Find block indices for heading, styled paragraph, and final paragraph
    let mut heading_idx = None;
    let mut styled_p_idx = None;
    let mut final_p_idx = None;

    for (i, b) in blocks.iter().enumerate() {
        match b {
            Block::Heading { content, .. }
                if flatten_inlines(content).contains("Chapter Title") =>
            {
                heading_idx = Some(i);
            }
            Block::Paragraph(content) if flatten_inlines(content).contains("Styled paragraph") => {
                styled_p_idx = Some(i);
            }
            Block::Paragraph(content) if flatten_inlines(content).contains("Final paragraph") => {
                final_p_idx = Some(i);
            }
            _ => {}
        }
    }

    assert!(heading_idx.is_some());
    assert!(styled_p_idx.is_some());
    assert!(final_p_idx.is_some());

    let h_style = styles
        .get(&heading_idx.unwrap())
        .expect("Heading style missing");
    assert_eq!(h_style.align, css::TextAlign::Center);
    assert!(h_style.is_bold);

    let p_style = styles
        .get(&styled_p_idx.unwrap())
        .expect("Styled para style missing");
    assert_eq!(p_style.align, css::TextAlign::Justify);
    assert_eq!(p_style.text_indent, 20.0);

    let final_style = styles
        .get(&final_p_idx.unwrap())
        .expect("Final para style missing");
    assert_eq!(final_style.align, css::TextAlign::Right);
    assert!(final_style.is_italic);
}

#[test]
fn test_extract_selective_images_percent_encoded_path() {
    use std::io::Cursor;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buf));
        let options = SimpleFileOptions::default();
        zip.start_file("OEBPS/images/my image with space.png", options)
            .unwrap();
        zip.write_all(b"fake image data").unwrap();
        zip.finish().unwrap();
    }

    let mut archive = zip::ZipArchive::new(Cursor::new(&buf)).unwrap();
    let blocks = vec![Block::Image {
        alt: "Image".to_string(),
        path: "images/my%20image%20with%20space.png".to_string(),
        link_url: None,
        width: None,
        height: None,
    }];

    let images = image::extract_selective_images(
        &mut archive,
        "OEBPS/content.opf",
        None,
        &blocks,
        Some("text/ch1.xhtml"),
    );

    // Verify image was found under the original percent-encoded block path
    assert!(
        images.contains_key("images/my%20image%20with%20space.png"),
        "Image mapping must contain original percent-encoded key"
    );
    assert_eq!(
        images.get("images/my%20image%20with%20space.png").unwrap(),
        b"fake image data"
    );
}

#[test]
fn test_convert_html_trailing_whitespace_inside_formatting_tags() {
    let html = r#"<p><span class="text_">Điều bạn sẽ rút ra:&nbsp;</span>Hiểu rõ những lý do sâu xa để trở nên năng suất hơn.</p>"#;
    let md = convert_html_to_markdown(html);
    assert!(
        md.starts_with("**Điều bạn sẽ rút ra:**"),
        "Delimiter must close before trailing whitespace to adhere to CommonMark right-flanking rules. Got: {md}"
    );

    let blocks = parse_to_blocks(&md);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Paragraph(inlines) => {
            // First inline must be bold, not text with literal `**`
            assert!(
                matches!(inlines[0], Inline::Bold(_)),
                "First inline should be parsed as Inline::Bold, got {:?}",
                inlines[0]
            );
        }
        other => panic!("Expected Paragraph block, got {other:?}"),
    }
}

#[test]
fn test_convert_html_leading_whitespace_inside_formatting_tags() {
    let html = r#"<p><span class="text_">Về bản chất, năng suất có nghĩa là bạn&nbsp;</span><span class="text_3">hoàn thành</span><span class="text_">&nbsp;được bao nhiêu.</span></p>"#;
    let md = convert_html_to_markdown(html);

    // Opening delimiter must come AFTER the leading non-breaking space (never `**\u{a0}được` or `** được`)
    assert!(
        !md.contains("**\u{a0}") && !md.contains("** được"),
        "Opening bold delimiter must come AFTER the leading whitespace, got: {md}"
    );
    assert!(
        md.contains("**được bao nhiêu.**"),
        "Delimiter should enclose 'được bao nhiêu.', got: {md}"
    );

    let blocks = parse_to_blocks(&md);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Paragraph(inlines) => {
            // There should be NO raw `**` text leaked in any inline
            for inline in inlines {
                if let Inline::Text(t) = inline {
                    assert!(
                        !t.contains("**"),
                        "Markdown delimiter `**` leaked as literal text: {t}"
                    );
                }
            }
        }
        other => panic!("Expected Paragraph block, got {other:?}"),
    }
}

#[test]
fn test_convert_html_table_cell_formatting() {
    let html = r#"<table>
        <tr>
            <th><b>Header 1</b></th>
            <th><i>Header 2</i></th>
        </tr>
        <tr>
            <td><u>Underlined</u> cell</td>
            <td><s>Strikethrough</s> &amp; <b>bold</b></td>
        </tr>
    </table>"#;

    let md = convert_html_to_markdown(html);
    assert!(
        md.contains("**Header 1**"),
        "Table header bold missing: {md}"
    );
    assert!(
        md.contains("*Header 2*"),
        "Table header italic missing: {md}"
    );
    assert!(
        md.contains("<u>Underlined</u>"),
        "Table cell underline missing: {md}"
    );
    assert!(
        md.contains("~~Strikethrough~~"),
        "Table cell strikethrough missing: {md}"
    );
    assert!(md.contains("**bold**"), "Table cell bold missing: {md}");
}

#[test]
fn test_convert_html_u_and_ins_tags() {
    let html = "<p>This is <u>underlined</u> and <ins>inserted</ins> text.</p>";
    let md = convert_html_to_markdown(html);
    assert_eq!(md, "This is <u>underlined</u> and <u>inserted</u> text.");

    let blocks = parse_to_blocks(&md);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Paragraph(inlines) => {
            let underline_count = inlines
                .iter()
                .filter(|i| matches!(i, Inline::Underline(_)))
                .count();
            assert_eq!(underline_count, 2);
        }
        other => panic!("Expected Paragraph block, got {other:?}"),
    }
}

#[test]
fn test_convert_html_span_style_text_decoration() {
    let html = r#"<p>Normal <span style="text-decoration: underline">underlined</span> and <span style="text-decoration: line-through">struck</span></p>"#;
    let md = convert_html_to_markdown(html);
    assert!(md.contains("<u>underlined</u>"));
    assert!(md.contains("~~struck~~"));

    let blocks = parse_to_blocks(&md);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Paragraph(inlines) => {
            assert!(inlines.iter().any(|i| matches!(i, Inline::Underline(_))));
            assert!(
                inlines
                    .iter()
                    .any(|i| matches!(i, Inline::Strikethrough(_)))
            );
        }
        other => panic!("Expected Paragraph block, got {other:?}"),
    }
}

#[test]
fn test_convert_html_with_stylesheet_classes() {
    use super::html::convert_html_to_markdown_with_stylesheet;

    let css = r#"
        .u-class {
            text-decoration: underline;
        }
        .strike-class {
            text-decoration-line: line-through;
        }
    "#;
    let stylesheet = css::parse_stylesheet(css);
    let html = r#"<p><span class="u-class">underlined via class</span> and <span class="strike-class">struck via class</span></p>"#;

    let md = convert_html_to_markdown_with_stylesheet(html, &stylesheet);
    assert!(md.contains("<u>underlined via class</u>"));
    assert!(md.contains("~~struck via class~~"));

    let blocks = parse_to_blocks(&md);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Paragraph(inlines) => {
            assert!(inlines.iter().any(|i| matches!(i, Inline::Underline(_))));
            assert!(
                inlines
                    .iter()
                    .any(|i| matches!(i, Inline::Strikethrough(_)))
            );
        }
        other => panic!("Expected Paragraph block, got {other:?}"),
    }
}

#[test]
fn test_convert_html_nested_formatting() {
    let html = "<p><b><u>bold underline</u></b> and <u><i>italic underline</i></u></p>";
    let md = convert_html_to_markdown(html);

    let blocks = parse_to_blocks(&md);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Paragraph(inlines) => {
            let has_bold_underline = inlines.iter().any(|i| match i {
                Inline::Bold(subs) => subs.iter().any(|s| matches!(s, Inline::Underline(_))),
                Inline::Underline(subs) => subs.iter().any(|s| matches!(s, Inline::Bold(_))),
                _ => false,
            });
            assert!(
                has_bold_underline,
                "Expected nested Bold and Underline inlines"
            );

            let has_italic_underline = inlines.iter().any(|i| match i {
                Inline::Italic(subs) => subs.iter().any(|s| matches!(s, Inline::Underline(_))),
                Inline::Underline(subs) => subs.iter().any(|s| matches!(s, Inline::Italic(_))),
                _ => false,
            });
            assert!(
                has_italic_underline,
                "Expected nested Italic and Underline inlines"
            );
        }
        other => panic!("Expected Paragraph block, got {other:?}"),
    }
}

#[test]
fn test_convert_html_empty_tags_no_leak() {
    let html = "<p><u></u><b></b><i></i>normal text</p>";
    let md = convert_html_to_markdown(html);
    assert_eq!(md, "normal text");
}

#[test]
fn test_block_style_underline_resolution() {
    let css = r#"
        p.underlined_para {
            text-decoration: underline;
        }
    "#;
    let stylesheet = css::parse_stylesheet(css);
    let html = r#"<p class="underlined_para">This is an entire underlined paragraph.</p>"#;
    let blocks = parse_to_blocks("This is an entire underlined paragraph.");
    let styles = extract_blocks_styles(html, &blocks, &stylesheet);
    assert_eq!(styles.len(), 1);
    let p_style = styles.get(&0).expect("expected style at index 0");
    assert!(p_style.is_underline);
}

#[test]
fn test_epub_why_nations_fail_notes_chapter15_style() {
    let epub_path =
        std::path::Path::new("/home/mintori/Downloads/Vì sao các quốc gia thất bại.epub");
    if !epub_path.exists() {
        return;
    }
    let (blocks, _, styles) =
        load_chapter_content_and_images_from_epub(epub_path, "Text/028.html", None)
            .expect("Must load 028.html");

    let mut found = false;
    for (i, b) in blocks.iter().enumerate() {
        if let Block::Paragraph(inlines) = b {
            let text = flatten_inlines(inlines);
            if text.contains("Chương 15:") {
                assert!(
                    !text.contains("****"),
                    "Text must not leak spurious asterisks: {text}"
                );
                let style = styles.get(&i);
                println!("FOUND CHUONG 15 AT BLOCK {i}: style = {style:?}, text = {text}");
                let s = style.expect("Chương 15 block must have style");
                assert_eq!(s.align, css::TextAlign::Center, "Must be centered");
                assert!(s.is_bold, "Must be bold");
                assert_eq!(s.font_scale, Some(1.2), "Must have 1.2x font scale");
                found = true;
                break;
            }
        }
    }
    assert!(found, "Must find Chương 15 block in 028.html");
}

#[test]
fn test_coalesce_adjacent_formatting_spans_vietnamese() {
    let mut stylesheet = HashMap::new();
    stylesheet.insert(
        "t15".to_string(),
        css::BlockStyle {
            is_bold: true,
            ..Default::default()
        },
    );
    stylesheet.insert(
        "t5".to_string(),
        css::BlockStyle {
            is_bold: true,
            ..Default::default()
        },
    );
    stylesheet.insert(
        "t3".to_string(),
        css::BlockStyle {
            is_bold: true,
            ..Default::default()
        },
    );

    // 1. Nguyễn Quang A
    let html1 = r#"<p><span class="t15">Nguy</span><span class="t15">ễn Quang A</span></p>"#;
    let md1 = super::html::convert_html_to_markdown_with_stylesheet(html1, &stylesheet);
    assert_eq!(md1, "**Nguyễn Quang A**");

    // 2. Kenneth J. Arrow, giải Nobel
    let html2 = r#"<p>- <span class="t5">Kenneth J. Arrow, gi</span><span class="t5">ải Nobel Kinh tế, 1972</span>.</p>"#;
    let md2 = super::html::convert_html_to_markdown_with_stylesheet(html2, &stylesheet);
    assert_eq!(md2, "\\- **Kenneth J. Arrow, giải Nobel Kinh tế, 1972**.");

    // 3. Vì sao các Quốc gia Thất bại
    let html3 =
        r#"<p><span class="t3">Vì sao các Qu</span><span class="t3">ốc gia Thất bại</span></p>"#;
    let md3 = super::html::convert_html_to_markdown_with_stylesheet(html3, &stylesheet);
    assert_eq!(md3, "**Vì sao các Quốc gia Thất bại**");

    // 4. Verify in real EPUB chapters 009.html and 003.html
    let epub_path =
        std::path::Path::new("/home/mintori/Downloads/Vì sao các quốc gia thất bại.epub");
    if epub_path.exists() {
        let (blocks_009, _, _) =
            load_chapter_content_and_images_from_epub(epub_path, "Text/009.html", None)
                .expect("Must load 009.html");
        let all_text_009: Vec<_> = blocks_009
            .iter()
            .filter_map(|b| match b {
                Block::Paragraph(inlines) => Some(flatten_inlines(inlines)),
                _ => None,
            })
            .collect();
        assert!(
            all_text_009
                .iter()
                .any(|t| t.contains("Nguyễn Quang A") && !t.contains("****")),
            "009.html must contain clean 'Nguyễn Quang A' without spurious ****: {all_text_009:?}"
        );

        let (blocks_003, _, _) =
            load_chapter_content_and_images_from_epub(epub_path, "Text/003.html", None)
                .expect("Must load 003.html");
        let all_text_003: Vec<_> = blocks_003
            .iter()
            .filter_map(|b| match b {
                Block::Paragraph(inlines) => Some(flatten_inlines(inlines)),
                _ => None,
            })
            .collect();
        assert!(
            all_text_003
                .iter()
                .any(|t| t.contains("Quốc gia Thất bại") && !t.contains("****")),
            "003.html must contain clean 'Quốc gia Thất bại' without spurious ****: {all_text_003:?}"
        );
        assert!(
            all_text_003
                .iter()
                .any(|t| t.contains("giải Nobel Kinh tế") && !t.contains("****")),
            "003.html must contain clean 'giải Nobel Kinh tế' without spurious ****: {all_text_003:?}"
        );
    }
}

#[test]
fn test_block_style_inherits_from_child_spans() {
    let html = r#"<p class="p26"><span class="t27">Chương 15: Hiểu sự Thịnh vượng</span></p>"#;
    let md = convert_html_to_markdown(html);
    let blocks = parse_to_blocks(&md);

    let mut stylesheet = HashMap::new();
    stylesheet.insert(
        "p26".to_string(),
        css::BlockStyle {
            align: css::TextAlign::Center,
            ..Default::default()
        },
    );
    stylesheet.insert(
        "t27".to_string(),
        css::BlockStyle {
            is_bold: true,
            font_scale: Some(1.2),
            ..Default::default()
        },
    );

    let styles = extract_blocks_styles(html, &blocks, &stylesheet);
    let s = styles.get(&0).expect("Block 0 must have styles");
    assert_eq!(s.align, css::TextAlign::Center);
    assert!(s.is_bold, "Block should inherit bold from child span");
    assert_eq!(
        s.font_scale,
        Some(1.2),
        "Block should inherit font_scale from child span"
    );
}

#[test]
fn test_block_style_does_not_inherit_drop_cap_from_child_spans_on_body_paragraph() {
    let html = r#"<p class="p2"><span class="t1">T</span>HÀNH PHỐ NOGALES bị cắt đôi bởi một hàng rào. Nếu bạn đứng cạnh nó và nhìn theo hướng bắc, bạn sẽ thấy Nogales, Arizona, nằm ở Quận Santa Cruz. Thu nhập của hộ gia đình trung bình ở đó là khoảng 30.000 USD một năm. Hầu hết trẻ vị thành niên ở trường học, và đa số những người lớn là những người đã tốt nghiệp trung học.</p>"#;
    let md = convert_html_to_markdown(html);
    let blocks = parse_to_blocks(&md);

    let mut stylesheet = HashMap::new();
    stylesheet.insert(
        "p2".to_string(),
        css::BlockStyle {
            text_indent: 0.0,
            ..Default::default()
        },
    );
    stylesheet.insert(
        "t1".to_string(),
        css::BlockStyle {
            is_bold: true,
            font_scale: Some(1.3),
            ..Default::default()
        },
    );

    let styles = extract_blocks_styles(html, &blocks, &stylesheet);
    if let Some(s) = styles.get(&0) {
        assert!(
            !s.is_bold,
            "Body paragraph should NOT inherit bold from drop-cap span"
        );
        assert_eq!(
            s.font_scale, None,
            "Body paragraph should NOT inherit font_scale from drop-cap span"
        );
    }

    // Paragraph with class="h" should NOT be falsely recognized as a heading candidate
    let html_h = r#"<p class="h"><span class="t1">A</span> short text.</p>"#;
    let md_h = convert_html_to_markdown(html_h);
    let blocks_h = parse_to_blocks(&md_h);
    let styles_h = extract_blocks_styles(html_h, &blocks_h, &stylesheet);
    if let Some(s) = styles_h.get(&0) {
        assert!(
            !s.is_bold,
            "Paragraph with class='h' must NOT inherit drop-cap bold"
        );
    }
}

#[test]
fn test_heading_with_br_keeps_multiline_softbreak() {
    let html = r#"<h1 class="h11">1.<br />GẦN THẾ VÀ VẪN RẤT KHÁC NHAU</h1>"#;
    let md = convert_html_to_markdown(html);
    let blocks = parse_to_blocks(&md);

    assert_eq!(blocks.len(), 1, "Must produce exactly one block");
    if let Block::Heading { level, content } = &blocks[0] {
        assert_eq!(*level, 1);
        assert!(
            content.iter().any(|i| matches!(i, Inline::SoftBreak)),
            "Heading must contain Inline::SoftBreak for <br>"
        );
    } else {
        panic!("Block must be a Heading");
    }
}

#[test]
fn test_epub_chapter_1_opening_paragraph_not_polluted_by_drop_cap() {
    let epub_path =
        std::path::Path::new("/home/mintori/Downloads/Vì sao các quốc gia thất bại.epub");
    if !epub_path.exists() {
        return;
    }
    let (blocks, _, styles) =
        load_chapter_content_and_images_from_epub(epub_path, "Text/011.html", None)
            .expect("Must load 011.html");

    // Check heading has softbreak
    let mut found_h1 = false;
    for b in &blocks {
        if let Block::Heading { level, content } = b
            && *level == 1
        {
            assert!(
                content.iter().any(|i| matches!(i, Inline::SoftBreak)),
                "Chapter 1 heading should contain SoftBreak for <br>"
            );
            found_h1 = true;
            break;
        }
    }
    assert!(found_h1, "Must find H1 in 011.html");

    // Check opening paragraph: NOGALES
    let mut found_para = false;
    for (i, b) in blocks.iter().enumerate() {
        if let Block::Paragraph(inlines) = b {
            let text = flatten_inlines(inlines);
            if text.contains("NOGALES") {
                if let Some(s) = styles.get(&i) {
                    assert!(
                        !s.is_bold,
                        "Opening paragraph must NOT be bold (no drop cap pollution)"
                    );
                    assert_ne!(
                        s.font_scale,
                        Some(1.3),
                        "Opening paragraph must NOT inherit drop-cap 1.3x scale"
                    );
                    assert_eq!(
                        s.font_scale,
                        Some(1.09),
                        "Opening paragraph must keep standard p tag 1.09x scale"
                    );
                }
                found_para = true;
                break;
            }
        }
    }
    assert!(found_para, "Must find NOGALES paragraph in 011.html");
}

#[test]
fn test_epub_chapter1_heading_center_and_tag_selector() {
    let css = r#"
        h1 {
            text-align: center;
            font-weight: bold;
            font-size: 145%;
        }
        .p9 {
            text-indent: 1.2em;
            text-align: center;
        }
    "#;
    let stylesheet = css::parse_stylesheet(css);
    assert!(
        stylesheet.contains_key("h1"),
        "Must parse tag selector 'h1'"
    );
    assert!(
        stylesheet.contains_key("p9"),
        "Must parse class selector 'p9'"
    );

    let h1_style = stylesheet.get("h1").unwrap();
    assert_eq!(h1_style.align, css::TextAlign::Center);
    assert!(h1_style.is_bold);
    assert_eq!(h1_style.font_scale, Some(1.45));

    let epub_path =
        std::path::Path::new("/home/mintori/Downloads/Vì sao các quốc gia thất bại.epub");
    if epub_path.exists() {
        let (blocks, _, styles) =
            load_chapter_content_and_images_from_epub(epub_path, "Text/011.html", None)
                .expect("Must load 011.html");

        let mut found_h1 = false;
        let mut found_subheading = false;

        for (i, b) in blocks.iter().enumerate().take(10) {
            match b {
                Block::Heading { level: 1, content } => {
                    let text = flatten_inlines(content);
                    if text.contains("1. GẦN THẾ VÀ VẪN RẤT KHÁC NHAU") {
                        let s = styles.get(&i).expect("H1 block must have style");
                        assert_eq!(s.align, css::TextAlign::Center, "H1 must be centered");
                        assert!(s.is_bold, "H1 must be bold");
                        assert_eq!(s.text_indent, 0.0, "Centered H1 must have text_indent = 0");
                        found_h1 = true;
                    }
                }
                Block::Paragraph(inlines) => {
                    let text = flatten_inlines(inlines);
                    if text.contains("NỀN KINH TẾ CỦA RIO GRANDE") {
                        let s = styles.get(&i).expect("Subheading must have style");
                        assert_eq!(
                            s.align,
                            css::TextAlign::Center,
                            "Subheading must be centered"
                        );
                        assert_eq!(
                            s.text_indent, 0.0,
                            "Centered subheading must have text_indent normalized to 0"
                        );
                        found_subheading = true;
                    }
                }
                _ => {}
            }
        }

        assert!(found_h1, "Must find centered H1 in 011.html");
        assert!(
            found_subheading,
            "Must find centered subheading in 011.html"
        );
    }
}

#[test]
fn test_comprehensive_scan_why_nations_fail_epub() {
    let epub_path =
        std::path::Path::new("/home/mintori/Downloads/Vì sao các quốc gia thất bại.epub");
    if !epub_path.exists() {
        eprintln!("EPUB file not found, skipping comprehensive scan.");
        return;
    }

    println!("\n=== STARTING COMPREHENSIVE SCAN OF WHY NATIONS FAIL EPUB ===");

    // 1. Check TOC / Chapters metadata via EpubParser
    let parser = EpubParser;
    let res = parser.parse(epub_path).expect("EpubParser must parse EPUB");
    let mut violations = Vec::new();

    if let ParsedContent::Epub {
        chapters, title, ..
    } = res
    {
        println!("Book Title: {title}");
        println!("Total TOC chapters extracted: {}", chapters.len());

        for (idx, (ch_title, level, anchor, href, _)) in chapters.iter().enumerate() {
            println!("  [TOC #{idx:02}] (L{level}) href='{href}' anchor={anchor:?}: '{ch_title}'");

            if ch_title.contains("****") {
                violations.push(format!("TOC #{idx} title contains '****': '{ch_title}'"));
            }
            if ch_title.contains("~~~~") {
                violations.push(format!("TOC #{idx} title contains '~~~~': '{ch_title}'"));
            }
            if ch_title.contains("</u>") || ch_title.contains("<u>") {
                violations.push(format!(
                    "TOC #{idx} title contains unhandled u tag: '{ch_title}'"
                ));
            }
        }
    } else {
        panic!("ParsedContent is not Epub!");
    }

    // 2. Open EPUB archive directly and list all HTML/XHTML entries
    let file = File::open(epub_path).expect("Failed to open EPUB file");
    let mut archive = zip::ZipArchive::new(BufReader::new(file)).expect("Failed to open zip");
    let opf_path = read_container_opf_path(&mut archive).expect("OPF path missing");
    let opf_xml = read_archive_entry_string(&mut archive, &opf_path).unwrap_or_default();
    let stylesheet_map = load_all_stylesheets(&mut archive, &opf_path, &opf_xml);

    let html_entries: Vec<String> = (0..archive.len())
        .filter_map(|i| {
            let entry = archive.by_index(i).ok()?;
            let name = entry.name().ok()?.to_string();
            if name.ends_with(".html") || name.ends_with(".xhtml") {
                Some(name)
            } else {
                None
            }
        })
        .collect();

    println!("\nFound {} HTML/XHTML files in EPUB.", html_entries.len());

    fn check_markdown_leaks(md: &str, loc: &str, violations: &mut Vec<String>) {
        if md.contains("****") {
            violations.push(format!(
                "{loc}: contains '****' -> snippet: {}",
                md.chars().take(120).collect::<String>()
            ));
        }
        if md.contains("******") {
            violations.push(format!(
                "{loc}: contains '******' -> snippet: {}",
                md.chars().take(120).collect::<String>()
            ));
        }
        if md.contains("~~~~") {
            violations.push(format!(
                "{loc}: contains '~~~~' -> snippet: {}",
                md.chars().take(120).collect::<String>()
            ));
        }
        // Check for orphaned </u> or mismatched underline tags
        let open_u_count = md.matches("<u>").count();
        let close_u_count = md.matches("</u>").count();
        if open_u_count != close_u_count {
            violations.push(format!(
                "{loc}: mismatched underline tags (<u>: {open_u_count}, </u>: {close_u_count})"
            ));
        }
    }

    use crate::parsers::markdown::flatten_inlines_visual;

    fn check_visual_text(visual: &str, loc: &str, violations: &mut Vec<String>) {
        if visual.contains("****") {
            violations.push(format!(
                "{loc} [Visual]: contains '****' -> snippet: {}",
                visual.chars().take(120).collect::<String>()
            ));
        }
        if visual.contains("~~~~") {
            violations.push(format!(
                "{loc} [Visual]: contains '~~~~' -> snippet: {}",
                visual.chars().take(120).collect::<String>()
            ));
        }
        if visual.contains("</u>") || visual.contains("<u>") {
            violations.push(format!(
                "{loc} [Visual]: leaked raw u tag in visual text -> snippet: {}",
                visual.chars().take(120).collect::<String>()
            ));
        }
    }

    fn check_block(b: &Block, loc: &str, violations: &mut Vec<String>) {
        match b {
            Block::Heading { content, .. } => {
                let md = flatten_inlines(content);
                check_markdown_leaks(&md, &format!("{loc} [Heading MD]"), violations);
                let visual = flatten_inlines_visual(content);
                check_visual_text(&visual, loc, violations);
            }
            Block::Paragraph(content) => {
                let md = flatten_inlines(content);
                check_markdown_leaks(&md, &format!("{loc} [Paragraph MD]"), violations);
                let visual = flatten_inlines_visual(content);
                check_visual_text(&visual, loc, violations);
            }
            Block::List { items, .. } => {
                for (item_i, item) in items.iter().enumerate() {
                    let md = flatten_inlines(&item.content);
                    check_markdown_leaks(
                        &md,
                        &format!("{loc} [List item {item_i} MD]"),
                        violations,
                    );
                    let visual = flatten_inlines_visual(&item.content);
                    check_visual_text(&visual, &format!("{loc} [List item {item_i}]"), violations);
                }
            }
            Block::Table(tbl) => {
                for h in &tbl.headers {
                    let md = flatten_inlines(&h.content);
                    check_markdown_leaks(&md, &format!("{loc} [Table Header MD]"), violations);
                    let visual = flatten_inlines_visual(&h.content);
                    check_visual_text(&visual, &format!("{loc} [Table Header]"), violations);
                }
                for row in &tbl.rows {
                    for cell in row {
                        let md = flatten_inlines(&cell.content);
                        check_markdown_leaks(&md, &format!("{loc} [Table Cell MD]"), violations);
                        let visual = flatten_inlines_visual(&cell.content);
                        check_visual_text(&visual, &format!("{loc} [Table Cell]"), violations);
                    }
                }
            }
            Block::Quote(blocks) => {
                for (q_i, q_b) in blocks.iter().enumerate() {
                    check_block(q_b, &format!("{loc} [Quote block {q_i}]"), violations);
                }
            }
            Block::FootnoteDefinition { content, .. } => {
                for (f_i, f_b) in content.iter().enumerate() {
                    check_block(f_b, &format!("{loc} [Footnote block {f_i}]"), violations);
                }
            }
            _ => {}
        }
    }

    for entry_name in &html_entries {
        let html = read_archive_entry_string(&mut archive, entry_name)
            .unwrap_or_else(|_| panic!("Failed to read {entry_name}"));

        // 1. Raw markdown conversion check
        let md = super::html::convert_html_to_markdown_with_stylesheet(&html, &stylesheet_map);
        check_markdown_leaks(&md, &format!("{entry_name} (raw md)"), &mut violations);

        // 2. High-level load_chapter_content_and_images_from_epub check
        let file_href = if let Some(stripped) = entry_name.strip_prefix("OEBPS/") {
            stripped
        } else {
            entry_name.as_str()
        };

        if let Ok((blocks, _, _)) =
            load_chapter_content_and_images_from_epub(epub_path, file_href, None)
        {
            for (b_idx, b) in blocks.iter().enumerate() {
                check_block(
                    b,
                    &format!("{entry_name} (Block #{b_idx})"),
                    &mut violations,
                );
            }
        } else {
            violations.push(format!(
                "load_chapter_content_and_images_from_epub failed for {file_href}"
            ));
        }
    }

    println!("\n=== SCAN RESULTS ===");
    println!("Total violations detected: {}", violations.len());
    for v in &violations {
        println!("  - {v}");
    }

    assert!(
        violations.is_empty(),
        "Detected {} typography/markdown leak violations:\n{:#?}",
        violations.len(),
        violations
    );
    println!("=== SCAN PASSED PERFECTLY ===");
}

#[test]
fn test_display_none_html_and_meta_filtering() {
    let css = r#"
        .invisible {
            display: none;
        }
        #hidden-banner {
            display: none;
        }
    "#;
    let stylesheet_map = css::parse_stylesheet(css);

    let html = r#"
        <div class="container">
            <h1>Visible Title</h1>
            <p class="invisible">This paragraph is hidden via class</p>
            <div id="hidden-banner">
                <p>Hidden inside div#hidden-banner</p>
            </div>
            <p style="display: none">Inline hidden paragraph</p>
            <p hidden>HTML5 hidden attribute</p>
            <p>Visible content <span>Visible inline</span> <span style="display:none">Hidden span</span></p>
        </div>
    "#;

    let md = convert_html_to_markdown_with_stylesheet(html, &stylesheet_map);
    assert!(md.contains("Visible Title"));
    assert!(md.contains("Visible content"));
    assert!(md.contains("Visible inline"));
    assert!(!md.contains("This paragraph is hidden via class"));
    assert!(!md.contains("Hidden inside div#hidden-banner"));
    assert!(!md.contains("Inline hidden paragraph"));
    assert!(!md.contains("HTML5 hidden attribute"));
    assert!(!md.contains("Hidden span"));

    // Verify extract_html_blocks_meta correctly filters out hidden blocks
    let metas = extract_html_blocks_meta(html, Some(&stylesheet_map));
    let non_empty_metas: Vec<_> = metas.into_iter().filter(|m| !m.is_empty).collect();
    // Only <h1>Visible Title</h1> and <p>Visible content...</p>
    assert_eq!(non_empty_metas.len(), 2);
    assert_eq!(non_empty_metas[0].tag, "h1");
    assert_eq!(non_empty_metas[1].tag, "p");
}

#[test]
fn test_list_style_type_none_rendering_flag() {
    let css = r#"
        ul.no-bullet {
            list-style-type: none;
        }
        #nav-list {
            list-style: none;
        }
    "#;
    let stylesheet_map = css::parse_stylesheet(css);

    let html = r#"
        <ul class="no-bullet">
            <li>First item</li>
            <li>Second item</li>
        </ul>
        <ol id="nav-list">
            <li>Numbered one</li>
        </ol>
    "#;

    let md = convert_html_to_markdown_with_stylesheet(html, &stylesheet_map);
    let blocks = parse_to_blocks(&md);
    let styles = extract_blocks_styles(html, &blocks, &stylesheet_map);

    // Style for first list (ul)
    let ul_style = styles.get(&0).expect("ul style should exist");
    assert!(ul_style.list_style_type_none);

    // Style for second list (ol)
    let ol_style = styles.get(&1).expect("ol style should exist");
    assert!(ol_style.list_style_type_none);
}

#[test]
fn test_void_tags_and_unclosed_tags_hidden_handling() {
    let css = r#"
        .invisible {
            display: none;
        }
    "#;
    let stylesheet_map = css::parse_stylesheet(css);

    // 1. Void tag <img class="invisible"> followed by regular paragraph
    let html_img = r#"
        <div>
            <img class="invisible" src="test.png">
            <p>Paragraph after hidden void img must remain visible</p>
        </div>
    "#;
    let md_img = convert_html_to_markdown_with_stylesheet(html_img, &stylesheet_map);
    assert!(!md_img.contains("test.png"));
    assert!(md_img.contains("Paragraph after hidden void img must remain visible"));

    let metas_img = extract_html_blocks_meta(html_img, Some(&stylesheet_map));
    let non_empty_img: Vec<_> = metas_img.into_iter().filter(|m| !m.is_empty).collect();
    assert_eq!(non_empty_img.len(), 1);
    assert_eq!(non_empty_img[0].tag, "p");

    // 2. Hidden container containing void tags <img src="x"><br> followed by visible p
    let html_container = r#"
        <div style="display:none">
            <img src="x">
            <br>
        </div>
        <p>Visible</p>
    "#;
    let md_container = convert_html_to_markdown_with_stylesheet(html_container, &stylesheet_map);
    assert!(!md_container.contains("![image](x)"));
    assert!(md_container.contains("Visible"));

    let metas_container = extract_html_blocks_meta(html_container, Some(&stylesheet_map));
    let non_empty_container: Vec<_> = metas_container
        .into_iter()
        .filter(|m| !m.is_empty)
        .collect();
    assert_eq!(non_empty_container.len(), 1);
    assert_eq!(non_empty_container[0].tag, "p");
}

#[test]
fn test_id_selector_camel_case_preservation() {
    let css = r#"
        #HeaderTitle {
            font-weight: bold;
            text-align: center;
        }
    "#;
    let map = css::parse_stylesheet(css);
    assert!(map.contains_key("#HeaderTitle"));

    let res =
        css::resolve_block_style(Some("h1"), &[], &[], Some("HeaderTitle"), None, &map, false);
    assert_eq!(res.align, css::TextAlign::Center);
    assert!(res.is_bold);
}

#[test]
fn test_vertical_align_super_sub_and_sub_tag() {
    use crate::features::common::parser::html::to_subscript_unicode;

    // 1. Unicode subscript helper test
    assert_eq!(to_subscript_unicode("0123456789+-=()"), "₀₁₂₃₄₅₆₇₈₉₊₋₌₍₎");
    assert_eq!(to_subscript_unicode("h2o"), "ₕ₂ₒ");

    // 2. <sub> tag output
    let html_sub = "<p>Formula: H<sub>2</sub>O and x<sub>i+1</sub></p>";
    let md_sub = convert_html_to_markdown(html_sub);
    assert!(md_sub.contains("H₂O"));
    assert!(md_sub.contains("xᵢ₊₁"));
    assert!(!md_sub.contains('~'));

    // 3. <span> with inline style vertical-align: sub
    let html_span_sub = r#"<p>Formula: H<span style="vertical-align: sub;">2</span>O</p>"#;
    let md_span_sub = convert_html_to_markdown(html_span_sub);
    assert!(md_span_sub.contains("H₂O"));

    // 4. <span> with inline style vertical-align: super footnote
    let html_span_sup_fn =
        r#"<p>Some claim<span style="vertical-align: super;">1</span> explained.</p>"#;
    let md_span_sup_fn = convert_html_to_markdown(html_span_sup_fn);
    assert!(md_span_sup_fn.contains("[^1]"));

    // 5. <span> with stylesheet vertical-align: super / sub
    let css = r#"
        .footnote-ref {
            vertical-align: super;
        }
        .chem-sub {
            vertical-align: sub;
        }
    "#;
    let map = css::parse_stylesheet(css);
    let html_css = r#"<p>Water H<span class="chem-sub">2</span>O is vital<span class="footnote-ref">2</span>.</p>"#;
    let md_css = convert_html_to_markdown_with_stylesheet(html_css, &map);
    assert!(md_css.contains("H₂O"));
    assert!(md_css.contains("[^2]"));
}

#[test]
fn test_body_style_inheritance_in_extract_blocks_styles() {
    let css = r#"
        body {
            line-height: 1.7;
            font-family: "Georgia", serif;
        }
        .special {
            text-align: center;
        }
    "#;
    let stylesheet_map = css::parse_stylesheet(css);
    let html = r#"
        <body>
            <p>Normal paragraph inheriting body style</p>
            <p class="special">Special centered paragraph</p>
        </body>
    "#;
    let blocks = parse_to_blocks(&convert_html_to_markdown_with_stylesheet(
        html,
        &stylesheet_map,
    ));
    let styles = extract_blocks_styles(html, &blocks, &stylesheet_map);

    // Both paragraphs should inherit body line_height and font_family
    let p0_style = styles.get(&0).expect("block 0 should have style");
    assert_eq!(p0_style.line_height, Some(1.7));
    assert_eq!(p0_style.font_family.as_deref(), Some("Georgia"));

    let p1_style = styles.get(&1).expect("block 1 should have style");
    assert_eq!(p1_style.line_height, Some(1.7));
    assert_eq!(p1_style.font_family.as_deref(), Some("Georgia"));
    assert_eq!(p1_style.align, css::TextAlign::Center);
}

#[test]
fn test_body_style_no_box_model_leakage() {
    let css = r#"
        body {
            line-height: 1.6;
            margin: 20px;
            padding: 15px;
            text-indent: 2em;
        }
        .custom {
            text-align: right;
        }
    "#;
    let stylesheet_map = css::parse_stylesheet(css);
    let html = r#"
        <body>
            <p>Normal paragraph inheriting typography only</p>
            <p class="custom">Custom aligned</p>
        </body>
    "#;
    let blocks = parse_to_blocks(&convert_html_to_markdown_with_stylesheet(
        html,
        &stylesheet_map,
    ));
    let styles = extract_blocks_styles(html, &blocks, &stylesheet_map);

    let p0_style = styles.get(&0).expect("block 0 should have style");
    assert_eq!(p0_style.line_height, Some(1.6));
    // Box model properties MUST NOT leak from body
    assert_eq!(p0_style.margin_top, None);
    assert_eq!(p0_style.margin_bottom, None);
    assert_eq!(p0_style.margin_left, None);
    assert_eq!(p0_style.margin_right, None);
    assert_eq!(p0_style.padding_top, None);
    assert_eq!(p0_style.text_indent, 0.0);

    let p1_style = styles.get(&1).expect("block 1 should have style");
    assert_eq!(p1_style.line_height, Some(1.6));
    assert_eq!(p1_style.align, css::TextAlign::Right);
    assert_eq!(p1_style.margin_top, None);
    assert_eq!(p1_style.padding_top, None);
}

#[test]
fn test_nested_spans_vertical_align() {
    let html = r#"<p>Outer <span style="vertical-align: super;">A<span style="font-style: italic;">B</span>C</span> done.</p>"#;
    let md = convert_html_to_markdown(html);
    // Outer super contains A, *B*, and C, which formats to superscript
    assert!(md.contains("Outer"));
    assert!(md.contains("done."));
}

#[test]
fn test_epub_css_colors_and_highlights() {
    let css = r#"
        .colored {
            color: #0070c0;
        }
        .highlighted {
            background-color: yellow;
        }
    "#;
    let stylesheet_map = css::parse_stylesheet(css);
    assert_eq!(
        stylesheet_map.get("colored").and_then(|s| s.color),
        Some([0, 112, 192, 255])
    );
    assert_eq!(
        stylesheet_map
            .get("highlighted")
            .and_then(|s| s.background_color),
        Some([255, 255, 0, 255])
    );

    let html =
        r#"<p>Text with <span class="colored">blue color</span> and <mark>highlight</mark>.</p>"#;
    let md = convert_html_to_markdown_with_stylesheet(html, &stylesheet_map);
    assert!(md.contains("color:#0070c0"));
    assert!(md.contains("background-color:#ffff00"));

    let blocks = parse_to_blocks(&md);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Paragraph(inlines) => {
            let has_styled = inlines.iter().any(|i| match i {
                Inline::Styled {
                    color, background, ..
                } => *color == Some([0, 112, 192, 255]) || *background == Some([255, 255, 0, 255]),
                _ => false,
            });
            assert!(has_styled, "Should contain Inline::Styled variants");
        }
        _ => panic!("Expected paragraph block"),
    }
}

#[test]
fn test_epub_meaningful_text_detection() {
    let empty_blocks = vec![
        Block::Paragraph(vec![Inline::Text(
            "<a id=\"book-content\"></a>".to_string(),
        )]),
        Block::Paragraph(vec![Inline::Text(
            "<a id=\"sbo-rt-content\"></a>".to_string(),
        )]),
        Block::Image {
            alt: String::new(),
            path: "assets/cover.png".to_string(),
            link_url: None,
            width: Some(1200),
            height: Some(1574),
        },
    ];
    assert!(!super::blocks_contain_meaningful_text(&empty_blocks));

    let readable_blocks = vec![
        Block::Heading {
            level: 1,
            content: vec![Inline::Text("Efficient Node.js".to_string())],
        },
        Block::Paragraph(vec![Inline::Text(
            "A Beyond-the-Basics Guide to Node programming".to_string(),
        )]),
    ];
    assert!(super::blocks_contain_meaningful_text(&readable_blocks));
}

#[test]
fn test_epub_parse_efficient_nodejs_file_if_present() {
    let path = std::path::Path::new(
        "/home/mintori/Documents/Efficient Node_ js_ A Beyond-The-Basics Guide -- Samer Buna -- 2025 -- O'Reilly Media, Incorporated -- isbn13 9781098145194 -- 65b5aab642b8157b28fa78792d9db892 -- Anna’s Archive.epub",
    );
    if !path.exists() {
        return;
    }

    let parser = super::EpubParser;
    let res = parser.parse(path).expect("Failed to parse EPUB");
    let crate::features::common::parser::types::ParsedContent::Epub {
        chapters, images, ..
    } = res
    else {
        panic!("Expected ParsedContent::Epub variant");
    };

    assert!(!chapters.is_empty());
    // Chapter 0 is cover
    assert_eq!(chapters[0].0, "Cover");
    // At least one subsequent chapter has loaded blocks
    let has_loaded_readable = chapters.iter().skip(1).any(|ch| !ch.4.is_empty());
    assert!(
        has_loaded_readable,
        "Should have loaded blocks for first readable chapter"
    );
    // Images must contain cover image
    assert!(!images.is_empty());

    let (ch01_blocks, _ch01_images, ch01_styles) =
        super::load_chapter_content_and_images_from_epub(path, "ch01.html", None)
            .expect("Failed to load ch01");
    // Verify that the body paragraph following Figure 1-1 (Block 45: "Node will read the line...")
    // is NOT polluted with italic or center alignment
    if let Some(p_block) = ch01_blocks.get(45) {
        let text = super::flatten_inlines(match p_block {
            Block::Paragraph(inlines) => inlines,
            _ => panic!("Expected Block::Paragraph at index 45"),
        });
        assert!(text.starts_with("Node will read the line"));
        if let Some(style) = ch01_styles.get(&45) {
            assert_ne!(
                style.align,
                css::TextAlign::Center,
                "Body paragraph following figure must not be centered"
            );
            assert!(
                !style.is_italic,
                "Body paragraph following figure must not be italicized"
            );
        }
    }

    // Verify paragraphs in ch01 do not have black color from `.p` class selector collision
    for (idx, block) in ch01_blocks.iter().enumerate() {
        if let Block::Paragraph(inlines) = block {
            let text = super::flatten_inlines(inlines);
            if (text.starts_with("In its core") || text.starts_with("However, this definition"))
                && let Some(style) = ch01_styles.get(&idx)
            {
                assert_ne!(
                    style.color,
                    Some([0, 0, 0, 255]),
                    "Paragraph at index {idx} must not have black color from .p collision"
                );
            }
        }
    }

    // Verify ch05.html around "Creating and Publishing Packages"
    let file = std::fs::File::open(path).unwrap();
    let reader = std::io::BufReader::new(file);
    let mut archive = zip::ZipArchive::new(reader).unwrap();
    let opf_path = super::read_container_opf_path(&mut archive).unwrap();
    let opf_xml = super::read_archive_entry_string(&mut archive, &opf_path).unwrap_or_default();
    let stylesheet_map = super::load_all_stylesheets(&mut archive, &opf_path, &opf_xml);
    let html = super::read_archive_entry_string(&mut archive, "OEBPS/ch05.html").unwrap();
    let md = convert_html_to_markdown_with_stylesheet(&html, &stylesheet_map);
    let blocks = parse_to_blocks(&md);
    let metas = extract_html_blocks_meta(&html, Some(&stylesheet_map));
    let non_empty_metas: Vec<_> = metas.into_iter().filter(|m| !m.is_empty).collect();
    println!(
        "TOTAL BLOCKS: {}, TOTAL NON-EMPTY METAS: {}",
        blocks.len(),
        non_empty_metas.len()
    );
    // Verify ch05.html paragraphs are strictly left aligned with 0 margin_left and standard font
    let styles = extract_blocks_styles(&html, &blocks, &stylesheet_map);
    for idx in 105..=110 {
        if let Some(style) = styles.get(&idx) {
            assert_eq!(
                style.align,
                css::TextAlign::Left,
                "Block {idx} must be left aligned"
            );
            assert_eq!(
                style.margin_left,
                Some(0.0),
                "Block {idx} must not have margin_left indentation from code block desync"
            );
            assert_ne!(
                style.font_family.as_deref(),
                Some("Ubuntu Mono"),
                "Block {idx} must not receive monospace font from pre desync"
            );
        }
    }
}

#[test]
fn test_css_class_tag_collision_no_black_color() {
    let css = r#"
        #sbo-rt-content pre code.p {
            color: #000000;
        }
    "#;
    let stylesheet_map = css::parse_stylesheet(css);
    let html = r#"
        <div id="sbo-rt-content">
            <p>Normal paragraph text</p>
        </div>
    "#;
    let blocks = parse_to_blocks(&convert_html_to_markdown_with_stylesheet(
        html,
        &stylesheet_map,
    ));
    let styles = extract_blocks_styles(html, &blocks, &stylesheet_map);
    if let Some(p_style) = styles.get(&0) {
        assert_ne!(
            p_style.color,
            Some([0, 0, 0, 255]),
            "Tag <p> must not inherit color: #000 from class .p"
        );
    }
}

#[test]
fn test_callout_internal_heading_does_not_desync_blocks() {
    let css = r#"
        h1.main {
            font-size: 2em;
            text-align: center;
        }
    "#;
    let stylesheet_map = css::parse_stylesheet(css);
    let html = r#"
        <body>
            <h1 class="main">Main Title</h1>
            <p>First paragraph</p>
            <div data-type="tip">
                <h6>Tip</h6>
                <p>Tip inner content</p>
            </div>
            <p>Second paragraph</p>
        </body>
    "#;
    let blocks = parse_to_blocks(&convert_html_to_markdown_with_stylesheet(
        html,
        &stylesheet_map,
    ));
    let styles = extract_blocks_styles(html, &blocks, &stylesheet_map);

    // Block 0: Heading Main Title (centered)
    assert!(matches!(&blocks[0], Block::Heading { .. }));
    let h_style = styles.get(&0).expect("heading style");
    assert_eq!(h_style.align, css::TextAlign::Center);

    // Block 1: First paragraph
    assert!(matches!(&blocks[1], Block::Paragraph(..)));

    // Block 2: Alert (Tip)
    assert!(matches!(&blocks[2], Block::Alert { .. }));

    // Block 3: Second paragraph MUST NOT inherit any heading or callout styles!
    assert!(matches!(&blocks[3], Block::Paragraph(..)));
    if let Some(p2_style) = styles.get(&3) {
        assert_ne!(
            p2_style.align,
            css::TextAlign::Center,
            "Second paragraph must not receive heading styles due to desync"
        );
    }
}

#[test]
fn test_convert_html_adjacent_code_tags_merged() {
    let html = r#"<p>Element <code class="inlineCode">&lt;</code><code class="inlineCode">div</code><code class="inlineCode"> id="app"/</code><code class="inlineCode">&gt;</code> rendered.</p>"#;
    let md = convert_html_to_markdown(html);
    assert_eq!(md, r#"Element `<div id="app"/>` rendered."#);

    let html_call = r#"<p>The <code class="inlineCode">onSubmit</code><code class="inlineCode">()</code> event.</p>"#;
    let md_call = convert_html_to_markdown(html_call);
    assert_eq!(md_call, "The `onSubmit()` event.");

    let html_path = r#"<p>Path: <code class="inlineCode">/</code><code class="inlineCode">api</code><code class="inlineCode">/videos</code> endpoint.</p>"#;
    let md_path = convert_html_to_markdown(html_path);
    assert_eq!(md_path, "Path: `/api/videos` endpoint.");

    // Real EPUB sample from Packt Spring Boot:
    let html_packt = r#"<li class="bulletList-level-1"><code class="inlineCode">&lt;</code><code class="inlineCode">div</code><code class="inlineCode"> id="app"/</code><code class="inlineCode">&gt;</code> is the element that the React component <code class="inlineCode">&lt;</code><code class="inlineCode">App</code><code class="inlineCode"> /</code><code class="inlineCode">&gt;</code> will be rendered at per the <code class="inlineCode">document.getElementById</code><code class="inlineCode">("app")</code> from earlier in this section.</li>"#;
    let md_packt = convert_html_to_markdown(html_packt);
    assert!(md_packt.contains("`<div id=\"app\"/>`"));
    assert!(md_packt.contains("`<App />`"));
    assert!(md_packt.contains("`document.getElementById(\"app\")`"));
    assert!(!md_packt.contains("``"));

    // Ensure separate code tags with actual text or space between them are NOT merged
    let html_separate = r#"<p>Call <code>foo()</code> or <code>bar()</code>.</p>"#;
    let md_separate = convert_html_to_markdown(html_separate);
    assert_eq!(md_separate, "Call `foo()` or `bar()`.");
}

#[test]

fn test_convert_html_coalesces_chapter_number_and_title() {
    let html = r#"<section class="chapter-first"><h1 class="chapterNumber" id="h1_3">1</h1>
<h1 class="chapterTitle" id="h1_4">Core Features of Spring Boot</h1>
<p class="normal">Rod Johnson</p></section>"#;
    let md = convert_html_to_markdown(html);
    assert!(
        md.contains("# Chapter 1: Core Features of Spring Boot"),
        "Output was:\n{md}"
    );
    assert!(
        md.contains("<a id=\"h1_3\"></a>"),
        "Should preserve anchor h1_3. Output was:\n{md}"
    );
    assert!(
        md.contains("<a id=\"h1_4\"></a>"),
        "Should preserve anchor h1_4. Output was:\n{md}"
    );

    // Chapter 2 test with different IDs
    let html2 = r#"<section class="chapter"><h1 class="chapterNumber" id="h1_19">2</h1>
<h1 class="chapterTitle" id="h1_20">Creating Web and API Applications with Spring Boot</h1></section>"#;
    let md2 = convert_html_to_markdown(html2);
    assert!(
        md2.contains("# Chapter 2: Creating Web and API Applications with Spring Boot"),
        "Output was:\n{md2}"
    );
    assert!(
        md2.contains("<a id=\"h1_19\"></a>"),
        "Should preserve anchor h1_19. Output was:\n{md2}"
    );
    assert!(
        md2.contains("<a id=\"h1_20\"></a>"),
        "Should preserve anchor h1_20. Output was:\n{md2}"
    );
}

#[test]
fn test_convert_html_demotes_subheadings_when_chapter_title_present() {
    let html = r#"<section class="chapter">
<h1 class="chapterNumber" id="h1_3">1</h1>
<h1 class="chapterTitle" id="h1_4">Core Features of Spring Boot</h1>
<h1 class="heading-1" id="h1_5">Technical requirements</h1>
<h2 class="heading-2" id="h2_6">Installing Java 25</h2>
<p>Some text</p>
</section>"#;
    let md = convert_html_to_markdown(html);
    assert!(
        md.contains("# Chapter 1: Core Features of Spring Boot"),
        "Output was:\n{md}"
    );
    assert!(
        md.contains("## Technical requirements"),
        "heading-1 should be demoted to level 2 (##). Output was:\n{md}"
    );
    assert!(
        md.contains("### Installing Java 25"),
        "heading-2 should be demoted to level 3 (###). Output was:\n{md}"
    );
    assert!(
        md.contains("<a id=\"h1_5\"></a>"),
        "Should preserve id h1_5. Output was:\n{md}"
    );
    assert!(
        md.contains("<a id=\"h2_6\"></a>"),
        "Should preserve id h2_6. Output was:\n{md}"
    );
}

#[test]
fn test_convert_html_does_not_demote_headings_without_chapter_title() {
    let html = r#"<section class="preface">
<h1 class="mainHeading" id="h1_1">Welcome</h1>
<h1 class="heading-1" id="h1_2">Learning Spring Boot 4</h1>
</section>"#;
    let md = convert_html_to_markdown(html);
    assert!(md.contains("# Welcome"), "Output was:\n{md}");
    assert!(
        md.contains("# Learning Spring Boot 4"),
        "heading-1 should remain level 1 (#) when no chapterTitle is present. Output was:\n{md}"
    );
    assert!(
        md.contains("<a id=\"h1_1\"></a>"),
        "Should preserve id h1_1. Output was:\n{md}"
    );
    assert!(
        md.contains("<a id=\"h1_2\"></a>"),
        "Should preserve id h1_2. Output was:\n{md}"
    );
}

#[test]
fn test_split_list_by_code_block_does_not_desync_following_paragraphs() {
    let css = r#"
        ol.steps { margin-left: 20px; }
        pre.snippet { font-family: "Courier"; }
        p.normal { font-family: "Crimson Pro"; font-size: 1.2em; }
    "#;
    let stylesheet_map = css::parse_stylesheet(css);
    let html = r#"
        <ol class="steps">
          <li>Step 1
            <pre class="snippet"><code>npm install</code></pre>
          </li>
          <li>Step 2</li>
          <li>Step 3</li>
        </ol>
        <p class="normal">Paragraph 1 text</p>
        <p class="normal">Paragraph 2 text</p>
    "#;

    let md = convert_html_to_markdown_with_stylesheet(html, &stylesheet_map);
    let blocks = parse_to_blocks(&md);
    let styles = extract_blocks_styles(html, &blocks, &stylesheet_map);

    // Verify blocks structure: List -> CodeBlock -> List continuation -> Paragraph 1 -> Paragraph 2
    assert!(blocks.len() >= 5);
    assert!(matches!(&blocks[0], Block::List { .. }));
    assert!(matches!(&blocks[1], Block::CodeBlock { .. }));
    assert!(matches!(&blocks[2], Block::List { .. }));
    assert!(matches!(&blocks[3], Block::Paragraph(..)));
    assert!(matches!(&blocks[4], Block::Paragraph(..)));

    // Paragraph 1 and Paragraph 2 MUST preserve p.normal style without desync
    let p1_style = styles.get(&3).expect("Paragraph 1 must have style");
    assert_eq!(p1_style.font_family.as_deref(), Some("Crimson Pro"));
    assert_eq!(p1_style.font_scale, Some(1.2));

    let p2_style = styles.get(&4).expect("Paragraph 2 must have style");
    assert_eq!(p2_style.font_family.as_deref(), Some("Crimson Pro"));
    assert_eq!(p2_style.font_scale, Some(1.2));
}

#[test]
fn test_custom_bullet_list_consumes_matching_p_metas() {
    let css = r#"
        p.normal { font-family: "Crimson Pro"; font-size: 1.1em; }
    "#;
    let stylesheet_map = css::parse_stylesheet(css);
    let html = r#"
        <p class="bullet">♦ Item 1</p>
        <p class="bullet">♦ Item 2</p>
        <p class="bullet">♦ Item 3</p>
        <p class="normal">Follow-up text</p>
    "#;

    let md = convert_html_to_markdown_with_stylesheet(html, &stylesheet_map);
    let blocks = parse_to_blocks(&md);
    let styles = extract_blocks_styles(html, &blocks, &stylesheet_map);

    assert_eq!(blocks.len(), 2);
    assert!(matches!(&blocks[0], Block::List { .. }));
    assert!(matches!(&blocks[1], Block::Paragraph(..)));

    let p_style = styles.get(&1).expect("Follow-up text must have style");
    assert_eq!(p_style.font_family.as_deref(), Some("Crimson Pro"));
    assert_eq!(p_style.font_scale, Some(1.1));
}

#[test]
fn test_packt_csrf_styling_uniformity_if_present() {
    let path = std::path::Path::new(
        "/home/mintori/Documents/Learning Spring Boot 4 - 4 Edition - Simplify the -- Wanderson Xesquevixos, Ranga Rao Karanam, Magnus Larsson, -- 2026 -- Packt -- 657ad18699818b1c33c0a6c6d7880e67 -- Anna’s Archive(1).epub",
    );
    if !path.exists() {
        return;
    }
    let res = super::load_chapter_content_and_images_from_epub(path, "Chapter_4.xhtml", None);
    let (blocks, _images, styles) = match res {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error loading chapter: {e:?}");
            return;
        }
    };

    // Find the heading "To CSRF or not to CSRF, that is the question"
    let heading_idx = blocks
        .iter()
        .position(|b| match b {
            Block::Heading { content, .. } => {
                flatten_inlines(content).contains("To CSRF or not to CSRF")
            }
            _ => false,
        })
        .expect("Should find CSRF heading");

    // Check subsequent paragraphs around CSRF section
    for offset in 1..=6 {
        let idx = heading_idx + offset;
        if let Some(Block::Paragraph(_)) = blocks.get(idx) {
            let style = styles.get(&idx);
            assert!(style.is_some(), "Paragraph {idx} should have style");
            let s = style.unwrap();
            assert_eq!(
                s.font_family.as_deref(),
                Some("Crimson Pro"),
                "Paragraph {idx} should be Crimson Pro"
            );
        }
    }

    // Find hierarchical fashion block (from Chapter 71/78 screenshot)
    let pos = blocks
        .iter()
        .position(|b| match b {
            Block::Paragraph(inlines) => flatten_inlines(inlines).contains("hierarchical fashion"),
            _ => false,
        })
        .expect("Should find hierarchical fashion");

    // All paragraphs in this section should consistently be "Crimson Pro"
    for idx in pos..=(pos + 4) {
        if let Some(Block::Paragraph(_)) = blocks.get(idx) {
            let style = styles.get(&idx);
            assert!(style.is_some(), "Paragraph {idx} should have style");
            let s = style.unwrap();
            assert_eq!(
                s.font_family.as_deref(),
                Some("Crimson Pro"),
                "Paragraph {idx} should have Crimson Pro font"
            );
        }
    }

    // Verify Summary section (from Chapter 4 screenshot)
    let summary_pos = blocks
        .iter()
        .position(|b| match b {
            Block::Heading { content, .. } => flatten_inlines(content).contains("Summary"),
            _ => false,
        })
        .expect("Should find Summary heading");

    // Summary heading must be Jost font (heading font)
    let summary_style = styles
        .get(&summary_pos)
        .expect("Summary heading must have style");
    assert_eq!(
        summary_style.font_family.as_deref(),
        Some("Jost"),
        "Summary heading should be Jost"
    );

    // All paragraphs under Summary must consistently have Crimson Pro font (no desync or alternating styles)
    for offset in 1..=3 {
        let idx = summary_pos + offset;
        if let Some(Block::Paragraph(_)) = blocks.get(idx) {
            let style = styles
                .get(&idx)
                .unwrap_or_else(|| panic!("Summary paragraph {idx} must have style"));
            assert_eq!(
                style.font_family.as_deref(),
                Some("Crimson Pro"),
                "Summary paragraph {idx} should have Crimson Pro font"
            );
        }
    }
}

#[test]
fn test_normalize_for_matching_whitespace_and_casing() {
    use super::normalize_for_matching;

    let text = "  Chapter   1: \n  The   BEGINNING  ";
    assert_eq!(normalize_for_matching(text), "chapter 1: the beginning");

    let text_with_anchor = " <a id=\"intro\"></a>  Introduction  to  Oxiview ";
    assert_eq!(
        normalize_for_matching(text_with_anchor),
        "introduction to oxiview"
    );

    let empty = "    ";
    assert_eq!(normalize_for_matching(empty), "");
}

#[test]
fn test_find_block_index_exact_vs_substring_priority() {
    use super::find_block_index;
    use crate::parsers::markdown::{Block, Inline};

    let blocks = vec![
        Block::Heading {
            level: 2,
            content: vec![Inline::Text(
                "Chapter 1: Overview and Introduction".to_string(),
            )],
        },
        Block::Paragraph(vec![Inline::Text("Body paragraph here".to_string())]),
        Block::Heading {
            level: 1,
            content: vec![Inline::Text("Chapter 1".to_string())],
        },
    ];

    // Searching for "Chapter 1" should exact-match block index 2, not substring-match block index 0
    let exact_match = find_block_index(&blocks, None, Some("Chapter 1"));
    assert_eq!(exact_match, Some(2));

    // Searching for "Overview" has no exact heading, so it should match substring block index 0
    let substr_match = find_block_index(&blocks, None, Some("Overview"));
    assert_eq!(substr_match, Some(0));

    // Searching for "Body paragraph" has no heading match, falls back to paragraph block index 1
    let para_match = find_block_index(&blocks, None, Some("Body paragraph"));
    assert_eq!(para_match, Some(1));
}

#[test]
fn test_load_chapter_content_and_images_cached_roundtrip() {
    use super::load_chapter_content_and_images_cached;
    use std::io::Write;

    let temp_dir = std::env::temp_dir();
    let test_epub_path = temp_dir.join(format!("test_cached_css_{}.epub", std::process::id()));

    let file = File::create(&test_epub_path).unwrap();
    let mut zip = ::zip::ZipWriter::new(file);
    let options = ::zip::write::SimpleFileOptions::default();

    zip.start_file("META-INF/container.xml", options).unwrap();
    zip.write_all(br#"<?xml version="1.0"?><container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#).unwrap();

    zip.start_file("content.opf", options).unwrap();
    zip.write_all(br#"<?xml version="1.0"?><package><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>Cache Test</dc:title></metadata><manifest><item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/><item id="style" href="style.css" media-type="text/css"/></manifest><spine><itemref idref="ch1"/></spine></package>"#).unwrap();

    zip.start_file("style.css", options).unwrap();
    zip.write_all(
        br#"
        h1 { font-weight: bold; }
        .intro { text-align: center; }
    "#,
    )
    .unwrap();

    zip.start_file("ch1.xhtml", options).unwrap();
    zip.write_all(br#"<html><head><link rel="stylesheet" href="style.css"/></head><body><h1>Chapter Title</h1><p class="intro">Welcome</p></body></html>"#).unwrap();

    zip.finish().unwrap();

    let first_load =
        load_chapter_content_and_images_cached(&test_epub_path, "ch1.xhtml", None, None);
    assert!(first_load.is_ok(), "Initial chapter load must succeed");
    let ((blocks, _, styles), parsed_css) = first_load.unwrap();
    assert!(!blocks.is_empty());
    assert!(!styles.is_empty());
    assert!(
        parsed_css.is_some(),
        "Initial load without cache must return parsed stylesheets"
    );

    let cached_map = parsed_css.unwrap();
    assert!(cached_map.contains_key("tag:h1") || cached_map.contains_key("h1"));
    assert!(cached_map.contains_key(".intro") || cached_map.contains_key("intro"));

    let second_load = load_chapter_content_and_images_cached(
        &test_epub_path,
        "ch1.xhtml",
        None,
        Some(&cached_map),
    );
    assert!(second_load.is_ok(), "Cached chapter load must succeed");
    let ((blocks2, _, styles2), returned_css) = second_load.unwrap();
    assert_eq!(blocks2.len(), blocks.len());
    assert_eq!(styles2.len(), styles.len());
    assert!(
        returned_css.is_none(),
        "Subsequent load with cache must not re-parse stylesheets"
    );

    let _ = std::fs::remove_file(test_epub_path);
}
