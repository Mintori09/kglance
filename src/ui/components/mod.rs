pub mod image_viewer;
pub mod media_viewer;
pub mod pdf_viewer;
pub mod table_viewer;
pub mod text_viewer;

pub use image_viewer::view_image;
pub use media_viewer::view_media;
pub use pdf_viewer::view_pdf;
pub use table_viewer::view_table;
pub use text_viewer::view_text;
