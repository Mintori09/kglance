use crate::app::Message;
use crate::core::types::GridThumbnail;
use crate::ui::views::grid::calculate::{calculate_column_count, calculate_horizontal_padding};
use crate::ui::views::grid::card::create_grid_card;
use crate::ui::views::grid::constants::ScaledDimensions;
use iced::widget::{column, responsive, row, scrollable};
use iced::{Element, Length};

pub fn render_grid_layout<'a>(
    filtered_thumbnails: Vec<(usize, &'a GridThumbnail)>,
    active_index: usize,
    scale: f32,
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

        let mut grid_column = column![]
            .spacing(dimensions.gap)
            .padding([dimensions.gap, horizontal_padding]);

        for chunk in filtered_thumbnails.chunks(columns_count) {
            let mut grid_row = row![].spacing(dimensions.gap);

            for &(original_index, item) in chunk {
                let is_active = original_index == active_index;
                let card_button = create_grid_card(item, original_index, is_active, &dimensions);
                grid_row = grid_row.push(card_button);
            }

            grid_column = grid_column.push(grid_row);
        }

        scrollable(grid_column)
            .id("grid_scroll")
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    })
    .into()
}
