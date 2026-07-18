use std::sync::{mpsc, Arc};

use zbus::interface;

use crate::parser::{ParseError, ParserRegistry};

pub enum DaemonCommand {
    ShowPreview {
        path: String,
        content: crate::parser::ParsedContent,
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

#[interface(name = "org.mintori.KiviewRust")]
impl DaemonService {
    async fn show_preview(&mut self, file_path: &str) -> zbus::fdo::Result<()> {
        let path = std::path::Path::new(file_path);
        let content = self.parser_registry.parse(path).map_err(|e| match e {
            ParseError::FileNotFound => {
                zbus::fdo::Error::Failed("File not found".into())
            }
            ParseError::PermissionDenied => {
                zbus::fdo::Error::Failed("Permission denied".into())
            }
            ParseError::UnsupportedFormat => {
                zbus::fdo::Error::Failed("Unsupported file format".into())
            }
            ParseError::TooLarge => zbus::fdo::Error::Failed("File too large".into()),
            ParseError::ParseFailed(msg) => zbus::fdo::Error::Failed(msg),
        })?;
        self.tx
            .send(DaemonCommand::ShowPreview {
                path: file_path.to_string(),
                content,
            })
            .map_err(|_| zbus::fdo::Error::Failed("Internal error".into()))?;
        Ok(())
    }

    async fn hide_preview(&mut self) -> zbus::fdo::Result<()> {
        self.tx
            .send(DaemonCommand::HidePreview)
            .map_err(|_| zbus::fdo::Error::Failed("Internal error".into()))?;
        Ok(())
    }
}
