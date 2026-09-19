mod html;
mod ncx;
mod opf;

#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::features::common::parser::traits::{ParseError, PreviewParser};
use crate::features::common::parser::types::{ParsedContent, ParsedEpubChapter};
use crate::features::markdown::parser::{Block, Inline, flatten_inlines, parse_to_blocks};

use html::{
    convert_html_to_markdown, decode_html_entities, extract_chapter_title_from_html,
    extract_filename, extract_headings_from_html, extract_tag_content,
};
use ncx::{NcxNavPoint, extract_epub3_navpoints, extract_ncx_navpoints};
use opf::{
    extract_cover_image_href, extract_nav_href, extract_ncx_href, extract_opf_path,
    extract_spine_items, resolve_relative_path,
};

const DEFAULT_TITLE: &str = "Unknown Title";
const DEFAULT_AUTHOR: &str = "Unknown Author";

pub struct EpubParser;

pub fn parse_chapter_content<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    opf_path: &str,
    file_href: &str,
    anchor: Option<&str>,
) -> Result<Vec<Block>, ParseError> {
    let resolved_path = resolve_relative_path(opf_path, file_href);
    let html = read_archive_entry_string(archive, &resolved_path)?;
    let md = convert_html_to_markdown(&html);
    let blocks = parse_to_blocks(&md);

    if let Some(anc) = anchor
        && !anc.is_empty()
        && let Some(start_idx) = find_block_index(&blocks, Some(anc), None)
    {
        return Ok(blocks[start_idx..].to_vec());
    }

    Ok(blocks)
}

pub type LoadedChapterContent = (Vec<Block>, HashMap<String, Vec<u8>>);

pub fn load_chapter_content_and_images_from_epub(
    path: &Path,
    file_href: &str,
    anchor: Option<&str>,
) -> Result<LoadedChapterContent, ParseError> {
    let file = File::open(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let opf_path = read_container_opf_path(&mut archive)?;
    let blocks = parse_chapter_content(&mut archive, &opf_path, file_href, anchor)?;
    let images = extract_selective_images(&mut archive, &opf_path, None, &blocks);
    Ok((blocks, images))
}

pub fn load_chapter_from_epub(
    path: &Path,
    file_href: &str,
    anchor: Option<&str>,
) -> Result<Vec<Block>, ParseError> {
    let file = File::open(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let opf_path = read_container_opf_path(&mut archive)?;
    parse_chapter_content(&mut archive, &opf_path, file_href, anchor)
}

pub fn load_images_for_blocks_from_epub(
    archive: &mut zip::ZipArchive<impl Read + std::io::Seek>,
    opf_path: &str,
    blocks: &[Block],
) -> HashMap<String, Vec<u8>> {
    extract_selective_images(archive, opf_path, None, blocks)
}

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
        let nav_href = extract_nav_href(&opf_xml);

        let mut nav_entries = ncx_href
            .and_then(|href| {
                let resolved = resolve_relative_path(&opf_path, &href);
                read_archive_entry_string(&mut archive, &resolved).ok()
            })
            .map(|xml| extract_ncx_navpoints(&xml))
            .unwrap_or_default();

        if nav_entries.is_empty()
            && let Some(href) = nav_href
        {
            let resolved = resolve_relative_path(&opf_path, &href);
            if let Ok(xml) = read_archive_entry_string(&mut archive, &resolved) {
                nav_entries = extract_epub3_navpoints(&xml);
            }
        }

        let mut chapters = if !nav_entries.is_empty() {
            build_lazy_chapters_from_ncx(&mut archive, &opf_path, &nav_entries)
        } else {
            build_lazy_chapters_from_spine(&mut archive, &opf_path, &spine_items, &title)
        };

        if chapters.is_empty() {
            chapters.push(create_fallback_chapter());
        }

        let cover_href = extract_cover_image_href(&opf_xml);
        let initial_blocks = chapters.first().map(|ch| ch.4.as_slice()).unwrap_or(&[]);
        let images = extract_selective_images(
            &mut archive,
            &opf_path,
            cover_href.as_deref(),
            initial_blocks,
        );

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

fn build_lazy_chapters_from_ncx<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    opf_path: &str,
    ncx_entries: &[NcxNavPoint],
) -> Vec<ParsedEpubChapter> {
    let mut chapters = Vec::new();

    for (index, entry) in ncx_entries.iter().enumerate() {
        let (label, level, file_part, anchor) = entry;
        let clean_title = decode_html_entities(label);

        let blocks = if index == 0 {
            parse_chapter_content(archive, opf_path, file_part, anchor.as_deref())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        chapters.push((
            clean_title,
            *level,
            anchor.clone(),
            file_part.clone(),
            blocks,
        ));
    }

    chapters
}

fn build_lazy_chapters_from_spine<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    opf_path: &str,
    spine_items: &[String],
    book_title: &str,
) -> Vec<ParsedEpubChapter> {
    let mut chapters = Vec::new();

    for (index, item_path) in spine_items.iter().enumerate() {
        if index == 0 {
            let resolved_path = resolve_relative_path(opf_path, item_path);
            if let Ok(html) = read_archive_entry_string(archive, &resolved_path) {
                let markdown_text = convert_html_to_markdown(&html);
                let blocks = parse_to_blocks(&markdown_text);
                let headings = extract_headings_from_html(&html);

                if !headings.is_empty() {
                    let total_headings = headings.len();
                    let mut last_end = 0;

                    for h_idx in 0..total_headings {
                        let h = &headings[h_idx];
                        let start_idx =
                            find_block_index(&blocks[last_end..], h.id.as_deref(), Some(&h.title))
                                .map(|offset| last_end + offset)
                                .unwrap_or(last_end);

                        let end_idx = if h_idx + 1 < total_headings {
                            let next_h = &headings[h_idx + 1];
                            find_block_index(
                                &blocks[start_idx..],
                                next_h.id.as_deref(),
                                Some(&next_h.title),
                            )
                            .map(|offset| start_idx + offset)
                        } else {
                            None
                        };

                        let chapter_blocks = match end_idx {
                            Some(end) if end > start_idx => {
                                last_end = end;
                                blocks[start_idx..end].to_vec()
                            }
                            _ => {
                                last_end = blocks.len();
                                blocks[start_idx..].to_vec()
                            }
                        };

                        chapters.push((
                            h.title.clone(),
                            h.level,
                            h.id.clone(),
                            item_path.clone(),
                            chapter_blocks,
                        ));
                    }
                } else {
                    let clean_title = extract_chapter_title_from_html(&html, book_title)
                        .map(|t| decode_html_entities(&t))
                        .unwrap_or_else(|| "Chapter 1".to_string());

                    chapters.push((clean_title, 1, None, item_path.clone(), blocks));
                }
            } else {
                chapters.push((
                    "Chapter 1".to_string(),
                    1,
                    None,
                    item_path.clone(),
                    Vec::new(),
                ));
            }
        } else {
            let clean_title = format!("Chapter {}", index + 1);
            chapters.push((clean_title, 1, None, item_path.clone(), Vec::new()));
        }
    }

    chapters
}

fn find_block_index(blocks: &[Block], anchor: Option<&str>, title: Option<&str>) -> Option<usize> {
    blocks.iter().position(|block| {
        let text = extract_text_from_block(block);

        let matches_anchor = anchor.is_some_and(|anc| !anc.is_empty() && text.contains(anc));
        let matches_title = title.is_some_and(|t| !t.is_empty() && text.contains(t));

        matches_anchor || matches_title
    })
}

fn extract_text_from_block(block: &Block) -> String {
    match block {
        Block::Heading { content, .. } | Block::Paragraph(content) => flatten_inlines(content),
        _ => String::new(),
    }
}

fn create_fallback_chapter() -> ParsedEpubChapter {
    (
        "Chapter 1".to_string(),
        1,
        None,
        String::new(),
        vec![Block::Paragraph(vec![Inline::Text(
            "[No readable content found in EPUB]".to_string(),
        )])],
    )
}

pub fn extract_images_from_blocks(blocks: &[Block]) -> Vec<String> {
    let mut image_paths = Vec::new();
    collect_images_from_blocks(blocks, &mut image_paths);
    image_paths
}

fn collect_images_from_blocks(blocks: &[Block], paths: &mut Vec<String>) {
    for block in blocks {
        match block {
            Block::Image { path, .. } => {
                paths.push(path.clone());
            }
            Block::Paragraph(inlines)
            | Block::Heading {
                content: inlines, ..
            } => {
                collect_images_from_inlines(inlines, paths);
            }
            Block::Quote(sub_blocks)
            | Block::Alert {
                content: sub_blocks,
                ..
            }
            | Block::FootnoteDefinition {
                content: sub_blocks,
                ..
            } => {
                collect_images_from_blocks(sub_blocks, paths);
            }
            Block::List { items, .. } => {
                for item in items {
                    collect_images_from_inlines(&item.content, paths);
                    collect_images_from_blocks(&item.sub_blocks, paths);
                }
            }
            Block::Table(table) => {
                for header in &table.headers {
                    collect_images_from_inlines(&header.content, paths);
                }
                for row in &table.rows {
                    for cell in row {
                        collect_images_from_inlines(&cell.content, paths);
                    }
                }
            }
            _ => {}
        }
    }
}

fn collect_images_from_inlines(inlines: &[Inline], paths: &mut Vec<String>) {
    for inline in inlines {
        match inline {
            Inline::Image { url, .. } => {
                paths.push(url.clone());
            }
            Inline::Bold(subs)
            | Inline::Italic(subs)
            | Inline::Strikethrough(subs)
            | Inline::Link { text: subs, .. } => {
                collect_images_from_inlines(subs, paths);
            }
            _ => {}
        }
    }
}

pub fn extract_selective_images<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    opf_path: &str,
    cover_href: Option<&str>,
    blocks: &[Block],
) -> HashMap<String, Vec<u8>> {
    let mut images = HashMap::new();
    let mut target_names: HashSet<String> = HashSet::new();

    if let Some(href) = cover_href {
        let resolved = resolve_relative_path(opf_path, href);
        target_names.insert(resolved.clone());
        target_names.insert(href.to_string());
        target_names.insert(extract_filename(href));
    }

    let block_imgs = extract_images_from_blocks(blocks);
    for img in block_imgs {
        let resolved = resolve_relative_path(opf_path, &img);
        target_names.insert(resolved);
        let filename = extract_filename(&img);
        target_names.insert(filename);
        target_names.insert(img);
    }

    let mut found_cover = cover_href.is_some();

    for index in 0..archive.len() {
        let Ok(file) = archive.by_index(index) else {
            continue;
        };
        let name = file.name().to_string();
        let filename = extract_filename(&name);

        let matches_target = target_names.contains(&name) || target_names.contains(&filename);
        let matches_cover = is_potential_cover_image(&name, found_cover);

        if is_image_extension(&name) && (matches_target || matches_cover) {
            drop(file);
            if let Ok(mut file) = archive.by_index(index) {
                let mut buffer = Vec::new();
                if file.read_to_end(&mut buffer).is_ok() {
                    if matches_cover && !found_cover {
                        found_cover = true;
                    }
                    if filename != name {
                        images.insert(filename, buffer.clone());
                    }
                    images.insert(name, buffer);
                }
            }
        }
    }

    images
}

fn is_potential_cover_image(name: &str, already_found_cover: bool) -> bool {
    !already_found_cover && name.to_ascii_lowercase().contains("cover")
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
