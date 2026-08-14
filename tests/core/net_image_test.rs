#[test]
fn test_is_remote_url() {
    assert!(kglance::core::net::is_remote_url("https://miro.medium.com/v2/resize:fit:1126/format:webp/1*3LYrhyjjzgcUWCX5y06r4g.png"));
    assert!(kglance::core::net::is_remote_url("http://example.com/image.png"));
    assert!(!kglance::core::net::is_remote_url("./assets/image.png"));
    assert!(!kglance::core::net::is_remote_url("/home/user/image.png"));
}
