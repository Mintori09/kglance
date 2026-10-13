use super::*;
use crate::app::test_util::test_app;
use crate::core::selection_action::{CustomAction, SelectionMenuState};

#[test]
fn test_show_when_enabled_and_non_empty() {
    let mut app = test_app(None);
    app.state.selection_menu_config.enabled = true;
    assert!(matches!(
        app.state.selection_menu,
        SelectionMenuState::Hidden
    ));

    let pos = iced::Point::new(200.0, 300.0);
    let _ = handle_selection_menu_msg(
        &mut app,
        SelectionMenuMsg::Show {
            pos,
            text: "Selected text".to_string(),
        },
    );

    match &app.state.selection_menu {
        SelectionMenuState::Toolbar {
            pos: p,
            selected_text,
        } => {
            assert_eq!(*p, pos);
            assert_eq!(selected_text, "Selected text");
        }
        _ => panic!("Expected Toolbar state"),
    }
}

#[test]
fn test_show_ignored_when_disabled_or_empty() {
    let mut app = test_app(None);
    app.state.selection_menu_config.enabled = false;

    let pos = iced::Point::new(200.0, 300.0);
    let _ = handle_selection_menu_msg(
        &mut app,
        SelectionMenuMsg::Show {
            pos,
            text: "Selected text".to_string(),
        },
    );
    assert!(matches!(
        app.state.selection_menu,
        SelectionMenuState::Hidden
    ));

    // Enabled but empty/whitespace text
    app.state.selection_menu_config.enabled = true;
    let _ = handle_selection_menu_msg(
        &mut app,
        SelectionMenuMsg::Show {
            pos,
            text: "   \n\t  ".to_string(),
        },
    );
    assert!(matches!(
        app.state.selection_menu,
        SelectionMenuState::Hidden
    ));
}

#[test]
fn test_dismiss_hides_menu() {
    let mut app = test_app(None);
    app.state.selection_menu = SelectionMenuState::Toolbar {
        pos: iced::Point::new(10.0, 10.0),
        selected_text: "Text".to_string(),
    };

    let _ = handle_selection_menu_msg(&mut app, SelectionMenuMsg::Dismiss);
    assert!(matches!(
        app.state.selection_menu,
        SelectionMenuState::Hidden
    ));
}

#[test]
fn test_dismiss_hides_menu_even_when_busy() {
    let mut app = test_app(None);
    app.state.selection_menu = SelectionMenuState::Running {
        pos: iced::Point::new(10.0, 10.0),
        selected_text: "Text".to_string(),
        action_name: "Action".to_string(),
    };
    let _ = handle_selection_menu_msg(&mut app, SelectionMenuMsg::Dismiss);
    assert!(matches!(
        app.state.selection_menu,
        SelectionMenuState::Hidden
    ));

    app.state.selection_menu = SelectionMenuState::Result {
        pos: iced::Point::new(10.0, 10.0),
        action_name: "Action".to_string(),
        content: "streaming".to_string(),
        blocks: Vec::new(),
        is_streaming: true,
    };
    let _ = handle_selection_menu_msg(&mut app, SelectionMenuMsg::Dismiss);
    assert!(matches!(
        app.state.selection_menu,
        SelectionMenuState::Hidden
    ));
}

#[test]
fn test_custom_action_clicked_sets_running() {
    let mut app = test_app(None);
    let pos = iced::Point::new(50.0, 80.0);
    app.state.selection_menu = SelectionMenuState::Toolbar {
        pos,
        selected_text: "Word".to_string(),
    };

    let action = CustomAction {
        name: "Save Note".to_string(),
        command: "cat >> notes.txt".to_string(),
        enabled: true,
        popup: false,
        toast: true,
        stream: false,
    };

    let _ = handle_selection_menu_msg(
        &mut app,
        SelectionMenuMsg::CustomActionClicked {
            action,
            text: "Word".to_string(),
        },
    );

    match &app.state.selection_menu {
        SelectionMenuState::Running {
            pos: p,
            selected_text,
            action_name,
        } => {
            assert_eq!(*p, pos);
            assert_eq!(selected_text, "Word");
            assert_eq!(action_name, "Save Note");
        }
        _ => panic!("Expected Running state"),
    }
}

#[test]
fn test_custom_action_clicked_non_stream_popup_starts_result_loading() {
    let mut app = test_app(None);
    let pos = iced::Point::new(50.0, 80.0);
    app.state.selection_menu = SelectionMenuState::Toolbar {
        pos,
        selected_text: "Word".to_string(),
    };

    let action = CustomAction {
        name: "Translate".to_string(),
        command: "trans -b :vi".to_string(),
        enabled: true,
        popup: true,
        toast: false,
        stream: false,
    };

    let _ = handle_selection_menu_msg(
        &mut app,
        SelectionMenuMsg::CustomActionClicked {
            action,
            text: "Word".to_string(),
        },
    );

    match &app.state.selection_menu {
        SelectionMenuState::Result {
            pos: p,
            action_name,
            content,
            is_streaming,
            ..
        } => {
            assert_eq!(*p, pos);
            assert_eq!(action_name, "Translate");
            assert!(content.is_empty());
            assert!(*is_streaming);
        }
        _ => panic!("Expected Result state with is_streaming: true"),
    }
}

#[test]
fn test_custom_action_clicked_stream_starts_result_streaming() {
    let mut app = test_app(None);
    let pos = iced::Point::new(50.0, 80.0);
    app.state.selection_menu = SelectionMenuState::Toolbar {
        pos,
        selected_text: "Word".to_string(),
    };

    let action = CustomAction {
        name: "AI Stream".to_string(),
        command: "ollama run llama3".to_string(),
        enabled: true,
        popup: true,
        toast: false,
        stream: true,
    };

    let _ = handle_selection_menu_msg(
        &mut app,
        SelectionMenuMsg::CustomActionClicked {
            action,
            text: "Word".to_string(),
        },
    );

    match &app.state.selection_menu {
        SelectionMenuState::Result {
            pos: p,
            action_name,
            content,
            is_streaming,
            ..
        } => {
            assert_eq!(*p, pos);
            assert_eq!(action_name, "AI Stream");
            assert!(content.is_empty());
            assert!(*is_streaming);
        }
        _ => panic!("Expected Result state with is_streaming: true"),
    }
}

#[test]
fn test_action_result_popup() {
    let mut app = test_app(None);
    let pos = iced::Point::new(50.0, 80.0);
    app.state.selection_menu = SelectionMenuState::Running {
        pos,
        selected_text: "Word".to_string(),
        action_name: "Translate".to_string(),
    };

    let _ = handle_selection_menu_msg(
        &mut app,
        SelectionMenuMsg::ActionResult {
            action_name: "Translate".to_string(),
            result: Ok("**Từ** (danh từ)".to_string()),
            popup: true,
            toast: false,
        },
    );

    match &app.state.selection_menu {
        SelectionMenuState::Result {
            pos: p,
            action_name,
            content,
            blocks,
            is_streaming,
        } => {
            assert_eq!(*p, pos);
            assert_eq!(action_name, "Translate");
            assert_eq!(content, "**Từ** (danh từ)");
            assert!(!blocks.is_empty());
            assert!(!*is_streaming);
        }
        _ => panic!("Expected Result state with parsed blocks"),
    }
}

#[test]
fn test_stream_chunks_and_finish() {
    let mut app = test_app(None);
    let pos = iced::Point::new(60.0, 90.0);
    app.state.selection_menu = SelectionMenuState::Result {
        pos,
        action_name: "AI Stream".to_string(),
        content: String::new(),
        blocks: Vec::new(),
        is_streaming: true,
    };

    // Receive first chunk
    let _ = handle_selection_menu_msg(
        &mut app,
        SelectionMenuMsg::StreamChunk {
            action_name: "AI Stream".to_string(),
            chunk: "Hello ".to_string(),
        },
    );

    match &app.state.selection_menu {
        SelectionMenuState::Result {
            content,
            blocks,
            is_streaming,
            ..
        } => {
            assert_eq!(content, "Hello ");
            assert!(blocks.is_empty());
            assert!(*is_streaming);
        }
        _ => panic!("Expected Result state streaming"),
    }

    // Receive second chunk
    let _ = handle_selection_menu_msg(
        &mut app,
        SelectionMenuMsg::StreamChunk {
            action_name: "AI Stream".to_string(),
            chunk: "world!".to_string(),
        },
    );

    match &app.state.selection_menu {
        SelectionMenuState::Result {
            content,
            blocks,
            is_streaming,
            ..
        } => {
            assert_eq!(content, "Hello world!");
            assert!(blocks.is_empty());
            assert!(*is_streaming);
        }
        _ => panic!("Expected Result state streaming"),
    }

    // Finish stream
    let _ = handle_selection_menu_msg(
        &mut app,
        SelectionMenuMsg::StreamFinished {
            action_name: "AI Stream".to_string(),
            toast: false,
        },
    );

    match &app.state.selection_menu {
        SelectionMenuState::Result {
            content,
            blocks,
            is_streaming,
            ..
        } => {
            assert_eq!(content, "Hello world!");
            assert!(!blocks.is_empty());
            assert!(!*is_streaming);
        }
        _ => panic!("Expected Result state not streaming"),
    }
}

#[test]
fn test_stream_finished_empty_output_fallback() {
    let mut app = test_app(None);
    let pos = iced::Point::new(60.0, 90.0);
    app.state.selection_menu = SelectionMenuState::Result {
        pos,
        action_name: "Empty Action".to_string(),
        content: String::new(),
        blocks: Vec::new(),
        is_streaming: true,
    };

    let _ = handle_selection_menu_msg(
        &mut app,
        SelectionMenuMsg::StreamFinished {
            action_name: "Empty Action".to_string(),
            toast: false,
        },
    );

    match &app.state.selection_menu {
        SelectionMenuState::Result {
            content,
            blocks,
            is_streaming,
            ..
        } => {
            assert_eq!(content, "(No output returned)");
            assert!(!blocks.is_empty());
            assert!(!*is_streaming);
        }
        _ => panic!("Expected Result state with fallback"),
    }
}

#[test]
fn test_stream_failed_empty_content() {
    let mut app = test_app(None);
    let pos = iced::Point::new(60.0, 90.0);
    app.state.selection_menu = SelectionMenuState::Result {
        pos,
        action_name: "Failed Action".to_string(),
        content: String::new(),
        blocks: Vec::new(),
        is_streaming: true,
    };

    let _ = handle_selection_menu_msg(
        &mut app,
        SelectionMenuMsg::StreamFailed {
            action_name: "Failed Action".to_string(),
            error: "API key missing".to_string(),
            toast: false,
        },
    );

    match &app.state.selection_menu {
        SelectionMenuState::Result {
            content,
            blocks,
            is_streaming,
            ..
        } => {
            assert_eq!(content, "**Error:** API key missing");
            assert!(!blocks.is_empty());
            assert!(!*is_streaming);
        }
        _ => panic!("Expected Result state with error"),
    }
}

#[test]
fn test_stream_failed_with_partial_chunks() {
    let mut app = test_app(None);
    let pos = iced::Point::new(60.0, 90.0);
    app.state.selection_menu = SelectionMenuState::Result {
        pos,
        action_name: "Partial Action".to_string(),
        content: String::new(),
        blocks: Vec::new(),
        is_streaming: true,
    };

    let _ = handle_selection_menu_msg(
        &mut app,
        SelectionMenuMsg::StreamChunk {
            action_name: "Partial Action".to_string(),
            chunk: "Partial output".to_string(),
        },
    );

    let _ = handle_selection_menu_msg(
        &mut app,
        SelectionMenuMsg::StreamFailed {
            action_name: "Partial Action".to_string(),
            error: "Connection lost".to_string(),
            toast: false,
        },
    );

    match &app.state.selection_menu {
        SelectionMenuState::Result {
            content,
            blocks,
            is_streaming,
            ..
        } => {
            assert_eq!(content, "Partial output\n\n**Error:** Connection lost");
            assert!(!blocks.is_empty());
            assert!(!*is_streaming);
        }
        _ => panic!("Expected Result state with partial content and error"),
    }
}

#[test]
fn test_stream_finished_and_failed_with_toast() {
    let mut app = test_app(None);
    let pos = iced::Point::new(60.0, 90.0);
    app.state.selection_menu = SelectionMenuState::Result {
        pos,
        action_name: "Toast Action".to_string(),
        content: "Done".to_string(),
        blocks: Vec::new(),
        is_streaming: true,
    };

    let _ = handle_selection_menu_msg(
        &mut app,
        SelectionMenuMsg::StreamFinished {
            action_name: "Toast Action".to_string(),
            toast: true,
        },
    );
    assert_eq!(app.state.toasts.len(), 1);
    assert_eq!(app.state.toasts[0].message, "Toast Action completed");

    let _ = handle_selection_menu_msg(
        &mut app,
        SelectionMenuMsg::StreamFailed {
            action_name: "Toast Action".to_string(),
            error: "Bad request".to_string(),
            toast: true,
        },
    );
    assert_eq!(app.state.toasts.len(), 2);
    assert_eq!(
        app.state.toasts[1].message,
        "Toast Action error: Bad request"
    );
}

#[test]
fn test_action_result_toast() {
    let mut app = test_app(None);
    app.state.selection_menu = SelectionMenuState::Running {
        pos: iced::Point::new(50.0, 80.0),
        selected_text: "Note content".to_string(),
        action_name: "Add Note".to_string(),
    };

    let _ = handle_selection_menu_msg(
        &mut app,
        SelectionMenuMsg::ActionResult {
            action_name: "Add Note".to_string(),
            result: Ok("".to_string()),
            popup: false,
            toast: true,
        },
    );

    assert!(matches!(
        app.state.selection_menu,
        SelectionMenuState::Hidden
    ));
    assert!(!app.state.toasts.is_empty());
    assert!(
        app.state
            .toasts
            .last()
            .unwrap()
            .message
            .contains("Add Note completed")
    );
}

#[test]
fn test_action_result_error() {
    let mut app = test_app(None);
    app.state.selection_menu = SelectionMenuState::Running {
        pos: iced::Point::new(50.0, 80.0),
        selected_text: "Test".to_string(),
        action_name: "Failing".to_string(),
    };

    let _ = handle_selection_menu_msg(
        &mut app,
        SelectionMenuMsg::ActionResult {
            action_name: "Failing".to_string(),
            result: Err("exit code 1".to_string()),
            popup: true,
            toast: false,
        },
    );

    assert!(matches!(
        app.state.selection_menu,
        SelectionMenuState::Hidden
    ));
    assert!(!app.state.toasts.is_empty());
    assert!(
        app.state
            .toasts
            .last()
            .unwrap()
            .message
            .contains("Failing error")
    );
}

#[test]
fn test_split_utf8_boundary() {
    use super::split_utf8_boundary;

    // Normal ASCII
    let (chunk, rem) = split_utf8_boundary(b"hello world");
    assert_eq!(chunk, "hello world");
    assert_eq!(rem, b"");

    // Multi-byte UTF-8 split across boundary
    // '€' is 3 bytes: 0xE2, 0x82, 0xAC
    let bytes = [b'a', 0xE2, 0x82];
    let (chunk, rem) = split_utf8_boundary(&bytes);
    assert_eq!(chunk, "a");
    assert_eq!(rem, &[0xE2, 0x82]);

    // Combining with the remaining byte
    let mut combined = rem.to_vec();
    combined.push(0xAC);
    let (chunk, rem) = split_utf8_boundary(&combined);
    assert_eq!(chunk, "€");
    assert_eq!(rem, b"");
}
