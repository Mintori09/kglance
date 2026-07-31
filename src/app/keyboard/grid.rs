use iced::Task;
use iced::widget::operation::AbsoluteOffset;

use super::Message;

use crate::app::KglanceApp;

impl KglanceApp {
    pub(super) fn handle_grid_navigation(
        &mut self,
        key: &iced::keyboard::Key,
    ) -> Option<Task<Message>> {
        use iced::keyboard::key::Named;

        let total = self.state.playlist.len();
        if total == 0 {
            return None;
        }

        let cols = self.state.grid_cols.max(1);
        let cur = self.state.current_index;

        match key {
            iced::keyboard::Key::Named(Named::ArrowRight) => {
                if cur + 1 < total {
                    self.state.current_index = cur + 1;
                }
                Some(self.grid_scroll_to_current(cols))
            }
            iced::keyboard::Key::Named(Named::ArrowLeft) => {
                if cur > 0 {
                    self.state.current_index = cur - 1;
                }
                Some(self.grid_scroll_to_current(cols))
            }
            iced::keyboard::Key::Named(Named::ArrowDown) => {
                if cur + cols < total {
                    self.state.current_index = cur + cols;
                } else if cur < total - 1 {
                    self.state.current_index = total - 1;
                }
                Some(self.grid_scroll_to_current(cols))
            }
            iced::keyboard::Key::Named(Named::ArrowUp) => {
                if cur >= cols {
                    self.state.current_index = cur - cols;
                } else {
                    self.state.current_index = 0;
                }
                Some(self.grid_scroll_to_current(cols))
            }
            iced::keyboard::Key::Named(Named::Enter) => Some(
                self.update(crate::app::messages::NavigationMsg::FileClickedInGrid(cur).into()),
            ),
            _ => None,
        }
    }

    fn grid_scroll_to_current(&self, cols: usize) -> Task<Message> {
        let row_h = crate::core::types::GRID_ROW_HEIGHT * self.state.grid_scale;
        let row_idx = (self.state.current_index / cols) as f32;
        let y = if row_idx == 0.0 { 0.0 } else { row_idx * row_h };
        iced::widget::operation::scroll_to("grid_scroll", AbsoluteOffset { x: 0.0, y })
    }
}
