# Kglance — Development Roadmap

> **Goal:** Become the macOS **QuickLook + Preview.app** equivalent for KDE Plasma 6 — providing instant file previews via a single keypress.

---

## Product Vision

| Feature | macOS QuickLook | macOS Preview.app | Kglance (Target) |
| --- | --- | --- | --- |
| **Activation** | Space key in Finder | Direct file open | Space key in Dolphin / CLI |
| **Speed** | Instant (<100ms) | Fast | Sub-10ms (Daemon mode) |
| **Modes** | Preview only | Edit / Annotate | QuickLook + Interactive Preview |
| **Formats** | Text, Image, PDF, Video, Archive | Image + PDF | Code, Image, PDF, Archive, Media, Office, Font |

---

## Feature Roadmap & Status

### Phase 1 — Core QuickLook Experience (v0.2.0)
- [x] Client-Daemon architecture over DBus (`zbus`)
- [x] Standalone fallback mode
- [x] Dolphin integration via KIO Service Menu (`Space` key trigger)
- [x] Text & source code syntax highlighting (`syntect`)
- [x] Image viewer (PNG, JPEG, WebP, GIF, BMP, SVG) with zoom, pan, and rotation
- [x] Continuous PDF rendering and page navigation (`mupdf`)
- [x] Archive tree view (ZIP, TAR, 7z) with inner file preview

### Phase 2 — Multimedia & Document Previews (v0.3.0)
- [x] EXIF metadata extraction & display sidebar
- [x] Audio metadata extraction & waveform display container
- [x] Video preview decoding & frame playback via GStreamer
- [x] Office document text extraction (DOCX, XLSX) & LibreOffice fallback
- [x] Markdown rendering with code block highlighting
- [x] Font sample rasterization & metadata view (`fontdue`)

### Phase 3 — Advanced Interactive Preview (v0.4.0)
- [x] Text searching (`Ctrl+F`) & line word wrap toggle
- [x] Multi-file navigation (Left/Right arrows in directory context)
- [x] History stack (Back/Forward navigation)
- [x] Automatic KDE Plasma Dark/Light theme synchronization

### Phase 4 — System Integration & Desktop Polish (v0.5.0)
- [ ] KDE Thumbnail Provider registration
- [ ] Freedesktop `xdg-desktop-portal` integration
- [ ] Package distributions (AUR, Flatpak, `.deb`, `.rpm`)
