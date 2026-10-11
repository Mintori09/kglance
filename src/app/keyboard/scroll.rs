use iced::Task;
use iced::widget::operation::{self, AbsoluteOffset, RelativeOffset};

use super::Message;
use crate::app::KglanceApp;
use crate::core::PreviewData;

const CONTENT_SCROLL_ID: &str = "content_scroll";
const SCROLL_LINE_AMOUNT: f32 = 80.0;

/// Pixel margin threshold below which scroll position is considered already at the boundary.
const SCROLL_EPSILON: f32 = 1.0;

/// Default viewport height fallback if neither the scroll container nor window dimensions are reported yet.
const DEFAULT_VIEWPORT_HEIGHT: f32 = 800.0;

/// Number of viewports used as the distance threshold for the optical landing glide on `gg`/`G`.
///
/// At ~800px viewport, 2 viewports equals ~1600px (~75-80 text lines), matching the text
/// viewer's virtualizer overscan buffer (`overscan = 80`) to guarantee that the landing glide
/// runs entirely within pre-parsed/tokenized cache without mid-flight frame drops.
const OPTICAL_WARP_VIEWPORT_FACTOR: f32 = 2.0;

/// Lower clamp bound for the optical warp landing distance.
///
/// Guarantees that smaller or split-screen windows still provide enough runway (~1.5 viewports)
/// for human eye perception of cubic ease-out deceleration.
const MIN_OPTICAL_WARP_DISTANCE: f32 = 1200.0;

/// Upper clamp bound for the optical warp landing distance.
///
/// Prevents large 4K / UHD vertical displays from exceeding the virtualizer overscan cache
/// or creating an excessively long animation (>300ms).
const MAX_OPTICAL_WARP_DISTANCE: f32 = 2400.0;

fn calculate_optical_warp_distance(target_vh: f32, fallback_vh: f32) -> f32 {
    let vh = if target_vh > 0.0 {
        target_vh
    } else {
        fallback_vh
    };
    (OPTICAL_WARP_VIEWPORT_FACTOR * vh).clamp(MIN_OPTICAL_WARP_DISTANCE, MAX_OPTICAL_WARP_DISTANCE)
}

struct ActiveScrollTarget<'a> {
    smooth_scroll: &'a mut crate::core::types::SmoothScrollState,
    scroll_controller: &'a mut crate::core::scroll::ScrollController,
    scroll_y: &'a mut f32,
    total_content_height: f32,
    viewport_height: f32,
}

impl KglanceApp {
    pub(super) fn handle_scroll_shortcuts(
        &mut self,
        key: &iced::keyboard::Key,
        modifiers: iced::keyboard::Modifiers,
    ) -> Option<Task<Message>> {
        if modifiers.control() || self.ctrl_held {
            return None;
        }

        use iced::keyboard::Key;
        use iced::keyboard::key::Named;

        match key {
            Key::Named(Named::ArrowDown) => Some(self.scroll_by(SCROLL_LINE_AMOUNT)),
            Key::Character(character) if character == "j" => {
                Some(self.scroll_by(SCROLL_LINE_AMOUNT))
            }
            Key::Named(Named::ArrowUp) => Some(self.scroll_by(-SCROLL_LINE_AMOUNT)),
            Key::Character(character) if character == "k" => {
                Some(self.scroll_by(-SCROLL_LINE_AMOUNT))
            }
            Key::Named(Named::PageDown) => Some(self.scroll_page(0.85)),
            Key::Character(character) if character == "d" => Some(self.scroll_page(0.5)),
            Key::Named(Named::PageUp) => Some(self.scroll_page(-0.85)),
            Key::Character(character) if character == "u" => Some(self.scroll_page(-0.5)),
            Key::Named(Named::ArrowLeft) => {
                Some(self.scroll_horizontal_by(-SCROLL_LINE_AMOUNT * 1.5))
            }
            Key::Named(Named::ArrowRight) => {
                Some(self.scroll_horizontal_by(SCROLL_LINE_AMOUNT * 1.5))
            }
            Key::Character(character)
                if character == "h"
                    && matches!(self.current_content, Some(PreviewData::Spreadsheet { .. })) =>
            {
                Some(self.scroll_horizontal_by(-SCROLL_LINE_AMOUNT * 1.5))
            }
            Key::Character(character)
                if character == "l"
                    && matches!(self.current_content, Some(PreviewData::Spreadsheet { .. })) =>
            {
                Some(self.scroll_horizontal_by(SCROLL_LINE_AMOUNT * 1.5))
            }
            Key::Character(character) if character == "g" && !modifiers.shift() => {
                self.handle_g_shortcut()
            }
            Key::Character(character)
                if character == "G" || (character == "g" && modifiers.shift()) =>
            {
                Some(self.snap_to_bottom())
            }
            Key::Named(Named::Home) => self.handle_home_shortcut(),
            Key::Named(Named::End) => Some(self.snap_to_bottom()),
            Key::Character(character) if character == "t" && self.pending_g => {
                self.toggle_sidebar()
            }
            Key::Character(character) if character == "e" && self.pending_g => {
                self.toggle_file_tree()
            }
            _ => {
                self.reset_scroll_pending();
                None
            }
        }
    }

    fn active_scroll_target_mut(&mut self) -> Option<ActiveScrollTarget<'_>> {
        match self.current_content {
            Some(PreviewData::Markdown { .. }) | Some(PreviewData::Epub { .. }) => {
                let state = crate::features::markdown::update::active_markdown_state_mut(self);
                Some(ActiveScrollTarget {
                    smooth_scroll: &mut state.smooth_scroll,
                    scroll_controller: &mut state.scroll_controller,
                    scroll_y: &mut state.scroll_y,
                    total_content_height: state.total_content_height,
                    viewport_height: state.viewport_height,
                })
            }
            Some(PreviewData::Text { .. }) => {
                let state = &mut self.state.text;
                Some(ActiveScrollTarget {
                    smooth_scroll: &mut state.smooth_scroll,
                    scroll_controller: &mut state.scroll_controller,
                    scroll_y: &mut state.scroll_y,
                    total_content_height: state.total_content_height,
                    viewport_height: state.viewport_height,
                })
            }
            Some(PreviewData::Pdf { .. }) => {
                let state = &mut self.state.pdf;
                Some(ActiveScrollTarget {
                    smooth_scroll: &mut state.smooth_scroll,
                    scroll_controller: &mut state.scroll_controller,
                    scroll_y: &mut state.scroll_y,
                    total_content_height: state.total_content_height,
                    viewport_height: state.viewport_height,
                })
            }
            Some(PreviewData::Typst { .. }) => {
                if self.state.typst.show_source || self.state.typst.error.is_some() {
                    let state = &mut self.state.typst.source_text;
                    Some(ActiveScrollTarget {
                        smooth_scroll: &mut state.smooth_scroll,
                        scroll_controller: &mut state.scroll_controller,
                        scroll_y: &mut state.scroll_y,
                        total_content_height: state.total_content_height,
                        viewport_height: state.viewport_height,
                    })
                } else {
                    let state = &mut self.state.pdf;
                    Some(ActiveScrollTarget {
                        smooth_scroll: &mut state.smooth_scroll,
                        scroll_controller: &mut state.scroll_controller,
                        scroll_y: &mut state.scroll_y,
                        total_content_height: state.total_content_height,
                        viewport_height: state.viewport_height,
                    })
                }
            }
            Some(PreviewData::Json { .. }) => {
                if !self.state.json.tree_mode {
                    let state = &mut self.state.json.raw_text;
                    Some(ActiveScrollTarget {
                        smooth_scroll: &mut state.smooth_scroll,
                        scroll_controller: &mut state.scroll_controller,
                        scroll_y: &mut state.scroll_y,
                        total_content_height: state.total_content_height,
                        viewport_height: state.viewport_height,
                    })
                } else {
                    let state = &mut self.state.json;
                    Some(ActiveScrollTarget {
                        smooth_scroll: &mut state.smooth_scroll,
                        scroll_controller: &mut state.scroll_controller,
                        scroll_y: &mut state.scroll_y,
                        total_content_height: state.total_content_height,
                        viewport_height: state.viewport_height,
                    })
                }
            }
            Some(PreviewData::Spreadsheet { .. }) => {
                let state = &mut self.state.spreadsheet;
                Some(ActiveScrollTarget {
                    smooth_scroll: &mut state.smooth_scroll,
                    scroll_controller: &mut state.scroll_controller,
                    scroll_y: &mut state.scroll_y,
                    total_content_height: state.total_content_height,
                    viewport_height: state.viewport_height,
                })
            }
            _ => None,
        }
    }

    pub(crate) fn active_scroll_y(&self) -> Option<f32> {
        match self.current_content {
            Some(PreviewData::Markdown { .. }) | Some(PreviewData::Epub { .. }) => {
                Some(crate::features::markdown::update::active_markdown_state(self).scroll_y)
            }
            Some(PreviewData::Text { .. }) => Some(self.state.text.scroll_y),
            Some(PreviewData::Pdf { .. }) => Some(self.state.pdf.scroll_y),
            Some(PreviewData::Typst { .. }) => {
                if self.state.typst.show_source || self.state.typst.error.is_some() {
                    Some(self.state.typst.source_text.scroll_y)
                } else {
                    Some(self.state.pdf.scroll_y)
                }
            }
            Some(PreviewData::Json { .. }) => {
                if !self.state.json.tree_mode {
                    Some(self.state.json.raw_text.scroll_y)
                } else {
                    Some(self.state.json.scroll_y)
                }
            }
            Some(PreviewData::Spreadsheet { .. }) => Some(self.state.spreadsheet.scroll_y),
            _ => None,
        }
    }

    pub(crate) fn stop_active_scroll_animations(&mut self) {
        if let Some(target) = self.active_scroll_target_mut() {
            let current_y = *target.scroll_y;
            target.scroll_controller.stop(current_y);
            target.smooth_scroll.stop(current_y);
        }
    }

    fn scroll_page(&mut self, fraction: f32) -> Task<Message> {
        self.reset_scroll_pending();

        if let Some(target) = self.active_scroll_target_mut() {
            if target.viewport_height <= 0.0 {
                return Task::none();
            }
            let max_y = crate::core::scroll::max_scroll_y(
                target.total_content_height,
                target.viewport_height,
            );
            let page_delta = target.viewport_height * fraction;
            let current_y = *target.scroll_y;
            target.scroll_controller.stop(current_y);
            let base_y = if target.smooth_scroll.is_animating
                && (target.smooth_scroll.target_y - current_y).signum() == fraction.signum()
            {
                target.smooth_scroll.target_y
            } else {
                current_y
            };
            let target_y = crate::core::scroll::clamp_target(base_y + page_delta, max_y);
            target
                .smooth_scroll
                .start_navigation(current_y, target_y, max_y);
            if !target.smooth_scroll.is_animating {
                let new_y = target.smooth_scroll.position_y();
                *target.scroll_y = new_y;
                return operation::scroll_to(
                    CONTENT_SCROLL_ID,
                    AbsoluteOffset { x: 0.0, y: new_y },
                );
            }
            return Task::none();
        }

        let vh = if self.state.current_window_size.height > 0.0 {
            self.state.current_window_size.height
        } else {
            768.0
        };

        operation::scroll_by(
            CONTENT_SCROLL_ID,
            AbsoluteOffset {
                x: 0.0,
                y: vh * fraction,
            },
        )
    }

    fn scroll_by(&mut self, vertical_offset: f32) -> Task<Message> {
        self.reset_scroll_pending();

        if let Some(target) = self.active_scroll_target_mut() {
            if target.viewport_height <= 0.0 {
                return Task::none();
            }
            let max_y = crate::core::scroll::max_scroll_y(
                target.total_content_height,
                target.viewport_height,
            );
            let current_y = *target.scroll_y;
            target.scroll_controller.stop(current_y);
            target
                .smooth_scroll
                .start_interactive(current_y, vertical_offset, max_y);
            if !target.smooth_scroll.is_animating {
                let new_y = target.smooth_scroll.position_y();
                *target.scroll_y = new_y;
                return operation::scroll_to(
                    CONTENT_SCROLL_ID,
                    AbsoluteOffset { x: 0.0, y: new_y },
                );
            }
            return Task::none();
        }

        operation::scroll_by(
            CONTENT_SCROLL_ID,
            AbsoluteOffset {
                x: 0.0,
                y: vertical_offset,
            },
        )
    }

    fn scroll_horizontal_by(&mut self, horizontal_offset: f32) -> Task<Message> {
        self.reset_scroll_pending();

        if let Some(PreviewData::Spreadsheet { .. }) = self.current_content {
            let state = &mut self.state.spreadsheet;
            if state.viewport_width <= 0.0 {
                return Task::none();
            }
            let max_x =
                crate::core::scroll::max_scroll_y(state.total_content_width, state.viewport_width);
            let current_x = state.scroll_x;
            state.scroll_controller_x.stop(current_x);
            state
                .smooth_scroll_x
                .start_interactive(current_x, horizontal_offset, max_x);
            if !state.smooth_scroll_x.is_animating {
                let new_x = state.smooth_scroll_x.position_y();
                state.scroll_x = new_x;
                return operation::scroll_to(
                    CONTENT_SCROLL_ID,
                    AbsoluteOffset {
                        x: new_x,
                        y: state.scroll_y,
                    },
                );
            }
            return Task::none();
        }

        operation::scroll_by(
            CONTENT_SCROLL_ID,
            AbsoluteOffset {
                x: horizontal_offset,
                y: 0.0,
            },
        )
    }

    fn handle_g_shortcut(&mut self) -> Option<Task<Message>> {
        self.pending_home = false;

        if self.pending_g {
            self.pending_g = false;
            Some(self.snap_to_top())
        } else {
            self.pending_g = true;
            None
        }
    }

    fn handle_home_shortcut(&mut self) -> Option<Task<Message>> {
        self.pending_g = false;
        self.pending_home = false;
        Some(self.snap_to_top())
    }

    fn fallback_viewport_height(&self) -> f32 {
        if self.state.current_window_size.height > 0.0 {
            self.state.current_window_size.height
        } else if self.state.window_height > 0.0 {
            self.state.window_height
        } else {
            DEFAULT_VIEWPORT_HEIGHT
        }
    }

    pub(super) fn snap_to_top(&mut self) -> Task<Message> {
        self.reset_scroll_pending();
        let fallback_vh = self.fallback_viewport_height();

        if let Some(target) = self.active_scroll_target_mut() {
            let current_y = *target.scroll_y;
            target.scroll_controller.stop(current_y);
            if current_y <= SCROLL_EPSILON {
                self.record_read_position();
                return operation::snap_to(CONTENT_SCROLL_ID, RelativeOffset { x: 0.0, y: 0.0 });
            }
            let max_y = crate::core::scroll::max_scroll_y(
                target.total_content_height,
                target.viewport_height,
            );
            let warp_distance =
                calculate_optical_warp_distance(target.viewport_height, fallback_vh);
            let start_y = if current_y > warp_distance {
                *target.scroll_y = warp_distance;
                target.scroll_controller.set_position_y(warp_distance);
                warp_distance
            } else {
                current_y
            };
            target.smooth_scroll.start_navigation(start_y, 0.0, max_y);
            if !target.smooth_scroll.is_animating {
                let new_y = target.smooth_scroll.position_y();
                *target.scroll_y = new_y;
                self.record_read_position();
                return operation::snap_to(CONTENT_SCROLL_ID, RelativeOffset { x: 0.0, y: 0.0 });
            }
            if matches!(self.current_content, Some(PreviewData::Epub { .. }))
                && self.state.epub.reading_mode == crate::core::config::EpubReadingMode::Continuous
            {
                self.state.epub.active_chapter = 0;
            }
            return Task::none();
        }

        self.record_read_position();
        operation::snap_to(CONTENT_SCROLL_ID, RelativeOffset { x: 0.0, y: 0.0 })
    }

    pub(super) fn snap_to_bottom(&mut self) -> Task<Message> {
        self.reset_scroll_pending();
        let fallback_vh = self.fallback_viewport_height();

        if let Some(target) = self.active_scroll_target_mut() {
            let current_y = *target.scroll_y;
            target.scroll_controller.stop(current_y);
            let max_y = crate::core::scroll::max_scroll_y(
                target.total_content_height,
                target.viewport_height,
            );
            if (max_y - current_y).abs() <= SCROLL_EPSILON {
                self.record_read_position();
                return operation::snap_to(CONTENT_SCROLL_ID, RelativeOffset { x: 0.0, y: 1.0 });
            }
            let warp_distance =
                calculate_optical_warp_distance(target.viewport_height, fallback_vh);
            let start_y = if max_y - current_y > warp_distance {
                let y = (max_y - warp_distance).max(0.0);
                *target.scroll_y = y;
                target.scroll_controller.set_position_y(y);
                y
            } else {
                current_y
            };
            target.smooth_scroll.start_navigation(start_y, max_y, max_y);
            if !target.smooth_scroll.is_animating {
                let new_y = target.smooth_scroll.position_y();
                *target.scroll_y = new_y;
                self.record_read_position();
                return operation::snap_to(CONTENT_SCROLL_ID, RelativeOffset { x: 0.0, y: 1.0 });
            }
            if matches!(self.current_content, Some(PreviewData::Epub { .. }))
                && self.state.epub.reading_mode == crate::core::config::EpubReadingMode::Continuous
            {
                self.state.epub.active_chapter = self.state.epub.chapters.len().saturating_sub(1);
            }
            return Task::none();
        }

        self.record_read_position();
        operation::snap_to(CONTENT_SCROLL_ID, RelativeOffset { x: 0.0, y: 1.0 })
    }

    fn toggle_sidebar(&mut self) -> Option<Task<Message>> {
        self.reset_scroll_pending();
        self.state.file_tree.visible = false;

        match self.current_content {
            Some(PreviewData::Markdown { .. }) => Some(Task::done(
                crate::app::messages::MarkdownMsg::TocToggled.into(),
            )),
            Some(PreviewData::Epub { .. }) => Some(Task::done(
                crate::app::messages::EpubMsg::SidebarToggled.into(),
            )),
            Some(PreviewData::Pdf { .. }) | Some(PreviewData::Typst { .. }) => Some(Task::done(
                crate::app::messages::PdfMsg::SidebarToggled.into(),
            )),
            Some(PreviewData::Text { .. }) => Some(Task::done(
                crate::app::messages::TextMsg::ToggleOutline.into(),
            )),
            _ => None,
        }
    }

    fn toggle_file_tree(&mut self) -> Option<Task<Message>> {
        self.reset_scroll_pending();
        Some(Task::done(crate::app::messages::FileTreeMsg::Toggle.into()))
    }

    fn reset_scroll_pending(&mut self) {
        self.pending_g = false;
        self.pending_home = false;
    }

    pub(super) fn handle_json_tree_navigation(
        &mut self,
        key: &iced::keyboard::Key,
        modifiers: iced::keyboard::Modifiers,
    ) -> Option<Task<Message>> {
        use iced::keyboard::Key;
        use iced::keyboard::key::Named;

        if !self.is_json_tree_nav_available() || modifiers.control() || modifiers.alt() {
            return None;
        }

        let visible = crate::features::json::view::tree::visible_node_indices(&self.state.json);
        if visible.is_empty() {
            return None;
        }

        match key {
            Key::Named(Named::ArrowDown) => {
                self.move_json_selection(&visible, 1);
                Some(Task::none())
            }
            Key::Character(character) if character == "j" => {
                self.move_json_selection(&visible, 1);
                Some(Task::none())
            }
            Key::Named(Named::ArrowUp) => {
                self.move_json_selection(&visible, -1);
                Some(Task::none())
            }
            Key::Character(character) if character == "k" => {
                self.move_json_selection(&visible, -1);
                Some(Task::none())
            }
            Key::Named(Named::ArrowRight) => {
                self.expand_or_enter_json_node();
                Some(Task::none())
            }
            Key::Character(character) if character == "l" => {
                self.expand_or_enter_json_node();
                Some(Task::none())
            }
            Key::Named(Named::ArrowLeft) => {
                self.collapse_or_exit_json_node();
                Some(Task::none())
            }
            Key::Character(character) if character == "h" => {
                self.collapse_or_exit_json_node();
                Some(Task::none())
            }
            Key::Named(Named::Enter) => {
                if let Some(active) = self.state.json.active_node
                    && self
                        .state
                        .json
                        .nodes
                        .get(active)
                        .is_some_and(|n| n.children_count > 0)
                {
                    return Some(
                        self.update(crate::app::messages::JsonMsg::ToggleNode(active).into()),
                    );
                }
                None
            }
            _ => None,
        }
    }

    fn is_json_tree_nav_available(&self) -> bool {
        matches!(self.current_content, Some(PreviewData::Json { .. }))
            && self.state.json.tree_mode
            && !self.state.json.search_visible
    }

    fn move_json_selection(&mut self, visible: &[usize], delta: isize) {
        let current_pos = self
            .state
            .json
            .active_node
            .and_then(|curr| visible.iter().position(|&idx| idx == curr));

        let new_pos = match current_pos {
            Some(pos) => {
                let target = pos as isize + delta;
                target.clamp(0, (visible.len() - 1) as isize) as usize
            }
            None => {
                if delta > 0 {
                    0
                } else {
                    visible.len() - 1
                }
            }
        };

        if let Some(&node_idx) = visible.get(new_pos) {
            self.state.json.active_node = Some(node_idx);
        }
    }

    fn expand_or_enter_json_node(&mut self) {
        if let Some(active) = self.state.json.active_node
            && let Some(node) = self.state.json.nodes.get(active)
            && node.children_count > 0
        {
            if !self.state.json.expanded.contains(&active) {
                self.state.json.expanded.insert(active);
            } else if active + 1 < self.state.json.nodes.len() {
                self.state.json.active_node = Some(active + 1);
            }
        }
    }

    fn collapse_or_exit_json_node(&mut self) {
        if let Some(active) = self.state.json.active_node {
            if self.state.json.expanded.contains(&active) {
                self.state.json.expanded.remove(&active);
            } else if let Some(parent) = self
                .state
                .json
                .nodes
                .get(active)
                .and_then(|n| n.parent_index)
            {
                self.state.json.active_node = Some(parent);
            }
        }
    }
}

#[cfg(test)]
#[path = "scroll_tests.rs"]
mod tests;
