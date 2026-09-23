use iced::Task;
use iced::widget::operation::{self, AbsoluteOffset, RelativeOffset};

use super::Message;
use crate::app::KglanceApp;
use crate::core::PreviewData;

const CONTENT_SCROLL_ID: &str = "content_scroll";
const SCROLL_LINE_AMOUNT: f32 = 80.0;

impl KglanceApp {
    pub(super) fn handle_scroll_shortcuts(
        &mut self,
        key: &iced::keyboard::Key,
        modifiers: iced::keyboard::Modifiers,
    ) -> Option<Task<Message>> {
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
            _ => {
                self.reset_scroll_pending();
                None
            }
        }
    }

    fn scroll_page(&mut self, fraction: f32) -> Task<Message> {
        self.reset_scroll_pending();

        if self.is_smooth_scrollable() {
            let state = crate::features::markdown::update::active_markdown_state_mut(self);
            if state.viewport_height <= 0.0 {
                return Task::none();
            }
            let max_y = crate::core::scroll::max_scroll_y(
                state.total_content_height,
                state.viewport_height,
            );
            let page_delta = state.viewport_height * fraction;
            let base_y = if state.smooth_scroll.is_animating
                && (state.smooth_scroll.target_y - state.scroll_y).signum() == fraction.signum()
            {
                state.smooth_scroll.target_y
            } else {
                state.scroll_y
            };
            let target_y = crate::core::scroll::clamp_target(base_y + page_delta, max_y);
            state
                .smooth_scroll
                .start_navigation(state.scroll_y, target_y, max_y);
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

        if self.is_smooth_scrollable() {
            let state = crate::features::markdown::update::active_markdown_state_mut(self);
            if state.viewport_height <= 0.0 {
                return Task::none();
            }
            let max_y = crate::core::scroll::max_scroll_y(
                state.total_content_height,
                state.viewport_height,
            );
            state
                .smooth_scroll
                .start_interactive(state.scroll_y, vertical_offset, max_y);
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

    pub(super) fn snap_to_top(&mut self) -> Task<Message> {
        self.reset_scroll_pending();

        if self.is_smooth_scrollable() {
            let state = crate::features::markdown::update::active_markdown_state_mut(self);
            if state.scroll_y <= 1.0 {
                self.record_read_position();
                return operation::snap_to(CONTENT_SCROLL_ID, RelativeOffset { x: 0.0, y: 0.0 });
            }
            let max_y = crate::core::scroll::max_scroll_y(
                state.total_content_height,
                state.viewport_height,
            );
            state
                .smooth_scroll
                .start_navigation(state.scroll_y, 0.0, max_y);
            return Task::none();
        }

        self.record_read_position();
        operation::snap_to(CONTENT_SCROLL_ID, RelativeOffset { x: 0.0, y: 0.0 })
    }

    pub(super) fn snap_to_bottom(&mut self) -> Task<Message> {
        self.reset_scroll_pending();

        if self.is_smooth_scrollable() {
            let state = crate::features::markdown::update::active_markdown_state_mut(self);
            let max_y = crate::core::scroll::max_scroll_y(
                state.total_content_height,
                state.viewport_height,
            );
            if (max_y - state.scroll_y).abs() <= 1.0 {
                self.record_read_position();
                return operation::snap_to(CONTENT_SCROLL_ID, RelativeOffset { x: 0.0, y: 1.0 });
            }
            state
                .smooth_scroll
                .start_navigation(state.scroll_y, max_y, max_y);
            return Task::none();
        }

        self.record_read_position();
        operation::snap_to(CONTENT_SCROLL_ID, RelativeOffset { x: 0.0, y: 1.0 })
    }

    fn toggle_sidebar(&mut self) -> Option<Task<Message>> {
        self.reset_scroll_pending();

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
            Key::Named(Named::ArrowUp) => {
                self.move_json_selection(&visible, -1);
                Some(Task::none())
            }
            Key::Named(Named::ArrowRight) => {
                self.expand_or_enter_json_node();
                Some(Task::none())
            }
            Key::Named(Named::ArrowLeft) => {
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
mod tests {
    use crate::app::test_util::{markdown_content, test_app};

    #[test]
    fn test_scroll_page_triggers_smooth_navigation() {
        let mut app = test_app(Some(markdown_content("# Heading\n\nContent paragraph")));
        let state = crate::features::markdown::update::active_markdown_state_mut(&mut app);
        state.viewport_height = 800.0;
        state.total_content_height = 3000.0;
        state.scroll_y = 100.0;

        let _ = app.scroll_page(0.85);

        let state = crate::features::markdown::update::active_markdown_state(&app);
        assert!(state.smooth_scroll.is_animating);
        assert_eq!(
            state.smooth_scroll.mode(),
            crate::core::SmoothScrollMode::Navigation
        );
        // 100.0 + 800.0 * 0.85 = 780.0
        assert_eq!(state.smooth_scroll.target_y(), 780.0);
    }

    #[test]
    fn test_scroll_half_page_triggers_smooth_navigation() {
        let mut app = test_app(Some(markdown_content("# Heading\n\nContent paragraph")));
        let state = crate::features::markdown::update::active_markdown_state_mut(&mut app);
        state.viewport_height = 800.0;
        state.total_content_height = 3000.0;
        state.scroll_y = 100.0;

        let _ = app.scroll_page(0.5);

        let state = crate::features::markdown::update::active_markdown_state(&app);
        assert!(state.smooth_scroll.is_animating);
        // 100.0 + 800.0 * 0.5 = 500.0
        assert_eq!(state.smooth_scroll.target_y(), 500.0);
    }

    #[test]
    fn test_scroll_page_up_clamps_at_top() {
        let mut app = test_app(Some(markdown_content("# Heading\n\nContent paragraph")));
        let state = crate::features::markdown::update::active_markdown_state_mut(&mut app);
        state.viewport_height = 800.0;
        state.total_content_height = 3000.0;
        state.scroll_y = 200.0;

        let _ = app.scroll_page(-0.85);

        let state = crate::features::markdown::update::active_markdown_state(&app);
        assert!(state.smooth_scroll.is_animating);
        assert_eq!(state.smooth_scroll.target_y(), 0.0);
    }

    #[test]
    fn test_scroll_page_accumulates_rapid_keystrokes() {
        let mut app = test_app(Some(markdown_content("# Heading\n\nContent paragraph")));
        let state = crate::features::markdown::update::active_markdown_state_mut(&mut app);
        state.viewport_height = 1000.0;
        state.total_content_height = 5000.0;
        state.scroll_y = 0.0;

        // First PageDown: target = 0 + 850 = 850
        let _ = app.scroll_page(0.85);
        assert_eq!(
            crate::features::markdown::update::active_markdown_state(&app)
                .smooth_scroll
                .target_y(),
            850.0
        );

        // While in flight (simulated current scroll_y = 200), second PageDown arrives
        crate::features::markdown::update::active_markdown_state_mut(&mut app).scroll_y = 200.0;
        let _ = app.scroll_page(0.85);

        // Target accumulates to 850 + 850 = 1700
        assert_eq!(
            crate::features::markdown::update::active_markdown_state(&app)
                .smooth_scroll
                .target_y(),
            1700.0
        );
    }

    #[test]
    fn test_snap_to_top_and_bottom_triggers_smooth_navigation() {
        let mut app = test_app(Some(markdown_content("# Heading\n\nContent paragraph")));
        let state = crate::features::markdown::update::active_markdown_state_mut(&mut app);
        state.viewport_height = 800.0;
        state.total_content_height = 3000.0;
        state.scroll_y = 500.0;

        // Snap to top (gg or Home)
        let _ = app.snap_to_top();
        let state = crate::features::markdown::update::active_markdown_state(&app);
        assert_eq!(state.smooth_scroll.target_y(), 0.0);
        assert_eq!(
            state.smooth_scroll.mode(),
            crate::core::SmoothScrollMode::Navigation
        );
        assert!(state.smooth_scroll.is_animating);

        // Snap to bottom (G or End)
        let _ = app.snap_to_bottom();
        let state = crate::features::markdown::update::active_markdown_state(&app);
        assert_eq!(state.smooth_scroll.target_y(), 2200.0); // 3000.0 - 800.0
        assert_eq!(
            state.smooth_scroll.mode(),
            crate::core::SmoothScrollMode::Navigation
        );
        assert!(state.smooth_scroll.is_animating);
    }

    #[test]
    fn test_scroll_down_to_end_with_j() {
        let md = std::fs::read_to_string("testing-file/markdown.md").unwrap();
        let mut app = test_app(Some(markdown_content(&md)));
        let state = crate::features::markdown::update::active_markdown_state_mut(&mut app);
        state.viewport_height = 800.0;
        let total_h = state.total_content_height;
        let max_scroll = total_h - 800.0;

        let mut now = std::time::Instant::now();
        for _ in 0..200 {
            let _ = app.scroll_by(super::SCROLL_LINE_AMOUNT);
            for _ in 0..10 {
                now += std::time::Duration::from_millis(16);
                let _ = crate::features::markdown::update::handle_smooth_scroll_tick(&mut app, now);
            }
        }
        let state = crate::features::markdown::update::active_markdown_state(&app);
        assert!(
            (state.scroll_y - max_scroll).abs() < 1.0,
            "Expected scroll_y to reach max_scroll {max_scroll}, got {}",
            state.scroll_y
        );
    }

    #[test]
    fn test_snap_to_top_and_bottom_reaches_boundaries() {
        let mut app = test_app(Some(markdown_content("# Heading\n\nContent paragraph")));
        let state = crate::features::markdown::update::active_markdown_state_mut(&mut app);
        state.viewport_height = 800.0;
        state.total_content_height = 3000.0;
        state.scroll_y = 1500.0;

        // Snap to bottom (G)
        let _ = app.snap_to_bottom();
        let mut now = std::time::Instant::now();
        for _ in 0..50 {
            now += std::time::Duration::from_millis(16);
            let _ = crate::features::markdown::update::handle_smooth_scroll_tick(&mut app, now);
        }
        let state = crate::features::markdown::update::active_markdown_state(&app);
        assert_eq!(state.scroll_y, 2200.0);
        assert!(!state.smooth_scroll.is_animating);

        // Snap to top (gg)
        let _ = app.snap_to_top();
        for _ in 0..50 {
            now += std::time::Duration::from_millis(16);
            let _ = crate::features::markdown::update::handle_smooth_scroll_tick(&mut app, now);
        }
        let state = crate::features::markdown::update::active_markdown_state(&app);
        assert_eq!(state.scroll_y, 0.0);
        assert!(!state.smooth_scroll.is_animating);
    }

    #[test]
    fn test_gg_and_g_key_press_with_scrolled_events() {
        let mut app = test_app(Some(markdown_content("# Heading\n\nContent paragraph")));
        let state = crate::features::markdown::update::active_markdown_state_mut(&mut app);
        state.viewport_height = 800.0;
        state.total_content_height = 3000.0;
        state.scroll_y = 0.0;

        let g_key = iced::keyboard::Key::Character("g".into());
        let cap_g_key = iced::keyboard::Key::Character("G".into());
        let empty_mod = iced::keyboard::Modifiers::default();

        // 1. Press 'G' to go to bottom
        let _ = app.handle_key_pressed(cap_g_key, empty_mod);
        let mut now = std::time::Instant::now();
        for _ in 0..50 {
            now += std::time::Duration::from_millis(16);
            let _ = crate::features::markdown::update::handle_smooth_scroll_tick(&mut app, now);
            let current_y = crate::features::markdown::update::active_markdown_state(&app).scroll_y;
            // Simulate Iced emitting Scrolled event on each frame
            let _ = crate::features::markdown::update::handle_markdown_scrolled(
                &mut app, current_y, 800.0, 3000.0,
            );
        }
        let state = crate::features::markdown::update::active_markdown_state(&app);
        assert_eq!(state.scroll_y, 2200.0);
        assert!(!state.smooth_scroll.is_animating);

        // 2. Press 'g' once -> sets pending_g
        let _ = app.handle_key_pressed(g_key.clone(), empty_mod);
        assert!(app.pending_g);

        // 3. Press 'g' twice -> triggers snap_to_top
        let _ = app.handle_key_pressed(g_key, empty_mod);
        assert!(!app.pending_g);

        for _ in 0..50 {
            now += std::time::Duration::from_millis(16);
            let _ = crate::features::markdown::update::handle_smooth_scroll_tick(&mut app, now);
            let current_y = crate::features::markdown::update::active_markdown_state(&app).scroll_y;
            let _ = crate::features::markdown::update::handle_markdown_scrolled(
                &mut app, current_y, 800.0, 3000.0,
            );
        }
        let state = crate::features::markdown::update::active_markdown_state(&app);
        assert_eq!(state.scroll_y, 0.0);
        assert!(!state.smooth_scroll.is_animating);
    }
}
