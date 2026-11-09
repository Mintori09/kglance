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
        let render_svg_to_icon = |svg_bytes: &[u8]| -> Option<iced::window::Icon> {
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
        };

        // Try looking up "kglance" from system icon theme or known icon directories
        if let Some(path) = ui::theme::icon_theme::resolve_icon_path("kglance")
            && let Ok(svg_bytes) = std::fs::read(&path)
            && let Some(icon) = render_svg_to_icon(&svg_bytes)
        {
            return Some(icon);
        }

        for path in [
            std::path::PathBuf::from("/usr/share/icons/hicolor/scalable/apps/kglance.svg"),
            std::path::PathBuf::from("/usr/local/share/icons/hicolor/scalable/apps/kglance.svg"),
        ] {
            if path.exists()
                && let Ok(svg_bytes) = std::fs::read(&path)
                && let Some(icon) = render_svg_to_icon(&svg_bytes)
            {
                return Some(icon);
            }
        }

        // Fallback to embedded data/kglance.svg
        let svg_bytes = include_bytes!("../data/kglance.svg");
        render_svg_to_icon(svg_bytes)
    });
    ICON.clone()
}
