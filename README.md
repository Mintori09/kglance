# Kglance (Oxiview)

A high-performance file preview application for **KDE Plasma 6** built in Rust and Iced. Inspired by macOS QuickLook, Kglance provides near-instantaneous file previews via a single keypress.

It operates in two modes:

- **Daemon Mode**: Long-running background service listening on DBus for instant window toggle (<10ms UI latency).
- **Standalone Mode**: Opens directly for previewing a file without requiring the daemon process, automatically exiting upon close.

---

## Key Features

- **Source Code & Text**: Syntax highlighting via `syntect` with line numbers, text search (`Ctrl+F`), and word wrap toggle.
- **Images**: Fast rendering of PNG, JPEG, WebP, GIF, BMP, and SVG (`resvg`). Full support for zoom, pan, rotation, and detailed EXIF metadata sidebar.
- **Documents & Office**: PDF continuous scrolling, page navigation, and thumbnail sidebar; text extraction for DOCX & XLSX with LibreOffice fallback.
- **Archives**: Interactive folder tree view for ZIP, Tar, GZ, and 7z archives with inner file preview.
- **Audio & Video**: Metadata extraction, waveform visualization, and inline media playback using GStreamer pipelines.
- **Fonts**: Font sample rendering (TTF, OTF, WOFF) and metadata display (`fontdue`).
- **KDE Plasma 6 Integration**: Automatic Dark/Light mode theme sync, Dolphin file manager integration (Space key preview via KIO Service Menu), and autostart daemon.

---

## Dependencies

### System Build Dependencies

To build Kglance from source on Linux (Debian/Ubuntu/Arch/Fedora), the following system development libraries are required:

| Component          | Library Dependency                                        | Description / Usage                          |
| ------------------ | --------------------------------------------------------- | -------------------------------------------- |
| **Fonts & Layout** | `libfontconfig1-dev` / `fontconfig`                       | Font matching and fallback configuration     |
| **FreeType**       | `libfreetype6-dev` / `freetype2`                          | Font rendering engine for text/font previews |
| **XKB Common**     | `libxkbcommon-dev` / `libxkbcommon`                       | Keyboard keycode handling for Wayland & X11  |
| **GStreamer**      | `libgstreamer1.0-dev`, `libgstreamer-plugins-base1.0-dev` | Audio and video decoding/playback pipeline   |
| **MuPDF**          | `libmupdf-dev` _(optional system bind)_                   | PDF rendering engine                         |

#### Installing Dependencies

- **Arch Linux:**
  ```bash
  sudo pacman -Syu fontconfig freetype2 libxkbcommon gstreamer gst-plugins-base gst-plugins-good gst-plugins-bad gst-libav gst-plugin-va gst-plugins-ugly
  ```
- **Ubuntu / Debian:**
  ```bash
  sudo apt install libfontconfig1-dev libfreetype6-dev libxkbcommon-dev libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev gstreamer1.0-plugins-good gstreamer1.0-plugins-bad
  ```
- **Fedora:**
  ```bash
  sudo dnf install fontconfig-devel freetype-devel libxkbcommon-devel gstreamer1-devel gstreamer1-plugins-base-devel gstreamer1-plugins-good
  ```

---

## Keyboard Shortcuts

Kglance offers rich keyboard navigation for navigating files, zooming images, scrolling PDFs, and searching text.

| Shortcut                         | Action                                           | Scope / Context                      |
| -------------------------------- | ------------------------------------------------ | ------------------------------------ |
| `Space` / `Escape`               | Close preview window                             | Global                              |
| `Ctrl` + `C`                     | Copy file path / Copy selected text              | Global / Text preview               |
| `Ctrl` + `A`                     | Select all text                                  | Text / Code preview                  |
| `Ctrl` + `F`                     | Open text search bar                             | Text / Code preview                  |
| `Ctrl` + `W`                     | Toggle line word wrap                            | Text / Code preview                  |
| `Ctrl` + `+` / `Ctrl` + `=`     | Zoom in / Increase font size                     | Image / PDF / Text preview           |
| `Ctrl` + `-`                     | Zoom out / Decrease font size                    | Image / PDF / Text preview           |
| `Ctrl` + `0`                     | Reset zoom to 100%                               | Image preview                        |
| `Shift` + `+` / `Shift` + `=`   | Reset font size to 14px                          | Text / Code preview                  |
| `Ctrl` + `T`                     | Toggle dark / light theme                        | Global                               |
| `←` / `→`                        | Go to parent dir / Preview selected file         | Folder view                          |
| `Left Arrow` / `PageUp`         | Previous file in directory / Previous PDF page   | Directory / Multi-file / PDF         |
| `Right Arrow` / `PageDown`      | Next file in directory / Next PDF page           | Directory / Multi-file / PDF         |
| `Enter`                          | Preview selected file / Open externally          | Folder view / Global                |
| `Arrow Up` / `k`                | Scroll up                                        | Scrollable content                   |
| `Arrow Down` / `j`              | Scroll down                                      | Scrollable content                   |
| `PageUp` / `u`                  | Scroll half page up                              | Scrollable content                   |
| `PageDown` / `d`                | Scroll half page down                            | Scrollable content                   |
| `gg` (double-tap `g`)            | Scroll to top                                    | Scrollable content                   |
| `G` / `Shift` + `g`             | Scroll to bottom                                 | Scrollable content                   |
| `Home` (double-tap)              | Scroll to top                                    | Scrollable content                   |
| `End`                            | Scroll to bottom                                 | Scrollable content                   |
| `Mouse Wheel`                    | Scroll / Zoom (with `Ctrl`)                      | All previews                         |

---

## Installation & Setup

### 1. Build from Source

Ensure Rust 1.85+ (Edition 2024) is installed.

```bash
cargo build --release
```

The resulting binary will be at `target/release/kglance`.

### 2. Configure Autostart (Daemon Mode)

To start the Kglance daemon automatically on KDE login:

```bash
mkdir -p ~/.config/autostart
cp data/kglance-daemon.desktop ~/.config/autostart/
```

_(Ensure the `Exec` path in the desktop file points to your `kglance` binary location)_

### 3. Install Dolphin Integration (KIO Service Menu)

To enable pressing **Space** in Dolphin to preview files:

```bash
mkdir -p ~/.local/share/kio/servicemenus
cp data/kglance-rust.desktop ~/.local/share/kio/servicemenus/
```

Restart Dolphin or log out and back in to apply the changes.

---

## Usage

```bash
# Start background Daemon (Listens on DBus: org.mintori.Kglance)
kglance daemon

# Preview a file (Auto-detects Daemon or falls back to Standalone)
kglance /path/to/file

# Force Standalone mode
kglance --standalone /path/to/file
```

---

## Project Architecture

```text
src/
├── app.rs            # Application state, Message handling & Iced event routing
├── lib.rs            # Core library entrypoint
├── main.rs           # Entrypoint CLI & Daemon/Standalone dispatcher
├── logger.rs         # Logging macros
├── core/             # Business logic (Clean Architecture)
│   ├── config.rs     # Application configuration
│   ├── handlers.rs   # Async handlers
│   ├── preview.rs    # FilePreviewer trait & PreviewData definitions
│   └── types.rs      # App state definitions (ImageState, TextState, etc.)
├── dbus/             # DBus service implementation (zbus)
├── parsers/          # Independent file format parsers
│   ├── archive.rs    # Zip, Tar, 7z tree parser
│   ├── audio.rs      # Audio metadata & waveform decoder
│   ├── image.rs      # Image & EXIF metadata parser
│   ├── markdown.rs   # Markdown block & syntax parser
│   ├── office.rs     # Office document extractor
│   ├── pdf.rs        # MuPDF continuous page renderer
│   ├── text.rs       # Syntax highlighting code parser
│   └── video.rs      # Video decoder & frame grabber
└── ui/               # Iced GUI views, themes, and custom widgets
```

---

## License

Distributed under the AGPL-3.0-only. See `LICENSE` for more information.
