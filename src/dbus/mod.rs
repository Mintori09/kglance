pub mod service;

pub use service::{DaemonCommand, DaemonService};

use std::sync::Arc;

pub fn send_via_dbus(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = zbus::blocking::Connection::session()?;

    conn.call_method(
        Some("org.mintori.Kglance"),
        "/org/mintori/Kglance",
        Some("org.mintori.Kglance"),
        "ShowPreview",
        &(path,),
    )?;

    Ok(())
}

pub async fn run_zbus(
    registry: Arc<crate::parser::ParserRegistry>,
    tx: std::sync::mpsc::Sender<DaemonCommand>,
) -> Result<(), String> {
    let service = DaemonService::new(registry, tx);
    let _conn = zbus::connection::Builder::session()
        .map_err(|e| format!("session: {e}"))?
        .name("org.mintori.Kglance")
        .map_err(|e| format!("name: {e}"))?
        .serve_at("/org/mintori/Kglance", service)
        .map_err(|e| format!("serve_at: {e}"))?
        .build()
        .await
        .map_err(|e| format!("build: {e}"))?;

    std::future::pending::<()>().await;

    Ok(())
}
