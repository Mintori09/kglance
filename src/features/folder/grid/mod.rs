pub mod calculate;
pub mod card;
pub mod constants;
pub mod layout;
pub mod thumbnails;

pub use thumbnails::get_freedesktop_thumbnail_path;

use crate::app::Message;
use crate::core::types::GridThumbnail;
use crate::features::folder::grid::constants::CARD_GRID_SPACING;
use crate::features::folder::grid::layout::render_grid_layout;
use crate::features::folder::grid::thumbnails::filter_thumbnails;
use crate::ui::components::search_bar::{SearchKind, search_bar};

use iced::widget::column;
use iced::{Element, Length};

pub fn view_grid<'a>(
    thumbnails: &'a [GridThumbnail],
    active_index: usize,
    scale: f32,
    search_visible: bool,
    search_query: &'a str,
) -> Element<'a, Message> {
    let filtered_thumbnails = filter_thumbnails(thumbnails, search_query);
    let grid_layout = render_grid_layout(filtered_thumbnails, active_index, scale);

    if search_visible {
        column![
            search_bar(SearchKind::Grid, search_query, None),
            grid_layout,
        ]
        .spacing(CARD_GRID_SPACING)
        .height(Length::Fill)
        .into()
    } else {
        grid_layout
    }
}
