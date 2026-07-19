#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {{
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        println!("[{}] [INFO] {}", now, format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {{
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        eprintln!("[{}] [ERROR] {}", now, format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {{
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        println!("[{}] [DEBUG] {}", now, format!($($arg)*));
    }};
}
