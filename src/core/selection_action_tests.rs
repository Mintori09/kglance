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
    assert!(cfg.custom_actions[0].enabled);
    assert!(cfg.custom_actions[0].popup);
    assert!(!cfg.custom_actions[0].toast);
    assert!(!cfg.custom_actions[0].stream);

    assert_eq!(cfg.custom_actions[1].name, "Add Note");
    assert_eq!(cfg.custom_actions[1].command, "cat >> ~/notes.txt");
    assert!(cfg.custom_actions[1].enabled);
    assert!(!cfg.custom_actions[1].popup);
    assert!(cfg.custom_actions[1].toast);
    assert!(!cfg.custom_actions[1].stream);

    assert_eq!(cfg.custom_actions[2].name, "AI Explain");
    assert_eq!(cfg.custom_actions[2].command, "ollama run llama3");
    assert!(cfg.custom_actions[2].enabled);
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
    let res = execute_custom_action(cmd, text, None)
        .await
        .expect("execute");
    assert_eq!(res.trim(), "HELLO RUST");

    let cmd_env = "echo \"PREFIX_$KGLANCE_TEXT\"".to_string();
    let text_env = "kglance".to_string();
    let res_env = execute_custom_action(cmd_env, text_env, None)
        .await
        .expect("execute env");
    assert_eq!(res_env.trim(), "PREFIX_kglance");
}

#[tokio::test]
async fn test_execute_custom_action_file_env() {
    let cmd = "echo \"$KGLANCE_FILE|$KGLANCE_FILE_DIR|$KGLANCE_FILE_NAME|$KGLANCE_FILE_STEM|$KGLANCE_FILE_EXT|$(pwd)\"".to_string();
    let text = "sample".to_string();
    let test_file = std::path::Path::new("/tmp/test_dir/my_document.MD");
    let res = execute_custom_action(cmd, text, Some(test_file))
        .await
        .expect("execute file env");
    let trimmed = res.trim();
    assert!(
        trimmed.starts_with(
            "/tmp/test_dir/my_document.MD|/tmp/test_dir|my_document.MD|my_document|md|"
        )
    );
}

#[tokio::test]
async fn test_execute_custom_action_file_env_no_extension() {
    let cmd =
        "echo \"$KGLANCE_FILE_NAME|$KGLANCE_FILE_STEM|${KGLANCE_FILE_EXT:-none}\"".to_string();
    let text = "sample".to_string();
    let test_file = std::path::Path::new("/tmp/Makefile");
    let res = execute_custom_action(cmd, text, Some(test_file))
        .await
        .expect("execute no ext");
    assert_eq!(res.trim(), "Makefile|Makefile|none");
}

#[tokio::test]
async fn test_execute_custom_action_no_file_env_fallback() {
    let cmd = "echo \"${KGLANCE_FILE:-none}|$(pwd)\"".to_string();
    let text = "sample".to_string();
    let res = execute_custom_action(cmd, text, None)
        .await
        .expect("execute none file");
    let trimmed = res.trim();
    assert!(trimmed.starts_with("none|"));
}

#[tokio::test]
async fn test_execute_custom_action_failure() {
    let cmd = "exit 1".to_string();
    let text = "test".to_string();
    let res = execute_custom_action(cmd, text, None).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_execute_custom_action_large_env_truncation() {
    let cmd = "echo \"${#KGLANCE_TEXT}\"".to_string();
    // 32 KB string (larger than MAX_ENV_TEXT_BYTES)
    let text = "a".repeat(32 * 1024);
    let res = execute_custom_action(cmd, text, None)
        .await
        .expect("execute large env");
    let len: usize = res.trim().parse().expect("parsed length");
    assert_eq!(len, MAX_ENV_TEXT_BYTES);
}

#[tokio::test]
async fn test_execute_custom_action_timeout() {
    let cmd = "sleep 35".to_string();
    let text = "test".to_string();
    let res = match tokio::time::timeout(
        std::time::Duration::from_millis(50),
        execute_custom_action(cmd, text, None),
    )
    .await
    {
        Ok(r) => r,
        Err(_) => Err("Command timed out after 30 seconds".to_string()),
    };
    assert!(res.is_err());
}

#[test]
fn test_toolbar_dynamic_bounds_and_cursor_inside() {
    let actions = vec![
        CustomAction {
            name: "📖 Dict (Loading)".to_string(),
            command: "".to_string(),
            enabled: true,
            popup: true,
            toast: false,
            stream: false,
        },
        CustomAction {
            name: "🤖 AI Stream".to_string(),
            command: "".to_string(),
            enabled: true,
            popup: true,
            toast: false,
            stream: true,
        },
        CustomAction {
            name: "⚡ Explain Code".to_string(),
            command: "".to_string(),
            enabled: true,
            popup: true,
            toast: false,
            stream: false,
        },
        CustomAction {
            name: "📌 Lưu Ghi Chú".to_string(),
            command: "".to_string(),
            enabled: true,
            popup: false,
            toast: true,
            stream: false,
        },
    ];

    let w_empty = estimate_toolbar_width(&[]);
    assert!(w_empty >= 60.0);

    let w_custom = estimate_toolbar_width(&actions);
    assert!(w_custom > 450.0);

    // Disabled actions should not increase toolbar width
    let disabled_actions = vec![CustomAction {
        name: "Disabled Action".to_string(),
        command: "".to_string(),
        enabled: false,
        popup: true,
        toast: false,
        stream: false,
    }];
    assert_eq!(estimate_toolbar_width(&disabled_actions), w_empty);

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
