use std::process::{Child, Command};
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== KGLANCE STARTUP PROFILER ===");

    // Step 1: Kill any existing daemon
    let _ = Command::new("pkill")
        .arg("-f")
        .arg("kglance daemon")
        .output();
    std::thread::sleep(Duration::from_millis(500));

    // Step 2: Spawn daemon with KGLANCE_PROBE=1
    println!("[TEST] Spawning kglance daemon with KGLANCE_PROBE=1...");
    let mut daemon: Child = Command::new("cargo")
        .args(["run", "--bin", "kglance", "--", "daemon"])
        .env("KGLANCE_PROBE", "1")
        .env("RUST_LOG", "info")
        .spawn()?;

    // Give daemon 3 seconds to register zbus interface & iced loop
    std::thread::sleep(Duration::from_secs(3));

    // Step 3: Trigger preview via CLI (DBus request)
    let test_file = "../../testing-file/markdown.md";
    println!("[TEST] Triggering preview via DBus for: {test_file}");
    let t_start = Instant::now();

    let output = Command::new("cargo")
        .args(["run", "--bin", "kglance", "--", test_file])
        .output()?;

    let t_cli_done = t_start.elapsed();
    println!(
        "[TEST] DBus CLI call returned in: {:.2?}ms",
        t_cli_done.as_secs_f64() * 1000.0
    );
    println!(
        "[TEST] CLI output: {}",
        String::from_utf8_lossy(&output.stdout).trim()
    );

    // Wait a moment for frame rendering output to print to stderr/daemon log
    std::thread::sleep(Duration::from_secs(2));

    // Step 4: Clean up daemon
    let _ = daemon.kill();
    let _ = daemon.wait();

    println!("=== PROFILING COMPLETE ===");
    Ok(())
}
