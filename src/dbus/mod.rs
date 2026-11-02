pub mod recipe;
pub mod service;

pub use service::{DaemonCommand, DaemonService};

use crate::{features::common::parser::traits::ParserRegistry, log_error, log_info};
use std::sync::Arc;

pub const DEFAULT_DBUS_NAME: &str = "org.mintori.Kglance";
pub const DEFAULT_DBUS_PATH: &str = "/org/mintori/Kglance";
pub const DEFAULT_DBUS_INTERFACE: &str = "org.mintori.Kglance";

#[derive(Debug, Clone)]
pub struct PreviewDisplayedEvent {
    pub file_path: String,
    pub render_time_ms: u64,
}

pub fn quit_via_dbus() -> Result<(), Box<dyn std::error::Error>> {
    quit_via_dbus_to(DEFAULT_DBUS_NAME)
}

pub fn quit_via_dbus_to(service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = zbus::blocking::Connection::session()?;
    let _ = conn.call_method(
        Some(service_name),
        DEFAULT_DBUS_PATH,
        Some(DEFAULT_DBUS_INTERFACE),
        "Quit",
        &(),
    );
    Ok(())
}

pub fn send_via_dbus(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    send_via_dbus_to(DEFAULT_DBUS_NAME, path)
}

pub fn send_via_dbus_to(service_name: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    log_info!("DBus: Connecting to session DBus to send file path: {path} to {service_name}");
    let conn = zbus::blocking::Connection::session()?;

    log_info!("DBus: Calling ShowPreview on {service_name} with argument: {path}");
    conn.call_method(
        Some(service_name),
        DEFAULT_DBUS_PATH,
        Some(DEFAULT_DBUS_INTERFACE),
        "ShowPreview",
        &(path,),
    )?;

    log_info!("DBus: ShowPreview method call succeeded");
    Ok(())
}

pub fn send_multiple_via_dbus(paths: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    send_multiple_via_dbus_to(DEFAULT_DBUS_NAME, paths)
}

pub fn send_multiple_via_dbus_to(
    service_name: &str,
    paths: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    if paths.is_empty() {
        return Err("No file paths provided".into());
    }
    log_info!(
        "DBus: Connecting to session DBus to send {} file(s) to {service_name}",
        paths.len()
    );
    let conn = zbus::blocking::Connection::session()?;

    if paths.len() == 1 {
        return send_via_dbus_to(service_name, &paths[0]);
    }

    log_info!(
        "DBus: Calling ShowMultiplePreviews on {service_name} with {} files",
        paths.len()
    );
    let owned: Vec<String> = paths.to_vec();
    conn.call_method(
        Some(service_name),
        DEFAULT_DBUS_PATH,
        Some(DEFAULT_DBUS_INTERFACE),
        "ShowMultiplePreviews",
        &(owned,),
    )?;

    log_info!("DBus: ShowMultiplePreviews method call succeeded");
    Ok(())
}

pub fn send_update_via_dbus(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    send_update_via_dbus_to(DEFAULT_DBUS_NAME, path)
}

pub fn send_update_via_dbus_to(
    service_name: &str,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    log_info!("DBus: Connecting to session DBus to update file path: {path} on {service_name}");
    let conn = zbus::blocking::Connection::session()?;

    log_info!("DBus: Calling UpdatePreview on {service_name} with argument: {path}");
    conn.call_method(
        Some(service_name),
        DEFAULT_DBUS_PATH,
        Some(DEFAULT_DBUS_INTERFACE),
        "UpdatePreview",
        &(path,),
    )?;

    log_info!("DBus: UpdatePreview method call succeeded");
    Ok(())
}

pub fn send_multiple_update_via_dbus(paths: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    send_multiple_update_via_dbus_to(DEFAULT_DBUS_NAME, paths)
}

pub fn send_multiple_update_via_dbus_to(
    service_name: &str,
    paths: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    if paths.is_empty() {
        return Err("No file paths provided".into());
    }
    log_info!(
        "DBus: Connecting to session DBus to send update for {} file(s) to {service_name}",
        paths.len()
    );
    let conn = zbus::blocking::Connection::session()?;

    if paths.len() == 1 {
        return send_update_via_dbus_to(service_name, &paths[0]);
    }

    log_info!(
        "DBus: Calling UpdateMultiplePreviews on {service_name} with {} files",
        paths.len()
    );
    let owned: Vec<String> = paths.to_vec();
    conn.call_method(
        Some(service_name),
        DEFAULT_DBUS_PATH,
        Some(DEFAULT_DBUS_INTERFACE),
        "UpdateMultiplePreviews",
        &(owned,),
    )?;

    log_info!("DBus: UpdateMultiplePreviews method call succeeded");
    Ok(())
}

pub fn is_gui_open() -> Result<bool, Box<dyn std::error::Error>> {
    is_gui_open_at(DEFAULT_DBUS_NAME)
}

pub fn is_gui_open_at(service_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let conn = zbus::blocking::Connection::session()?;
    let reply: bool = conn
        .call_method(
            Some(service_name),
            DEFAULT_DBUS_PATH,
            Some(DEFAULT_DBUS_INTERFACE),
            "IsGuiOpen",
            &(),
        )?
        .body()
        .deserialize()?;
    Ok(reply)
}

pub async fn run_zbus(
    registry: Arc<ParserRegistry>,
    tx: tokio::sync::mpsc::Sender<DaemonCommand>,
    is_gui_open: Arc<std::sync::atomic::AtomicBool>,
    perf_rx: Option<tokio::sync::mpsc::Receiver<PreviewDisplayedEvent>>,
) -> Result<(), String> {
    run_zbus_with_name(DEFAULT_DBUS_NAME, registry, tx, is_gui_open, perf_rx).await
}

pub async fn run_zbus_with_name(
    service_name: &str,
    registry: Arc<ParserRegistry>,
    tx: tokio::sync::mpsc::Sender<DaemonCommand>,
    is_gui_open: Arc<std::sync::atomic::AtomicBool>,
    perf_rx: Option<tokio::sync::mpsc::Receiver<PreviewDisplayedEvent>>,
) -> Result<(), String> {
    log_info!("DBus: Starting zbus connection setup for {service_name}...");
    let service = DaemonService::new(registry, tx, is_gui_open);
    let conn = zbus::connection::Builder::session()
        .map_err(|e| {
            log_error!("DBus error during session build: {e}");
            format!("session: {e}")
        })?
        .name(service_name)
        .map_err(|e| {
            log_error!("DBus error registering name: {e}");
            format!("name: {e}")
        })?
        .serve_at(DEFAULT_DBUS_PATH, service)
        .map_err(|e| {
            log_error!("DBus error setting object path: {e}");
            format!("serve_at: {e}")
        })?
        .build()
        .await
        .map_err(|e| {
            log_error!("DBus connection build failed: {e}");
            format!("build: {e}")
        })?;

    log_info!("DBus: zbus service successfully running and serving {service_name}");

    if let Some(mut rx) = perf_rx {
        let emitter = zbus::object_server::SignalEmitter::new(&conn, DEFAULT_DBUS_PATH)
            .map_err(|e| format!("emitter: {e}"))?;
        tokio::spawn(async move {
            while let Some(evt) = rx.recv().await {
                let _ =
                    DaemonService::preview_displayed(&emitter, &evt.file_path, evt.render_time_ms)
                        .await;
            }
        });
    }

    std::future::pending::<()>().await;

    Ok(())
}
