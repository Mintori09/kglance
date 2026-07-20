use std::sync::Arc;
use tokio::sync::mpsc;

use zbus::interface;

use crate::core::preview::{FilePreviewer, PreviewData};
use crate::parsers::{ParseError, ParserRegistry};
use crate::{log_error, log_info};

pub enum DaemonCommand {
    ShowPreview { path: String, content: PreviewData },
    HidePreview,
}

pub struct DaemonService {
    parser_registry: Arc<ParserRegistry>,
    tx: mpsc::Sender<DaemonCommand>,
}

impl DaemonService {
    pub fn new(parser_registry: Arc<ParserRegistry>, tx: mpsc::Sender<DaemonCommand>) -> Self {
        Self {
            parser_registry,
            tx,
        }
    }
}

#[interface(name = "org.mintori.Kglance")]
impl DaemonService {
    async fn show_preview(&mut self, file_path: &str) -> zbus::fdo::Result<()> {
        log_info!(
            "DaemonService: show_preview request received for path: {}",
            file_path
        );
        let path = std::path::Path::new(file_path);
        let content = FilePreviewer::parse(&*self.parser_registry, path).map_err(|e| {
            log_error!("DaemonService: Failed to parse path {}: {:?}", file_path, e);
            match e {
                ParseError::FileNotFound => zbus::fdo::Error::Failed("File not found".into()),
                ParseError::PermissionDenied => {
                    zbus::fdo::Error::Failed("Permission denied".into())
                }
                ParseError::UnsupportedFormat => {
                    zbus::fdo::Error::Failed("Unsupported file format".into())
                }
                ParseError::TooLarge => zbus::fdo::Error::Failed("File too large".into()),
                ParseError::ParseFailed(msg) => zbus::fdo::Error::Failed(msg),
            }
        })?;

        log_info!(
            "DaemonService: Sending ShowPreview event for: {}",
            file_path
        );
        self.tx
            .send(DaemonCommand::ShowPreview {
                path: file_path.to_string(),
                content,
            })
            .await
            .map_err(|err| {
                log_error!(
                    "DaemonService: Failed to send ShowPreview to channels: {:?}",
                    err
                );
                zbus::fdo::Error::Failed("Internal error".into())
            })?;
        Ok(())
    }

    async fn hide_preview(&mut self) -> zbus::fdo::Result<()> {
        log_info!("DaemonService: hide_preview request received");
        self.tx
            .send(DaemonCommand::HidePreview)
            .await
            .map_err(|err| {
                log_error!(
                    "DaemonService: Failed to send HidePreview to channels: {:?}",
                    err
                );
                zbus::fdo::Error::Failed("Internal error".into())
            })?;
        Ok(())
    }
}
