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

        if total_items == 0 {
            return None;
        }

        let column_count = self.state.grid_cols.max(MIN_GRID_COLUMNS);
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

    fn scroll_grid_to_current(&self, column_count: usize) -> Task<Message> {
        let row_height = crate::core::types::GRID_ROW_HEIGHT * self.state.grid_scale;
        let current_row = self.state.current_index / column_count;
        let scroll_offset_y = current_row as f32 * row_height;

        iced::widget::operation::scroll_to(
            GRID_SCROLL_ID,
            AbsoluteOffset {
                x: 0.0,
                y: scroll_offset_y,
            },
        )
    }
}
