mod html;
mod ncx;
mod opf;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::parsers::{ParseError, ParsedContent, PreviewParser};

use html::{
    convert_html_to_markdown, decode_html_entities, extract_chapter_title_from_html,
    extract_filename, extract_tag_content,
};
use ncx::{NcxNavPoint, extract_ncx_navpoints};
use opf::{extract_ncx_href, extract_opf_path, extract_spine_items, resolve_relative_path};

const DEFAULT_TITLE: &str = "Unknown Title";
const DEFAULT_AUTHOR: &str = "Unknown Author";

pub struct EpubParser;

type SpineCache = HashMap<String, (String, Vec<crate::parsers::markdown::Block>)>;

impl PreviewParser for EpubParser {
    fn supported_extensions(&self) -> &[&str] {
        &["epub"]
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let file = File::open(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        let opf_path = read_container_opf_path(&mut archive)?;
        let opf_xml = read_archive_entry_string(&mut archive, &opf_path)?;

        let title =
            extract_tag_content(&opf_xml, "dc:title").unwrap_or_else(|| DEFAULT_TITLE.to_string());
        let author = extract_tag_content(&opf_xml, "dc:creator")
            .unwrap_or_else(|| DEFAULT_AUTHOR.to_string());

        let spine_items = extract_spine_items(&opf_xml);
        let ncx_href = extract_ncx_href(&opf_xml);

        let ncx_entries = ncx_href
            .and_then(|href| {
                let resolved = resolve_relative_path(&opf_path, &href);
                read_archive_entry_string(&mut archive, &resolved).ok()
            })
            .map(|xml| extract_ncx_navpoints(&xml))
            .unwrap_or_default();

        let spine_cache = cache_spine_contents(&mut archive, &opf_path, &spine_items);

        let mut chapters = build_chapters_from_ncx(&ncx_entries, &spine_cache);
        if chapters.is_empty() {
            chapters = build_chapters_from_spine(&spine_items, &spine_cache, &title);
        }
        if chapters.is_empty() {
            chapters.push(create_fallback_chapter());
        }

        let images = extract_images_from_archive(&mut archive);

        Ok(ParsedContent::Epub {
            title,
            author,
            chapters,
            images,
        })
    }
}

fn read_container_opf_path<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> Result<String, ParseError> {
    let xml = read_archive_entry_string(archive, "META-INF/container.xml")
        .map_err(|_| ParseError::ParseFailed("Missing META-INF/container.xml".into()))?;

    extract_opf_path(&xml)
        .ok_or_else(|| ParseError::ParseFailed("Could not locate OPF file in container.xml".into()))
}

fn read_archive_entry_string<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    entry_name: &str,
) -> Result<String, ParseError> {
    let mut entry = archive
        .by_name(entry_name)
        .map_err(|_| ParseError::ParseFailed(format!("Missing file in archive: {entry_name}")))?;

    let mut bytes = Vec::new();
    entry
        .read_to_end(&mut bytes)
        .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

    Ok(read_bytes_to_string(&bytes))
}

fn cache_spine_contents<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    opf_path: &str,
    spine_items: &[String],
) -> SpineCache {
    let mut cache = HashMap::new();

    for item_path in spine_items {
        let resolved_path = resolve_relative_path(opf_path, item_path);
        if let Ok(html) = read_archive_entry_string(archive, &resolved_path) {
            let markdown_text = convert_html_to_markdown(&html);
            let blocks = crate::parsers::markdown::parse_to_blocks(&markdown_text);
            let filename = extract_filename(item_path);

            cache.insert(filename, (html, blocks));
        }
    }

    cache
}

fn build_chapters_from_ncx(
    ncx_entries: &[NcxNavPoint],
    spine_cache: &SpineCache,
) -> Vec<(
    String,
    u8,
    Option<String>,
    Vec<crate::parsers::markdown::Block>,
)> {
    let mut chapters = Vec::new();
    let total_entries = ncx_entries.len();

    for index in 0..total_entries {
        let (label, level, file_part, anchor) = &ncx_entries[index];
        let filename = extract_filename(file_part);

        let Some((_html, blocks)) = spine_cache.get(&filename) else {
            continue;
        };

        if blocks.is_empty() {
            continue;
        }

        let clean_title = decode_html_entities(label);
        let start_idx =
            find_block_index(blocks, anchor.as_deref(), Some(&clean_title)).unwrap_or(0);

        let next_entry = ncx_entries.get(index + 1);
        let end_idx = calculate_next_ncx_end_index(next_entry, &filename, blocks, start_idx);

        let chapter_blocks = match end_idx {
            Some(end) if end > start_idx => blocks[start_idx..end].to_vec(),
            _ => blocks[start_idx..].to_vec(),
        };

        if !chapter_blocks.is_empty() {
            chapters.push((clean_title, *level, anchor.clone(), chapter_blocks));
        }
    }

    chapters
}

fn calculate_next_ncx_end_index(
    next_entry: Option<&NcxNavPoint>,
    current_filename: &str,
    blocks: &[crate::parsers::markdown::Block],
    start_idx: usize,
) -> Option<usize> {
    let (_next_label, _next_level, next_file_part, next_anchor) = next_entry?;
    let next_filename = extract_filename(next_file_part);

    if next_filename == current_filename {
        let next_anc = next_anchor.as_deref()?;
        find_block_index(&blocks[start_idx..], Some(next_anc), None)
            .map(|offset| start_idx + offset)
    } else {
        None
    }
}

fn find_block_index(
    blocks: &[crate::parsers::markdown::Block],
    anchor: Option<&str>,
    title: Option<&str>,
) -> Option<usize> {
    blocks.iter().position(|block| {
        let text = extract_text_from_block(block);

        let matches_anchor = anchor.is_some_and(|anc| text.contains(anc));
        let matches_title = title.is_some_and(|t| text.contains(t));

        matches_anchor || matches_title
    })
}

fn extract_text_from_block(block: &crate::parsers::markdown::Block) -> String {
    use crate::parsers::markdown::Block;

    match block {
        Block::Heading { content, .. } | Block::Paragraph(content) => {
            crate::parsers::markdown::flatten_inlines(content)
        }
        _ => String::new(),
    }
}

fn build_chapters_from_spine(
    spine_items: &[String],
    spine_cache: &SpineCache,
    book_title: &str,
) -> Vec<(
    String,
    u8,
    Option<String>,
    Vec<crate::parsers::markdown::Block>,
)> {
    let mut chapters = Vec::new();

    for (index, item_path) in spine_items.iter().enumerate() {
        let filename = extract_filename(item_path);

        if let Some((html, blocks)) = spine_cache.get(&filename) {
            if blocks.is_empty() {
                continue;
            }

            let raw_title = extract_chapter_title_from_html(html, book_title)
                .unwrap_or_else(|| format!("Chapter {}", index + 1));
            let clean_title = decode_html_entities(&raw_title);

            chapters.push((clean_title, 1, None, blocks.clone()));
        }
    }

    chapters
}

fn create_fallback_chapter() -> (
    String,
    u8,
    Option<String>,
    Vec<crate::parsers::markdown::Block>,
) {
    (
        "Chapter 1".to_string(),
        1,
        None,
        vec![crate::parsers::markdown::Block::Paragraph(vec![
            crate::parsers::markdown::Inline::Text(
                "[No readable content found in EPUB]".to_string(),
            ),
        ])],
    )
}

fn extract_images_from_archive<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> HashMap<String, Vec<u8>> {
    let mut images = HashMap::new();

    for index in 0..archive.len() {
        if let Ok(mut file) = archive.by_index(index) {
            let name = file.name().to_string();

            if is_image_extension(&name) {
                let mut buffer = Vec::new();
                if file.read_to_end(&mut buffer).is_ok() {
                    let filename = extract_filename(&name);
                    images.insert(name, buffer.clone());
                    images.insert(filename, buffer);
                }
            }
        }
    }

    images
}

fn is_image_extension(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".png")
        || lower.ends_with(".gif")
        || lower.ends_with(".webp")
        || lower.ends_with(".bmp")
        || lower.ends_with(".svg")
}

fn read_bytes_to_string(bytes: &[u8]) -> String {
    let data = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &bytes[3..]
    } else {
        bytes
    };

    String::from_utf8_lossy(data).into_owned()
}
