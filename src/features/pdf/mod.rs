pub mod cache;
pub mod compress;
pub mod dimensions;
pub mod geometry;
pub mod goto_dialog;
pub mod handler;
pub mod layout;
pub mod lazy_handler;
pub mod outline;
pub mod page_view;
pub mod parser;
pub mod rich_text;
pub mod search;
pub mod selection;
pub mod state;
pub mod types;
pub mod update;
pub mod view;
pub mod viewport;

pub use cache::{PdfDiskCache, clean_stale_pdf_caches};
pub use goto_dialog::{GotoPageState, view_goto_dialog};
pub use handler::lazy_load_thumbnails;
pub use lazy_handler::lazy_load_pages;
pub use outline::{
    PdfOutlineNode, PdfOutlineState, build_outline_tree, collect_visible_outline_nodes,
    render_pdf_outline, resolve_outline_page,
};
pub use page_view::PdfPageWidget;
pub use parser::*;
pub use rich_text::extract_selected_content;
pub use search::{PdfSearchMatch, PdfSearchState, SEARCH_INPUT_ID};
pub use selection::{
    PdfChar, PdfLine, PdfPageText, PdfPosition, PdfSelection, compute_selection_rects,
    extract_selected_text,
};
pub use state::*;
pub use types::*;
pub use update::*;
pub use view::view_pdf;
