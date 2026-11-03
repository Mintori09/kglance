use crate::app::KglanceApp;
use crate::app::messages::Message;
use iced::Task;

pub fn handle_open_image_lightbox(app: &mut KglanceApp, path: String) -> Task<Message> {
    let all_images = collect_all_epub_image_paths(app);
    let filename = std::path::Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&path);

    let current_index = all_images
        .iter()
        .position(|p| {
            p == &path
                || std::path::Path::new(p)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(p)
                    == filename
        })
        .unwrap_or(0);

    let Some(image_bytes) = get_epub_image_bytes(app, &path) else {
        return Task::none();
    };

    let image_state = create_lightbox_image_state(&image_bytes);

    app.state.epub.lightbox = Some(crate::features::epub::types::EpubLightboxState {
        active_path: path,
        all_image_paths: all_images,
        current_index,
        image_state,
    });

    Task::none()
}

pub fn handle_close_image_lightbox(app: &mut KglanceApp) -> Task<Message> {
    app.state.epub.lightbox = None;
    Task::none()
}

pub fn handle_next_lightbox_image(app: &mut KglanceApp) -> Task<Message> {
    let (next_idx, next_path) = {
        let Some(lightbox) = &app.state.epub.lightbox else {
            return Task::none();
        };

        if lightbox.all_image_paths.is_empty() {
            return Task::none();
        }

        let idx = (lightbox.current_index + 1) % lightbox.all_image_paths.len();
        let path = lightbox.all_image_paths[idx].clone();
        (idx, path)
    };

    if let Some(bytes) = get_epub_image_bytes(app, &next_path)
        && let Some(lightbox) = &mut app.state.epub.lightbox
    {
        lightbox.active_path = next_path;
        lightbox.current_index = next_idx;
        lightbox.image_state = create_lightbox_image_state(&bytes);
    }

    Task::none()
}

pub fn handle_prev_lightbox_image(app: &mut KglanceApp) -> Task<Message> {
    let (prev_idx, prev_path) = {
        let Some(lightbox) = &app.state.epub.lightbox else {
            return Task::none();
        };

        if lightbox.all_image_paths.is_empty() {
            return Task::none();
        }

        let len = lightbox.all_image_paths.len();
        let idx = if lightbox.current_index == 0 {
            len.saturating_sub(1)
        } else {
            lightbox.current_index - 1
        };
        let path = lightbox.all_image_paths[idx].clone();
        (idx, path)
    };

    if let Some(bytes) = get_epub_image_bytes(app, &prev_path)
        && let Some(lightbox) = &mut app.state.epub.lightbox
    {
        lightbox.active_path = prev_path;
        lightbox.current_index = prev_idx;
        lightbox.image_state = create_lightbox_image_state(&bytes);
    }

    Task::none()
}

pub fn handle_sidebar_tab_changed(
    app: &mut KglanceApp,
    tab: crate::features::epub::types::EpubSidebarTab,
) -> Task<Message> {
    app.state.epub.sidebar_tab = tab;
    Task::none()
}

fn collect_all_epub_image_paths(app: &KglanceApp) -> Vec<String> {
    let mut paths = Vec::new();

    // From active continuous blocks or all chapters
    if app.state.epub.reading_mode == crate::core::config::EpubReadingMode::Continuous {
        paths = crate::features::epub::parser::extract_images_from_blocks(
            &app.state.epub.continuous_blocks,
        );
    } else {
        for ch in &app.state.epub.chapters {
            let ch_paths = crate::features::epub::parser::extract_images_from_blocks(&ch.blocks);
            for p in ch_paths {
                if !paths.contains(&p) {
                    paths.push(p);
                }
            }
        }
    }

    // Also include any images in PreviewData that might not be directly in blocks
    if let Some(crate::core::PreviewData::Epub { images, .. }) = &app.current_content {
        for k in images.keys() {
            if !paths.contains(k) {
                paths.push(k.clone());
            }
        }
    }

    paths
}

pub fn get_epub_image_bytes(app: &KglanceApp, path: &str) -> Option<Vec<u8>> {
    let filename = std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path);

    if let Some(crate::core::PreviewData::Epub { images, .. }) = &app.current_content
        && let Some(bytes) = images.get(path).or_else(|| images.get(filename))
    {
        return Some(bytes.clone());
    }

    // Fallback: extract on-demand from the EPUB archive on disk
    if !app.state.file_name.is_empty() {
        let epub_path = std::path::Path::new(&app.state.file_name);
        if let Ok(file) = std::fs::File::open(epub_path)
            && let Ok(mut archive) = zip::ZipArchive::new(file)
        {
            let normalized_target = path.replace('\\', "/");
            let target_filename = filename.to_lowercase();

            let file_names: Vec<String> = archive
                .file_names()
                .flatten()
                .map(|name| name.to_string())
                .collect();

            for entry_name in file_names {
                let entry_clean = entry_name.replace('\\', "/");
                let entry_file = std::path::Path::new(&entry_clean)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&entry_clean)
                    .to_lowercase();

                let matches = entry_clean == normalized_target
                    || entry_clean.ends_with(&normalized_target)
                    || normalized_target.ends_with(&entry_clean)
                    || entry_file == target_filename;

                if matches && let Ok(mut zf) = archive.by_name(&entry_name) {
                    use std::io::Read;
                    let mut buf = Vec::new();
                    if zf.read_to_end(&mut buf).is_ok() {
                        return Some(buf);
                    }
                }
            }
        }
    }

    None
}

fn create_lightbox_image_state(bytes: &[u8]) -> crate::features::image::ImageState {
    use std::sync::Arc;
    use std::sync::atomic::AtomicU64;

    let (display_handle, width, height) =
        if let Some((h, w, h_dim)) = crate::features::image::decode_to_rgba_handle(bytes) {
            (Some(h), w, h_dim)
        } else {
            (
                Some(iced::widget::image::Handle::from_bytes(bytes.to_vec())),
                0,
                0,
            )
        };

    crate::features::image::ImageState {
        handle: display_handle.clone(),
        preview_handle: None,
        image_bytes: bytes.to_vec(),
        exif_content: String::new(),
        exif_parsed_lines: Vec::new(),
        format_info: String::new(),
        load_state: crate::features::image::ImageLoadState::Ready,
        load_id: Arc::new(AtomicU64::new(0)),
        display_handle,
        display_width: width,
        display_height: height,
        camera: crate::features::image::Camera::new(),
        show_info: false,
        animated_gif: None,
        animation_start_time: None,
        prev_frame_handle: None,
    }
}
