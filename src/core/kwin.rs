use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::oneshot;
use zbus::interface;

static SCRIPT_COUNTER: AtomicU64 = AtomicU64::new(0);

fn is_kde_wayland() -> bool {
    let is_wayland = std::env::var_os("WAYLAND_DISPLAY").is_some()
        || std::env::var("XDG_SESSION_TYPE")
            .map(|v| v.eq_ignore_ascii_case("wayland"))
            .unwrap_or(false);

    if !is_wayland {
        return false;
    }

    std::env::var("XDG_CURRENT_DESKTOP")
        .map(|v| v.to_ascii_uppercase().contains("KDE"))
        .unwrap_or(false)
        || std::env::var_os("KDE_FULL_SESSION").is_some()
}

struct DesktopReporter {
    tx: Arc<tokio::sync::Mutex<Option<oneshot::Sender<bool>>>>,
}

#[interface(name = "org.kde.kwin.KglanceDesktop")]
impl DesktopReporter {
    async fn report(&self, on_current: bool) {
        if let Some(tx) = self.tx.lock().await.take() {
            let _ = tx.send(on_current);
        }
    }
}

/// Checks whether the Kglance window is on the active virtual desktop.
/// If it is on the current desktop, KWin focuses it.
/// Returns `Some(true)` if on current desktop, `Some(false)` if on another desktop,
/// or `None` if running outside KDE Wayland or if query fails.
pub async fn is_on_current_desktop() -> Option<bool> {
    if !is_kde_wayland() {
        return None;
    }

    let conn = zbus::Connection::session().await.ok()?;
    let unique_name = conn.unique_name()?.to_string();

    let pid = std::process::id();
    let counter = SCRIPT_COUNTER.fetch_add(1, Ordering::Relaxed);
    let plugin_name = format!("kglance_desktop_query_{pid}_{counter}");
    let obj_path = format!("/org/kde/kwin/KglanceDesktop/{pid}_{counter}");

    let (tx, rx) = oneshot::channel();
    let reporter = DesktopReporter {
        tx: Arc::new(tokio::sync::Mutex::new(Some(tx))),
    };

    conn.object_server()
        .at(obj_path.as_str(), reporter)
        .await
        .ok()?;

    let script_content = format!(
        r#"
(function() {{
    var targetPid = {pid};
    var onCurrent = false;
    for (var i = 0; i < workspace.stackingOrder.length; ++i) {{
        var win = workspace.stackingOrder[i];
        if (win.pid === targetPid || win.resourceClass === "kglance") {{
            for (var j = 0; j < win.desktops.length; ++j) {{
                if (win.desktops[j] === workspace.currentDesktop || (win.desktops[j].id && win.desktops[j].id === workspace.currentDesktop.id)) {{
                    onCurrent = true;
                    break;
                }}
            }}
            if (onCurrent) {{
                workspace.activeWindow = win;
            }}
            break;
        }}
    }}
    callDBus("{unique_name}", "{obj_path}", "org.kde.kwin.KglanceDesktop", "report", onCurrent);
}})();
"#
    );

    let script_path = std::env::temp_dir().join(format!("{plugin_name}.js"));
    if std::fs::write(&script_path, script_content).is_err() {
        let _ = conn
            .object_server()
            .remove::<DesktopReporter, _>(obj_path.as_str())
            .await;
        return None;
    }

    let run_res = run_script_via_dbus(&conn, &script_path, &plugin_name).await;
    let _ = std::fs::remove_file(&script_path);

    let query_result = if run_res.is_ok() {
        match tokio::time::timeout(Duration::from_millis(200), rx).await {
            Ok(Ok(val)) => Some(val),
            _ => None,
        }
    } else {
        None
    };

    let _ = conn
        .object_server()
        .remove::<DesktopReporter, _>(obj_path.as_str())
        .await;
    query_result
}

/// Synchronous wrapper around `is_on_current_desktop()`.
/// If called within or outside an async runtime, executes the query safely with a fallback.
pub fn is_on_current_desktop_sync() -> Option<bool> {
    if !is_kde_wayland() {
        return None;
    }

    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        std::thread::scope(|s| {
            s.spawn(|| handle.block_on(is_on_current_desktop()))
                .join()
                .unwrap_or(None)
        })
    } else {
        match tokio::runtime::Runtime::new() {
            Ok(rt) => rt.block_on(is_on_current_desktop()),
            Err(_) => None,
        }
    }
}

async fn run_script_via_dbus(
    conn: &zbus::Connection,
    script_path: &std::path::Path,
    plugin_name: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let path_str = match script_path.to_str() {
        Some(s) => s,
        None => return Err("Invalid temp script path".into()),
    };

    let reply = conn
        .call_method(
            Some("org.kde.KWin"),
            "/Scripting",
            Some("org.kde.kwin.Scripting"),
            "loadScript",
            &(path_str, plugin_name),
        )
        .await?;

    let script_id: i32 = reply.body().deserialize()?;
    let script_object_path = format!("/Scripting/Script{script_id}");

    let run_res = conn
        .call_method(
            Some("org.kde.KWin"),
            script_object_path.as_str(),
            Some("org.kde.kwin.Script"),
            "run",
            &(),
        )
        .await;

    // Small delay to ensure KWin engine processes the script execution
    tokio::time::sleep(Duration::from_millis(15)).await;

    let _ = conn
        .call_method(
            Some("org.kde.KWin"),
            script_object_path.as_str(),
            Some("org.kde.kwin.Script"),
            "stop",
            &(),
        )
        .await;

    let _ = conn
        .call_method(
            Some("org.kde.KWin"),
            "/Scripting",
            Some("org.kde.kwin.Scripting"),
            "unloadScript",
            &(plugin_name,),
        )
        .await;

    run_res?;
    Ok(())
}
