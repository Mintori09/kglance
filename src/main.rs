use std::sync::Arc;

use kglance::app::KglanceApp;
use kglance::{dbus, log_error, log_info, parsers};

fn build_registry() -> parsers::ParserRegistry {
    let mut r = parsers::ParserRegistry::new();
    r.register(Box::new(parsers::markdown::MarkdownParser::new()));
    r.register(Box::new(parsers::text::TextParser::new()));
    r.register(Box::new(parsers::image::ImageParser));
    r.register(Box::new(parsers::svg::SvgParser));
    r.register(Box::new(parsers::pdf::PdfParser));
    r.register(Box::new(parsers::archive::ArchiveParser));
    r.register(Box::new(parsers::folder::FolderParser));
    r.register(Box::new(parsers::font::FontParser));
    r.register(Box::new(parsers::audio::AudioParser));
    r.register(Box::new(parsers::video::VideoParser));
    r.register(Box::new(parsers::office::OfficeParser));
    r
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    log_info!("Starting kglance with arguments: {:?}", args);
    match args.get(1).map(|s| s.as_str()) {
        Some("daemon") => {
            log_info!("Running in daemon mode");
            run_daemon()
        }
        Some(path) if !path.starts_with('-') => {
            let resolved = std::fs::canonicalize(path)?;
            let resolved_str = resolved.to_string_lossy();
            log_info!("Attempting to preview file via DBus: {}", resolved_str);
            if dbus::send_via_dbus(&resolved_str).is_ok() {
                log_info!("Successfully requested preview via DBus");
                return Ok(());
            }
            log_info!(
                "DBus connection failed or daemon not running. Falling back to standalone mode."
            );
            run_standalone(&resolved_str)
        }
        _ => {
            eprintln!("Usage:");
            eprintln!("  kglance daemon                Start preview daemon (autostart)");
            eprintln!("  kglance <file-path>           Preview file (DBus or standalone)");
            std::process::exit(1);
        }
    }
}

fn run_daemon() -> Result<(), Box<dyn std::error::Error>> {
    log_info!("Initializing daemon...");
    let registry = Arc::new(build_registry());
    let (tx, rx) = tokio::sync::mpsc::channel::<dbus::DaemonCommand>(100);

    let zbus_registry = registry.clone();
    let _zbus_thread = std::thread::spawn(move || {
        log_info!("Starting DBus zbus thread...");
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                log_error!("Failed to create tokio runtime: {e}");
                return;
            }
        };
        if let Err(e) = rt.block_on(dbus::run_zbus(zbus_registry, tx)) {
            log_error!("zbus daemon error: {e}");
        }
    });

    log_info!("Running Iced GUI in daemon mode...");
    iced::application(KglanceApp::title, KglanceApp::update, KglanceApp::view)
        .window(iced::window::Settings {
            visible: false,
            exit_on_close_request: false,
            decorations: false,
            ..Default::default()
        })
        .subscription(KglanceApp::subscription)
        .theme(KglanceApp::theme)
        .run_with(move || KglanceApp::new(registry, Some(rx), None, true))?;

    log_info!("Daemon event loop quit.");
    Ok(())
}

fn run_standalone(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    log_info!("Running standalone preview for: {}", path);
    let start_time = std::time::Instant::now();
    let registry = std::sync::Arc::new(build_registry());
    let path_str = path.to_string();

    log_info!("Running Iced GUI in standalone mode...");
    iced::application(KglanceApp::title, KglanceApp::update, KglanceApp::view)
        .window(iced::window::Settings {
            visible: true,
            exit_on_close_request: true,
            decorations: false,
            ..Default::default()
        })
        .subscription(KglanceApp::subscription)
        .theme(KglanceApp::theme)
        .run_with(move || {
            let (app, task) = KglanceApp::new(registry, None, Some(&path_str), false);
            let duration = start_time.elapsed();
            log_info!("[PERF] PreviewWindow GUI initialized in: {:?}", duration);
            (app, task)
        })?;

    log_info!("Standalone event loop quit.");
    Ok(())
}
