# System Architecture — Kglance (Oxiview)

Kglance operates on a **Client–Daemon** architecture over **DBus** to achieve sub-10ms UI latency and minimize resource consumption when repeatedly previewing files.

---

## Directory Structure

```text
Kglance/
├── Cargo.toml
├── AGENTS.md
├── ARCHITECTURE.md
├── PERFORMANCE.md
├── PROJECT_GUIDE.md
├── README.md
├── roadmap.md
├── data/
│   ├── kglance.desktop          # AppImage & application menu entry
│   └── kglance-rust.desktop     # KIO Service Menu for Dolphin
└── src/
    ├── main.rs                  # Entrypoint (dispatches daemon vs standalone mode)
    ├── lib.rs                   # Library module exports
    ├── logger.rs                # Timestamped logging macros
    ├── app/                     # Core Application state, Message enum, update(), view()
    │   ├── mod.rs
    │   ├── keyboard.rs          # Keyboard shortcut handling
    │   ├── media.rs             # Media playback integration
    │   ├── probe.rs             # File probing and validation
    │   └── window.rs            # Window management
    ├── core/                    # Core business logic (Clean Architecture)
    │   ├── mod.rs
    │   ├── config.rs            # App configuration & manager
    │   ├── config_watcher.rs    # Dynamic configuration watcher
    │   ├── file_watcher.rs      # Filesystem change watcher
    │   ├── handlers/            # Event handling logic
    │   │   ├── mod.rs
    │   │   └── pdf.rs           # PDF page rendering handler
    │   ├── navigation.rs        # File navigation (prev/next, grid, siblings)
    │   ├── preloader.rs         # Proactive file preloading
    │   ├── preview.rs           # FilePreviewer trait & PreviewData definitions
    │   └── types.rs             # Data structures (KglanceState, ImageState, TextState, …)
    ├── dbus/                    # Inter-process communication
    │   ├── mod.rs               # Client: connect & call ShowPreview
    │   ├── recipe.rs            # Iced subscription Recipe for DBus events
    │   └── service.rs           # Server: zbus interface exposing ShowPreview
    ├── parsers/                 # File format extractors → PreviewData
    │   ├── mod.rs               # ParserRegistry & format matching
    │   ├── archive.rs           # ZIP, TAR, 7Z tree parser & entry extraction
    │   ├── audio.rs             # Audio metadata & waveform generation
    │   ├── csv.rs               # CSV / Spreadsheet parser
    │   ├── epub.rs              # EPUB document parser
    │   ├── folder.rs            # Directory item listing & sorting
    │   ├── font.rs              # Font sample rasterization & metadata
    │   ├── image.rs             # Image & EXIF metadata parser
    │   ├── json.rs              # JSON tree viewer
    │   ├── markdown.rs          # Markdown block parsing & syntax highlighting
    │   ├── office.rs            # DOCX, XLSX, ODT extraction & LibreOffice fallback
    │   ├── pdf.rs               # MuPDF page rendering
    │   ├── svg.rs               # SVG vector rasterization (resvg)
    │   ├── text.rs              # Source code syntax highlighter (syntect)
    │   └── video.rs             # Video metadata & thumbnail extraction
    ├── preview/                 # Image preview pipeline
    │   ├── mod.rs
    │   └── image/
    │       ├── mod.rs
    │       ├── camera.rs        # EXIF camera metadata extraction
    │       ├── canvas.rs        # Canvas rendering logic
    │       ├── controller.rs    # Zoom/pan/rotation control
    │       ├── renderer.rs      # Image rendering pipeline
    │       └── state.rs         # Image load state management
    └── ui/                      # Iced view components
        ├── mod.rs               # Module entrypoint & helpers
        ├── window.rs            # Window-level layout
        ├── components/          # Reusable UI widgets
        │   ├── mod.rs
        │   └── button.rs        # Custom button widgets
        └── handlers/            # Async media processing
            ├── mod.rs
            └── image.rs         # Image loading handler
```

---

## Execution Modes

1. **Daemon Mode (`kglance daemon`):**
   Runs as a background process initialized at system boot. Keeps the UI window warm in memory for instant show/hide toggling (<10ms response time) triggered via DBus calls.

2. **Standalone Mode (`kglance <file>`):**
   Parses and displays the specified file directly without needing the DBus daemon. Exits automatically when the window is closed. Automatically falls back to Standalone mode if the Daemon is not currently active.

---

## System Workflow & Core Components

### 1. Window & Event Management (Iced)
- Window instances are preserved in memory during Daemon execution.
- Pressing `Esc` or losing focus triggers a window `hide()` instead of terminating the process.
- UI state is centralized in `KglanceState` with sub-states:
  - `ImageState`: Handles zoom, pan, rotation, and EXIF metadata sidebar.
  - `TextState`: Manages scroll line numbers, word wrap, and search queries.
  - `PdfState`: Manages active page, page cache, thumbnails, and continuous scroll.
  - `TableState`: Manages file list rows for directory/archive navigation and column sorting.
  - `MediaState`: Controls audio/video playback, seeking, and progress tracking.
  - `HistoryState`: Tracks preview navigation stack for back/forward operations.

### 2. DBus IPC Layer (`zbus`)
- **Service Name:** `org.mintori.Kglance` registered on the Session Bus.
- **Exposed Methods:**
  - `ShowPreview(file_path: String)` — opens preview for a single file.
  - `ShowMultiplePreviews(files: Vec<String>)` — opens preview with sibling navigation.

### 3. KDE Plasma 6 & Dolphin Integration
Integration is achieved via a custom KIO Service Menu (`~/.local/share/kio/servicemenus/kglance-rust.desktop`) mapping the `Space` shortcut in Dolphin to `kglance "%f"`.

---

## Data Flow

```text
Dolphin (Space key)
      │
      ▼
  kglance <file>
      │
      ├── Daemon running? ──Yes──► DBus call: ShowPreview(path)
      │                               │
      │                               ▼
      │                          DBus Service (zbus)
      │                               │
      │                         ParserRegistry.match()
      │                               │
      │                         FilePreviewer.parse()
      │                               │
      │                         PreviewData
      │                               │
      │                         mpsc channel ──► iced event loop
      │                                               │
      │                                         update()
      │                                               │
      │                                         view()
      │                                               │
      │                                         hide() / exit
      │
      └── No ──► Standalone mode
                    │
               direct parse + show
```

## Key Design Principles

- **Daemon runs concurrently** — the zbus event loop lives in a separate tokio task, keeping UI responsive.
- **Preview is parsed before the window opens** — the DBus service parses the file, then sends the `PreviewData` over an mpsc channel. The window never blocks on I/O.
- **Sibling navigation** — the daemon can precompute a playlist of adjacent files, enabling instant Prev/Next without re-parsing.
- **Parser registry is extensible** — `ParserRegistry` iterates registered parsers by extension and `is_supported()`, returning the first match.
