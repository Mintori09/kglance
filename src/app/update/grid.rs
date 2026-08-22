use crate::app::KglanceApp;
use crate::app::messages::Message;
use iced::Task;
use std::path::Path;

const IMAGE_EXTENSIONS: [&str; 6] = ["png", "jpg", "jpeg", "webp", "bmp", "gif"];
const VIDEO_EXTENSIONS: [&str; 5] = ["mp4", "mkv", "avi", "mov", "webm"];

pub fn handle_grid_thumbnail_loaded(
    app: &mut KglanceApp,
    index: usize,
    thumbnail_handle: Option<iced::widget::image::Handle>,
) -> Task<Message> {
    if let crate::core::ViewMode::Grid(thumbnails) = &mut app.state.view_mode
        && let Some(thumbnail) = thumbnails.get_mut(index)
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
        .enumerate()
        .map(|(index, path)| {
            let thumbnail = create_grid_thumbnail(path);
            thumbnail_tasks.push(create_thumbnail_task(index, path.clone()));
            thumbnail
        })
        .collect();

    app.state.view_mode = crate::core::ViewMode::Grid(thumbnails);

    Task::batch(thumbnail_tasks)
}

fn create_grid_thumbnail(path: &str) -> crate::core::GridThumbnail {
    let name = Path::new(path)
        .file_name()
        .map(|file_name| file_name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_owned());

    crate::core::GridThumbnail {
        path: path.to_owned(),
        name,
        thumbnail_handle: None,
        is_loading: true,
    }
}

fn create_thumbnail_task(index: usize, path: String) -> Task<Message> {
    Task::perform(
        async move {
            let thumbnail_handle =
                tokio::task::spawn_blocking(move || load_thumbnail_handle(&path))
                    .await
                    .ok()
                    .flatten();

            crate::app::messages::NavigationMsg::GridThumbnailLoaded {
                index,
                handle: thumbnail_handle,
            }
            .into()
        },
        |message| message,
    )
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
