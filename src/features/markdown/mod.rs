pub mod content;
pub mod parser;
pub mod syntax;
pub mod types;
pub mod view;

pub use parser::{
    MarkdownParser, extract_toc, flatten_inlines, parse_to_blocks, render_mermaid_to_png,
};
pub use types::{Block, Inline, ListItem, TableBlock, TableCell};
pub use view::view_markdown;
