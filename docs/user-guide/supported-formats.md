# Supported File Formats

Kglance automatically routes files to specialized parsers and rendering features based on their file extension and MIME type.

---

## Format Matrix

| Category | File Extensions | Engine / Parser Library | Key Capabilities |
| :--- | :--- | :--- | :--- |
| **Source Code & Text** | `.rs`, `.py`, `.js`, `.ts`, `.jsx`, `.tsx`, `.c`, `.cpp`, `.h`, `.hpp`, `.go`, `.java`, `.kt`, `.swift`, `.rb`, `.php`, `.lua`, `.sh`, `.bash`, `.zsh`, `.fish`, `.html`, `.css`, `.scss`, `.sql`, `.graphql`, `.proto`, `.vim`, `.ps1`, `.bat`, `.dockerfile`, `.makefile`, `.cmake`, `.gradle`, `.cfg`, `.ini`, `.conf`, `.txt`, `.log`, `.diff`, `.patch` | `syntect` + Virtualized Line Renderer | Fast syntax highlighting, line numbers, word wrap, regex/text search (`Ctrl+F`), font scaling, multiline text selection & floating action menu. |
| **Markdown** | `.md`, `.markdown`, `.mdown`, `.mdwn`, `.mkd`, `.mkdn` | `pulldown-cmark` | Full CommonMark + GFM (tables, interactive task lists, strikethrough), math formulas, code blocks, Table of Contents sidebar (`gt`), HTML `<img>` rendering (with dimensions, styles, base64 data URIs), animated GIF rendering, Furigana / Ruby text (`<ruby>`), adjustable reading width, multiline text selection & floating action menu. |
| **Raster Images** | `.png`, `.jpg`, `.jpeg`, `.webp`, `.gif`, `.bmp`, `.ico` | `image` crate | GPU accelerated rendering, zoom, pan, lossless rotation (`r`/`Shift+R`), flip horizontal/vertical (`h`/`v`), reset (`0`), copy bitmap pixels directly to clipboard (`Ctrl+C`), animated GIF frame timing playback engine, EXIF metadata sidebar (`i`). |
| **Vector Graphics**| `.svg` | `resvg` / `usvg` | Crisp infinite zooming and vector fidelity. |
| **Creative Files** | `.psd` (Photoshop), `.kra` (Krita), `.ora` (OpenRaster) | Pure-Rust PSD parser / Zip extraction (`mergedimage.png`) | Instant high-res merged raster preview without launching Photoshop or Krita. |
| **PDF Documents** | `.pdf` | `mupdf` / pure-rust pipeline | Multi-page continuous scrolling, thumbnail sidebar, page navigation, zoom, text selection (bounding box detection, double/triple click), copy plain text & rich HTML (tables, paragraphs) to clipboard, floating selection context menu. |
| **EPUB Books** | `.epub` | `rbook` / `epub` parser | Chapter navigation, Table of Contents, reader typography, single chapter & continuous reading modes with position preservation, multiline text selection, Furigana / Ruby text rendering. |
| **Typst Documents**| `.typ` | `typst` CLI | Instant side-by-side or toggleable compiled preview vs raw source code (`s` key). |
| **Databases** | `.sqlite`, `.sqlite3`, `.db`, `.db3`, `.s3db`, `.sl3` | `rusqlite` | Read-only inspection of tables/views rendered into multi-tab interactive spreadsheets with cell truncation and schema info. |
| **Spreadsheets** | `.csv`, `.tsv`, `.xlsx`, `.ods` | `calamine` / `csv` | Interactive data table with column resize, search, sheet selector, dynamic font size scaling (`Ctrl+=`, `Ctrl+-`, `Ctrl+Wheel`). |
| **Structured Data**| `.json` | `serde_json` | Dual mode: interactive collapsible tree viewer (`Ctrl+P`, `Ctrl+e`, `Ctrl+E`) or syntax-highlighted formatted JSON with text search. |
| **Office Documents**| `.docx`, `.pptx`, `.odt`, `.odp` | Pure-rust extraction + LibreOffice fallback | Extracts formatted text/layout; falls back to LibreOffice PDF export if installed. |
| **Archives** | `.zip`, `.tar`, `.gz`, `.tgz`, `.xz`, `.txz`, `.7z` | `zip`, `tar`, `sevenz-rust` | Interactive hierarchical folder tree, archive statistics, entry navigation. |
| **Fonts** | `.ttf`, `.otf`, `.woff`, `.woff2` | `fontdue` | Rendered glyph waterfalls, character maps, font metadata (Family, PostScript name, weights). |
| **Audio** | `.mp3`, `.wav`, `.flac`, `.ogg`, `.aac`, `.m4a`, `.opus` | `gstreamer` + `symphonia` | Audio playback, waveform visualization, album artwork extraction, ID3 tags. |
| **Video** | `.mp4`, `.mkv`, `.avi`, `.mov`, `.wmv`, `.webm`, `.flv`, `.m4v` | `gstreamer` | Hardware-accelerated video playback, transport controls, audio track selection. |
| **Directories** | Folders | `std::fs` | Interactive folder browser, file counts, total size calculation, quick traversal. |

---

## Fallback Strategy

If a file has an unknown extension or unsupported binary format:
1. **Plain Text Check**: Kglance checks if the file content is valid UTF-8 text. If yes, it falls back to the generic text viewer with full syntax search, line numbers, and navigation support.
2. **Binary Inspection**: If the file contains non-printable binary bytes, Kglance shows a file summary and metadata inspector rather than crashing or freezing.
