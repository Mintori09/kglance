use super::*;
use crate::core::{SelectionPoint, SelectionRange};
use crate::parsers::markdown::{Block, parse_to_blocks};

#[test]
fn test_selected_text_extraction_with_inline_math() {
    let src = ". $\\Rightarrow$ **cấu trúc thuật toán ANN, nhu cầu can thiệp của con người thấp hơn và yêu cầu dữ liệu lớn hơn.**";
    let blocks = parse_to_blocks(src);

    let range = SelectionRange {
        start: SelectionPoint {
            block: 0,
            offset: 0,
        },
        end: SelectionPoint {
            block: 0,
            offset: 500,
        },
    };

    let selected = build_selected_text(&blocks, range);
    assert!(selected.is_some(), "selected text should not be None");
    let text = selected.unwrap();
    assert_eq!(
        text,
        ". ⇒ cấu trúc thuật toán ANN, nhu cầu can thiệp của con người thấp hơn và yêu cầu dữ liệu lớn hơn."
    );
}

#[test]
fn test_user_bullet_item_copy_with_inline_math() {
    let src = "- $\\Rightarrow$ **cấu trúc thuật toán ANN, nhu cầu can thiệp của con người thấp hơn và yêu cầu dữ liệu lớn hơn.**";
    let blocks = parse_to_blocks(src);
    assert_eq!(blocks.len(), 1);
    assert!(matches!(blocks[0], Block::List { .. }));

    // Test clicking & selecting the list item which has block_index = 1 (since list is at base_idx 0)
    let range = SelectionRange {
        start: SelectionPoint {
            block: 1,
            offset: 0,
        },
        end: SelectionPoint {
            block: 1,
            offset: 500,
        },
    };

    let selected = build_selected_text(&blocks, range);
    assert!(
        selected.is_some(),
        "selected text should not be None for list item"
    );
    let text = selected.unwrap();
    assert!(text.contains("cấu trúc thuật toán ANN"));
}

#[test]
fn test_partial_selection_phrase_thuat_toan_ann() {
    let src = "- $\\Rightarrow$ **cấu trúc thuật toán ANN, nhu cầu can thiệp của con người thấp hơn và yêu cầu dữ liệu lớn hơn.**";
    let blocks = parse_to_blocks(src);
    assert_eq!(blocks.len(), 1);

    let visual_text = crate::parsers::markdown::flatten_inlines_visual(
        if let Block::List { items, .. } = &blocks[0] {
            &items[0].content
        } else {
            panic!("Expected List block");
        },
    );

    let target = "thuật toán ANN";
    let start_offset = visual_text
        .find(target)
        .expect("Must find 'thuật toán ANN'");
    let end_offset = start_offset + target.len();

    let range = SelectionRange {
        start: SelectionPoint {
            block: 1,
            offset: start_offset,
        },
        end: SelectionPoint {
            block: 1,
            offset: end_offset,
        },
    };

    let selected = build_selected_text(&blocks, range);
    assert!(
        selected.is_some(),
        "selected_text should not be None when partial text is selected"
    );

    let text = selected.unwrap();
    assert_eq!(
        text, "thuật toán ANN",
        "Extracted text must match exactly the selected substring"
    );
}

#[test]
fn test_nested_bullet_item_copy_with_inline_math() {
    let src = "- Mục cha\n  - $\\Rightarrow$ **cấu trúc thuật toán ANN, nhu cầu can thiệp của con người thấp hơn và yêu cầu dữ liệu lớn hơn.**";
    let blocks = parse_to_blocks(src);
    assert_eq!(blocks.len(), 1);
    assert!(matches!(blocks[0], Block::List { .. }));

    // Item 0 is at block 1, its sub-list items are at block 3 (base_idx 2 + 1)
    let range = SelectionRange {
        start: SelectionPoint {
            block: 3,
            offset: 0,
        },
        end: SelectionPoint {
            block: 3,
            offset: 500,
        },
    };

    let selected = build_selected_text(&blocks, range);
    assert!(
        selected.is_some(),
        "selected text should not be None for nested list item"
    );
    let text = selected.unwrap();
    assert!(text.contains("cấu trúc thuật toán ANN"));
    assert!(text.contains("⇒"));
}

#[test]
fn test_smooth_scroll_step_and_start() {
    let mut state = crate::core::MarkdownState {
        total_content_height: 2000.0,
        viewport_height: 800.0,
        ..Default::default()
    };

    // Test start_smooth_scroll clamping
    start_smooth_scroll(&mut state, 3000.0);
    assert_eq!(state.smooth_scroll.target_y, 1200.0);
    assert!(state.smooth_scroll.is_animating);

    // When starting navigation to -50.0 (clamped to 0.0) with non-zero scroll_y:
    state.scroll_y = 500.0;
    start_smooth_scroll(&mut state, -50.0);
    assert_eq!(state.smooth_scroll.target_y, 0.0);
    assert!(state.smooth_scroll.is_animating);

    // When target distance is below threshold, animation is skipped
    state.scroll_y = 0.0;
    start_smooth_scroll(&mut state, 0.0);
    assert!(!state.smooth_scroll.is_animating);
}

#[test]
fn test_smooth_scroll_reaches_target() {
    let mut scroller = crate::core::scroll::SmoothScroller::default();
    let mut current: f32 = 0.0;
    let target: f32 = 500.0;
    let max_y: f32 = 1000.0;
    scroller.start_navigation(current, target, max_y);

    let start = std::time::Instant::now();
    for step in 1..=50 {
        let now = start + std::time::Duration::from_millis(step * 16);
        if let Some(next_y) = scroller.tick(current, now, max_y) {
            current = next_y;
        } else {
            break;
        }
    }
    assert!(
        (current - target).abs() < 0.1,
        "Scroller should reach target {target}, got {current}"
    );
    assert!(!scroller.is_animating);
}

#[test]
fn test_handle_smooth_wheel_scrolled_lines_and_pixels() {
    use crate::app::test_util::{markdown_content, test_app};

    let mut app = test_app(Some(markdown_content("# Heading\n\nContent paragraph")));
    app.state.markdown.viewport_height = 800.0;
    app.state.markdown.total_content_height = 3000.0;
    app.state.markdown.scroll_y = 100.0;

    // Lines: 1 notch down (-1.0) -> step = 1.0 * (800 * 0.15) = 120.0 px
    let delta_lines = iced::mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 };
    let _ = handle_smooth_wheel_scrolled(&mut app, delta_lines);
    assert_eq!(app.state.markdown.smooth_scroll.target_y, 220.0);
    assert!(app.state.markdown.smooth_scroll.is_animating);

    // Pixels: trackpad touch events produce direct displacement via scroll_controller
    let delta_pixels = iced::mouse::ScrollDelta::Pixels { x: 0.0, y: -20.0 };
    let _ = handle_smooth_wheel_scrolled(&mut app, delta_pixels);
    assert_eq!(app.state.markdown.scroll_y, 150.0);
    assert_eq!(
        app.state.markdown.scroll_controller.state(),
        crate::core::scroll::GestureState::Dragging
    );

    // Second fast swipe in rapid succession
    std::thread::sleep(std::time::Duration::from_millis(15));
    let delta_pixels_fast = iced::mouse::ScrollDelta::Pixels { x: 0.0, y: -30.0 };
    let _ = handle_smooth_wheel_scrolled(&mut app, delta_pixels_fast);
    assert_eq!(app.state.markdown.scroll_y, 225.0);
    assert_eq!(
        app.state.markdown.scroll_controller.state(),
        crate::core::scroll::GestureState::Dragging
    );

    // Opportunistic instant fling on zero delta
    let delta_pixels_zero = iced::mouse::ScrollDelta::Pixels { x: 0.0, y: 0.0 };
    let _ = handle_smooth_wheel_scrolled(&mut app, delta_pixels_zero);
    assert!(app.state.markdown.scroll_controller.is_animating());
    assert_eq!(
        app.state.markdown.scroll_controller.state(),
        crate::core::scroll::GestureState::Flinging
    );

    // Ctrl held down: ignores wheel scrolling
    app.ctrl_held = true;
    let _ = handle_smooth_wheel_scrolled(&mut app, delta_lines);
}

#[test]
fn test_markdown_touchpad_hold_finger_does_not_fling() {
    use crate::app::test_util::{markdown_content, test_app};

    let mut app = test_app(Some(markdown_content("# Heading\n\nContent paragraph")));
    app.state.markdown.viewport_height = 800.0;
    app.state.markdown.total_content_height = 3000.0;
    app.state.markdown.scroll_y = 100.0;

    let delta_pixels = iced::mouse::ScrollDelta::Pixels { x: 0.0, y: -20.0 };
    let _ = handle_smooth_wheel_scrolled(&mut app, delta_pixels);
    assert_eq!(app.state.markdown.scroll_y, 150.0);
    assert_eq!(
        app.state.markdown.scroll_controller.state(),
        crate::core::scroll::GestureState::Dragging
    );

    // User keeps finger held on touchpad (45ms pass without new events)
    let now = std::time::Instant::now() + std::time::Duration::from_millis(45);
    let _ = handle_smooth_scroll_tick(&mut app, now);

    assert_eq!(
        app.state.markdown.scroll_controller.state(),
        crate::core::scroll::GestureState::Idle
    );
    assert!(!app.state.markdown.scroll_controller.is_animating());
    assert_eq!(app.state.markdown.scroll_y, 150.0);
}

#[test]
fn test_markdown_scrolled_preserves_animation_and_handles_manual_scroll() {
    use crate::app::test_util::{markdown_content, test_app};

    let mut app = test_app(Some(markdown_content("# Heading\n\nContent paragraph")));
    app.state.markdown.smooth_scroll.is_animating = true;
    app.state.markdown.smooth_scroll.target_y = 1000.0;
    app.state.markdown.smooth_scroll.last_applied_y = 100.0;
    app.state.markdown.scroll_y = 100.0;
    app.state.markdown.viewport_height = 800.0;
    app.state.markdown.total_content_height = 3000.0;

    // While animating, asynchronous on_scroll does not clobber target_y or scroll_y
    let _ = handle_markdown_scrolled(&mut app, 250.0, 800.0, 3000.0);
    assert_eq!(app.state.markdown.scroll_y, 100.0);
    assert_eq!(app.state.markdown.smooth_scroll.target_y, 1000.0);
    assert!(app.state.markdown.smooth_scroll.is_animating);

    // When not animating, on_scroll (e.g. manual scrollbar drag) updates scroll_y
    app.state.markdown.smooth_scroll.is_animating = false;
    let _ = handle_markdown_scrolled(&mut app, 400.0, 800.0, 3000.0);
    assert_eq!(app.state.markdown.scroll_y, 400.0);
    assert_eq!(app.state.markdown.smooth_scroll.target_y, 400.0);
    assert!(!app.state.markdown.smooth_scroll.is_animating);
}

#[test]
fn test_smooth_wheel_scroll_reaches_exact_bottom_without_overshoot() {
    use crate::app::test_util::{markdown_content, test_app};

    let mut app = test_app(Some(markdown_content("# Heading\n\nContent paragraph")));
    app.state.markdown.viewport_height = 800.0;
    app.state.markdown.total_content_height = 2000.0;
    app.state.markdown.scroll_y = 1100.0;

    let max_y = crate::core::scroll::max_scroll_y(
        app.state.markdown.total_content_height,
        app.state.markdown.viewport_height,
    );
    assert_eq!(max_y, 1200.0);

    // Scroll down 2 notches: 2 * (800 * 0.15) = 240px -> target = 1100 + 240 = 1340, clamped to 1200.0
    let delta_lines = iced::mouse::ScrollDelta::Lines { x: 0.0, y: -2.0 };
    let _ = handle_smooth_wheel_scrolled(&mut app, delta_lines);
    assert_eq!(app.state.markdown.smooth_scroll.target_y, 1200.0);

    // Tick to completion
    let start = std::time::Instant::now();
    for step in 1..=30 {
        let now = start + std::time::Duration::from_millis(step * 16);
        let _ = handle_smooth_scroll_tick(&mut app, now);
        if !app.state.markdown.smooth_scroll.is_animating {
            break;
        }
    }
    assert_eq!(app.state.markdown.scroll_y, 1200.0);
    assert!(!app.state.markdown.smooth_scroll.is_animating);
}

#[test]
fn test_markdown_scrolled_does_not_corrupt_virtualized_height() {
    use crate::app::test_util::{markdown_content, test_app};

    let mut app = test_app(Some(markdown_content("# Heading\n\nContent paragraph")));
    // Simulate a virtualized document with 100 blocks
    app.state.markdown.block_y_offsets = vec![0.0; 100];
    app.state.markdown.viewport_height = 800.0;
    app.state.markdown.total_content_height = 10_000.0;
    app.state.markdown.scroll_y = 9_500.0; // Near bottom
    app.state.markdown.smooth_scroll.is_animating = false;

    // Even though scroll_y is near bottom, virtualized document must NOT clobber total_content_height
    let _ = handle_markdown_scrolled(&mut app, 9_500.0, 800.0, 10_250.0);
    assert_eq!(app.state.markdown.total_content_height, 10_000.0);

    // For non-virtualized document (e.g. 10 blocks), total_content_height IS updated from exact bounds
    app.state.markdown.block_y_offsets = vec![0.0; 10];
    let _ = handle_markdown_scrolled(&mut app, 100.0, 800.0, 1_500.0);
    assert_eq!(app.state.markdown.total_content_height, 1_500.0);
}

#[test]
fn test_shift_wheel_scroll_resizes_page_width_lines() {
    use crate::app::test_util::{markdown_content, test_app};

    let mut app = test_app(Some(markdown_content(
        "# Heading\n\nContent paragraph with lots of words for testing.",
    )));
    app.state.max_text_width = Some(820.0);
    app.shift_held = true;

    // 1. Scroll Up (y = 1.0) -> Increases width by PAGE_WIDTH_STEP (40px) -> 860px
    let delta_up = iced::mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 };
    let _ = handle_smooth_wheel_scrolled(&mut app, delta_up);
    assert_eq!(app.state.max_text_width, Some(860.0));
    assert!(
        app.state
            .toasts
            .iter()
            .any(|t| t.message == "Page Width: 860px")
    );

    // 2. Scroll Down (y = -2.0) -> Decreases width by 80px -> 780px
    let delta_down = iced::mouse::ScrollDelta::Lines { x: 0.0, y: -2.0 };
    let _ = handle_smooth_wheel_scrolled(&mut app, delta_down);
    assert_eq!(app.state.max_text_width, Some(780.0));
    assert!(
        app.state
            .toasts
            .iter()
            .any(|t| t.message == "Page Width: 780px")
    );

    // 3. Excessive downward scroll clamps to MIN_PAGE_WIDTH (400px)
    let delta_excess_down = iced::mouse::ScrollDelta::Lines { x: 0.0, y: -100.0 };
    let _ = handle_smooth_wheel_scrolled(&mut app, delta_excess_down);
    assert_eq!(app.state.max_text_width, Some(MIN_PAGE_WIDTH));

    // 4. Excessive upward scroll clamps to MAX_PAGE_WIDTH (2400px)
    let delta_excess_up = iced::mouse::ScrollDelta::Lines { x: 0.0, y: 100.0 };
    let _ = handle_smooth_wheel_scrolled(&mut app, delta_excess_up);
    assert_eq!(app.state.max_text_width, Some(MAX_PAGE_WIDTH));
}

#[test]
fn test_shift_wheel_scroll_horizontal_axis_fallback() {
    use crate::app::test_util::{markdown_content, test_app};

    let mut app = test_app(Some(markdown_content("# Heading\n\nContent paragraph.")));
    app.state.max_text_width = Some(820.0);
    app.shift_held = true;

    // When compositor reports horizontal delta (x = 1.0, y = 0.0) with Shift held:
    let delta_x_up = iced::mouse::ScrollDelta::Lines { x: 1.0, y: 0.0 };
    let _ = handle_smooth_wheel_scrolled(&mut app, delta_x_up);
    assert_eq!(app.state.max_text_width, Some(860.0));

    let delta_x_down = iced::mouse::ScrollDelta::Lines { x: -1.0, y: 0.0 };
    let _ = handle_smooth_wheel_scrolled(&mut app, delta_x_down);
    assert_eq!(app.state.max_text_width, Some(820.0));
}

#[test]
fn test_shift_wheel_scroll_touchpad_pixels() {
    use crate::app::test_util::{markdown_content, test_app};

    let mut app = test_app(Some(markdown_content("# Heading\n\nContent paragraph.")));
    app.state.max_text_width = Some(820.0);
    app.shift_held = true;

    // Touchpad pixel scrolling: 10px * 1.5 = +15px
    let delta_pixel = iced::mouse::ScrollDelta::Pixels { x: 0.0, y: 10.0 };
    let _ = handle_smooth_wheel_scrolled(&mut app, delta_pixel);
    assert_eq!(app.state.max_text_width, Some(835.0));
}

#[test]
fn test_shift_wheel_scroll_stops_in_flight_vertical_animation() {
    use crate::app::test_util::{markdown_content, test_app};

    let mut app = test_app(Some(markdown_content("# Heading\n\nContent paragraph.")));
    app.state.markdown.smooth_scroll.is_animating = true;
    app.state.markdown.smooth_scroll.target_y = 1000.0;
    app.state.markdown.scroll_y = 200.0;
    app.state.max_text_width = Some(820.0);
    app.shift_held = true;

    let delta = iced::mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 };
    let _ = handle_smooth_wheel_scrolled(&mut app, delta);

    // Smooth vertical scroll must be stopped in place and re-anchored
    assert!(!app.state.markdown.smooth_scroll.is_animating);
    assert_eq!(
        app.state.markdown.smooth_scroll.target_y,
        app.state.markdown.scroll_y
    );
    assert_eq!(app.state.max_text_width, Some(860.0));
}

#[test]
fn test_shift_zero_resets_page_width() {
    use crate::app::test_util::{markdown_content, test_app};

    let mut app = test_app(Some(markdown_content("# Heading\n\nContent paragraph.")));
    app.state.max_text_width = Some(1400.0);

    let key = iced::keyboard::Key::Character("0".into());
    let modifiers = iced::keyboard::Modifiers::SHIFT;
    let _ = app.handle_key_pressed(key, modifiers);

    assert_eq!(app.state.max_text_width, Some(DEFAULT_PAGE_WIDTH));
    assert!(
        app.state
            .toasts
            .iter()
            .any(|t| t.message == "Page Width: 820px")
    );
}

#[test]
fn test_epub_page_width_resize_anchoring() {
    use crate::app::test_util::test_app;
    use crate::core::types::EpubChapterInfo;

    let mut app = test_app(None);
    let blocks = vec![
        Block::Heading {
            level: 1,
            content: vec![],
        },
        Block::Paragraph(vec![]),
    ];
    let chapters = vec![EpubChapterInfo {
        title: "Chapter 1".into(),
        level: 1,
        anchor: None,
        file_href: "ch1.xhtml".into(),
        blocks,
    }];
    app.state.epub.chapters = chapters.clone();
    app.state.epub.active_chapter = 0;
    app.current_content = Some(crate::core::PreviewData::Epub {
        title: "Test Book".into(),
        author: "Author".into(),
        chapters,
        active_chapter: 0,
        images: std::collections::HashMap::new(),
    });
    app.state.max_text_width = Some(800.0);
    app.shift_held = true;

    let delta = iced::mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 };
    let _ = handle_smooth_wheel_scrolled(&mut app, delta);
    assert_eq!(app.state.max_text_width, Some(840.0));
}

#[test]
fn test_codeblock_selection_multiline() {
    let src = "```rust\nuse iced::font::Weight;\npub enum Foo {\n    Bar,\n}\n```";
    let blocks = parse_to_blocks(src);
    assert_eq!(blocks.len(), 1);
    assert!(matches!(blocks[0], Block::CodeBlock { .. }));

    let code_str = "use iced::font::Weight;\npub enum Foo {\n    Bar,\n}";
    let target = "pub enum Foo {\n    Bar,\n}";
    let start_offset = code_str.find(target).expect("Must find target");
    let end_offset = start_offset + target.len();

    let range = SelectionRange {
        start: SelectionPoint {
            block: 1,
            offset: start_offset,
        },
        end: SelectionPoint {
            block: 1,
            offset: end_offset,
        },
    };

    let selected = build_selected_text(&blocks, range);
    assert!(selected.is_some());
    assert_eq!(selected.unwrap(), target);
}

#[test]
fn test_double_click_selection_lifecycle_not_cleared_by_mouse_pressed() {
    use crate::app::test_util::test_app;

    let mut app = test_app(None);
    let src = "Hello world from Kglance";
    let blocks = parse_to_blocks(src);
    app.current_content = Some(crate::core::PreviewData::Markdown {
        raw_text: src.to_string(),
        blocks,
    });

    // 1. Double click triggers drag start and drag update with word bounds (e.g. "world" at 6..11)
    let _ = handle_selection_drag_start(&mut app, 0, 6);
    let _ = handle_selection_drag_update(&mut app, 0, 11);
    let _ = handle_selection_drag_end(&mut app);

    assert_eq!(app.state.markdown.selected_text.as_deref(), Some("world"));
    assert_eq!(
        app.state.markdown.selection_range,
        Some(SelectionRange {
            start: SelectionPoint {
                block: 0,
                offset: 6
            },
            end: SelectionPoint {
                block: 0,
                offset: 11
            },
        })
    );

    // 2. Global MousePressed subscription fires in the same click cycle.
    // selection_started_in_press must protect the selection from being cleared.
    let _ = crate::app::update::misc::handle_mouse_pressed(&mut app, 0.0, 0.0);

    assert_eq!(app.state.markdown.selected_text.as_deref(), Some("world"));
    assert_eq!(
        app.state.markdown.selection_range,
        Some(SelectionRange {
            start: SelectionPoint {
                block: 0,
                offset: 6
            },
            end: SelectionPoint {
                block: 0,
                offset: 11
            },
        })
    );
}

#[test]
fn test_triple_click_paragraph_lifecycle() {
    use crate::app::test_util::test_app;

    let mut app = test_app(None);
    let src = "Hello world from Kglance";
    let blocks = parse_to_blocks(src);
    let block_len = src.len();
    app.current_content = Some(crate::core::PreviewData::Markdown {
        raw_text: src.to_string(),
        blocks,
    });

    // Triple click selects entire paragraph block (0..24)
    let _ = handle_selection_drag_start(&mut app, 0, 0);
    let _ = handle_selection_drag_update(&mut app, 0, block_len);
    let _ = handle_selection_drag_end(&mut app);

    assert_eq!(
        app.state.markdown.selected_text.as_deref(),
        Some("Hello world from Kglance")
    );
    assert_eq!(
        app.state.markdown.selection_range,
        Some(SelectionRange {
            start: SelectionPoint {
                block: 0,
                offset: 0
            },
            end: SelectionPoint {
                block: 0,
                offset: block_len
            },
        })
    );

    // Global MousePressed must not clear selection
    let _ = crate::app::update::misc::handle_mouse_pressed(&mut app, 0.0, 0.0);

    assert_eq!(
        app.state.markdown.selected_text.as_deref(),
        Some("Hello world from Kglance")
    );
    assert_eq!(
        app.state.markdown.selection_range,
        Some(SelectionRange {
            start: SelectionPoint {
                block: 0,
                offset: 0
            },
            end: SelectionPoint {
                block: 0,
                offset: block_len
            },
        })
    );
}

#[test]
fn test_mouse_pressed_outside_clears_selection() {
    use crate::app::test_util::test_app;

    let mut app = test_app(None);
    let src = "Hello world";
    let blocks = parse_to_blocks(src);
    app.current_content = Some(crate::core::PreviewData::Markdown {
        raw_text: src.to_string(),
        blocks,
    });

    app.state.markdown.selection_range = Some(SelectionRange {
        start: SelectionPoint {
            block: 0,
            offset: 0,
        },
        end: SelectionPoint {
            block: 0,
            offset: 5,
        },
    });
    app.state.markdown.selected_text = Some("Hello".to_string());
    app.state.markdown.selection_started_in_press = false;

    // Mouse pressed outside text selection
    let _ = crate::app::update::misc::handle_mouse_pressed(&mut app, 0.0, 0.0);

    assert_eq!(app.state.markdown.selection_range, None);
    assert_eq!(app.state.markdown.selected_text, None);
}

#[test]
fn test_single_click_without_drag_clears_selection_on_end() {
    use crate::app::test_util::test_app;

    let mut app = test_app(None);
    let src = "Hello world";
    let blocks = parse_to_blocks(src);
    app.current_content = Some(crate::core::PreviewData::Markdown {
        raw_text: src.to_string(),
        blocks,
    });

    // Single click at offset 5 without dragging: start == end
    let _ = handle_selection_drag_start(&mut app, 0, 5);
    let _ = handle_selection_drag_end(&mut app);

    assert_eq!(app.state.markdown.selection_range, None);
    assert_eq!(app.state.markdown.selected_text, None);
}

#[test]
fn test_epub_double_click_selection_lifecycle() {
    use crate::app::test_util::test_app;
    use crate::core::types::EpubChapterInfo;

    let mut app = test_app(None);
    let src = "EPUB chapter text content";
    let blocks = parse_to_blocks(src);
    let chapters = vec![EpubChapterInfo {
        title: "Chapter 1".into(),
        level: 1,
        anchor: None,
        file_href: "ch1.xhtml".into(),
        blocks,
    }];
    app.state.epub.chapters = chapters.clone();
    app.state.epub.active_chapter = 0;
    app.current_content = Some(crate::core::PreviewData::Epub {
        title: "Test EPUB".into(),
        author: "Author".into(),
        chapters,
        active_chapter: 0,
        images: std::collections::HashMap::new(),
    });

    // Double-click word "chapter" at 5..12
    let _ = handle_selection_drag_start(&mut app, 0, 5);
    let _ = handle_selection_drag_update(&mut app, 0, 12);
    let _ = handle_selection_drag_end(&mut app);

    assert_eq!(
        app.state.epub.markdown_state.selected_text.as_deref(),
        Some("chapter")
    );

    let _ = crate::app::update::misc::handle_mouse_pressed(&mut app, 0.0, 0.0);
    assert_eq!(
        app.state.epub.markdown_state.selected_text.as_deref(),
        Some("chapter")
    );
}

#[test]
fn test_cross_block_drag_selection_lifecycle() {
    use crate::app::test_util::test_app;

    let mut app = test_app(None);
    let src = "First paragraph block.\n\nSecond paragraph block.";
    let blocks = parse_to_blocks(src);
    assert_eq!(blocks.len(), 2);
    app.current_content = Some(crate::core::PreviewData::Markdown {
        raw_text: src.to_string(),
        blocks,
    });

    // Start drag in block 0 at offset 6 ("paragraph")
    let _ = handle_selection_drag_start(&mut app, 0, 6);
    assert!(app.state.markdown.is_dragging_selection);

    // Drag into block 1 (base index 1000) at offset 6 ("paragraph")
    let _ = handle_selection_drag_update(&mut app, 1000, 6);
    assert_eq!(
        app.state.markdown.selection_range,
        Some(SelectionRange {
            start: SelectionPoint {
                block: 0,
                offset: 6
            },
            end: SelectionPoint {
                block: 1000,
                offset: 6
            },
        })
    );

    // End drag
    let _ = handle_selection_drag_end(&mut app);
    assert!(!app.state.markdown.is_dragging_selection);
    assert_eq!(
        app.state.markdown.selection_range,
        Some(SelectionRange {
            start: SelectionPoint {
                block: 0,
                offset: 6
            },
            end: SelectionPoint {
                block: 1000,
                offset: 6
            },
        })
    );
    assert!(app.state.markdown.selected_text.is_some());
    let selected = app.state.markdown.selected_text.as_ref().unwrap();
    assert!(selected.contains("paragraph block"));
    assert!(selected.contains("Second"));
}

#[test]
fn test_update_selected_text_preserves_fallback_when_range_is_none() {
    use crate::app::test_util::test_app;

    let mut app = test_app(None);
    app.state.markdown.selected_text = Some("fallback text".to_string());
    app.state.markdown.selection_range = None;

    update_selected_text_from_range(&mut app);

    // Must preserve fallback text when selection_range is None
    assert_eq!(
        app.state.markdown.selected_text.as_deref(),
        Some("fallback text")
    );
}

#[test]
fn test_wheel_scroll_lines_interrupts_active_kinetic_scroll() {
    use crate::app::test_util::test_app;

    let mut app = test_app(None);
    app.state.markdown.viewport_height = 800.0;
    app.state.markdown.total_content_height = 5000.0;
    app.state.markdown.scroll_y = 100.0;

    // 1. Start a kinetic fling
    app.state
        .markdown
        .scroll_controller
        .start_kinetic(100.0, 1500.0, 4200.0);
    assert!(app.state.markdown.scroll_controller.is_animating());

    // 2. User turns the mouse wheel (Lines)
    let delta = iced::mouse::ScrollDelta::Lines { x: 0.0, y: -2.0 };
    let _ = handle_smooth_wheel_scrolled(&mut app, delta);

    // Active kinetic fling must be immediately cancelled
    assert!(!app.state.markdown.scroll_controller.is_animating());
    assert!(app.state.markdown.smooth_scroll.is_animating);

    // 3. Frame tick must advance smooth scroll towards wheel target, NOT kinetic position
    let now = std::time::Instant::now();
    let _ = handle_smooth_scroll_tick(&mut app, now);
    assert!(!app.state.markdown.scroll_controller.is_animating());
    assert!(app.state.markdown.scroll_y > 100.0);
}

#[test]
fn test_toc_heading_click_interrupts_active_kinetic_scroll() {
    use crate::app::test_util::test_app;

    let mut app = test_app(None);
    app.state.markdown.viewport_height = 800.0;
    app.state.markdown.total_content_height = 6000.0;
    app.state.markdown.scroll_y = 200.0;
    app.state.markdown.block_y_offsets = vec![0.0, 1500.0, 3000.0];

    // 1. Start a kinetic fling
    app.state
        .markdown
        .scroll_controller
        .start_kinetic(200.0, 2500.0, 5200.0);
    assert!(app.state.markdown.scroll_controller.is_animating());

    // 2. User clicks a TOC heading
    let _ = handle_toc_heading_clicked(&mut app, 1);

    // Active kinetic fling must be stopped
    assert!(!app.state.markdown.scroll_controller.is_animating());
    assert!(app.state.markdown.smooth_scroll.is_animating);
    assert_eq!(
        app.state.markdown.smooth_scroll.target_y(),
        crate::parsers::markdown::layout_constants::CONTENT_PADDING + 1500.0
    );

    // 3. Frame tick must advance towards TOC target without snapping back
    let now = std::time::Instant::now();
    let _ = handle_smooth_scroll_tick(&mut app, now);
    assert!(!app.state.markdown.scroll_controller.is_animating());
}

#[test]
fn test_handle_markdown_scrolled_interruption_stops_active_kinetic_scroll() {
    use crate::app::test_util::test_app;

    let mut app = test_app(None);
    app.state.markdown.viewport_height = 800.0;
    app.state.markdown.total_content_height = 6000.0;
    app.state.markdown.scroll_y = 200.0;

    // 1. Start a kinetic fling
    app.state
        .markdown
        .scroll_controller
        .start_kinetic(200.0, 2000.0, 5200.0);
    assert!(app.state.markdown.scroll_controller.is_animating());

    // 2. User drags the scrollbar to 2500.0
    let _ = handle_markdown_scrolled(&mut app, 2500.0, 800.0, 6000.0);

    // Active kinetic fling must be interrupted and stopped at 2500.0
    assert!(!app.state.markdown.scroll_controller.is_animating());
    assert_eq!(app.state.markdown.scroll_y, 2500.0);

    // Next tick must not jump back
    let now = std::time::Instant::now();
    let _ = handle_smooth_scroll_tick(&mut app, now);
    assert_eq!(app.state.markdown.scroll_y, 2500.0);
}
