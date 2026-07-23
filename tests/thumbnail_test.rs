use kglance::ui::views::grid::get_freedesktop_thumbnail_path;

#[test]
fn test_freedesktop_hash() {
    let path = "/home/user/test.png";
    let thumb_path = get_freedesktop_thumbnail_path(path);
    if let Some(p) = thumb_path {
        assert!(p.to_string_lossy().contains(".cache/thumbnails/"));
    }
}
