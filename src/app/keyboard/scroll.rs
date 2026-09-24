use iced::Task;
use iced::widget::operation::{self, AbsoluteOffset, RelativeOffset};

use super::Message;
use crate::app::KglanceApp;
use crate::core::PreviewData;

const CONTENT_SCROLL_ID: &str = "content_scroll";
const SCROLL_LINE_AMOUNT: f32 = 80.0;

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
            Some(PreviewData::Pdf { .. }) | Some(PreviewData::Typst { .. }) => {
                let state = crate::features::pdf::update::active_pdf_state_mut(self);
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

        if let Some(target) = self.active_scroll_target_mut() {
            let current_y = *target.scroll_y;
            target.scroll_controller.stop(current_y);
            if current_y <= 1.0 {
                self.record_read_position();
                return operation::snap_to(CONTENT_SCROLL_ID, RelativeOffset { x: 0.0, y: 0.0 });
            }
            let max_y = crate::core::scroll::max_scroll_y(
                target.total_content_height,
                target.viewport_height,
            );
            target.smooth_scroll.start_navigation(current_y, 0.0, max_y);
            return Task::none();
        }

        self.record_read_position();
        operation::snap_to(CONTENT_SCROLL_ID, RelativeOffset { x: 0.0, y: 0.0 })
    }

    pub(super) fn snap_to_bottom(&mut self) -> Task<Message> {
        self.reset_scroll_pending();

        if let Some(target) = self.active_scroll_target_mut() {
            let current_y = *target.scroll_y;
            target.scroll_controller.stop(current_y);
            let max_y = crate::core::scroll::max_scroll_y(
                target.total_content_height,
                target.viewport_height,
            );
            if (max_y - current_y).abs() <= 1.0 {
                self.record_read_position();
                return operation::snap_to(CONTENT_SCROLL_ID, RelativeOffset { x: 0.0, y: 1.0 });
            }
            target
                .smooth_scroll
                .start_navigation(current_y, max_y, max_y);
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

    #[test]
    fn test_text_scroll_page_and_vim_navigation() {
        use crate::app::test_util::text_content;

        let code = (0..200)
            .map(|i| format!("fn line_{i}() {{ println!(\"Hello {i}\"); }}"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut app = test_app(Some(text_content(&code, "rs")));
        app.state.text.viewport_height = 800.0;
        app.state.text.total_content_height = 4000.0;
        app.state.text.scroll_y = 0.0;

        // 1. Scroll page down (PageDown or 'd')
        let _ = app.scroll_page(0.5);
        assert!(app.state.text.smooth_scroll.is_animating);
        assert_eq!(app.state.text.smooth_scroll.target_y(), 400.0);

        // 2. Step smooth scroll animation
        let mut now = std::time::Instant::now();
        for _ in 0..30 {
            now += std::time::Duration::from_millis(16);
            let _ = crate::features::text::update::handle_smooth_scroll_tick(&mut app, now);
        }
        assert_eq!(app.state.text.scroll_y, 400.0);
        assert!(!app.state.text.smooth_scroll.is_animating);

        // 3. Snap to bottom (G)
        let _ = app.snap_to_bottom();
        assert_eq!(app.state.text.smooth_scroll.target_y(), 3200.0); // 4000.0 - 800.0
        for _ in 0..50 {
            now += std::time::Duration::from_millis(16);
            let _ = crate::features::text::update::handle_smooth_scroll_tick(&mut app, now);
        }
        assert_eq!(app.state.text.scroll_y, 3200.0);

        // 4. Snap to top (gg)
        let _ = app.snap_to_top();
        assert_eq!(app.state.text.smooth_scroll.target_y(), 0.0);
        for _ in 0..50 {
            now += std::time::Duration::from_millis(16);
            let _ = crate::features::text::update::handle_smooth_scroll_tick(&mut app, now);
        }
        assert_eq!(app.state.text.scroll_y, 0.0);
    }

    #[test]
    fn test_text_touchpad_velocity_kinetic_scroll() {
        use crate::app::test_util::text_content;

        let code = (0..500)
            .map(|i| format!("fn line_{i}() {{ println!(\"Code Line {i}\"); }}"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut app = test_app(Some(text_content(&code, "rs")));
        app.state.text.viewport_height = 800.0;
        app.state.text.total_content_height = 10000.0;
        app.state.text.scroll_y = 0.0;

        // 1. First swipe
        let _ = crate::features::text::update::handle_wheel_scrolled(
            &mut app,
            iced::mouse::ScrollDelta::Pixels { x: 0.0, y: -20.0 },
        );
        assert_eq!(app.state.text.scroll_y, 50.0);
        assert_eq!(
            app.state.text.scroll_controller.state(),
            crate::core::scroll::GestureState::Dragging
        );

        // 2. Second rapid swipe
        std::thread::sleep(std::time::Duration::from_millis(15));
        let _ = crate::features::text::update::handle_wheel_scrolled(
            &mut app,
            iced::mouse::ScrollDelta::Pixels { x: 0.0, y: -30.0 },
        );
        assert_eq!(app.state.text.scroll_y, 125.0);
        assert_eq!(
            app.state.text.scroll_controller.state(),
            crate::core::scroll::GestureState::Dragging
        );

        // 3. Opportunistic zero delta on finger release
        let _ = crate::features::text::update::handle_wheel_scrolled(
            &mut app,
            iced::mouse::ScrollDelta::Pixels { x: 0.0, y: 0.0 },
        );
        assert!(app.state.text.scroll_controller.is_animating());
        assert_eq!(
            app.state.text.scroll_controller.state(),
            crate::core::scroll::GestureState::Flinging
        );

        // 4. Step physics frames
        let mut now = std::time::Instant::now();
        for _ in 0..60 {
            now += std::time::Duration::from_millis(16);
            let _ = crate::features::text::update::handle_smooth_scroll_tick(&mut app, now);
        }

        // Must glide significantly further than swipe displacement
        assert!(app.state.text.scroll_y > 125.0 + 100.0);
    }
}
