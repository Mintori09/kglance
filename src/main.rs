use std::sync::Arc;

use iced::{Font, Size};
use kglance::app::KglanceApp;
use kglance::{dbus, log_error, log_info, parsers};

fn build_registry() -> parsers::ParserRegistry {
    let mut r = parsers::ParserRegistry::new();
    r.register(Box::new(parsers::markdown::MarkdownParser::new()));
    r.register(Box::new(parsers::json::JsonParser));
    r.register(Box::new(parsers::font::FontParser));
    r.register(Box::new(parsers::text::TextParser::new()));
    r.register(Box::new(parsers::image::ImageParser));
    r.register(Box::new(parsers::svg::SvgParser));
    r.register(Box::new(parsers::pdf::PdfParser));
    r.register(Box::new(parsers::archive::ArchiveParser));
    r.register(Box::new(parsers::folder::FolderParser));
    r.register(Box::new(parsers::audio::AudioParser));
    r.register(Box::new(parsers::video::VideoParser));
    r.register(Box::new(parsers::epub::EpubParser));
    r.register(Box::new(parsers::csv::CsvParser));
    r.register(Box::new(parsers::office::OfficeParser));
    r.register(Box::new(parsers::typst::TypstParser));
    r
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    log_info!("Starting kglance with arguments: {:?}", args);

    let file_paths: Vec<String> = args
        .iter()
        .skip(1)
        .filter(|s| !s.starts_with('-') && s.as_str() != "daemon")
        .filter_map(|s| std::fs::canonicalize(s).ok())
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("daemon") => {
            log_info!("Running in daemon mode");
            run_daemon()
        }
        Some(path) if !path.starts_with('-') => {
            if file_paths.is_empty() {
                log_error!("No valid file paths provided: {:?}", path);
                eprintln!("Usage:");
                eprintln!("  kglance daemon                  Start preview daemon (autostart)");
                eprintln!("  kglance <file-path> [...]       Preview file(s)");
                std::process::exit(1);
            }

            log_info!("Attempting to preview files via DBus: {:?}", file_paths);
            if dbus::send_multiple_via_dbus(&file_paths).is_ok() {
                log_info!("Successfully requested preview via DBus");
                return Ok(());
            }
            log_info!(
                "DBus connection failed or daemon not running. Falling back to standalone mode."
            );
            run_standalone(&file_paths)
        }
        _ => {
            eprintln!("Usage:");
            eprintln!("  kglance daemon                  Start preview daemon (autostart)");
            eprintln!("  kglance <file-path> [...]       Preview file(s)");
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

    let config = kglance::core::config::ConfigManager::load_or_create();
    let default_font = config
        .ui
        .font_family
        .as_deref()
        .map(|name| Font::with_name(Box::leak(name.to_string().into_boxed_str())))
        .unwrap_or(Font::DEFAULT);

    log_info!("Running Iced daemon (no initial window)...");
    let reg = registry.clone();
    let rx = std::cell::Cell::new(Some(rx));
    iced::daemon(
        move || KglanceApp::new(reg.clone(), rx.take(), &[], true),
        KglanceApp::update,
        KglanceApp::view_daemon,
    )
    .default_font(default_font)
    .subscription(KglanceApp::subscription)
    .theme(KglanceApp::theme_daemon)
    .run()?;

    log_info!("Daemon event loop quit.");
    Ok(())
}

fn run_standalone(paths: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    log_info!("Running standalone preview for: {:?}", paths);
    let start_time = std::time::Instant::now();
    let registry = std::sync::Arc::new(build_registry());

    let owned_paths: Vec<String> = paths.to_vec();
    let config = kglance::core::config::ConfigManager::load_or_create();

    let primary = &paths[0];
    let resolved = std::path::Path::new(primary);
    let mut initial_size = Size::new(
        config.ui.default_width as f32,
        config.ui.default_height as f32,
    );
    if let Ok(kglance::parsers::ParsedContent::Image { width, height, .. }) =
        registry.parse(resolved)
    {
        initial_size =
            kglance::ui::views::image_view::helpers::calculate_window_size(width, height);
    }

    let default_font = config
        .ui
        .font_family
        .as_deref()
        .map(|name| Font::with_name(Box::leak(name.to_string().into_boxed_str())))
        .unwrap_or(Font::DEFAULT);

    let window_icon = kglance::load_app_icon();

    log_info!("Running Iced GUI in standalone mode...");
    let reg = registry.clone();
    iced::application(
        move || {
            let (app, task) = KglanceApp::new(reg.clone(), None, &owned_paths, false);
            let duration = start_time.elapsed();
            log_info!("[PERF] PreviewWindow GUI initialized in: {:?}", duration);
            (app, task)
        },
        KglanceApp::update,
        KglanceApp::view,
    )
    .window(iced::window::Settings {
        visible: true,
        min_size: Some(Size::new(
            config.ui.min_width as f32,
            config.ui.min_height as f32,
        )),
        size: initial_size,
        icon: window_icon,
        exit_on_close_request: true,
        decorations: true,
        ..Default::default()
    })
    .default_font(default_font)
    .title(KglanceApp::title)
    .subscription(KglanceApp::subscription)
    .theme(KglanceApp::theme)
    .run()?;

    log_info!("Standalone event loop quit.");
    Ok(())
}
