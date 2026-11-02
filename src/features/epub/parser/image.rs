use std::collections::{HashMap, HashSet};
use std::io::{Read, Seek};

use super::opf::resolve_relative_path;
use crate::features::markdown::parser::{Block, Inline};

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
    let file_names: Vec<String> = archive
        .file_names()
        .flatten()
        .map(|name| name.into_owned())
        .collect();

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

pub fn is_potential_cover_image(name: &str, already_found_cover: bool) -> bool {
    !already_found_cover && name.to_ascii_lowercase().contains("cover")
}

pub fn is_image_extension(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".png")
        || lower.ends_with(".gif")
        || lower.ends_with(".webp")
        || lower.ends_with(".bmp")
        || lower.ends_with(".svg")
}

pub fn extract_filename(path: &str) -> String {
    path.rsplit_once('/')
        .map(|(_, name)| name.to_string())
        .unwrap_or_else(|| path.to_string())
}
