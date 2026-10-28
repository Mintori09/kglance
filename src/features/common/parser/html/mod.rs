pub mod converter;
pub mod entities;
pub mod extract;

pub use converter::{
    CUSTOM_BULLET_CHARS, HtmlToMarkdownConverter, convert_html_to_markdown, strip_html_tags,
};
pub use entities::{decode_html_entities, normalize_nfc};
pub use extract::{
    HtmlHeading, RawHtmlBlockMeta, extract_attribute, extract_chapter_title_from_html,
    extract_classes, extract_filename, extract_first_paragraph_snippet, extract_headings_from_html,
    extract_html_blocks_meta, extract_id_or_name, extract_inline_style_attr, extract_tag_content,
};
