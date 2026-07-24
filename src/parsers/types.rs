use crate::core::error::ParseError;
use crate::core::preview::PreviewContent;
use std::path::Path;

pub trait PreviewParser<Message>: Send + Sync {
    fn supported_extensions(&self) -> &[&str];
    fn is_supported(&self, path: &Path) -> bool;
    fn parse(&self, path: &Path) -> Result<Box<dyn PreviewContent<Message>>, ParseError>;
}
