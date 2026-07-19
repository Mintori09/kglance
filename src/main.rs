use std::sync::Arc;

use kglance::{dbus, log_error, log_info, parser, ui};

fn build_registry() -> parser::ParserRegistry {
    let mut r = parser::ParserRegistry::new();
    r.register(Box::new(parser::markdown::MarkdownParser::new()));
    r.register(Box::new(parser::text::TextParser::new()));
    r.register(Box::new(parser::image::ImageParser));
    r.register(Box::new(parser::svg::SvgParser));
    r.register(Box::new(parser::pdf::PdfParser));
    r.register(Box::new(parser::archive::ArchiveParser));
    r.register(Box::new(parser::folder::FolderParser));
    r.register(Box::new(parser::font::FontParser));
    r.register(Box::new(parser::audio::AudioParser));
    r.register(Box::new(parser::video::VideoParser));
    r.register(Box::new(parser::office::OfficeParser));
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
    let (tx, rx) = std::sync::mpsc::channel::<dbus::DaemonCommand>();

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

    log_info!("Creating Slint preview window...");
    let window = ui::PreviewWindow::new(false)?;
    let window_rc = std::rc::Rc::new(window);

    {
        let reg = registry.clone();
        let win = window_rc.clone();
        window_rc.set_file_selected_handler(move |path| {
            log_info!("File selected in UI: {}", path);
            match reg.parse(std::path::Path::new(&path)) {
                Ok(content) => {
                    log_info!("Successfully parsed selected file: {}", path);
                    win.show(&path, &content);
                }
                Err(e) => {
                    log_error!("Failed to parse selected file {}: {:?}", path, e);
                }
            }
        });
    }

    let rx = Arc::new(std::sync::Mutex::new(rx));

    let timer = slint::Timer::default();

    log_info!("Starting daemon command receiver loop timer...");
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
                                log_info!("Daemon received ShowPreview command for: {}", path);
                                window.show(&path, &content);
                            }
                            dbus::DaemonCommand::HidePreview => {
                                log_info!("Daemon received HidePreview command");
                                window.hide();
                            }
                        }
                    }
                }
            }
        },
    );

    log_info!("Running Slint event loop...");
    slint::run_event_loop_until_quit()?;
    log_info!("Daemon event loop quit.");
    Ok(())
}

fn run_standalone(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    log_info!("Running standalone preview for: {}", path);
    let start_time = std::time::Instant::now();

    let registry = std::sync::Arc::new(build_registry());
    log_info!("Parsing file: {}", path);
    let content = registry.parse(std::path::Path::new(path)).map_err(|e| {
        log_error!("Failed to parse file: {e}");
        format!("Cannot preview: {e}")
    })?;

    log_info!("Creating Slint preview window...");
    let window = std::rc::Rc::new(ui::PreviewWindow::new(true)?);

    {
        let reg = registry.clone();
        let win = window.clone();
        window.set_file_selected_handler(move |p| {
            log_info!("File selected in UI: {}", p);
            match reg.parse(std::path::Path::new(&p)) {
                Ok(content) => {
                    log_info!("Successfully parsed selected file: {}", p);
                    win.show(&p, &content);
                }
                Err(e) => {
                    log_error!("Failed to parse selected file {}: {:?}", p, e);
                }
            }
        });
    }

    window.show(path, &content);

    slint::invoke_from_event_loop(move || {
        let duration = start_time.elapsed();
        log_info!("[PERF] PreviewWindow GUI fully loaded in: {:?}", duration);
    })
    .unwrap();

    log_info!("Running Slint event loop...");
    slint::run_event_loop()?;
    log_info!("Standalone event loop quit.");
    Ok(())
}
