use crate::app::KglanceApp;
use crate::app::messages::Message;
use iced::Task;
use std::path::Path;

const IMAGE_EXTENSIONS: [&str; 6] = ["png", "jpg", "jpeg", "webp", "bmp", "gif"];
const VIDEO_EXTENSIONS: [&str; 5] = ["mp4", "mkv", "avi", "mov", "webm"];

pub fn handle_grid_thumbnail_loaded(
    app: &mut KglanceApp,
    path: &str,
    thumbnail_handle: Option<iced::widget::image::Handle>,
) -> Task<Message> {
    if let crate::core::ViewMode::Grid(thumbnails) = &mut app.state.view_mode
        && let Some(thumbnail) = thumbnails.iter_mut().find(|t| t.path == path)
    {
        thumbnail.thumbnail_handle = thumbnail_handle;
        thumbnail.is_loading = false;
    }

    Task::none()
}

pub fn handle_toggle_view_mode(app: &mut KglanceApp) -> Task<Message> {
    match &app.state.view_mode {
        crate::core::ViewMode::Detail => enter_grid_view(app),
        crate::core::ViewMode::Grid(_) | crate::core::ViewMode::Settings => {
            app.state.view_mode = crate::core::ViewMode::Detail;
            Task::none()
        }
    }
}

fn enter_grid_view(app: &mut KglanceApp) -> Task<Message> {
    let mut thumbnail_tasks = Vec::with_capacity(app.state.playlist.len());
    let thumbnails = app
        .state
        .playlist
        .iter()
        .map(|path| {
            let thumbnail = create_grid_thumbnail(path);
            thumbnail_tasks.push(create_thumbnail_task(path.clone()));
            thumbnail
        })
        .collect();

    app.state.view_mode = crate::core::ViewMode::Grid(thumbnails);

    Task::batch(thumbnail_tasks)
}

pub(crate) fn create_grid_thumbnail(path: &str) -> crate::core::GridThumbnail {
    let name = Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string());

    crate::core::GridThumbnail {
        path: path.to_string(),
        name,
        thumbnail_handle: None,
        is_loading: true,
    }
}

pub(crate) fn create_thumbnail_task(path: String) -> Task<Message> {
    Task::perform(
        async move {
            let thumbnail_handle = tokio::task::spawn_blocking({
                let path = path.clone();
                move || load_thumbnail_handle(&path)
            })
            .await
            .ok()
            .flatten();

            crate::app::messages::NavigationMsg::GridThumbnailLoaded {
                path,
                handle: thumbnail_handle,
            }
            .into()
        },
        |message| message,
    )
}

pub(crate) fn sync_grid_thumbnails(
    old_thumbnails: &[crate::core::GridThumbnail],
    new_playlist: &[String],
) -> (Vec<crate::core::GridThumbnail>, Vec<Task<Message>>) {
    let mut old_map = std::collections::HashMap::with_capacity(old_thumbnails.len());
    for thumb in old_thumbnails {
        old_map.insert(thumb.path.as_str(), thumb);
    }

    let mut new_thumbnails = Vec::with_capacity(new_playlist.len());
    let mut tasks = Vec::new();

    for path in new_playlist {
        if let Some(existing) = old_map.get(path.as_str()) {
            new_thumbnails.push((*existing).clone());
        } else {
            new_thumbnails.push(create_grid_thumbnail(path));
            tasks.push(create_thumbnail_task(path.clone()));
        }
    }

    (new_thumbnails, tasks)
}

fn load_thumbnail_handle(path: &str) -> Option<iced::widget::image::Handle> {
    if let Some(thumbnail_path) = crate::features::grid::get_freedesktop_thumbnail_path(path) {
        return Some(iced::widget::image::Handle::from_path(thumbnail_path));
    }

    let lowercased_path = path.to_lowercase();

    if has_extension(&lowercased_path, &IMAGE_EXTENSIONS) {
        return Some(iced::widget::image::Handle::from_path(path));
    }

    if lowercased_path.ends_with(".typ") {
        return load_typst_thumbnail(path);
    }

    if has_extension(&lowercased_path, &VIDEO_EXTENSIONS) {
        return load_video_thumbnail(path);
    }

    None
}

fn has_extension(path: &str, extensions: &[&str]) -> bool {
    extensions
        .iter()
        .any(|extension| path.ends_with(&format!(".{extension}")))
}

fn load_typst_thumbnail(path: &str) -> Option<iced::widget::image::Handle> {
    let (_, _, first_page, _, _) =
        crate::parsers::typst::compile_typst_to_pdf(Path::new(path)).ok()?;

    if first_page.data.is_empty() {
        return None;
    }

    Some(iced::widget::image::Handle::from_bytes(first_page.data))
}

fn load_video_thumbnail(path: &str) -> Option<iced::widget::image::Handle> {
    let bytes = crate::parsers::video::extract_video_thumbnail(Path::new(path))?;

    Some(iced::widget::image::Handle::from_bytes(bytes))
}

pub fn handle_grid_search_query_changed(app: &mut KglanceApp, query: String) -> Task<Message> {
    app.state.grid_search_query = query;
    Task::none()
}

pub fn handle_grid_search_closed(app: &mut KglanceApp) -> Task<Message> {
    app.state.grid_search_visible = false;
    app.state.grid_search_query.clear();
    Task::none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_grid_thumbnails_preserves_existing_and_adds_new() {
        let old_thumbnails = vec![
            crate::core::GridThumbnail {
                path: "/tmp/a.png".to_string(),
                name: "a.png".to_string(),
                thumbnail_handle: None,
                is_loading: false,
            },
            crate::core::GridThumbnail {
                path: "/tmp/b.png".to_string(),
                name: "b.png".to_string(),
                thumbnail_handle: None,
                is_loading: true,
            },
        ];
        let new_playlist = vec!["/tmp/b.png".to_string(), "/tmp/c.png".to_string()];

        let (synced, tasks) = sync_grid_thumbnails(&old_thumbnails, &new_playlist);
        assert_eq!(synced.len(), 2);
        assert_eq!(synced[0].path, "/tmp/b.png");
        assert_eq!(synced[1].path, "/tmp/c.png");
        // /tmp/b.png was already present (even if is_loading), so only /tmp/c.png creates a thumbnail task
        assert_eq!(tasks.len(), 1);
    }
}
