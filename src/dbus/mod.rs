pub mod recipe;
pub mod service;

pub use service::{DaemonCommand, DaemonService};

use crate::{features::common::parser::traits::ParserRegistry, log_error, log_info};
use std::sync::Arc;

pub fn send_via_dbus(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    log_info!(
        "DBus: Connecting to session DBus to send file path: {}",
        path
    );
    let conn = zbus::blocking::Connection::session()?;

    log_info!(
        "DBus: Calling ShowPreview on org.mintori.Kglance with argument: {}",
        path
    );
    conn.call_method(
        Some("org.mintori.Kglance"),
        "/org/mintori/Kglance",
        Some("org.mintori.Kglance"),
        "ShowPreview",
        &(path,),
    )?;

    log_info!("DBus: ShowPreview method call succeeded");
    Ok(())
}

pub fn send_multiple_via_dbus(paths: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if paths.is_empty() {
        return Err("No file paths provided".into());
    }
    log_info!(
        "DBus: Connecting to session DBus to send {} file(s)",
        paths.len()
    );
    let conn = zbus::blocking::Connection::session()?;

    if paths.len() == 1 {
        return send_via_dbus(&paths[0]);
    }

    log_info!(
        "DBus: Calling ShowMultiplePreviews on org.mintori.Kglance with {} files",
        paths.len()
    );
    let owned: Vec<String> = paths.to_vec();
    conn.call_method(
        Some("org.mintori.Kglance"),
        "/org/mintori/Kglance",
        Some("org.mintori.Kglance"),
        "ShowMultiplePreviews",
        &(owned,),
    )?;

    log_info!("DBus: ShowMultiplePreviews method call succeeded");
    Ok(())
}

pub async fn run_zbus(
    registry: Arc<ParserRegistry>,
    tx: tokio::sync::mpsc::Sender<DaemonCommand>,
) -> Result<(), String> {
    log_info!("DBus: Starting zbus connection setup for org.mintori.Kglance...");
    let service = DaemonService::new(registry, tx);
    let _conn = zbus::connection::Builder::session()
        .map_err(|e| {
            log_error!("DBus error during session build: {e}");
            format!("session: {e}")
        })?
        .name("org.mintori.Kglance")
        .map_err(|e| {
            log_error!("DBus error registering name: {e}");
            format!("name: {e}")
        })?
        .serve_at("/org/mintori/Kglance", service)
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

    log_info!("DBus: zbus service successfully running and serving org.mintori.Kglance");
    std::future::pending::<()>().await;

    Ok(())
}
