mod registry;
pub mod types;
pub use self::types::PreviewParser;
pub use registry::*;

pub use crate::core::error::ParseError;

// Re-export parsers from features (backward compatibility)
pub use crate::features::archive::parser as archive;
pub use crate::features::audio::parser as audio;
pub use crate::features::document::csv;
pub use crate::features::document::epub;
pub use crate::features::document::folder;
pub use crate::features::document::office;
pub use crate::features::document::text;
pub use crate::features::image::parser as image;
pub use crate::features::markdown::parser as markdown;
pub use crate::features::pdf::parser as pdf;
pub use crate::features::vector::font;
pub use crate::features::vector::svg;
pub use crate::features::video::parser as video;
