pub mod app;
pub mod core;
pub mod dbus;
pub mod logger;
pub mod parsers;
pub mod preview;
pub mod ui;

pub fn load_app_icon() -> Option<iced::window::Icon> {
    let svg_bytes = include_bytes!("../data/kglance.svg");
    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(svg_bytes, &opt).ok()?;
    let pixmap_size = tree.size().to_int_size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );
    let width = pixmap.width();
    let height = pixmap.height();
    let rgba = pixmap.take();
    iced::window::icon::from_rgba(rgba, width, height).ok()
}
