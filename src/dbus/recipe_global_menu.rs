use crate::app::Message;
use iced_futures::subscription::{self, Recipe};
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

pub struct GlobalMenuRecipe {
    rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<String>>>>,
}

impl GlobalMenuRecipe {
    pub fn new(rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<String>>>>) -> Self {
        Self { rx }
    }
}

impl Recipe for GlobalMenuRecipe {
    type Output = Message;

    fn hash(&self, state: &mut subscription::Hasher) {
        "global_menu_subscription".hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: subscription::EventStream,
    ) -> iced_futures::BoxStream<Self::Output> {
        let rx_arc = self.rx.clone();
        iced_futures::boxed_stream(iced::stream::channel(
            100,
            move |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
                use iced::futures::SinkExt;
                let mut rx_opt = rx_arc.lock().await;
                if let Some(mut rx) = rx_opt.take() {
                    drop(rx_opt);
                    while let Some(action) = rx.recv().await {
                        let _ = output
                            .send(
                                crate::app::messages::GlobalMenuMsg::ActionTriggered(action).into(),
                            )
                            .await;
                    }
                }
            },
        ))
    }
}
