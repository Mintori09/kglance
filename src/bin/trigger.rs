use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: kiview-trigger <file-path>");
        std::process::exit(1);
    }

    let path = &args[1];
    let status = Command::new("busctl")
        .args([
            "--user",
            "call",
            "org.mintori.KiviewRust",
            "/org/mintori/KiviewRust",
            "org.mintori.KiviewRust",
            "ShowPreview",
            "s",
            path,
        ])
        .status();

    match status {
        Ok(s) if s.success() => {}
        Ok(s) => {
            eprintln!("busctl exited with code: {}", s);
            std::process::exit(s.code().unwrap_or(1));
        }
        Err(e) => {
            eprintln!("Failed to execute busctl: {e}");
            std::process::exit(1);
        }
    }
}
