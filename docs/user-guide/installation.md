# Installation Guide

This guide explains how to install system build dependencies, compile Kglance from source, and set up optional external format renderers.

---

## 1. System Build Dependencies

Kglance uses hardware acceleration via [Iced](https://github.com/iced-rs/iced), font rendering engines, and multimedia decoding pipelines.

### Arch Linux / Manjaro

```bash
sudo pacman -Syu --needed \
    base-devel \
    fontconfig \
    freetype2 \
    libxkbcommon \
    gstreamer \
    gst-plugins-base \
    gst-plugins-good \
    gst-plugins-bad \
    gst-libav \
    gst-plugin-va \
    gst-plugins-ugly \
    libreoffice-fresh \
    typst
```

### Ubuntu / Debian (22.04 LTS or newer)

```bash
sudo apt update && sudo apt install -y \
    build-essential \
    pkg-config \
    libfontconfig1-dev \
    libfreetype-dev \
    libxkbcommon-dev \
    libgstreamer1.0-dev \
    libgstreamer-plugins-base1.0-dev \
    gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad \
    libreoffice \
    typst
```

### Fedora / RHEL

```bash
sudo dnf install -y \
    gcc \
    pkg-config \
    fontconfig-devel \
    freetype-devel \
    libxkbcommon-devel \
    gstreamer1-devel \
    gstreamer1-plugins-base-devel \
    gstreamer1-plugins-good \
    gstreamer1-plugins-bad-free \
    gstreamer1-plugins-bad-free-devel \
    libreoffice \
    typst
```

---

## 2. Compiling from Source

Kglance requires Rust 1.85+ (Rust Edition 2024).

```bash
# Clone repository
git clone https://github.com/Mintori09/kglance.git
cd kglance

# Build release binary (uses mold linker by default if available)
cargo build --release
```

The compiled binary will be placed at `target/release/kglance`.

To install the binary globally to your user path:

```bash
install -Dm755 target/release/kglance ~/.local/bin/kglance
```

> [!NOTE]
> Make sure `~/.local/bin` is in your `$PATH` environment variable.

---

## 3. Optional External Tools

* **Typst CLI (`typst`)**: Enables live rendering of `.typ` documents into vector/bitmap previews.
* **LibreOffice (`libreoffice`)**: Provides fallback conversions for legacy Office formats (`.docx`, `.pptx`, `.odt`, `.ods`, `.odp`).
* **Mermaid CLI (`mmdc`)**: Enables rendering embedded mermaid diagrams in Markdown when `prefer_mermaid_cli` is enabled.
