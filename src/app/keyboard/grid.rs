use iced::Task;
use iced::keyboard::{Key, key::Named};
use iced::widget::operation::AbsoluteOffset;

use super::Message;
use crate::app::KglanceApp;

const MIN_GRID_COLUMNS: usize = 1;
const GRID_SCROLL_ID: &str = "grid_scroll";

impl KglanceApp {
    pub(super) fn handle_grid_navigation(&mut self, key: &Key) -> Option<Task<Message>> {
        let total_items = self.state.playlist.len();

        if self.ctrl_held || total_items == 0 {
            return None;
        }

        let dimensions =
            crate::features::grid::constants::ScaledDimensions::new(self.state.grid_scale);
        let effective_width = if self.state.grid_viewport_width > 0.0 {
            self.state.grid_viewport_width
        } else if self.state.window_width > 0.0 {
            self.state.window_width
        } else {
            800.0
        };
        let column_count = crate::features::grid::calculate::calculate_column_count(
            effective_width,
            dimensions.item_width,
            dimensions.gap,
        )
        .max(MIN_GRID_COLUMNS);
        let current_index = self.state.current_index;

        match key {
            Key::Named(Named::ArrowRight) => {
                self.select_next_item(current_index, total_items);
                Some(self.scroll_grid_to_current(column_count))
            }
            Key::Named(Named::ArrowLeft) => {
                self.select_previous_item(current_index);
                Some(self.scroll_grid_to_current(column_count))
            }
            Key::Named(Named::ArrowDown) => {
                self.select_item_below(current_index, column_count, total_items);
                Some(self.scroll_grid_to_current(column_count))
            }
            Key::Named(Named::ArrowUp) => {
                self.select_item_above(current_index, column_count);
                Some(self.scroll_grid_to_current(column_count))
            }
            Key::Named(Named::Home) => {
                self.state.current_index = 0;
                Some(self.scroll_grid_to_current(column_count))
            }
            Key::Named(Named::End) => {
                self.state.current_index = total_items - 1;
                Some(self.scroll_grid_to_current(column_count))
            }
            Key::Named(Named::PageDown) => {
                let page_items = self.grid_page_rows(dimensions.row_step()) * column_count;
                self.state.current_index = (current_index + page_items).min(total_items - 1);
                Some(self.scroll_grid_to_current(column_count))
            }
            Key::Named(Named::PageUp) => {
                let page_items = self.grid_page_rows(dimensions.row_step()) * column_count;
                self.state.current_index = current_index.saturating_sub(page_items);
                Some(self.scroll_grid_to_current(column_count))
            }
            Key::Named(Named::Enter) => Some(self.update(
                crate::app::messages::NavigationMsg::FileClickedInGrid(current_index).into(),
            )),
            _ => None,
        }
    }

    fn select_next_item(&mut self, current_index: usize, total_items: usize) {
        if current_index + 1 < total_items {
            self.state.current_index = current_index + 1;
        }
    }

    fn select_previous_item(&mut self, current_index: usize) {
        if current_index > 0 {
            self.state.current_index = current_index - 1;
        }
    }

    fn select_item_below(&mut self, current_index: usize, column_count: usize, total_items: usize) {
        self.state.current_index = if current_index + column_count < total_items {
            current_index + column_count
        } else if current_index < total_items - 1 {
            total_items - 1
        } else {
            current_index
        };
    }

    fn select_item_above(&mut self, current_index: usize, column_count: usize) {
        self.state.current_index = current_index.saturating_sub(column_count);
    }

    fn grid_page_rows(&self, row_step: f32) -> usize {
        let vh = if self.state.grid_viewport_height > 0.0 {
            self.state.grid_viewport_height
        } else {
            self.state.window_height
        };
        if vh <= 0.0 || row_step <= 0.0 {
            return 3;
        }
        (vh / row_step).floor().max(1.0) as usize
    }

    fn scroll_grid_to_current(&mut self, column_count: usize) -> Task<Message> {
        let dimensions =
            crate::features::grid::constants::ScaledDimensions::new(self.state.grid_scale);
        let row_step = dimensions.row_step();
        let current_row = self.state.current_index / column_count;
        let row_top = current_row as f32 * row_step;
        let row_bottom = row_top + row_step;
        let viewport_height = if self.state.grid_viewport_height > 0.0 {
            self.state.grid_viewport_height
        } else {
            self.state.window_height
        };

        let new_scroll_y = if row_top < self.state.grid_scroll_y {
            Some(row_top)
        } else if viewport_height > 0.0 && row_bottom > self.state.grid_scroll_y + viewport_height {
            Some((row_bottom - viewport_height).max(0.0))
        } else {
            None
        };

        let Some(target_y) = new_scroll_y else {
            return Task::none();
        };

        self.state.grid_scroll_y = target_y;
        let scroll_task = iced::widget::operation::scroll_to(
            GRID_SCROLL_ID,
            AbsoluteOffset {
                x: 0.0,
                y: target_y,
            },
        );
        let load_task = crate::app::update::grid::load_visible_grid_thumbnails(self);
        Task::batch([scroll_task, load_task])
    }
}
