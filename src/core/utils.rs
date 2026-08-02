pub fn human_time(datetime: std::time::SystemTime) -> String {
    let now = chrono::Local::now();
    let dt = chrono::DateTime::<chrono::Local>::from(datetime);
    let duration = now.signed_duration_since(dt);

    if duration.num_seconds() < 0 {
        return dt.format("%b %d").to_string();
    }

    if duration.num_minutes() < 1 {
        return "Just now".to_string();
    } else if duration.num_hours() < 1 {
        return format!("{}m ago", duration.num_minutes());
    } else if duration.num_days() < 1 {
        return format!("{}h ago", duration.num_hours());
    } else if duration.num_days() == 1 {
        return "Yesterday".to_string();
    } else if duration.num_days() < 7 {
        return format!("{}d ago", duration.num_days());
    }

    dt.format("%b %d, %Y").to_string()
}

pub fn format_timestamp(secs: u64) -> String {
    let dur = std::time::Duration::from_secs(secs);
    let sys_time = std::time::UNIX_EPOCH + dur;
    let dt: chrono::DateTime<chrono::Local> = sys_time.into();
    dt.format("%Y-%m-%d %H:%M").to_string()
}

pub fn human_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    if unit_idx == 0 {
        format!("{} B", bytes)
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
}
