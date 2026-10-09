mod html;
mod ncx;
mod opf;

#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::Path;

use rayon::prelude::*;

use crate::features::common::parser::traits::{ParseError, PreviewParser};
use crate::features::common::parser::types::{ParsedContent, ParsedEpubChapter};
use crate::features::markdown::parser::{
    Block, Inline, flatten_inlines, is_standalone_anchor_block, parse_to_blocks,
};

use html::{
    convert_html_to_markdown, decode_html_entities, extract_chapter_title_from_html,
    extract_filename, extract_headings_from_html, extract_tag_content,
};
use ncx::{NcxNavPoint, decode_url_component, extract_epub3_navpoints, extract_ncx_navpoints};
use opf::{
    extract_cover_image_href, extract_nav_href, extract_ncx_href, extract_opf_path,
    extract_spine_items, resolve_relative_path,
};

const DEFAULT_TITLE: &str = "Unknown Title";
const DEFAULT_AUTHOR: &str = "Unknown Author";

pub struct EpubParser;

pub fn parse_chapter_content<R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
    opf_path: &str,
    file_href: &str,
    _anchor: Option<&str>,
) -> Result<Vec<Block>, ParseError> {
    let resolved_path = resolve_relative_path(opf_path, file_href);
    let html = read_archive_entry_string(archive, &resolved_path)?;
    let md = convert_html_to_markdown(&html);
    let blocks = parse_to_blocks(&md);
    Ok(blocks)
}

pub type LoadedChapterContent = (Vec<Block>, HashMap<String, Vec<u8>>);

pub fn load_chapter_content_and_images_from_epub(
    path: &Path,
    file_href: &str,
    anchor: Option<&str>,
) -> Result<LoadedChapterContent, ParseError> {
    let file = File::open(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let reader = BufReader::new(file);
    let mut archive =
        zip::ZipArchive::new(reader).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let opf_path = read_container_opf_path(&mut archive)?;
    let blocks = parse_chapter_content(&mut archive, &opf_path, file_href, anchor)?;
    let images = extract_selective_images(&mut archive, &opf_path, None, &blocks, Some(file_href));
    Ok((blocks, images))
}

pub fn load_chapter_from_epub(
    path: &Path,
    file_href: &str,
    anchor: Option<&str>,
) -> Result<Vec<Block>, ParseError> {
    let file = File::open(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let reader = BufReader::new(file);
    let mut archive =
        zip::ZipArchive::new(reader).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let opf_path = read_container_opf_path(&mut archive)?;
    parse_chapter_content(&mut archive, &opf_path, file_href, anchor)
}

pub type LoadedAllChaptersContent = (
    Vec<crate::core::types::EpubChapterInfo>,
    HashMap<String, Vec<u8>>,
);

pub fn load_all_chapters_and_images_from_epub(
    path: &Path,
    chapters: &[crate::core::types::EpubChapterInfo],
) -> Result<LoadedAllChaptersContent, ParseError> {
    let file = File::open(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let reader = BufReader::new(file);
    let mut archive =
        zip::ZipArchive::new(reader).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
    let opf_path = read_container_opf_path(&mut archive)?;

    // 1. Identify all unique file hrefs that need to be read/parsed
    let mut needed_hrefs = HashSet::new();
    for ch in chapters {
        if ch.blocks.is_empty() && !ch.file_href.is_empty() {
            needed_hrefs.insert(ch.file_href.clone());
        }
    }

    // 2. Read distinct XHTML files in 1 sequential pass from the zip archive
    let mut raw_files: Vec<(String, String)> = Vec::with_capacity(needed_hrefs.len());
    for href in needed_hrefs {
        let resolved = resolve_relative_path(&opf_path, &href);
        if let Ok(content) = read_archive_entry_string(&mut archive, &resolved) {
            raw_files.push((href, content));
        }
    }

    // 3. Parse HTML to markdown and then to blocks in parallel using rayon
    let parsed_files: HashMap<String, Vec<Block>> = raw_files
        .into_par_iter()
        .map(|(href, html)| {
            let md = convert_html_to_markdown(&html);
            let blocks = parse_to_blocks(&md);
            (href, blocks)
        })
        .collect();

    // 4. Construct the loaded chapter list and collect all referenced images
    let mut loaded_chapters = Vec::with_capacity(chapters.len());
    let mut target_names: HashSet<String> = HashSet::new();
    let mut target_to_orig: HashMap<String, Vec<String>> = HashMap::new();

    for ch in chapters {
        let blocks = if !ch.blocks.is_empty() {
            ch.blocks.clone()
        } else if let Some(parsed) = parsed_files.get(&ch.file_href) {
            parsed.clone()
        } else {
            Vec::new()
        };

        let block_imgs = extract_images_from_blocks(&blocks);
        for img in block_imgs {
            if !ch.file_href.is_empty() {
                let ch_resolved = resolve_relative_path(&ch.file_href, &img);
                let full_resolved = resolve_relative_path(&opf_path, &ch_resolved);
                target_to_orig
                    .entry(full_resolved.clone())
                    .or_default()
                    .push(img.clone());
                target_to_orig
                    .entry(ch_resolved.clone())
                    .or_default()
                    .push(img.clone());
                target_names.insert(full_resolved);
                target_names.insert(ch_resolved);
            }
            let resolved = resolve_relative_path(&opf_path, &img);
            target_to_orig
                .entry(resolved.clone())
                .or_default()
                .push(img.clone());
            target_names.insert(resolved);
            let filename = extract_filename(&img);
            target_names.insert(filename);
            target_names.insert(img);
        }

        loaded_chapters.push(crate::core::types::EpubChapterInfo {
            title: ch.title.clone(),
            level: ch.level,
            anchor: ch.anchor.clone(),
            file_href: ch.file_href.clone(),
            blocks,
        });
    }

    // 5. Extract all matching images in a single pass over archive.file_names()
    let file_names: Vec<String> = archive.file_names().map(ToString::to_string).collect();
    let mut all_images = HashMap::new();

    for name in file_names {
        let filename = extract_filename(&name);
        let matches_target = target_names.contains(&name) || target_names.contains(&filename);
        let matches_cover = is_potential_cover_image(&name, false);

        if is_image_extension(&name)
            && (matches_target || matches_cover)
            && let Ok(mut file) = archive.by_name(&name)
        {
            let mut buffer = Vec::new();
            if file.read_to_end(&mut buffer).is_ok() {
                if filename != name {
                    all_images.insert(filename.clone(), buffer.clone());
                }
                if let Some(origs) = target_to_orig.get(&name) {
                    for orig in origs {
                        if orig != &name && orig != &filename {
                            all_images.insert(orig.clone(), buffer.clone());
                        }
                    }
                }
                all_images.insert(name, buffer);
            }
        }
    }

    Ok((loaded_chapters, all_images))
}

pub fn load_images_for_blocks_from_epub(
    archive: &mut zip::ZipArchive<impl Read + Seek>,
    opf_path: &str,
    blocks: &[Block],
    chapter_file_href: Option<&str>,
) -> HashMap<String, Vec<u8>> {
    extract_selective_images(archive, opf_path, None, blocks, chapter_file_href)
}

impl PreviewParser for EpubParser {
    fn supported_extensions(&self) -> &[&str] {
        &["epub"]
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let file = File::open(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        let reader = BufReader::new(file);
        let mut archive =
            zip::ZipArchive::new(reader).map_err(|e| ParseError::ParseFailed(e.to_string()))?;

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
        let first_chapter = chapters.first();
        let initial_blocks = first_chapter.map(|ch| ch.4.as_slice()).unwrap_or(&[]);
        let first_chapter_href = first_chapter.map(|ch| ch.3.as_str());
        let images = extract_selective_images(
            &mut archive,
            &opf_path,
            cover_href.as_deref(),
            initial_blocks,
            first_chapter_href,
        );

        Ok(ParsedContent::Epub {
            title,
            author,
            chapters,
            images,
        })
    }
}

fn read_container_opf_path<R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> Result<String, ParseError> {
    let xml = read_archive_entry_string(archive, "META-INF/container.xml")
        .map_err(|_| ParseError::ParseFailed("Missing META-INF/container.xml".into()))?;

    extract_opf_path(&xml)
        .ok_or_else(|| ParseError::ParseFailed("Could not locate OPF file in container.xml".into()))
}

fn read_archive_entry_string<R: Read + Seek>(
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

fn build_lazy_chapters_from_ncx<R: Read + Seek>(
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

fn build_lazy_chapters_from_spine<R: Read + Seek>(
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

pub fn find_block_index(
    blocks: &[Block],
    anchor: Option<&str>,
    title: Option<&str>,
) -> Option<usize> {
    if let Some(anc) = anchor
        && !anc.is_empty()
    {
        let anc_trimmed = anc.trim_start_matches('#');
        let anc_decoded = decode_url_component(anc_trimmed);

        for (i, block) in blocks.iter().enumerate() {
            if block_contains_anchor(block, anc_trimmed)
                || (!anc_decoded.is_empty() && block_contains_anchor(block, &anc_decoded))
            {
                if is_standalone_anchor_block(block) && i + 1 < blocks.len() {
                    return Some(i + 1);
                }
                return Some(i);
            }
        }
    }

    if let Some(t) = title
        && !t.is_empty()
    {
        let t_norm = normalize_for_matching(t);
        if !t_norm.is_empty() {
            // Pass 1: Prioritize Block::Heading exact normalized match
            for (i, block) in blocks.iter().enumerate() {
                if let Block::Heading { content, .. } = block {
                    let h_text = normalize_for_matching(&flatten_inlines(content));
                    if h_text == t_norm {
                        return Some(i);
                    }
                }
            }

            // Pass 2: Block::Heading substring match
            for (i, block) in blocks.iter().enumerate() {
                if let Block::Heading { content, .. } = block {
                    let h_text = normalize_for_matching(&flatten_inlines(content));
                    if !h_text.is_empty() && (h_text.contains(&t_norm) || t_norm.contains(&h_text))
                    {
                        return Some(i);
                    }
                }
            }

            // Pass 3: Fallback to non-heading blocks (e.g. bold paragraph pseudo-headings)
            for (i, block) in blocks.iter().enumerate() {
                if let Block::Paragraph(content) = block {
                    let p_text = normalize_for_matching(&flatten_inlines(content));
                    if p_text == t_norm || p_text.starts_with(&t_norm) {
                        return Some(i);
                    }
                }
            }
        }
    }

    None
}

fn normalize_for_matching(text: &str) -> String {
    let stripped = crate::parsers::markdown::strip_anchor_tags(text);
    stripped
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn contains_attr_value_case_insensitive(haystack: &str, attr_prefix: &str, value: &str) -> bool {
    let len = haystack.len();
    let val_len = value.len();
    let p_len = attr_prefix.len();
    if len < p_len + val_len + 3 {
        return false;
    }

    for (i, _) in haystack.match_indices(attr_prefix) {
        if i > 0 {
            let prev = haystack.as_bytes()[i - 1];
            if !prev.is_ascii_whitespace() && prev != b'<' {
                continue;
            }
        }
        let after = &haystack[i + p_len..];
        if after.starts_with("=\"") || after.starts_with("='") {
            let quote = after.as_bytes()[1];
            let val_part = &after[2..];
            if val_part.len() >= val_len
                && val_part.as_bytes().get(val_len) == Some(&quote)
                && val_part[..val_len].eq_ignore_ascii_case(value)
            {
                return true;
            }
        }
    }
    false
}

fn html_contains_anchor(html: &str, anchor: &str) -> bool {
    contains_attr_value_case_insensitive(html, "id", anchor)
        || contains_attr_value_case_insensitive(html, "xml:id", anchor)
        || contains_attr_value_case_insensitive(html, "name", anchor)
        || contains_attr_value_case_insensitive(html, "ID", anchor)
        || contains_attr_value_case_insensitive(html, "NAME", anchor)
}

fn block_contains_anchor(block: &Block, anchor: &str) -> bool {
    if anchor.is_empty() {
        return false;
    }
    match block {
        Block::Html(h) => html_contains_anchor(h, anchor),
        Block::Heading { content, .. } | Block::Paragraph(content) => {
            let text = flatten_inlines(content);
            html_contains_anchor(&text, anchor) || text.contains(anchor)
        }
        Block::Quote(sub) | Block::Alert { content: sub, .. } => {
            sub.iter().any(|b| block_contains_anchor(b, anchor))
        }
        Block::List { items, .. } => items.iter().any(|item| {
            let text = flatten_inlines(&item.content);
            html_contains_anchor(&text, anchor)
                || text.contains(anchor)
                || item
                    .sub_blocks
                    .iter()
                    .any(|b| block_contains_anchor(b, anchor))
        }),
        Block::Table(table) => {
            table.headers.iter().any(|h| {
                let text = flatten_inlines(&h.content);
                html_contains_anchor(&text, anchor) || text.contains(anchor)
            }) || table.rows.iter().any(|row| {
                row.iter().any(|cell| {
                    let text = flatten_inlines(&cell.content);
                    html_contains_anchor(&text, anchor) || text.contains(anchor)
                })
            })
        }
        _ => false,
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

pub fn extract_selective_images<R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
    opf_path: &str,
    cover_href: Option<&str>,
    blocks: &[Block],
    chapter_file_href: Option<&str>,
) -> HashMap<String, Vec<u8>> {
    let mut images = HashMap::new();
    let mut target_names: HashSet<String> = HashSet::new();

    if let Some(href) = cover_href {
        let resolved = resolve_relative_path(opf_path, href);
        target_names.insert(resolved.clone());
        target_names.insert(href.to_string());
        target_names.insert(extract_filename(href));
    }

    let mut target_to_orig: HashMap<String, Vec<String>> = HashMap::new();
    let block_imgs = extract_images_from_blocks(blocks);
    for img in block_imgs {
        if let Some(ch_href) = chapter_file_href {
            let ch_resolved = resolve_relative_path(ch_href, &img);
            let full_resolved = resolve_relative_path(opf_path, &ch_resolved);
            target_to_orig
                .entry(full_resolved.clone())
                .or_default()
                .push(img.clone());
            target_to_orig
                .entry(ch_resolved.clone())
                .or_default()
                .push(img.clone());
            target_names.insert(full_resolved);
            target_names.insert(ch_resolved);
        }
        let resolved = resolve_relative_path(opf_path, &img);
        target_to_orig
            .entry(resolved.clone())
            .or_default()
            .push(img.clone());
        target_names.insert(resolved);
        let filename = extract_filename(&img);
        target_names.insert(filename);
        target_names.insert(img);
    }

    let mut found_cover = cover_href.is_some();
    let file_names: Vec<String> = archive.file_names().map(ToString::to_string).collect();

    for name in file_names {
        let filename = extract_filename(&name);
        let matches_target = target_names.contains(&name) || target_names.contains(&filename);
        let matches_cover = is_potential_cover_image(&name, found_cover);

        if is_image_extension(&name)
            && (matches_target || matches_cover)
            && let Ok(mut file) = archive.by_name(&name)
        {
            let mut buffer = Vec::new();
            if file.read_to_end(&mut buffer).is_ok() {
                if matches_cover && !found_cover {
                    found_cover = true;
                }
                if filename != name {
                    images.insert(filename.clone(), buffer.clone());
                }
                if let Some(origs) = target_to_orig.get(&name) {
                    for orig in origs {
                        if orig != &name && orig != &filename {
                            images.insert(orig.clone(), buffer.clone());
                        }
                    }
                }
                images.insert(name, buffer);
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

#[test]
fn test_standalone_anchor_block() {
    use crate::features::markdown::parser::is_empty_anchor_html;

    assert!(is_empty_anchor_html(r#"<a id="toc-C0"></a>"#));
    assert!(is_empty_anchor_html(r#"<a id="toc-C0"/>"#));

    assert!(!is_empty_anchor_html(r#"<a id="toc-C0">Chapter 1</a>"#));
}
