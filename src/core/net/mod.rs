pub fn is_remote_url(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://")
}

pub async fn fetch_remote_image(url: &str) -> Option<Vec<u8>> {
    if !is_remote_url(url) {
        return None;
    }

    if let Some(cached_path) =
        crate::core::disk_cache::get_cached_url_path("remote_images", url, "")
        && let Ok(bytes) = std::fs::read(&cached_path)
    {
        return if bytes.is_empty() { None } else { Some(bytes) };
    }

    let url_str = url.to_string();
    let fetched = tokio::task::spawn_blocking(move || {
        let mut command = std::process::Command::new("curl");
        command
            .arg("-sSL")
            .arg("--max-time")
            .arg("10")
            .arg(&url_str);
        let output = command.output().ok()?;
        if output.status.success() && !output.stdout.is_empty() {
            Some(output.stdout)
        } else {
            None
        }
    })
    .await
    .ok()??;

    let _ = crate::core::disk_cache::put_url_bytes("remote_images", url, "", &fetched);
    Some(fetched)
}
