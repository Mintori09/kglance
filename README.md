# Kglance (Oxiview)

![image](./data/image.png)

A high-performance file preview application for **KDE Plasma 6** built in Rust and Iced. Inspired by macOS QuickLook, Kglance provides near-instantaneous file previews via a single keypress.

It operates in two modes:

- **Daemon Mode**: Long-running background service listening on DBus for instant window toggle (<10ms UI latency).
- **Standalone Mode**: Opens directly for previewing a file without requiring the daemon process, automatically exiting upon close.

---

## Key Features

- **Source Code & Text**: Syntax highlighting via `syntect` with line numbers, text search (`Ctrl+F`), and word wrap toggle.
- **Images**: Fast rendering of PNG, JPEG, WebP, GIF, BMP, and SVG (`resvg`). Full support for zoom, pan, rotation, and detailed EXIF metadata sidebar.
- **Documents & Office**: PDF continuous scrolling, page navigation, and thumbnail sidebar; text extraction for DOCX & XLSX with LibreOffice fallback; `.typ` document rendering via `typst` CLI (if installed).
- **Archives**: Interactive folder tree view for ZIP, Tar, GZ, and 7z archives with inner file preview.
- **Audio & Video**: Metadata extraction, waveform visualization, and inline media playback using GStreamer pipelines.
- **Fonts**: Font sample rendering (TTF, OTF, WOFF) and metadata display (`fontdue`).
- **KDE Plasma 6 Integration**: Automatic Dark/Light mode theme sync, Dolphin file manager integration (Space key preview via KIO Service Menu), and autostart daemon.

---

## Supported File Formats

| Category         | Formats                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| ---------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Code**         | `.rs`, `.py`, `.js`, `.ts`, `.jsx`, `.tsx`, `.html`, `.css`, `.scss`, `.json`, `.toml`, `.yml/.yaml`, `.xml`, `.sh`, `.bash`, `.zsh`, `.fish`, `.c`, `.h`, `.cpp`, `.hpp`, `.java`, `.kt`, `.swift`, `.go`, `.rb`, `.php`, `.pl`, `.pm`, `.lua`, `.r`, `.sql`, `.graphql`, `.proto`, `.tex`, `.bib`, `.dockerfile`, `.makefile`, `.cmake`, `.gradle`, `.cfg`, `.ini`, `.conf`, `.txt`, `.log`, `.diff`, `.patch`, `.vim`, `.ps1`, `.bat` |
| **Markdown**     | `.md`, `.markdown`, `.mdown`, `.mdwn`, `.mkd`, `.mkdn`                                                                                                                                                                                                                                                                                                                                                                                   |
| **Images**       | `.png`, `.jpg`, `.jpeg`, `.webp`, `.gif`, `.bmp`, `.ico`                                                                                                                                                                                                                                                                                                                                                                                 |
| **Vector**       | `.svg`                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| **Krita**        | `.kra`                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| **OpenRaster**   | `.ora`                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| **Documents**    | `.pdf`, `.epub`, `.typ` (requires `typst`)                                                                                                                                                                                                                                                                                                                                                                                               |
| **Office**       | `.docx`, `.xlsx`, `.pptx`, `.odt`, `.ods`, `.odp`                                                                                                                                                                                                                                                                                                                                                                                        |
| **Spreadsheets** | `.csv`, `.xlsx`, `.ods`                                                                                                                                                                                                                                                                                                                                                                                                                  |
| **Archives**     | `.zip`, `.tar`, `.gz`, `.tgz`, `.xz`, `.txz`, `.7z`                                                                                                                                                                                                                                                                                                                                                                                      |
| **Fonts**        | `.ttf`, `.otf`, `.woff`, `.woff2`                                                                                                                                                                                                                                                                                                                                                                                                        |
| **Audio**        | `.mp3`, `.wav`, `.flac`, `.ogg`, `.aac`, `.m4a`, `.opus`                                                                                                                                                                                                                                                                                                                                                                                 |
| **Video**        | `.mp4`, `.mkv`, `.avi`, `.mov`, `.wmv`, `.webm`, `.flv`, `.m4v`                                                                                                                                                                                                                                                                                                                                                                          |
| **Folders**      | Any directory — browse and navigate its contents                                                                                                                                                                                                                                                                                                                                                                                         |

Files without a matching extension fall back to plain text rendering.

---

## Dependencies

### System Build Dependencies

To build Kglance from source on Linux (Debian/Ubuntu/Arch/Fedora), the following system development libraries are required:

| Component          | Library Dependency                                        | Description / Usage                           |
| ------------------ | --------------------------------------------------------- | --------------------------------------------- |
| **Fonts & Layout** | `libfontconfig1-dev` / `fontconfig`                       | Font matching and fallback configuration      |
| **FreeType**       | `libfreetype6-dev` / `freetype2`                          | Font rendering engine for text/font previews  |
| **XKB Common**     | `libxkbcommon-dev` / `libxkbcommon`                       | Keyboard keycode handling for Wayland & X11   |
| **GStreamer**      | `libgstreamer1.0-dev`, `libgstreamer-plugins-base1.0-dev` | Audio and video decoding/playback pipeline    |
| **MuPDF**          | `libmupdf-dev` _(optional system bind)_                   | PDF rendering engine                          |
| **Typst**          | `typst` _(optional CLI binary)_                           | Typst (`.typ`) document compilation & preview |

#### Installing Dependencies

- **Arch Linux:**

  ```bash

  sudo pacman -Syu fontconfig freetype2 libxkbcommon gstreamer gst-plugins-base gst-plugins-good gst-plugins-bad gst-libav gst-plugin-va gst-plugins-ugly

  ```

- **Ubuntu / Debian:**

  ```bash

  sudo apt install libfontconfig1-dev libfreetype-dev libxkbcommon-dev libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev gstreamer1.0-plugins-good gstreamer1.0-plugins-bad

  ```

- **Fedora:**

  ```bash

  sudo dnf install fontconfig-devel freetype-devel libxkbcommon-devel gstreamer1-devel gstreamer1-plugins-base-devel gstreamer1-plugins-good gstreamer1-plugins-bad-free gstreamer1-plugins-bad-free-devel

  ```

---

## Keyboard Shortcuts

Kglance offers rich keyboard navigation for navigating files, zooming images, scrolling PDFs, and searching text.

| Shortcut                       | Action                                                   | Scope / Context                  |
| :----------------------------- | :------------------------------------------------------- | :------------------------------- |
| **General & Window**           |                                                          |                                  |
| `Space` / `Escape`             | Close preview window (or close search if open)           | Global                           |
| `Tab`                          | Toggle View Mode (Detail ↔ Grid)                         | Global                           |
| `Enter`                        | Open file in default desktop app / Open folder entry     | Global / Folder view             |
| `Ctrl` + `,`                   | Open Settings view                                       | Global                           |
| `Ctrl` + `T`                   | Cycle color theme (Dark → Light → Nord)                  | Global                           |
| `Ctrl` + `C`                   | Copy file path / Copy selected text                      | Global / Text & Documents        |
| `Ctrl` + `A`                   | Select all text                                          | Text, Code, Markdown, EPUB       |
| **Navigation & Playlist**      |                                                          |                                  |
| `←` / `Left Arrow`             | Previous file in directory / Previous chapter / Seek -5s | Playlist / EPUB / Video          |
| `→` / `Right Arrow`            | Next file in directory / Next chapter / Seek +5s         | Playlist / EPUB / Video          |
| `←` / `→`                      | Go to parent directory / Open folder or preview file     | Folder view                      |
| **Scrolling & Vim Navigation** |                                                          |                                  |
| `↑` / `k`                      | Scroll up (80px)                                         | Scrollable content               |
| `↓` / `j`                      | Scroll down (80px)                                       | Scrollable content               |
| `PageUp` / `u`                 | Scroll half page up                                      | Scrollable content               |
| `PageDown` / `d`               | Scroll half page down                                    | Scrollable content               |
| `gg` / `Home` (double-tap)     | Scroll to top                                            | Scrollable content               |
| `G` / `Shift` + `G` / `End`    | Scroll to bottom                                         | Scrollable content               |
| `gt`                           | Toggle Table of Contents / Sidebar                       | Markdown, EPUB, PDF              |
| `Mouse Wheel`                  | Scroll / Zoom (while holding `Ctrl`)                     | All previews                     |
| **Search**                     |                                                          |                                  |
| `Ctrl` + `F` / `/`             | Open search bar                                          | Text, Markdown, JSON, CSV, Grid  |
| `Escape`                       | Close search bar and clear query                         | Search active                    |
| Type characters                | Immediate type-to-search                                 | Spreadsheets, Grid view          |
| **Zoom & Font Scaling**        |                                                          |                                  |
| `Ctrl` + `+` / `Ctrl` + `=`    | Zoom in / Increase font size                             | Image, PDF, Text, Code, Markdown |
| `Ctrl` + `-`                   | Zoom out / Decrease font size                            | Image, PDF, Text, Code, Markdown |
| `Ctrl` + Scroll                | Zoom in/out / Resize                                     | Image, PDF, Text, Code, Markdown |
| `=`                            | Fit image to window                                      | Image preview                    |
| `Shift` + `+` / `Shift` + `=`  | Reset font size to default (14px)                        | Text & Code preview              |
| **Specific Formats**           |                                                          |                                  |
| `Ctrl` + `W`                   | Toggle word wrap                                         | Text, Code, JSON, Typst          |
| `Ctrl` + `e`                   | Expand all JSON tree nodes                               | JSON preview                     |
| `Ctrl` + `E` (`Shift` + `e`)   | Collapse all JSON tree nodes                             | JSON preview                     |
| `Ctrl` + `P` (`Shift` + `p`)   | Toggle JSON display (Tree vs Raw formatted)              | JSON preview                     |
| `s` / `S`                      | Toggle between Compiled Preview and Source               | Typst (`.typ`) preview           |

---

## Installation & Setup

### 1. Build from Source

Ensure Rust 1.85+ (Edition 2024) is installed.

```bash
cargo build --release
```

The resulting binary will be at `target/release/kglance`.

### 2. Install Dolphin Integration (KIO Service Menu)

> **Prerequisite**: Ensure `kglance` is in your `PATH` or set `BIN=/path/to/kglance`.

To enable pressing **Space** in Dolphin to preview files, use the setup script:

```bash
# If you have the repo cloned:
./scripts/dolphin-setup.sh install

# Or fetch directly from GitHub:
bash <(curl -s https://raw.githubusercontent.com/Mintori09/kglance/main/scripts/dolphin-setup.sh) install
```

Restart Dolphin (`killall dolphin`) or log out and back in to apply the changes.

### 3. Configure Dolphin Keyboard Shortcut (Option)

To preview files by pressing **Space**:

1. Open Dolphin → **Settings** → **Configure Keyboard Shortcuts**…
2. Search for `Quick Preview` or `Quick Preview (KIO)`
3. Assign the **Space** key as the shortcut

Now select any supported file and press **Space** to preview.

To remove:

```bash
./scripts/dolphin-setup.sh uninstall

# Or fetch directly from GitHub:
bash <(curl -s https://raw.githubusercontent.com/Mintori09/kglance/main/scripts/dolphin-setup.sh) uninstall
```

### 4. Setup Yazi Terminal File Manager Integration (Option)

To integrate Kglance with [Yazi](https://github.com/sxyazi/yazi) for real-time hover sync and `<Space>` preview:

#### Step 1: Download the plugin

Download `main.lua` into your Yazi plugins directory:

```bash
mkdir -p ~/.config/yazi/plugins/kglance.yazi
curl -fsSL https://raw.githubusercontent.com/Mintori09/kglance/main/kglance.yazi/main.lua -o ~/.config/yazi/plugins/kglance.yazi/main.lua
```

#### Step 2: Enable hover sync in `init.lua`

Add the following line to `~/.config/yazi/init.lua` (create the file if it does not exist):

```lua
require("kglance"):setup()
```

#### Step 3: Configure `<Space>` shortcut in `keymap.toml`

Add the keymap under `[[mgr.prepend_keymap]]` in `~/.config/yazi/keymap.toml`:

```toml
[[mgr.prepend_keymap]]
on   = [ "<Space>" ]
run  = "shell --orphan -- kglance %h"
desc = "Open hovered file with kglance"
```

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

## Configuration

Kglance reads a JSON config file from:

| Platform  | Path                            |
| --------- | ------------------------------- |
| **Linux** | `~/.config/kglance/config.json` |

The config file is auto-created with defaults on first run. See `data/examples/config.example.json` for all available options:

```json
{
  "ui": {
    "theme": "Dark",
    "font_size": 14.0,
    "default_width": 1024,
    "default_height": 768,
    "min_width": 800,
    "min_height": 600,
    "font_family": "Noto Sans",
    "font_family_mono": "Fira Code",
    "epub_font_family": "Noto Serif",
    "max_text_width": 820.0,
    "prefer_mermaid_cli": false,
    "word_wrap": false,
    "json_tree_view": false,
    "max_preview_size_mb": null
  }
}
```

For a full reference, see [`data/examples/config.example.json`](data/examples/config.example.json).

---

## Roadmap

- [x] implement selection text for epub. (without virtual loading)

---

## License

Distributed under the GNU AGPL-3.0-only. See `LICENSE` for more information.
