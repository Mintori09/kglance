use std::sync::Arc;

use iced::{Font, Size};
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
    r.register(Box::new(parsers::epub::EpubParser));
    r.register(Box::new(parsers::csv::CsvParser));
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
    kglance::app::probe::init();
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

    log_info!("Running Iced daemon (no initial window)...");
    let reg = registry.clone();
    let rx = std::cell::Cell::new(Some(rx));
    iced::daemon(
        move || KglanceApp::new(reg.clone(), rx.take(), None, true),
        KglanceApp::update,
        KglanceApp::view_daemon,
    )
    .default_font(iced::Font::with_name("Inter"))
    .subscription(KglanceApp::subscription)
    .theme(KglanceApp::theme_daemon)
    .run()?;

    log_info!("Daemon event loop quit.");
    Ok(())
}

fn run_standalone(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    log_info!("Running standalone preview for: {}", path);
    let start_time = std::time::Instant::now();
    let registry = std::sync::Arc::new(build_registry());
    let path_str = path.to_string();

    let resolved = std::path::Path::new(path);
    let mut initial_size = Size::new(1024.0, 768.0);
    if let Ok(kglance::parsers::ParsedContent::Image { width, height, .. }) =
        registry.parse(resolved)
    {
        initial_size = kglance::ui::handlers::image::calculate_window_size(width, height);
    }

    log_info!("Running Iced GUI in standalone mode...");
    let reg = registry.clone();
    iced::application(
        move || {
            let (app, task) = KglanceApp::new(reg.clone(), None, Some(&path_str), false);
            let duration = start_time.elapsed();
            log_info!("[PERF] PreviewWindow GUI initialized in: {:?}", duration);
            (app, task)
        },
        KglanceApp::update,
        KglanceApp::view,
    )
    .window(iced::window::Settings {
        visible: true,
        min_size: Some(Size::new(400.0, 300.0)),
        size: initial_size,
        exit_on_close_request: true,
        decorations: true,
        ..Default::default()
    })
    .default_font(Font::with_name("Inter"))
    .title(KglanceApp::title)
    .subscription(KglanceApp::subscription)
    .theme(KglanceApp::theme)
    .run()?;

    log_info!("Standalone event loop quit.");
    Ok(())
}
