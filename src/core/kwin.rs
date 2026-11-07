use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

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

pub async fn bring_to_current_desktop_and_focus() {
    if !is_kde_wayland() {
        return;
    }

    let pid = std::process::id();
    let counter = SCRIPT_COUNTER.fetch_add(1, Ordering::Relaxed);
    let plugin_name = format!("kglance_focus_{pid}_{counter}");

    if let Err(e) = execute_kwin_focus_script(&plugin_name, pid).await {
        crate::log_debug!("KWin focus script error: {e}");
    }
}

async fn execute_kwin_focus_script(
    plugin_name: &str,
    pid: u32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let script_content = format!(
        r#"
(function() {{
    var targetPid = {pid};
    for (var i = 0; i < workspace.stackingOrder.length; ++i) {{
        var win = workspace.stackingOrder[i];
        if (win.pid === targetPid || win.resourceClass === "kglance") {{
            var onCurrent = false;
            for (var j = 0; j < win.desktops.length; ++j) {{
                if (win.desktops[j] === workspace.currentDesktop) {{
                    onCurrent = true;
                    break;
                }}
            }}
            if (!onCurrent) {{
                win.desktops = [workspace.currentDesktop];
            }}
            workspace.activeWindow = win;
            break;
        }}
    }}
}})();
"#
    );

    let script_path = std::env::temp_dir().join(format!("{plugin_name}.js"));
    std::fs::write(&script_path, script_content)?;

    let result = run_script_via_dbus(&script_path, plugin_name).await;

    let _ = std::fs::remove_file(&script_path);

    result
}

async fn run_script_via_dbus(
    script_path: &std::path::Path,
    plugin_name: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let conn = zbus::Connection::session().await?;
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
