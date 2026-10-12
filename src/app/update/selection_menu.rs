use iced::Task;

use crate::app::KglanceApp;
use crate::app::messages::{Message, SelectionMenuMsg};
use crate::core::selection_action::{SelectionMenuState, execute_custom_action};

pub fn handle_selection_menu_msg(app: &mut KglanceApp, msg: SelectionMenuMsg) -> Task<Message> {
    match msg {
        SelectionMenuMsg::Show { pos, text } => {
            if app.state.selection_menu_config.enabled && !text.trim().is_empty() {
                app.state.selection_menu = SelectionMenuState::Toolbar {
                    pos,
                    selected_text: text,
                };
            }
            Task::none()
        }
        SelectionMenuMsg::Dismiss => {
            app.state.selection_menu.hide();
            Task::none()
        }
        SelectionMenuMsg::CopyClicked(text) => {
            app.state.selection_menu.hide();
            crate::app::update::misc::handle_copy_text(app, text)
        }
        SelectionMenuMsg::CustomActionClicked { action, text } => {
            let pos = app
                .state
                .selection_menu
                .position()
                .unwrap_or(iced::Point::new(100.0, 100.0));

            let action_name = action.name;
            let popup = action.popup;
            let toast = action.toast;

            // If popup is requested (streaming or non-streaming), open popup card immediately with loading state
            if popup {
                app.state.selection_menu = SelectionMenuState::Result {
                    pos,
                    action_name: action_name.clone(),
                    content: String::new(),
                    blocks: Vec::new(),
                    is_streaming: true,
                };

                if action.stream {
                    stream_custom_action(action_name, action.command, text)
                } else {
                    Task::perform(
                        async move {
                            let result = execute_custom_action(action.command, text).await;
                            SelectionMenuMsg::ActionResult {
                                action_name,
                                result,
                                popup,
                                toast,
                            }
                        },
                        Message::SelectionMenu,
                    )
                }
            } else {
                app.state.selection_menu = SelectionMenuState::Running {
                    pos,
                    selected_text: text.clone(),
                    action_name: action_name.clone(),
                };

                Task::perform(
                    async move {
                        let result = execute_custom_action(action.command, text).await;
                        SelectionMenuMsg::ActionResult {
                            action_name,
                            result,
                            popup,
                            toast,
                        }
                    },
                    Message::SelectionMenu,
                )
            }
        }
        SelectionMenuMsg::StreamChunk { action_name, chunk } => {
            if let SelectionMenuState::Result {
                action_name: cur_name,
                content,
                ..
            } = &mut app.state.selection_menu
                && cur_name == &action_name
            {
                content.push_str(&chunk);
            }
            Task::none()
        }
        SelectionMenuMsg::StreamFinished { action_name } => {
            if let SelectionMenuState::Result {
                action_name: cur_name,
                content,
                blocks,
                is_streaming,
                ..
            } = &mut app.state.selection_menu
                && cur_name == &action_name
            {
                let (parsed_blocks, _) = crate::features::markdown::parser::parse_markdown(
                    content,
                    std::path::Path::new(""),
                );
                *blocks = parsed_blocks;
                *is_streaming = false;
            }
            Task::none()
        }
        SelectionMenuMsg::ActionResult {
            action_name,
            result,
            popup,
            toast,
        } => match result {
            Err(err) => {
                app.state.selection_menu.hide();
                app.show_toast(format!("{action_name} error: {err}"))
            }
            Ok(output) => {
                if toast && !popup {
                    app.state.selection_menu.hide();
                    let msg = if output.trim().is_empty() {
                        format!("{action_name} completed")
                    } else {
                        output.trim().to_string()
                    };
                    app.show_toast(msg)
                } else if popup {
                    let toast_task = if toast {
                        app.show_toast(format!("{action_name} completed"))
                    } else {
                        Task::none()
                    };
                    if output.trim().is_empty() || !app.state.selection_menu.is_visible() {
                        app.state.selection_menu.hide();
                        toast_task
                    } else {
                        let pos = app
                            .state
                            .selection_menu
                            .position()
                            .unwrap_or(iced::Point::new(100.0, 100.0));
                        let (blocks, _) = crate::features::markdown::parser::parse_markdown(
                            &output,
                            std::path::Path::new(""),
                        );
                        app.state.selection_menu = SelectionMenuState::Result {
                            pos,
                            action_name,
                            content: output,
                            blocks,
                            is_streaming: false,
                        };
                        toast_task
                    }
                } else {
                    app.state.selection_menu.hide();
                    Task::none()
                }
            }
        },
    }
}

pub fn stream_custom_action(action_name: String, command: String, text: String) -> Task<Message> {
    use iced::futures::SinkExt;
    use iced::futures::channel::mpsc::Sender;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let stream = iced::stream::channel(100, |mut output: Sender<Message>| async move {
        let env_text = if text.len() > crate::core::selection_action::MAX_ENV_TEXT_BYTES {
            let mut byte_count = 0;
            let mut safe = String::new();
            for ch in text.chars() {
                let next_len = ch.len_utf8();
                if byte_count + next_len > crate::core::selection_action::MAX_ENV_TEXT_BYTES {
                    break;
                }
                safe.push(ch);
                byte_count += next_len;
            }
            safe
        } else {
            text.clone()
        };

        let spawn_res = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&command)
            .env("KGLANCE_TEXT", &env_text)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn();

        let mut child = match spawn_res {
            Ok(child) => child,
            Err(e) => {
                let _ = output
                    .send(Message::SelectionMenu(SelectionMenuMsg::ActionResult {
                        action_name,
                        result: Err(format!("Failed to spawn command: {e}")),
                        popup: true,
                        toast: false,
                    }))
                    .await;
                return;
            }
        };

        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes()).await;
            let _ = stdin.flush().await;
            drop(stdin);
        }

        if let Some(mut stdout) = child.stdout.take() {
            let mut buf = [0u8; 1024];
            let read_stream = async {
                loop {
                    match stdout.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => {
                            let chunk = String::from_utf8_lossy(&buf[..n]).into_owned();
                            let _ = output
                                .send(Message::SelectionMenu(SelectionMenuMsg::StreamChunk {
                                    action_name: action_name.clone(),
                                    chunk,
                                }))
                                .await;
                        }
                        Err(_) => break,
                    }
                }
            };

            let _ = tokio::time::timeout(
                std::time::Duration::from_secs(
                    crate::core::selection_action::CUSTOM_ACTION_TIMEOUT_SECS,
                ),
                read_stream,
            )
            .await;
        }

        let _ = tokio::time::timeout(std::time::Duration::from_secs(5), child.wait()).await;

        let _ = output
            .send(Message::SelectionMenu(SelectionMenuMsg::StreamFinished {
                action_name,
            }))
            .await;
    });

    Task::run(stream, |msg| msg)
}

#[cfg(test)]
#[path = "selection_menu_tests.rs"]
mod tests;
