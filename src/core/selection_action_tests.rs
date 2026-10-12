use super::*;

#[test]
fn test_default_selection_menu_config() {
    let cfg = SelectionMenuConfig::default();
    assert!(cfg.enabled);
    assert!(cfg.custom_actions.is_empty());
}

#[test]
fn test_deserialize_selection_menu_config() {
    let json = r#"{
        "enabled": true,
        "custom_actions": [
            {
                "name": "Translate",
                "command": "trans -b :vi",
                "popup": true
            },
            {
                "name": "Add Note",
                "command": "cat >> ~/notes.txt",
                "toast": true
            },
            {
                "name": "AI Explain",
                "command": "ollama run llama3",
                "popup": true,
                "stream": true
            }
        ]
    }"#;

    let cfg: SelectionMenuConfig = serde_json::from_str(json).expect("valid json");
    assert!(cfg.enabled);
    assert_eq!(cfg.custom_actions.len(), 3);
    assert_eq!(cfg.custom_actions[0].name, "Translate");
    assert_eq!(cfg.custom_actions[0].command, "trans -b :vi");
    assert!(cfg.custom_actions[0].popup);
    assert!(!cfg.custom_actions[0].toast);
    assert!(!cfg.custom_actions[0].stream);

    assert_eq!(cfg.custom_actions[1].name, "Add Note");
    assert_eq!(cfg.custom_actions[1].command, "cat >> ~/notes.txt");
    assert!(!cfg.custom_actions[1].popup);
    assert!(cfg.custom_actions[1].toast);
    assert!(!cfg.custom_actions[1].stream);

    assert_eq!(cfg.custom_actions[2].name, "AI Explain");
    assert_eq!(cfg.custom_actions[2].command, "ollama run llama3");
    assert!(cfg.custom_actions[2].popup);
    assert!(!cfg.custom_actions[2].toast);
    assert!(cfg.custom_actions[2].stream);
}

#[test]
fn test_selection_menu_state_transitions() {
    let mut state = SelectionMenuState::Hidden;
    assert!(!state.is_visible());
    assert_eq!(state.position(), None);
    assert_eq!(state.selected_text(), None);

    let pos = iced::Point::new(150.0, 200.0);
    state = SelectionMenuState::Toolbar {
        pos,
        selected_text: "hello world".to_string(),
    };
    assert!(state.is_visible());
    assert_eq!(state.position(), Some(pos));
    assert_eq!(state.selected_text(), Some("hello world"));

    state = SelectionMenuState::Running {
        pos,
        selected_text: "hello world".to_string(),
        action_name: "Translate".to_string(),
    };
    assert!(state.is_visible());
    assert_eq!(state.position(), Some(pos));

    state = SelectionMenuState::Result {
        pos,
        action_name: "Translate".to_string(),
        content: "xin chào thế giới".to_string(),
        blocks: Vec::new(),
        is_streaming: false,
    };
    assert!(state.is_visible());
    assert_eq!(state.position(), Some(pos));
    assert_eq!(state.selected_text(), None);

    state.hide();
    assert!(!state.is_visible());
}

#[tokio::test]
async fn test_execute_custom_action_stdin_and_env() {
    let cmd = "tr '[:lower:]' '[:upper:]'".to_string();
    let text = "hello rust".to_string();
    let res = execute_custom_action(cmd, text).await.expect("execute");
    assert_eq!(res.trim(), "HELLO RUST");

    let cmd_env = "echo \"PREFIX_$KGLANCE_TEXT\"".to_string();
    let text_env = "kglance".to_string();
    let res_env = execute_custom_action(cmd_env, text_env)
        .await
        .expect("execute env");
    assert_eq!(res_env.trim(), "PREFIX_kglance");
}

#[tokio::test]
async fn test_execute_custom_action_failure() {
    let cmd = "exit 1".to_string();
    let text = "test".to_string();
    let res = execute_custom_action(cmd, text).await;
    assert!(res.is_err());
}

#[test]
fn test_toolbar_dynamic_bounds_and_cursor_inside() {
    let actions = vec![
        CustomAction {
            name: "📖 Dict (Loading)".to_string(),
            command: "".to_string(),
            popup: true,
            toast: false,
            stream: false,
        },
        CustomAction {
            name: "🤖 AI Stream".to_string(),
            command: "".to_string(),
            popup: true,
            toast: false,
            stream: true,
        },
        CustomAction {
            name: "⚡ Explain Code".to_string(),
            command: "".to_string(),
            popup: true,
            toast: false,
            stream: false,
        },
        CustomAction {
            name: "📌 Lưu Ghi Chú".to_string(),
            command: "".to_string(),
            popup: false,
            toast: true,
            stream: false,
        },
    ];

    let w_empty = estimate_toolbar_width(&[]);
    assert!(w_empty >= 60.0);

    let w_custom = estimate_toolbar_width(&actions);
    assert!(w_custom > 450.0);

    let pos = iced::Point::new(100.0, 200.0);
    let state = SelectionMenuState::Toolbar {
        pos,
        selected_text: "test".to_string(),
    };

    let bounds = state.bounds(1920.0, 1080.0, &actions).expect("bounds");
    assert!(bounds.width >= w_custom);

    // Point located at x = 500.0 (inside the wide toolbar, but outside a 320px box)
    let pt_far_right = iced::Point::new(500.0, bounds.y + 10.0);
    assert!(state.is_cursor_inside(Some(pt_far_right), 1920.0, 1080.0, &actions));

    // Empty actions: point at x = 500.0 should NOT be inside
    assert!(!state.is_cursor_inside(Some(pt_far_right), 1920.0, 1080.0, &[]));
}

#[test]
fn test_bounds_clamping_matches_toolbar_height() {
    // When pos.y is 93.0, with TOOLBAR_HEIGHT = 36.0:
    // 93.0 - 36.0 - 12.0 = 45.0 >= 44.0 -> placed ABOVE cursor (y = 45.0)
    let state = SelectionMenuState::Toolbar {
        pos: iced::Point::new(100.0, 93.0),
        selected_text: "test".to_string(),
    };
    let bounds = state.bounds(1920.0, 1080.0, &[]).expect("bounds");
    // With 4px padding expansion, y should be around 41.0
    assert_eq!(bounds.y, 45.0 - 4.0);
    assert_eq!(bounds.height, TOOLBAR_HEIGHT + 8.0);

    // Cursor at toolbar visual center (x = 100.0, y = 45.0 + 18.0 = 63.0) is inside
    assert!(state.is_cursor_inside(Some(iced::Point::new(100.0, 63.0)), 1920.0, 1080.0, &[]));
}
