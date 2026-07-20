use std::sync::Arc;
use tokio::sync::mpsc;

use zbus::interface;

use crate::core::preview::FilePreviewer;
use crate::parsers::{ParseError, ParserRegistry};
use crate::{log_error, log_info};

/// Commands sent from the DBus service thread to the Iced event loop.
pub enum DaemonCommand {
    /// Instructs the UI to open (or un-hide) the window immediately.
    /// The window shows a loading state while parsing continues.
    OpenWindow {
        path: String,
    },
    /// File content is ready — update the UI.
    ShowPreview {
        path: String,
        content: crate::core::preview::PreviewData,
    },
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

        let path = file_path.to_string();

        // Step 1: Signal UI to open the window immediately (loading state).
        // Iced runs on a separate thread, so it processes this message while
        // we block below on parsing — giving the appearance of instant window open.
        self.tx
            .send(DaemonCommand::OpenWindow { path: path.clone() })
            .await
            .map_err(|_| zbus::fdo::Error::Failed("Internal error".into()))?;

        // Step 2: Parse synchronously on this zbus thread.
        // We cannot use tokio::task::spawn_blocking here because zbus uses its
        // own Connection executor, not the Tokio runtime context.
        // Blocking here is safe: Iced's event loop is on a separate thread.
        let p = std::path::Path::new(file_path);
        let content = FilePreviewer::parse(&*self.parser_registry, p).map_err(|e| {
            log_error!("DaemonService: Failed to parse path {}: {:?}", path, e);
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

        // Step 3: Send parsed content to update the already-open window.
        log_info!("DaemonService: Sending ShowPreview event for: {}", path);
        self.tx
            .send(DaemonCommand::ShowPreview { path, content })
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
