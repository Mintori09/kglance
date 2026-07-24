use std::hash::Hash;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::app::Message;
use crate::features::video::VideoEvent;
use iced_futures::subscription::{self, Recipe};

pub struct VideoRecipe {
    rx: Arc<Mutex<Option<mpsc::Receiver<VideoEvent>>>>,
}

impl VideoRecipe {
    pub fn new(rx: Arc<Mutex<Option<mpsc::Receiver<VideoEvent>>>>) -> Self {
        Self { rx }
    }
}

impl Recipe for VideoRecipe {
    type Output = Message;

    fn hash(&self, state: &mut subscription::Hasher) {
        "video_subscription".hash(state);
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
                    while let Some(event) = rx.recv().await {
                        if output
                            .send(Message::VideoEventReceived(event))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                },
            )),
            None => iced_futures::boxed_stream(iced::futures::stream::empty()),
        }
    }
}
