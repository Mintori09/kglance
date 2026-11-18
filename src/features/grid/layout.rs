use crate::app::Message;
use crate::app::messages::GridMsg;
use crate::core::types::GridThumbnail;
use crate::features::grid::calculate::{
    calculate_column_count, calculate_horizontal_padding, calculate_visible_row_range,
};
use crate::features::grid::card::create_grid_card;
use crate::features::grid::constants::ScaledDimensions;

use iced::widget::{Space, column, responsive, row, scrollable};
use iced::{Element, Length};

const OVERSCAN_ROWS: usize = 2;

pub fn render_grid_layout<'a>(
    filtered_thumbnails: Vec<(usize, &'a GridThumbnail)>,
    active_index: usize,
    scale: f32,
    scroll_y: f32,
    viewport_height: f32,
) -> Element<'a, Message> {
    responsive(move |bounds| {
        let dimensions = ScaledDimensions::new(scale);
        let columns_count =
            calculate_column_count(bounds.width, dimensions.item_width, dimensions.gap);
        let horizontal_padding = calculate_horizontal_padding(
            bounds.width,
            columns_count,
            dimensions.item_width,
            dimensions.gap,
        );

        let row_step = dimensions.row_step();
        let total_rows = filtered_thumbnails.len().div_ceil(columns_count);
        let effective_vh = if bounds.height > 0.0 {
            bounds.height
        } else {
            viewport_height
        };
        let (start_row, end_row) = calculate_visible_row_range(
            total_rows,
            row_step,
            scroll_y,
            effective_vh,
            OVERSCAN_ROWS,
        );

        let mut grid_column = column![]
            .spacing(dimensions.gap)
            .padding([dimensions.gap, horizontal_padding]);

        if start_row > 0 {
            let top_spacer = (start_row as f32 * row_step - dimensions.gap).max(0.0);
            grid_column = grid_column.push(Space::new().height(Length::Fixed(top_spacer)));
        }

        for row_idx in start_row..end_row {
            let start_item = row_idx * columns_count;
            let end_item = (start_item + columns_count).min(filtered_thumbnails.len());
            let mut grid_row = row![].spacing(dimensions.gap);

            for &(original_index, item) in &filtered_thumbnails[start_item..end_item] {
                let is_active = original_index == active_index;
                let card_button = create_grid_card(item, original_index, is_active, &dimensions);
                grid_row = grid_row.push(card_button);
            }

            grid_column = grid_column.push(grid_row);
        }

        if end_row < total_rows {
            let bottom_spacer =
                ((total_rows - end_row) as f32 * row_step - dimensions.gap).max(0.0);
            grid_column = grid_column.push(Space::new().height(Length::Fixed(bottom_spacer)));
        }

        scrollable(grid_column)
            .id("grid_scroll")
            .height(Length::Fill)
            .width(Length::Fill)
            .on_scroll(|vp| Message::from(GridMsg::Scrolled(vp)))
            .into()
    })
    .into()
}
