pub mod converter;
pub mod entities;
pub mod extract;

pub use converter::{HtmlToMarkdownConverter, convert_html_to_markdown, strip_html_tags};
pub use entities::{decode_html_entities, normalize_nfc};
pub use extract::{
    HtmlHeading, extract_attribute, extract_chapter_title_from_html, extract_filename,
    extract_first_paragraph_snippet, extract_headings_from_html, extract_id_or_name,
    extract_tag_content,
};
