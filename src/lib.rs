pub mod app;
pub mod core;
pub mod dbus;
pub mod engine;
pub mod features;
pub use features as parsers;
pub mod logger;
pub mod ui;

use std::sync::LazyLock;

pub fn load_app_icon() -> Option<iced::window::Icon> {
    static ICON: LazyLock<Option<iced::window::Icon>> = LazyLock::new(|| {
        let svg_bytes = include_bytes!("../data/kglance.svg");
        let opt = resvg::usvg::Options::default();
        let tree = resvg::usvg::Tree::from_data(svg_bytes, &opt).ok()?;
        let target_size = 128u32;
        let mut pixmap = resvg::tiny_skia::Pixmap::new(target_size, target_size)?;
        let sx = target_size as f32 / tree.size().width();
        let sy = target_size as f32 / tree.size().height();
        let transform = resvg::tiny_skia::Transform::from_scale(sx, sy);
        resvg::render(&tree, transform, &mut pixmap.as_mut());
        let rgba = pixmap.take();
        iced::window::icon::from_rgba(rgba, target_size, target_size).ok()
    });
    ICON.clone()
}
