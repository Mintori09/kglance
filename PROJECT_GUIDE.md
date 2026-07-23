# Kglance Developer Guide

## System Requirements

- **KDE Plasma 6** (Wayland or X11)
- **Rust 1.85+** (Edition 2024)

## Architecture Overview

Kglance uses a **Client-Daemon** architecture over DBus (`zbus`).

```text
File Manager (Dolphin)
         │
         ▼ (Space Key / KIO Service Menu)
    DBus Client
         │
         ▼ (org.mintori.Kglance)
   Kglance Daemon
         │
         ▼
   Preview Engine
         │
         ▼
      Iced UI
```

- **Daemon Mode (`kglance daemon`):** Background process listening on DBus for instant window toggle (<10ms UI latency).
- **Standalone Mode (`kglance <path>`):** Direct preview window creation without requiring DBus daemon.

## Build & Test Commands

```bash
# Check code
cargo check

# Run Clippy lints
cargo clippy --all-targets -- -D warnings

# Format code
cargo fmt

# Run unit tests
cargo test

# Build release binary
cargo build --release
```
