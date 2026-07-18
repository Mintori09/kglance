mod dbus;
mod parser;
mod ui;

use std::sync::Arc;

fn build_registry() -> parser::ParserRegistry {
    let mut r = parser::ParserRegistry::new();
    r.register(Box::new(parser::text::TextParser::new()));
    r.register(Box::new(parser::image::ImageParser));
    r.register(Box::new(parser::svg::SvgParser));
    r.register(Box::new(parser::pdf::PdfParser));
    r.register(Box::new(parser::archive::ArchiveParser));
    r.register(Box::new(parser::folder::FolderParser));
    r
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = Arc::new(build_registry());
    let (tx, rx) = std::sync::mpsc::channel::<dbus::DaemonCommand>();

    let zbus_registry = registry.clone();
    let _zbus_thread = std::thread::spawn(move || {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!("Failed to create tokio runtime: {e}");
                return;
            }
        };
        if let Err(e) = rt.block_on(run_zbus(zbus_registry, tx)) {
            eprintln!("zbus daemon error: {e}");
        }
    });

    let window = ui::PreviewWindow::new()?;

    let rx = Arc::new(std::sync::Mutex::new(rx));
    let window_rc = std::rc::Rc::new(window);

    let timer = slint::Timer::default();
    timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(50),
        {
            let window = window_rc.clone();
            let rx = rx.clone();
            move || {
                if let Ok(guard) = rx.lock() {
                    while let Ok(cmd) = guard.try_recv() {
                        match cmd {
                            dbus::DaemonCommand::ShowPreview { path, content } => {
                                window.show(&path, &content);
                            }
                            dbus::DaemonCommand::HidePreview => {
                                window.hide();
                            }
                        }
                    }
                }
            }
        },
    );

    let _ = window_rc.window().show();
    slint::run_event_loop()?;
    Ok(())
}

async fn run_zbus(
    registry: Arc<parser::ParserRegistry>,
    tx: std::sync::mpsc::Sender<dbus::DaemonCommand>,
) -> Result<(), String> {
    let service = dbus::DaemonService::new(registry, tx);
    let _conn = zbus::connection::Builder::session()
        .map_err(|e| format!("session: {e}"))?
        .name("org.mintori.KiviewRust")
        .map_err(|e| format!("name: {e}"))?
        .serve_at("/org/mintori/KiviewRust", service)
        .map_err(|e| format!("serve_at: {e}"))?
        .build()
        .await
        .map_err(|e| format!("build: {e}"))?;
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
    }
}
