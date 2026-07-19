use std::sync::Arc;

use kglance::{dbus, parser, ui};

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
    match args.get(1).map(|s| s.as_str()) {
        Some("daemon") => run_daemon(),
        Some(path) if !path.starts_with('-') => {
            let resolved = std::fs::canonicalize(path)?;
            let resolved_str = resolved.to_string_lossy();
            if dbus::send_via_dbus(&resolved_str).is_ok() {
                return Ok(());
            }
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
        if let Err(e) = rt.block_on(dbus::run_zbus(zbus_registry, tx)) {
            eprintln!("zbus daemon error: {e}");
        }
    });

    let window = ui::PreviewWindow::new(false)?;
    let window_rc = std::rc::Rc::new(window);

    {
        let reg = registry.clone();
        let win = window_rc.clone();
        window_rc.set_file_selected_handler(move |path| {
            if let Ok(content) = reg.parse(std::path::Path::new(&path)) {
                win.show(&path, &content);
            }
        });
    }

    let rx = Arc::new(std::sync::Mutex::new(rx));

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

    slint::run_event_loop_until_quit()?;
    Ok(())
}

fn run_standalone(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let registry = std::sync::Arc::new(build_registry());
    let content = registry
        .parse(std::path::Path::new(path))
        .map_err(|e| format!("Cannot preview: {e}"))?;
    let window = std::rc::Rc::new(ui::PreviewWindow::new(true)?);
    {
        let reg = registry.clone();
        let win = window.clone();
        window.set_file_selected_handler(move |p| {
            if let Ok(content) = reg.parse(std::path::Path::new(&p)) {
                win.show(&p, &content);
            }
        });
    }
    window.show(path, &content);
    slint::run_event_loop()?;
    Ok(())
}
