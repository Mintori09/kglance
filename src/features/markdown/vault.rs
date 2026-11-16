use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, RwLock};
use std::time::SystemTime;

use crate::features::image::types::ImageRef;
use crate::features::markdown::parser::Inline;
use crate::features::markdown::parser::html::decode_entity_if_present;

const MEDIA_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "svg", "webp", "bmp", "ico", "tiff", "avif", "pdf",
];

/// Returns true if the file extension is considered a media/attachment format.
pub fn is_media_extension(ext: &str) -> bool {
    let lower = ext.to_ascii_lowercase();
    MEDIA_EXTENSIONS.contains(&lower.as_str())
}

/// Returns true if the path has a supported media extension.
pub fn is_media_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(is_media_extension)
        .unwrap_or(false)
}

/// Discovers all enclosing Obsidian vault roots (from closest ancestor to farthest).
pub fn find_vault_roots(start_path: &Path) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    let start_dir = if start_path.is_file() {
        start_path.parent().unwrap_or(start_path)
    } else {
        start_path
    };

    for ancestor in start_dir.ancestors() {
        if ancestor.join(".obsidian").is_dir() {
            roots.push(ancestor.to_path_buf());
        }
    }

    roots
}

/// Returns the primary (closest) vault root, if any.
pub fn find_vault_root(start_path: &Path) -> Option<PathBuf> {
    find_vault_roots(start_path).into_iter().next()
}

/// In-memory cache indexing all media files in an Obsidian vault.
#[derive(Debug, Default)]
pub struct VaultMediaIndex {
    pub vault_root: PathBuf,
    pub attachment_folder: Option<PathBuf>,
    /// Relative path (normalized lowercase with forward slashes) -> Canonical PathBuf
    pub rel_map: HashMap<String, PathBuf>,
    /// Base filename (lowercase) -> Canonical PathBuf
    pub filename_map: HashMap<String, PathBuf>,
    /// Filename stem without extension (lowercase) -> Canonical PathBuf
    pub stem_map: HashMap<String, PathBuf>,
}

impl VaultMediaIndex {
    pub fn new(vault_root: PathBuf) -> Self {
        let attachment_folder = read_attachment_folder(&vault_root);
        let mut index = Self {
            vault_root: vault_root.clone(),
            attachment_folder,
            rel_map: HashMap::new(),
            filename_map: HashMap::new(),
            stem_map: HashMap::new(),
        };

        scan_directory_media(&vault_root, &vault_root, 0, &mut index);
        index
    }

    /// Queries the index for a matching real absolute path.
    pub fn lookup(&self, target: &str) -> Option<PathBuf> {
        let cleaned = target.replace('\\', "/");
        let lower = cleaned.to_ascii_lowercase();

        // 1. Direct relative path lookup
        if let Some(path) = self.rel_map.get(&lower) {
            return Some(path.clone());
        }

        // 2. Lookup by base filename
        let file_name = Path::new(&cleaned)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&cleaned);
        let lower_file_name = file_name.to_ascii_lowercase();

        if let Some(path) = self.filename_map.get(&lower_file_name) {
            return Some(path.clone());
        }

        // 3. Fallback: Lookup by stem (e.g. note refers to .png but file on disk is .jpg)
        if let Some(stem) = Path::new(file_name).file_stem().and_then(|s| s.to_str()) {
            let lower_stem = stem.to_ascii_lowercase();
            if let Some(path) = self.stem_map.get(&lower_stem) {
                return Some(path.clone());
            }
        }

        None
    }
}

fn read_attachment_folder(vault_root: &Path) -> Option<PathBuf> {
    let app_json_path = vault_root.join(".obsidian").join("app.json");
    let content = fs::read_to_string(app_json_path).ok()?;
    let parsed: serde_json::Value = serde_json::from_str(&content).ok()?;
    let folder = parsed.get("attachmentFolderPath")?.as_str()?.trim();

    if folder.is_empty() || folder == "/" {
        Some(vault_root.to_path_buf())
    } else if folder == "./" {
        None
    } else if let Some(stripped) = folder.strip_prefix("./") {
        Some(vault_root.join(stripped))
    } else {
        Some(vault_root.join(folder))
    }
}

fn scan_directory_media(
    vault_root: &Path,
    current_dir: &Path,
    depth: usize,
    index: &mut VaultMediaIndex,
) {
    if depth > 10 {
        return;
    }

    let Ok(entries) = fs::read_dir(current_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let file_name = entry.file_name().to_string_lossy().to_string();
        if file_name.starts_with('.')
            || file_name.eq_ignore_ascii_case("node_modules")
            || file_name.eq_ignore_ascii_case("target")
        {
            continue;
        }

        let path = entry.path();
        if path.is_dir() {
            scan_directory_media(vault_root, &path, depth + 1, index);
        } else if is_media_file(&path) {
            let lower_name = file_name.to_ascii_lowercase();

            if let Ok(rel) = path.strip_prefix(vault_root) {
                let rel_str = rel.to_string_lossy().to_string().replace('\\', "/");
                index
                    .rel_map
                    .entry(rel_str.to_ascii_lowercase())
                    .or_insert_with(|| path.clone());
            }

            index
                .filename_map
                .entry(lower_name)
                .or_insert_with(|| path.clone());

            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                index
                    .stem_map
                    .entry(stem.to_ascii_lowercase())
                    .or_insert_with(|| path.clone());
            }
        }
    }
}

type CachedVaultEntry = (SystemTime, Arc<VaultMediaIndex>);
type VaultCacheMap = HashMap<PathBuf, CachedVaultEntry>;

static VAULT_CACHE: LazyLock<RwLock<VaultCacheMap>> = LazyLock::new(|| RwLock::new(HashMap::new()));

/// Retrieves or builds the cached `VaultMediaIndex` for the given vault root.
pub fn get_or_build_vault_index(vault_root: &Path) -> Arc<VaultMediaIndex> {
    let mtime = fs::metadata(vault_root)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);

    if let Ok(cache) = VAULT_CACHE.read()
        && let Some((cached_mtime, index)) = cache.get(vault_root)
        && *cached_mtime == mtime
    {
        return Arc::clone(index);
    }

    let index = Arc::new(VaultMediaIndex::new(vault_root.to_path_buf()));

    if let Ok(mut cache) = VAULT_CACHE.write() {
        cache.insert(vault_root.to_path_buf(), (mtime, Arc::clone(&index)));
    }

    index
}

/// Cleans and decodes a percent-encoded path string (e.g. "%20" -> " ").
pub fn decode_percent_str(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(hex_str) = std::str::from_utf8(&bytes[i + 1..i + 3])
            && let Ok(byte) = u8::from_str_radix(hex_str, 16)
        {
            decoded.push(byte);
            i += 3;
            continue;
        }
        decoded.push(bytes[i]);
        i += 1;
    }

    String::from_utf8(decoded).unwrap_or_else(|_| s.to_string())
}

/// Resolves an Obsidian link or image path to its real absolute path on disk.
pub fn resolve_vault_path(target_str: &str, file_or_parent: &Path) -> Option<PathBuf> {
    let roots = find_vault_roots(file_or_parent);
    if roots.is_empty() {
        return None;
    }

    // Clean up percent encoding, wikilink brackets, anchors, and sizing pipes
    let decoded = decode_percent_str(target_str);
    let mut cleaned = decoded.trim();

    if cleaned.starts_with("![[") && cleaned.ends_with("]]") {
        cleaned = &cleaned[3..cleaned.len() - 2];
    } else if cleaned.starts_with("[[") && cleaned.ends_with("]]") {
        cleaned = &cleaned[2..cleaned.len() - 2];
    }

    // Extract path before any pipe '|' (e.g. "image.png|300" -> "image.png")
    if let Some((path_part, _)) = cleaned.split_once('|') {
        cleaned = path_part.trim();
    }

    // Extract path before any anchor '#' (e.g. "image.png#heading" -> "image.png")
    if let Some((path_part, _)) = cleaned.split_once('#') {
        cleaned = path_part.trim();
    }

    let cleaned_no_slash = cleaned.trim_start_matches('/');

    for vault_root in &roots {
        // 1. Direct path relative to current note folder
        let note_dir = if file_or_parent.is_file() {
            file_or_parent.parent().unwrap_or(file_or_parent)
        } else {
            file_or_parent
        };

        let note_relative = note_dir.join(cleaned_no_slash);
        if note_relative.is_file() {
            return Some(note_relative);
        }

        // 2. Direct path relative to vault root
        let root_relative = vault_root.join(cleaned_no_slash);
        if root_relative.is_file() {
            return Some(root_relative);
        }

        // 3. Vault media index lookup
        let index = get_or_build_vault_index(vault_root);
        if let Some(found) = index.lookup(cleaned_no_slash) {
            return Some(found);
        }
    }

    None
}

/// Resolves an image path taking into account remote URLs, Obsidian vaults, and parent relative directories.
pub fn resolve_image_path(url: &str, parent: Option<&Path>) -> String {
    let is_remote = crate::core::net::is_remote_url(url);
    if is_remote {
        return url.to_string();
    }

    if let Some(parent) = parent {
        if let Some(vault_path) = resolve_vault_path(url, parent) {
            return vault_path.to_string_lossy().to_string();
        }
        if url.starts_with('/') {
            url.to_string()
        } else {
            parent.join(url).to_string_lossy().to_string()
        }
    } else {
        url.to_string()
    }
}

/// Represents an extracted wikilink embed or link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedWikilink {
    Image {
        target: String,
        alt: String,
        width: Option<u32>,
        height: Option<u32>,
    },
    Link {
        target: String,
        label: String,
    },
}

/// Parses an Obsidian wikilink syntax (e.g. `![[image.png|300]]` or `[[note|Title]]`).
pub fn parse_wikilink(raw: &str) -> Option<ParsedWikilink> {
    let trimmed = raw.trim();
    let is_embed = trimmed.starts_with('!');
    let content = if is_embed {
        trimmed.strip_prefix("![[")?.strip_suffix("]]")?
    } else {
        trimmed.strip_prefix("[[")?.strip_suffix("]]")?
    };

    let (target, param) = match content.split_once('|') {
        Some((t, p)) => (t.trim(), Some(p.trim())),
        None => (content.trim(), None),
    };

    if target.is_empty() {
        return None;
    }

    if is_embed {
        let (alt, width, height) = parse_embed_params(param, target);
        Some(ParsedWikilink::Image {
            target: target.to_string(),
            alt,
            width,
            height,
        })
    } else {
        let label = param.unwrap_or(target).to_string();
        Some(ParsedWikilink::Link {
            target: target.to_string(),
            label,
        })
    }
}

fn parse_embed_params(
    param: Option<&str>,
    default_alt: &str,
) -> (String, Option<u32>, Option<u32>) {
    let Some(param) = param else {
        return (default_alt.to_string(), None, None);
    };

    // Check for "300x200"
    if let Some((w_str, h_str)) = param.split_once('x')
        && let Ok(w) = w_str.trim().parse::<u32>()
        && let Ok(h) = h_str.trim().parse::<u32>()
    {
        return (default_alt.to_string(), Some(w), Some(h));
    }

    // Check for "300"
    if let Ok(w) = param.parse::<u32>() {
        return (default_alt.to_string(), Some(w), None);
    }

    // Otherwise, it's custom alt text
    (param.to_string(), None, None)
}

/// Extracts dimension overrides if alt text is numeric (e.g. "500" or "600x400").
pub fn extract_image_dimensions(
    alt: &str,
    fallback_alt: &str,
) -> (String, Option<u32>, Option<u32>) {
    let trimmed = alt.trim();
    if let Some((w_str, h_str)) = trimmed.split_once('x')
        && let Ok(w) = w_str.trim().parse::<u32>()
        && let Ok(h) = h_str.trim().parse::<u32>()
    {
        return (fallback_alt.to_string(), Some(w), Some(h));
    }

    if let Ok(w) = trimmed.parse::<u32>() {
        return (fallback_alt.to_string(), Some(w), None);
    }

    (alt.to_string(), None, None)
}

/// Parses a text segment that may contain Obsidian wikilinks (`![[...]` or `[[...]]`).
pub fn parse_text_inlines(
    text: &str,
    parent: Option<&Path>,
    images: &mut Vec<ImageRef>,
) -> Vec<Inline> {
    if !text.contains("[[") {
        return vec![Inline::Text(decode_entity_if_present(text))];
    }

    let mut result = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        let embed_pos = remaining.find("![[");
        let link_pos = remaining.find("[[");

        let next_match = match (embed_pos, link_pos) {
            (Some(e), Some(l)) => {
                if e <= l {
                    Some((e, true))
                } else {
                    Some((l, false))
                }
            }
            (Some(e), None) => Some((e, true)),
            (None, Some(l)) => Some((l, false)),
            (None, None) => None,
        };

        let Some((start_idx, is_embed)) = next_match else {
            result.push(Inline::Text(decode_entity_if_present(remaining)));
            break;
        };

        if start_idx > 0 {
            result.push(Inline::Text(decode_entity_if_present(
                &remaining[..start_idx],
            )));
        }

        let prefix_len = if is_embed { 3 } else { 2 };
        let after_prefix = &remaining[start_idx + prefix_len..];

        if let Some(end_offset) = after_prefix.find("]]") {
            let full_token = &remaining[start_idx..start_idx + prefix_len + end_offset + 2];
            remaining = &remaining[start_idx + prefix_len + end_offset + 2..];

            if let Some(parsed) = parse_wikilink(full_token) {
                match parsed {
                    ParsedWikilink::Image {
                        target,
                        alt,
                        width,
                        height,
                    } => {
                        let resolved = resolve_image_path(&target, parent);
                        images.push(ImageRef {
                            alt_text: alt.clone(),
                            path: resolved.clone(),
                        });
                        result.push(Inline::Image {
                            alt,
                            url: resolved,
                            width,
                            height,
                        });
                    }
                    ParsedWikilink::Link { target, label } => {
                        result.push(Inline::Link {
                            text: vec![Inline::Text(label)],
                            url: target,
                        });
                    }
                }
            } else {
                result.push(Inline::Text(full_token.to_string()));
            }
        } else {
            result.push(Inline::Text(decode_entity_if_present(
                &remaining[start_idx..],
            )));
            break;
        }
    }

    result
}

#[cfg(test)]
#[path = "vault_tests.rs"]
mod tests;
