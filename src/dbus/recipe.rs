use tokio::sync::mpsc;

use iced_futures::subscription::{self, Recipe};
use std::hash::Hash;

use crate::app::Message;
use crate::dbus::DaemonCommand;

pub struct DaemonRecipe {
    rx: Option<mpsc::Receiver<DaemonCommand>>,
}

impl DaemonRecipe {
    pub fn new(rx: Option<mpsc::Receiver<DaemonCommand>>) -> Self {
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
        let rx = self.rx;
        match rx {
            Some(mut rx) => iced_futures::boxed_stream(iced::stream::channel(
                100,
                move |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
                    use iced::futures::SinkExt;
                    while let Some(cmd) = rx.recv().await {
                        match cmd {
                            DaemonCommand::OpenWindow { path } => {
                                let _ = output.send(Message::DaemonOpenWindow { path }).await;
                            }
                            DaemonCommand::ShowPreview { path, content } => {
                                let _ = output.send(Message::FileLoaded { path, content }).await;
                            }
                            DaemonCommand::HidePreview => {
                                let _ = output.send(Message::CloseRequested).await;
                            }
                        }
                    }
                },
            )),
            None => iced_futures::boxed_stream(iced::futures::stream::empty()),
        }
    }
}
