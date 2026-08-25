use tokio::sync::mpsc;

use iced_futures::subscription::{self, Recipe};
use std::hash::Hash;

use crate::app::Message;
use crate::dbus::DaemonCommand;

use std::sync::{Arc, Mutex};

pub struct DaemonRecipe {
    rx: Arc<Mutex<Option<mpsc::Receiver<DaemonCommand>>>>,
}

impl DaemonRecipe {
    pub fn new(rx: Arc<Mutex<Option<mpsc::Receiver<DaemonCommand>>>>) -> Self {
        Self { rx }
    }
}

impl Recipe for DaemonRecipe {
    type Output = Message;

    fn hash(&self, state: &mut subscription::Hasher) {
        "dbus_subscription".hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: subscription::EventStream,
    ) -> iced_futures::BoxStream<Self::Output> {
        let rx = self.rx.lock().unwrap().take();
        match rx {
            Some(mut rx) => iced_futures::boxed_stream(iced::stream::channel(
                100,
                move |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
                    use iced::futures::SinkExt;
                    while let Some(cmd) = rx.recv().await {
                        match cmd {
                            // Single merged event: open window + load content in one Iced cycle.
                            // Route through DaemonOpenWithPlaylist (no window-closed guard) so
                            // handle_file_loaded is always reached — which calls
                            // prepare_window_tasks → create_new_window() when needed.
                            // DaemonUpdateWindow had an early-return guard that silently dropped
                            // content when window_id.is_none(), breaking the first preview.
                            DaemonCommand::OpenWindowWithContent { path, content } => {
                                let playlist = vec![path.clone()];
                                let _ = output
                                    .send(
                                        crate::app::messages::SystemMsg::DaemonOpenWithPlaylist {
                                            path,
                                            content,
                                            playlist,
                                        }
                                        .into(),
                                    )
                                    .await;
                            }
                            // Open window with content + pre-populated playlist.
                            DaemonCommand::OpenWindowWithPlaylist {
                                path,
                                content,
                                playlist,
                            } => {
                                let _ = output
                                    .send(
                                        crate::app::messages::SystemMsg::DaemonOpenWithPlaylist {
                                            path,
                                            content,
                                            playlist,
                                        }
                                        .into(),
                                    )
                                    .await;
                            }
                            DaemonCommand::UpdateWindowWithContent { path, content } => {
                                let _ = output
                                    .send(
                                        crate::app::messages::SystemMsg::DaemonUpdateWindow {
                                            path,
                                            content,
                                        }
                                        .into(),
                                    )
                                    .await;
                            }
                            DaemonCommand::UpdateWindowWithPlaylist {
                                path,
                                content,
                                playlist,
                            } => {
                                let _ = output
                                    .send(
                                        crate::app::messages::SystemMsg::DaemonUpdateWithPlaylist {
                                            path,
                                            content,
                                            playlist,
                                        }
                                        .into(),
                                    )
                                    .await;
                            }
                            // Kept for future use (e.g. reloading without window re-open).
                            // Also routed through DaemonUpdateWindow to avoid stale-gen discard.
                            DaemonCommand::ShowPreviewExisting { path, content } => {
                                let _ = output
                                    .send(
                                        crate::app::messages::SystemMsg::DaemonUpdateWindow {
                                            path,
                                            content,
                                        }
                                        .into(),
                                    )
                                    .await;
                            }
                            DaemonCommand::HidePreview => {
                                let _ = output
                                    .send(crate::app::messages::ActionMsg::CloseRequested.into())
                                    .await;
                            }
                        }
                    }
                },
            )),
            None => iced_futures::boxed_stream(iced::futures::stream::empty()),
        }
    }
}
