pub fn is_remote_url(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://")
}

pub fn url_cache_key(url: &str) -> String {
    format!("{:x}", md5::compute(url.as_bytes()))
}

pub async fn fetch_remote_image(url: &str) -> Option<Vec<u8>> {
    if !is_remote_url(url) {
        return None;
    }

    let cache_dir = dirs::cache_dir()?.join("kglance").join("remote_images");
    let _ = std::fs::create_dir_all(&cache_dir);
    let key = url_cache_key(url);
    let cache_file = cache_dir.join(key);

    if let Ok(bytes) = std::fs::read(&cache_file) {
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

    let _ = std::fs::write(&cache_file, &fetched);
    Some(fetched)
}
