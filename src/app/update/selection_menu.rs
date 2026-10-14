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
            clear_document_selection(app);
            Task::none()
        }
        SelectionMenuMsg::CopyClicked(text) => {
            app.state.selection_menu.hide();
            clear_document_selection(app);
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

            let file_path = if app.state.file_name.is_empty() {
                None
            } else {
                Some(std::path::PathBuf::from(&app.state.file_name))
            };

            // If popup is requested (streaming or non-streaming), open popup card immediately with loading state
            if popup {
                app.state.selection_menu = SelectionMenuState::Result {
                    pos,
                    action_name: action_name.clone(),
                    content: String::new(),
                    blocks: Vec::new(),
                    is_streaming: true,
                    selected_text: None,
                    selection_range: None,
                    is_dragging: false,
                    auto_scroll_delta: None,
                };

                if action.stream {
                    stream_custom_action(
                        action_name,
                        action.command,
                        text,
                        file_path.as_deref(),
                        toast,
                    )
                } else {
                    Task::perform(
                        async move {
                            let result =
                                execute_custom_action(action.command, text, file_path.as_deref())
                                    .await;
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
                        let result =
                            execute_custom_action(action.command, text, file_path.as_deref()).await;
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
        SelectionMenuMsg::StreamFinished { action_name, toast } => {
            if let SelectionMenuState::Result {
                action_name: cur_name,
                content,
                blocks,
                is_streaming,
                selection_range,
                selected_text,
                is_dragging,
                auto_scroll_delta,
                ..
            } = &mut app.state.selection_menu
                && cur_name == &action_name
            {
                if content.trim().is_empty() {
                    content.push_str("(No output returned)");
                }
                let (parsed_blocks, _) = crate::features::markdown::parser::parse_markdown(
                    content,
                    std::path::Path::new(""),
                );
                *blocks = parsed_blocks;
                *is_streaming = false;
                *selection_range = None;
                *selected_text = None;
                *is_dragging = false;
                *auto_scroll_delta = None;
            }
            if toast {
                app.show_toast(format!("{action_name} completed"))
            } else {
                Task::none()
            }
        }
        SelectionMenuMsg::StreamFailed {
            action_name,
            error,
            toast,
        } => {
            if let SelectionMenuState::Result {
                action_name: cur_name,
                content,
                blocks,
                is_streaming,
                selection_range,
                selected_text,
                is_dragging,
                auto_scroll_delta,
                ..
            } = &mut app.state.selection_menu
                && cur_name == &action_name
            {
                if content.trim().is_empty() {
                    *content = format!("**Error:** {error}");
                } else {
                    use std::fmt::Write as _;
                    let _ = write!(content, "\n\n**Error:** {error}");
                }
                let (parsed_blocks, _) = crate::features::markdown::parser::parse_markdown(
                    content,
                    std::path::Path::new(""),
                );
                *blocks = parsed_blocks;
                *is_streaming = false;
                *selection_range = None;
                *selected_text = None;
                *is_dragging = false;
                *auto_scroll_delta = None;
            }
            if toast {
                app.show_toast(format!("{action_name} error: {error}"))
            } else {
                Task::none()
            }
        }
        SelectionMenuMsg::ActionResult {
            action_name,
            result,
            popup,
            toast,
        } => match result {
            Err(err) => {
                app.state.selection_menu.hide();
                if !popup {
                    clear_document_selection(app);
                }
                app.show_toast(format!("{action_name} error: {err}"))
            }
            Ok(output) => {
                if toast && !popup {
                    app.state.selection_menu.hide();
                    clear_document_selection(app);
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
                            selected_text: None,
                            selection_range: None,
                            is_dragging: false,
                            auto_scroll_delta: None,
                        };
                        toast_task
                    }
                } else {
                    app.state.selection_menu.hide();
                    clear_document_selection(app);
                    Task::none()
                }
            }
        },
        SelectionMenuMsg::ResultSelectionChanged(selected) => {
            if let SelectionMenuState::Result { selected_text, .. } = &mut app.state.selection_menu
            {
                *selected_text = selected;
            }
            Task::none()
        }
        SelectionMenuMsg::ResultDragStart { block, offset } => {
            if let SelectionMenuState::Result {
                selection_range,
                is_dragging,
                selected_text,
                ..
            } = &mut app.state.selection_menu
            {
                let pt = crate::core::SelectionPoint { block, offset };
                *selection_range = Some(crate::core::SelectionRange { start: pt, end: pt });
                *is_dragging = true;
                *selected_text = None;
            }
            Task::none()
        }
        SelectionMenuMsg::ResultDragUpdate { block, offset } => {
            if let SelectionMenuState::Result {
                content,
                blocks,
                selection_range: Some(range),
                is_dragging,
                selected_text,
                ..
            } = &mut app.state.selection_menu
            {
                range.end = crate::core::SelectionPoint { block, offset };
                *is_dragging = true;
                *selected_text = sync_result_selected_text(content, blocks, Some(*range));
            }
            Task::none()
        }
        SelectionMenuMsg::ResultDragEnd => {
            if let SelectionMenuState::Result {
                content,
                blocks,
                selection_range,
                is_dragging,
                selected_text,
                auto_scroll_delta,
                ..
            } = &mut app.state.selection_menu
            {
                *is_dragging = false;
                *auto_scroll_delta = None;
                if let Some(range) = *selection_range {
                    if range.start != range.end {
                        *selected_text = sync_result_selected_text(content, blocks, Some(range));
                    } else {
                        *selection_range = None;
                    }
                    if selected_text.is_none() {
                        *selection_range = None;
                    }
                }
            }
            Task::none()
        }
        SelectionMenuMsg::ResultSelectionClear => {
            if let SelectionMenuState::Result {
                selection_range,
                is_dragging,
                selected_text,
                auto_scroll_delta,
                ..
            } = &mut app.state.selection_menu
            {
                *selection_range = None;
                *is_dragging = false;
                *selected_text = None;
                *auto_scroll_delta = None;
            }
            Task::none()
        }
        SelectionMenuMsg::ResultAutoScrollTick => {
            let Some(delta) = (match &app.state.selection_menu {
                SelectionMenuState::Result {
                    auto_scroll_delta, ..
                } => *auto_scroll_delta,
                _ => None,
            }) else {
                return Task::none();
            };

            iced::widget::operation::scroll_by(
                "selection_result_scroll",
                iced::widget::operation::AbsoluteOffset { x: 0.0, y: delta },
            )
        }
    }
}

pub(crate) fn sync_result_selected_text(
    content: &str,
    blocks: &[crate::parsers::markdown::Block],
    selection_range: Option<crate::core::SelectionRange>,
) -> Option<String> {
    let range = selection_range?;
    if range.start == range.end {
        return None;
    }
    if blocks.is_empty() {
        let min_off = range.start.offset.min(range.end.offset);
        let max_off = range.start.offset.max(range.end.offset);
        let start = crate::features::markdown::update::char_boundary(content, min_off);
        let end = crate::features::markdown::update::char_boundary(content, max_off);
        if start < end && end <= content.len() {
            Some(content[start..end].to_string())
        } else {
            None
        }
    } else {
        crate::features::markdown::update::build_selected_text(blocks, range)
    }
}

pub fn stream_custom_action(
    action_name: String,
    command: String,
    text: String,
    file_path: Option<&std::path::Path>,
    toast: bool,
) -> Task<Message> {
    use iced::futures::SinkExt;
    use iced::futures::channel::mpsc::Sender;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let file_path = file_path.map(std::path::Path::to_path_buf);

    let stream = iced::stream::channel(100, move |mut output: Sender<Message>| async move {
        let spawn_res = crate::core::selection_action::prepare_custom_action_command(
            &command,
            &text,
            file_path.as_deref(),
        )
        .spawn();

        let mut child = match spawn_res {
            Ok(child) => child,
            Err(e) => {
                let _ = output
                    .send(Message::SelectionMenu(SelectionMenuMsg::StreamFailed {
                        action_name,
                        error: format!("Failed to spawn command: {e}"),
                        toast,
                    }))
                    .await;
                return;
            }
        };

        let stdin = child.stdin.take();
        let stdin_task = async move {
            if let Some(mut stdin) = stdin {
                let _ = stdin.write_all(text.as_bytes()).await;
                let _ = stdin.flush().await;
                drop(stdin);
            }
        };

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let mut out_sender = output.clone();
        let act_name = action_name.clone();

        let stdout_task = async move {
            let mut total_chunks = 0usize;
            if let Some(mut reader) = stdout {
                let mut buf = [0u8; 1024];
                let mut carry = Vec::new();
                loop {
                    match reader.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => {
                            let (chunk, rem) = if carry.is_empty() {
                                let (c, r) = split_utf8_boundary(&buf[..n]);
                                (c.to_string(), r.to_vec())
                            } else {
                                carry.extend_from_slice(&buf[..n]);
                                let (c, r) = split_utf8_boundary(&carry);
                                (c.to_string(), r.to_vec())
                            };
                            if !chunk.is_empty() {
                                total_chunks += 1;
                                let _ = out_sender
                                    .send(Message::SelectionMenu(SelectionMenuMsg::StreamChunk {
                                        action_name: act_name.clone(),
                                        chunk,
                                    }))
                                    .await;
                            }
                            carry = rem;
                        }
                        Err(_) => break,
                    }
                }
                if !carry.is_empty() {
                    let chunk = String::from_utf8_lossy(&carry).into_owned();
                    total_chunks += 1;
                    let _ = out_sender
                        .send(Message::SelectionMenu(SelectionMenuMsg::StreamChunk {
                            action_name: act_name.clone(),
                            chunk,
                        }))
                        .await;
                }
            }
            total_chunks
        };

        let stderr_task = async move {
            let mut err_buf = Vec::new();
            if let Some(mut reader) = stderr {
                let _ = reader.read_to_end(&mut err_buf).await;
            }
            String::from_utf8_lossy(&err_buf).trim().to_string()
        };

        let timeout_secs = crate::core::selection_action::CUSTOM_ACTION_TIMEOUT_SECS;
        let io_res = tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), async {
            tokio::join!(stdin_task, stdout_task, stderr_task)
        })
        .await;

        let (total_chunks, stderr_str) = match io_res {
            Ok((_, chunks, err_text)) => (chunks, err_text),
            Err(_) => {
                let _ = child.kill().await;
                let _ = output
                    .send(Message::SelectionMenu(SelectionMenuMsg::StreamFailed {
                        action_name,
                        error: format!("Command timed out after {timeout_secs} seconds"),
                        toast,
                    }))
                    .await;
                return;
            }
        };

        let exit_status =
            tokio::time::timeout(std::time::Duration::from_secs(5), child.wait()).await;

        let is_err = match &exit_status {
            Ok(Ok(status)) => !status.success(),
            _ => {
                let _ = child.kill().await;
                true
            }
        };

        if is_err {
            let error_msg = if !stderr_str.is_empty() {
                stderr_str
            } else if let Ok(Ok(status)) = exit_status {
                let code = status.code().unwrap_or(-1);
                format!("Process exited with status {code}")
            } else {
                "Process exited abnormally".to_string()
            };

            let _ = output
                .send(Message::SelectionMenu(SelectionMenuMsg::StreamFailed {
                    action_name,
                    error: error_msg,
                    toast,
                }))
                .await;
        } else if total_chunks == 0 && !stderr_str.is_empty() {
            let _ = output
                .send(Message::SelectionMenu(SelectionMenuMsg::StreamFailed {
                    action_name,
                    error: stderr_str,
                    toast,
                }))
                .await;
        } else {
            let _ = output
                .send(Message::SelectionMenu(SelectionMenuMsg::StreamFinished {
                    action_name,
                    toast,
                }))
                .await;
        }
    });

    Task::run(stream, |msg| msg)
}

pub(crate) fn split_utf8_boundary(bytes: &[u8]) -> (&str, &[u8]) {
    match std::str::from_utf8(bytes) {
        Ok(s) => (s, &[]),
        Err(e) => {
            let valid_len = e.valid_up_to();
            if e.error_len().is_none() {
                let valid = std::str::from_utf8(&bytes[..valid_len]).unwrap_or("");
                (valid, &bytes[valid_len..])
            } else {
                let valid = std::str::from_utf8(&bytes[..valid_len]).unwrap_or("");
                (valid, &[])
            }
        }
    }
}

pub(crate) fn clear_document_selection(app: &mut KglanceApp) {
    let s = crate::features::markdown::update::active_markdown_state_mut(app);
    s.selection_range = None;
    s.selected_text = None;
    s.is_dragging_selection = false;
    s.auto_scroll_delta = None;

    app.state.pdf.clear_selection();
    app.state.pdf.auto_scroll_delta = None;

    app.state.text.clear_selection();
    app.state.text.is_dragging_selection = false;
    app.state.text.auto_scroll_delta = None;
    app.state.text.drag_start = None;
}

#[cfg(test)]
#[path = "selection_menu_tests.rs"]
mod tests;
