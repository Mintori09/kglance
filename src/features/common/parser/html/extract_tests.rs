use super::*;

#[test]
fn test_extract_tag_content_kelvin_symbol_panic_fix() {
    // Kelvin symbol 'K' (U+212A) is 3 bytes in UTF-8, but its lowercase 'k' is 1 byte in ASCII.
    // In the old implementation, lower_xml = xml.to_lowercase() caused byte offsets to shift,
    // causing a panic: 'byte index ... is not a char boundary'.
    let xml = "K<dc:title>Temperature Analysis</dc:title>";
    let result = extract_tag_content(xml, "dc:title");
    assert_eq!(result, Some("Temperature Analysis".to_string()));
}

#[test]
fn test_extract_tag_content_german_sharp_s_panic_fix() {
    // German capital sharp S 'ẞ' (U+1E9E) is 3 bytes in UTF-8, lowercase 'ß' is 2 bytes.
    let xml = "ẞ<dc:title>Große Erwartungen</dc:title>";
    let result = extract_tag_content(xml, "dc:title");
    assert_eq!(result, Some("Große Erwartungen".to_string()));
}

#[test]
fn test_extract_tag_content_cherokee_panic_fix() {
    // Cherokee letter 'Ꭰ' (U+13A0) is 3 bytes in UTF-8, lowercase is 4 bytes.
    let xml = "\u{13A0}<dc:title>Cherokee History</dc:title>";
    let result = extract_tag_content(xml, "dc:title");
    assert_eq!(result, Some("Cherokee History".to_string()));
}

#[test]
fn test_extract_tag_content_turkish_i_panic_fix() {
    // Turkish 'İ' (U+0130) is 2 bytes, lowercase is 3 bytes.
    let xml = "\u{0130}<dc:title>İstanbul Hatırası</dc:title>";
    let result = extract_tag_content(xml, "dc:title");
    assert_eq!(result, Some("İstanbul Hatırası".to_string()));
}

#[test]
fn test_extract_tag_content_unicode_inside_content() {
    let xml = "<dc:title>Physics: 300K &amp; Groẞe Entdeckungen</dc:title>";
    let result = extract_tag_content(xml, "dc:title");
    assert_eq!(
        result,
        Some("Physics: 300K & Groẞe Entdeckungen".to_string())
    );
}

#[test]
fn test_extract_tag_content_case_insensitive() {
    assert_eq!(
        extract_tag_content("<DC:TITLE>War and Peace</DC:TITLE>", "dc:title"),
        Some("War and Peace".to_string())
    );
    assert_eq!(
        extract_tag_content("<dc:title>War and Peace</dc:title>", "DC:TITLE"),
        Some("War and Peace".to_string())
    );
    assert_eq!(
        extract_tag_content("<H1>Heading One</H1>", "h1"),
        Some("Heading One".to_string())
    );
}

#[test]
fn test_extract_tag_content_with_attributes() {
    let xml = r#"<dc:title id="title-1" xml:lang="en">Rust Programming</dc:title>"#;
    assert_eq!(
        extract_tag_content(xml, "dc:title"),
        Some("Rust Programming".to_string())
    );

    let xml_quotes = r#"<dc:title attr="val>more">Complex Quotes</dc:title>"#;
    assert_eq!(
        extract_tag_content(xml_quotes, "dc:title"),
        Some("Complex Quotes".to_string())
    );
}

#[test]
fn test_extract_tag_content_nested_html_and_entities() {
    let xml = "<h1>Chapter 1: <i>The Beginning</i> &amp; End</h1>";
    assert_eq!(
        extract_tag_content(xml, "h1"),
        Some("Chapter 1: *The Beginning* & End".to_string())
    );
}

#[test]
fn test_extract_tag_content_distinguishes_tag_prefixes() {
    let xml = "<titlepage>Cover Info</titlepage><title>Real Title</title>";
    assert_eq!(
        extract_tag_content(xml, "title"),
        Some("Real Title".to_string())
    );

    let xml_h = "<h10>Subheading</h10><h1>Header 1</h1>";
    assert_eq!(
        extract_tag_content(xml_h, "h1"),
        Some("Header 1".to_string())
    );
}

#[test]
fn test_extract_tag_content_skips_comments() {
    let xml = "<!-- <dc:title>Old Draft</dc:title> --> <dc:title>Published Title</dc:title>";
    assert_eq!(
        extract_tag_content(xml, "dc:title"),
        Some("Published Title".to_string())
    );
}

#[test]
fn test_extract_tag_content_self_closing_and_empty() {
    assert_eq!(extract_tag_content("<dc:title/>", "dc:title"), None);
    assert_eq!(
        extract_tag_content("<dc:title></dc:title>", "dc:title"),
        None
    );
    assert_eq!(
        extract_tag_content("<dc:title>   </dc:title>", "dc:title"),
        None
    );

    let xml = "<dc:title/> <dc:title>Actual Title</dc:title>";
    assert_eq!(
        extract_tag_content(xml, "dc:title"),
        Some("Actual Title".to_string())
    );
}

#[test]
fn test_extract_tag_content_closing_tag_with_whitespace() {
    let xml = "<dc:title>Whitespace Close</dc:title  >";
    assert_eq!(
        extract_tag_content(xml, "dc:title"),
        Some("Whitespace Close".to_string())
    );
}
