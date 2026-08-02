use crate::app::KglanceApp;
use crate::app::messages::Message;
use iced::Task;
use std::path::Path;

pub fn handle_grid_thumbnail_loaded(
    app: &mut KglanceApp,
    index: usize,
    handle: Option<iced::widget::image::Handle>,
) -> Task<Message> {
    if let crate::core::ViewMode::Grid(ref mut thumbnails) = app.state.view_mode
        && let Some(item) = thumbnails.get_mut(index)
    {
        item.thumbnail_handle = handle;
        item.is_loading = false;
    }
    Task::none()
}

pub fn handle_toggle_view_mode(app: &mut KglanceApp) -> Task<Message> {
    match app.state.view_mode {
        crate::core::ViewMode::Detail => {
            let mut tasks = Vec::new();
            let thumbnails: Vec<crate::core::GridThumbnail> = app
                .state
                .playlist
                .iter()
                .enumerate()
                .map(|(idx, p)| {
                    let name = Path::new(p)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| p.clone());

                    let path_for_task = p.clone();
                    tasks.push(Task::perform(
                        async move {
                            let handle = tokio::task::spawn_blocking(move || {
                                if let Some(thumb_path) =
                                    crate::features::grid::get_freedesktop_thumbnail_path(
                                        &path_for_task,
                                    )
                                {
                                    return Some(iced::widget::image::Handle::from_path(
                                        thumb_path,
                                    ));
                                }
                                let lower = path_for_task.to_lowercase();
                                if lower.ends_with(".png")
                                    || lower.ends_with(".jpg")
                                    || lower.ends_with(".jpeg")
                                    || lower.ends_with(".webp")
                                    || lower.ends_with(".bmp")
                                    || lower.ends_with(".gif")
                                {
                                    return Some(iced::widget::image::Handle::from_path(
                                        &path_for_task,
                                    ));
                                }
                                if lower.ends_with(".typ")
                                    && let Ok((_temp, _count, first_page, _)) =
                                        crate::parsers::typst::compile_typst_to_pdf(
                                            std::path::Path::new(&path_for_task),
                                        )
                                    && !first_page.data.is_empty()
                                {
                                    return Some(iced::widget::image::Handle::from_bytes(
                                        first_page.data,
                                    ));
                                }
                                if (lower.ends_with(".mp4")
                                    || lower.ends_with(".mkv")
                                    || lower.ends_with(".avi")
                                    || lower.ends_with(".mov")
                                    || lower.ends_with(".webm"))
                                    && let Some(bytes) =
                                        crate::parsers::video::extract_video_thumbnail(
                                            std::path::Path::new(&path_for_task),
                                        )
                                {
                                    return Some(iced::widget::image::Handle::from_bytes(bytes));
                                }
                                None
                            })
                            .await
                            .ok()
                            .flatten();

                            crate::app::messages::NavigationMsg::GridThumbnailLoaded {
                                index: idx,
                                handle,
                            }
                            .into()
                        },
                        |msg| msg,
                    ));

                    crate::core::GridThumbnail {
                        path: p.clone(),
                        name,
                        thumbnail_handle: None,
                        is_loading: true,
                    }
                })
                .collect();
            app.state.view_mode = crate::core::ViewMode::Grid(thumbnails);
            Task::batch(tasks)
        }
        crate::core::ViewMode::Grid(_) | crate::core::ViewMode::Settings => {
            app.state.view_mode = crate::core::ViewMode::Detail;
            Task::none()
        }
    }
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
