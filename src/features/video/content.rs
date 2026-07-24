use crate::app::Message;
use crate::core::preview::{ContentType, PreviewContent};
use crate::core::types::KglanceState;
use iced::widget::{container, image, text};
use iced::{Element, Length};

pub struct MediaContent {
    pub path: String,
    pub duration: f64,
    pub thumbnail: Vec<u8>,
    pub metadata: String,
    pub waveform: Vec<u8>,
    pub waveform_width: u32,
    pub waveform_height: u32,
    pub is_video: bool,
}

impl PreviewContent<Message> for MediaContent {
    fn populate_state(&self, state: &mut KglanceState) {
        state.media = crate::core::MediaState::default();
        state.media.metadata = self.metadata.clone();
        state.media.has_video = self.is_video;
        state.file_type_text = if self.is_video {
            "Video File".to_string()
        } else {
            "Audio File".to_string()
        };
    }

    fn view<'a>(&'a self, state: &'a KglanceState) -> Element<'a, Message> {
        let display: Element<'a, Message> = if !self.thumbnail.is_empty() {
            image(image::Handle::from_bytes(self.thumbnail.clone()))
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(iced::ContentFit::Contain)
                .into()
        } else if !state.media.metadata.is_empty() {
            text(&state.media.metadata).size(14).into()
        } else {
            text(if self.is_video { "Video" } else { "Audio" })
                .size(14)
                .into()
        };

        container(display)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(0)
            .into()
    }

    fn content_type(&self) -> ContentType {
        if self.is_video {
            ContentType::Video
        } else {
            ContentType::Audio
        }
    }

    fn is_media(&self) -> bool {
        true
    }

    fn needs_media_player(&self) -> bool {
        true
    }
}
