use std::path::Component;
use std::{
    env,
    path::{Path, PathBuf},
};

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

#[cfg(target_os = "linux")]
#[inline]
pub fn trim_process_memory() {
    let _ = std::thread::Builder::new()
        .name("kglance-trim-mem".into())
        .spawn(|| {
            unsafe extern "C" {
                fn malloc_trim(pad: usize) -> i32;
            }
            // SAFETY: `malloc_trim` is a standard GNU C library extension function that releases
            // free memory back to the operating system; passing 0 is always well-defined.
            unsafe {
                malloc_trim(0);
            }
        });
}

#[cfg(not(target_os = "linux"))]
#[inline]
pub fn trim_process_memory() {}

pub(crate) fn resolve_path(path: &str, file_path: &str) -> PathBuf {
    let path = path.trim();

    let expanded = if path == "~" || path == "$HOME" {
        env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(path))
    } else if let Some(rest) = path.strip_prefix("~/") {
        env::var_os("HOME")
            .map(|home| PathBuf::from(home).join(rest))
            .unwrap_or_else(|| PathBuf::from(path))
    } else if let Some(rest) = path.strip_prefix("$HOME/") {
        env::var_os("HOME")
            .map(|home| PathBuf::from(home).join(rest))
            .unwrap_or_else(|| PathBuf::from(path))
    } else {
        PathBuf::from(path)
    };

    let joined = if expanded.is_absolute() {
        expanded
    } else {
        Path::new(file_path)
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(expanded)
    };

    let mut result = PathBuf::new();

    for component in joined.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                let last = result.components().next_back();
                match last {
                    Some(Component::Normal(_)) => {
                        result.pop();
                    }
                    Some(Component::RootDir) => {}
                    _ => result.push(component),
                }
            }
            _ => result.push(component),
        }
    }

    result
}

pub fn path_to_file_uri(path: &Path) -> String {
    if let Ok(url) = url::Url::from_file_path(path) {
        return url.to_string();
    }

    let mut uri = String::from("file://");
    let path_str = path.to_string_lossy();
    for byte in path_str.as_bytes() {
        match *byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                uri.push(*byte as char);
            }
            b => {
                use std::fmt::Write;
                let _ = write!(uri, "%{b:02X}");
            }
        }
    }
    uri
}

pub fn reveal_in_file_manager(path: &Path) -> std::io::Result<()> {
    let abs_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    };

    let uri = path_to_file_uri(&abs_path);

    let dbus_result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let conn = zbus::blocking::Connection::session()?;
        conn.call_method(
            Some("org.freedesktop.FileManager1"),
            "/org/freedesktop/FileManager1",
            Some("org.freedesktop.FileManager1"),
            "ShowItems",
            &(vec![uri.as_str()], ""),
        )?;
        Ok(())
    })();

    if dbus_result.is_ok() {
        return Ok(());
    }

    let target_dir = if abs_path.is_file() {
        abs_path.parent().unwrap_or(&abs_path)
    } else {
        &abs_path
    };

    std::process::Command::new("xdg-open")
        .arg(target_dir)
        .spawn()
        .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::path::PathBuf;

    const HOME: &str = "/home/user";
    const FILE: &str = "/home/user/Documents/project/document.md";

    fn resolve(path: &str) -> PathBuf {
        // SAFETY: tests run single-threaded for this environment variable.
        unsafe { env::set_var("HOME", HOME) };
        resolve_path(path, FILE)
    }

    #[test]
    fn absolute_path() {
        assert_eq!(resolve("/tmp/image.png"), PathBuf::from("/tmp/image.png"));
    }

    #[test]
    fn relative_path() {
        assert_eq!(
            resolve("image.png"),
            PathBuf::from("/home/user/Documents/project/image.png")
        );
    }

    #[test]
    fn current_directory() {
        assert_eq!(
            resolve("./image.png"),
            PathBuf::from("/home/user/Documents/project/image.png")
        );
    }

    #[test]
    fn parent_directory() {
        assert_eq!(
            resolve("../image.png"),
            PathBuf::from("/home/user/Documents/image.png")
        );
    }

    #[test]
    fn multiple_parent_directories() {
        assert_eq!(
            resolve("../../image.png"),
            PathBuf::from("/home/user/image.png")
        );
    }

    #[test]
    fn tilde() {
        assert_eq!(
            resolve("~/Pictures/image.png"),
            PathBuf::from("/home/user/Pictures/image.png")
        );
    }

    #[test]
    fn home_variable() {
        assert_eq!(
            resolve("$HOME/Pictures/image.png"),
            PathBuf::from("/home/user/Pictures/image.png")
        );
    }

    #[test]
    fn home_itself() {
        assert_eq!(resolve("~"), PathBuf::from("/home/user"));
    }

    #[test]
    fn whitespace() {
        assert_eq!(
            resolve("  ./image.png  "),
            PathBuf::from("/home/user/Documents/project/image.png")
        );
    }

    #[test]
    fn file_uri_formatting() {
        let path = Path::new("/home/user/my file.txt");
        assert_eq!(path_to_file_uri(path), "file:///home/user/my%20file.txt");

        let path_special = Path::new("/tmp/test#1&2?.png");
        let uri = path_to_file_uri(path_special);
        assert!(uri.starts_with("file:///tmp/test"));
        assert!(uri.contains("%23") || uri.contains('#'));
    }
}
