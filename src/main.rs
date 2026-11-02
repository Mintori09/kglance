use std::sync::Arc;

use iced::{Font, Size};
use kglance::app::KglanceApp;
use kglance::engine::build_registry;
use kglance::{dbus, log_error, log_info};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Prefer pure Vulkan backend if not explicitly overridden, bypassing GL/GLES probing.
    if std::env::var_os("WGPU_BACKEND").is_none() {
        // SAFETY: Called at the very beginning of main before any threads are spawned.
        unsafe {
            std::env::set_var("WGPU_BACKEND", "vulkan");
        }
    }
    let args: Vec<String> = std::env::args().collect();

    let file_paths: Vec<String> = args
        .iter()
        .skip(1)
        .filter(|s| !s.starts_with('-') && s.as_str() != "daemon" && s.as_str() != "update")
        .filter_map(|s| std::fs::canonicalize(s).ok())
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let is_standalone = args.iter().any(|s| s == "--standalone" || s == "-s");
    let is_bench = args
        .iter()
        .any(|s| s == "--bench" || s == "--profile-startup");
    let is_force_daemon = args.iter().any(|s| s == "--daemon" || s == "-d");

    if is_bench {
        if file_paths.is_empty() {
            eprintln!("Usage: kglance --bench [--daemon|--standalone] <file-path>");
            std::process::exit(1);
        }
        if is_force_daemon {
            return run_bench_daemon(&file_paths[0]);
        } else {
            return run_bench_standalone(&file_paths[0]);
        }
    }

    match args.get(1).map(|s| s.as_str()) {
        Some("version" | "-v" | "--version") => {
            const VERSION: &str = env!("CARGO_PKG_VERSION");
            println!("Kglance: v{VERSION}");
            Ok(())
        }
        Some("help" | "-h" | "--help") => {
            println!("Usage:");
            println!("  kglance daemon                  Start preview daemon (autostart)");
            println!(
                "  kglance update <file-path> [...] Update currently open preview (daemon mode only)"
            );
            println!("  kglance <file-path> [...]       Preview file(s)");
            println!("  kglance --standalone <file-path> [...] Preview file(s) in standalone mode");
            println!("  kglance version                 Display version");
            Ok(())
        }
        Some("daemon") => {
            log_info!("Running in daemon mode");
            run_daemon()
        }
        Some("update") => {
            if file_paths.is_empty() {
                log_error!("No valid file paths provided for update");
                eprintln!("Usage: kglance update <file-path> [...]");
                std::process::exit(1);
            }

            match dbus::is_gui_open() {
                Ok(true) => {
                    log_info!("Attempting to update preview via DBus: {file_paths:?}");
                    if let Err(e) = dbus::send_multiple_update_via_dbus(&file_paths) {
                        log_error!("Daemon update via DBus failed: {e:?}");
                        eprintln!(
                            "Failed to update preview via DBus (daemon not running or not responding): {e}"
                        );
                        std::process::exit(1);
                    }
                    log_info!("Successfully requested update via DBus");
                }
                Ok(false) => {
                    log_info!("Preview GUI is not open/initialized. Skipping DBus update.");
                }
                Err(e) => {
                    log_info!(
                        "Daemon is not running or DBus unavailable: {e}. Skipping DBus update."
                    );
                }
            }
            Ok(())
        }
        Some("--standalone" | "-s") => {
            if file_paths.is_empty() {
                log_error!("No valid file paths provided for standalone preview");
                eprintln!("Usage: kglance --standalone <file-path> [...]");
                std::process::exit(1);
            }

            run_standalone(&file_paths)
        }
        Some(path) if !path.starts_with('-') => {
            if file_paths.is_empty() {
                log_error!("No valid file paths provided: {path:?}");
                eprintln!("Usage:");
                eprintln!("  kglance daemon                  Start preview daemon (autostart)");
                eprintln!(
                    "  kglance update <file-path> [...] Update currently open preview (daemon mode only)"
                );
                eprintln!("  kglance <file-path> [...]       Preview file(s)");
                eprintln!(
                    "  kglance --standalone <file-path> [...] Preview file(s) in standalone mode"
                );
                std::process::exit(1);
            }

            if is_standalone {
                return run_standalone(&file_paths);
            }

            log_info!("Attempting to preview files via DBus: {file_paths:?}");
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
            eprintln!(
                "  kglance update <file-path> [...] Update currently open preview (daemon mode only)"
            );
            eprintln!("  kglance <file-path> [...]       Preview file(s)");
            eprintln!(
                "  kglance --standalone <file-path> [...] Preview file(s) in standalone mode"
            );
            std::process::exit(1);
        }
    }
}

fn run_daemon() -> Result<(), Box<dyn std::error::Error>> {
    log_info!("Initializing daemon...");
    let registry = Arc::new(build_registry());
    let (tx, rx) = tokio::sync::mpsc::channel::<dbus::DaemonCommand>(100);
    let (perf_tx, perf_rx) = tokio::sync::mpsc::channel::<kglance::dbus::PreviewDisplayedEvent>(32);

    let is_gui_open = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let daemon_gui_open = is_gui_open.clone();

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
        if let Err(e) = rt.block_on(dbus::run_zbus(
            zbus_registry,
            tx,
            daemon_gui_open,
            Some(perf_rx),
        )) {
            log_error!("zbus daemon error: {e}");
        }
    });

    let config = kglance::core::config::ConfigManager::load_or_create();
    let max_disk_bytes = (config.cache.max_disk_cache_mb as u64) * 1024 * 1024;
    kglance::core::disk_cache::start_disk_cache_pruner_daemon(max_disk_bytes);

    let default_font = config
        .ui
        .font_family
        .as_deref()
        .map(|name| Font::with_name(Box::leak(name.to_string().into_boxed_str())))
        .unwrap_or(Font::DEFAULT);

    log_info!("Running Iced daemon (no initial window)...");
    let reg = registry.clone();
    let rx = std::cell::Cell::new(Some(rx));
    let daemon_config = config.clone();
    iced::daemon(
        move || {
            let (mut app, task) = KglanceApp::new(
                reg.clone(),
                rx.take(),
                &[],
                true,
                is_gui_open.clone(),
                Some(daemon_config.clone()),
            );
            app.perf_tx = Some(perf_tx.clone());
            (app, task)
        },
        KglanceApp::update,
        KglanceApp::view_daemon,
    )
    .default_font(default_font)
    .title(KglanceApp::title_daemon)
    .subscription(KglanceApp::subscription)
    .theme(KglanceApp::theme_daemon)
    .run()?;

    log_info!("Daemon event loop quit.");
    Ok(())
}

fn run_standalone(paths: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    run_standalone_internal(paths, None)
}

fn run_standalone_internal(
    paths: &[String],
    bench_start_time: Option<std::time::Instant>,
) -> Result<(), Box<dyn std::error::Error>> {
    log_info!("Running standalone preview for: {:?}", paths);
    let start_time = bench_start_time.unwrap_or_else(std::time::Instant::now);
    let is_bench = bench_start_time.is_some();
    let registry = std::sync::Arc::new(build_registry());

    let owned_paths: Vec<String> = paths.to_vec();
    let config = kglance::core::config::ConfigManager::load_or_create();

    let primary = &paths[0];
    let resolved = std::path::Path::new(primary);
    let mut initial_size = Size::new(
        config.ui.default_width as f32,
        config.ui.default_height as f32,
    );

    let ext = resolved
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();

    if matches!(
        ext.as_str(),
        "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp"
    ) && let Ok((width, height)) = image::image_dimensions(resolved)
    {
        initial_size = kglance::features::image::view::calculate_window_size(
            config.ui.default_width as f32,
            config.ui.default_height as f32,
            width,
            height,
        );
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
    let is_gui_open = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let standalone_config = config.clone();
    iced::application(
        move || {
            let (mut app, task) = KglanceApp::new(
                reg.clone(),
                None,
                &owned_paths,
                false,
                is_gui_open.clone(),
                Some(standalone_config.clone()),
            );
            if is_bench {
                app.is_bench_mode = true;
                app.bench_start_time = Some(start_time);
            }
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

fn run_bench_standalone(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let paths = vec![path.to_string()];
    run_standalone_internal(&paths, Some(std::time::Instant::now()))
}

fn run_bench_daemon(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let t0 = std::time::Instant::now();
    let mut _temp_guard = None;

    if dbus::is_gui_open().is_err() {
        println!("[BENCH] Daemon is not running. Spawning temporary daemon...");
        let current_exe = std::env::current_exe()?;
        let mut cmd = std::process::Command::new(current_exe);
        cmd.arg("daemon");

        #[cfg(target_os = "linux")]
        unsafe {
            use std::os::unix::process::CommandExt;
            cmd.pre_exec(|| {
                libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM);
                Ok(())
            });
        }

        let child = cmd.spawn()?;
        struct TempDaemon(std::process::Child);
        impl Drop for TempDaemon {
            fn drop(&mut self) {
                let _ = kglance::dbus::quit_via_dbus();
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
        _temp_guard = Some(TempDaemon(child));

        let wait_start = std::time::Instant::now();
        while dbus::is_gui_open().is_err() {
            if wait_start.elapsed() > std::time::Duration::from_secs(4) {
                return Err("Timeout waiting for temporary daemon to acquire DBus name".into());
            }
            std::thread::sleep(std::time::Duration::from_millis(15));
        }
    }

    println!("[BENCH] Connecting to DBus Session Bus...");
    let conn = zbus::blocking::Connection::session()?;
    let proxy: zbus::blocking::Proxy = zbus::blocking::Proxy::new(
        &conn,
        kglance::dbus::DEFAULT_DBUS_NAME,
        kglance::dbus::DEFAULT_DBUS_PATH,
        kglance::dbus::DEFAULT_DBUS_INTERFACE,
    )?;

    let mut signal_iterator = proxy.receive_signal("PreviewDisplayed")?;
    let start_bench = std::time::Instant::now();
    let _ = kglance::dbus::send_via_dbus(path);

    // Wait for signal with 5-second timeout
    let timeout = std::time::Duration::from_secs(5);
    let wait_start = std::time::Instant::now();

    while wait_start.elapsed() < timeout {
        if let Some(msg) = signal_iterator.next() {
            let total_dur = start_bench.elapsed();
            let (_file, daemon_ms): (String, u64) = msg.body().deserialize().unwrap_or_default();
            let total_ms = total_dur.as_secs_f64() * 1000.0;
            let tier = match total_ms {
                t if t < 100.0 => "\x1b[32m🟢 [FAST] (< 100ms)\x1b[0m",
                t if t <= 300.0 => "\x1b[33m🟡 [ACCEPTABLE] (100-300ms)\x1b[0m",
                _ => "\x1b[31m🔴 [SLOW] (> 300ms)\x1b[0m",
            };

            println!("\n[BENCHMARK] Mode: Daemon | Target: {path}");
            println!("  Status:              {tier}");
            println!(
                "  Time-to-First-Frame: {:.2} ms (Daemon render: {daemon_ms} ms)",
                total_ms
            );
            println!(
                "  Overall Duration:    {:.2} ms\n",
                t0.elapsed().as_secs_f64() * 1000.0
            );

            // Clean up: hide/close window on daemon
            let _ = conn.call_method(
                Some(kglance::dbus::DEFAULT_DBUS_NAME),
                kglance::dbus::DEFAULT_DBUS_PATH,
                Some(kglance::dbus::DEFAULT_DBUS_INTERFACE),
                "HidePreview",
                &(),
            );
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    Err("Timed out waiting for PreviewDisplayed DBus signal".into())
}
