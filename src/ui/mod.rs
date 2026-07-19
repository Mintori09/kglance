pub mod generated {
    slint::include_modules!();
}

mod helpers;
mod image_handler;
mod table;
mod theme;
mod video_player;
mod window;

pub use generated::TableRow;
pub use window::PreviewWindow;
