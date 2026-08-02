use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;

use zbus::interface;

use crate::core::preview::FilePreviewer;
use crate::features::common::parser::traits::{ParseError, ParserRegistry};
use crate::{log_debug, log_error, log_info};

fn to_fdo_error(e: ParseError) -> zbus::fdo::Error {
    match e {
        ParseError::FileNotFound => zbus::fdo::Error::Failed("File not found".into()),
        ParseError::PermissionDenied => zbus::fdo::Error::Failed("Permission denied".into()),
        ParseError::UnsupportedFormat => zbus::fdo::Error::Failed("Unsupported file format".into()),
        ParseError::TooLarge => zbus::fdo::Error::Failed("File too large".into()),
        ParseError::ParseFailed(msg) => zbus::fdo::Error::Failed(msg),
    }
}

/// Commands sent from the DBus service thread to the Iced event loop.
pub enum DaemonCommand {
    /// Instructs the UI to open (or un-hide) the window with content already parsed.
    /// Sending a single merged event avoids two separate Iced subscription poll cycles.
    OpenWindowWithContent {
        path: String,
        content: crate::core::preview::PreviewData,
    },
    /// Open window with content and a pre-populated playlist for navigation.
    OpenWindowWithPlaylist {
        path: String,
        content: crate::core::preview::PreviewData,
        playlist: Vec<String>,
    },
    /// Re-show an already-open window for the same content path without re-parsing.
    ShowPreviewExisting {
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
        let t0 = Instant::now();
        log_info!(
            "DaemonService: show_preview request received for path: {}",
            file_path
        );

        let path = file_path.to_string();
        let p = std::path::Path::new(file_path);

        // Parse first (826µs for markdown — negligible).
        // Then send a single merged event so Iced only needs ONE subscription poll cycle
        // instead of two (OpenWindow + ShowPreview).
        let content = FilePreviewer::parse(&*self.parser_registry, p).map_err(|e| {
            log_error!("DaemonService: Failed to parse path {}: {:?}", path, e);
            to_fdo_error(e)
        })?;

        log_debug!(
            "[PERF] parse completed in {:?}, sending OpenWindowWithContent",
            t0.elapsed()
        );

        self.tx
            .send(DaemonCommand::OpenWindowWithContent { path, content })
            .await
            .map_err(|err| {
                log_error!(
                    "DaemonService: Failed to send OpenWindowWithContent: {:?}",
                    err
                );
                zbus::fdo::Error::Failed("Internal error".into())
            })?;

        log_info!(
            "[PERF] show_preview total daemon-side latency: {:?}",
            t0.elapsed()
        );
        Ok(())
    }

    async fn show_multiple_previews(&mut self, file_paths: Vec<String>) -> zbus::fdo::Result<()> {
        if file_paths.is_empty() {
            return Err(zbus::fdo::Error::Failed("No files provided".into()));
        }

        let primary = &file_paths[0];
        let t0 = Instant::now();
        log_info!(
            "DaemonService: show_multiple_previews for {} files, primary: {}",
            file_paths.len(),
            primary
        );

        let p = std::path::Path::new(primary);
        let content = FilePreviewer::parse(&*self.parser_registry, p).map_err(|e| {
            log_error!("DaemonService: Failed to parse path {}: {:?}", primary, e);
            to_fdo_error(e)
        })?;

        let path = primary.clone();
        let playlist: Vec<String> = file_paths;

        self.tx
            .send(DaemonCommand::OpenWindowWithPlaylist {
                path,
                content,
                playlist,
            })
            .await
            .map_err(|err| {
                log_error!(
                    "DaemonService: Failed to send OpenWindowWithPlaylist: {:?}",
                    err
                );
                zbus::fdo::Error::Failed("Internal error".into())
            })?;

        log_info!(
            "[PERF] show_multiple_previews total daemon-side latency: {:?}",
            t0.elapsed()
        );
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
