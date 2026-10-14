# Keyboard Shortcuts Cheatsheet

Kglance is designed for keyboard-first efficiency. Most interactions can be completed without reaching for the mouse.

---

## 1. General & Window Controls

| Shortcut | Action | Scope |
| :--- | :--- | :--- |
| `Space` / `Escape` | Close preview window (or dismiss search if active) | Global |
| `Tab` | Toggle between Detail view and Grid/Thumbnail view | Global |
| `Enter` | Open active file in system default application / Enter directory | Global / Folder |
| `Ctrl` + `,` | Open Settings configuration screen | Global |
| `Ctrl` + `T` | Cycle color theme (`Dark` → `Light` → `Nord`) | Global |
| `Ctrl` + `C` | Copy file path / Copy selected text / Copy bitmap image (PNG) | Global / Text, PDF, Markdown, EPUB, Image |
| `Ctrl` + `A` | Select all text | Text, Code, Markdown, EPUB, PDF |

---

## 2. Navigation & Playlist

When multiple files are previewed or navigation within a folder is active:

| Shortcut | Action | Context |
| :--- | :--- | :--- |
| `Ctrl` + `Shift` + `←` / `Ctrl` + `PageUp` | Previous file in directory / playlist | Playlist / Global |
| `Ctrl` + `Shift` + `→` / `Ctrl` + `PageDown` | Next file in directory / playlist | Playlist / Global |
| `←` / `Left Arrow` | Previous EPUB chapter / Seek -5s (Video, Audio) | Reader & Media |
| `→` / `Right Arrow` | Next EPUB chapter / Seek +5s (Video, Audio) | Reader & Media |
| `←` | Go to parent directory | Folder View |
| `→` | Open selected folder or preview selected file | Folder View |

---

## 3. Smooth Scrolling, Sidebars & Vim Motions

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| `j` / `↓` | Scroll Down | Smooth scroll down by 80px |
| `k` / `↑` | Scroll Up | Smooth scroll up by 80px |
| `d` / `PageDown` | Half Page Down | Jump scroll down by half viewport height |
| `u` / `PageUp` | Half Page Up | Jump scroll up by half viewport height |
| `gg` / `Home` | Scroll to Top | Double-tap `g` or hit `Home` to spring to start |
| `G` / `End` | Scroll to Bottom | `Shift+G` or `End` to spring to the end of document |
| `gt` | Toggle Table of Contents | Opens/closes outline sidebar in Markdown, EPUB, PDF |
| `ge` | Toggle File Tree Sidebar | Opens/closes directory file tree sidebar |
| `Ctrl` + `Mouse Wheel` | Zoom in / Zoom out | Dynamic viewport scaling or font resizing |

---

## 4. Search & Filter

| Shortcut | Action | Context |
| :--- | :--- | :--- |
| `Ctrl` + `F` / `/` | Open Search Bar | Text, Code, Markdown, JSON, CSV, Grid |
| `Enter` / `F3` | Next search match | Search active |
| `Shift` + `Enter` / `Shift` + `F3` | Previous search match | Search active |
| `Escape` | Dismiss search bar and clear highlights | Search active |
| Type characters directly | Live filter / Type-to-search | Grid / Folder view |

---

## 5. Format-Specific Shortcuts

### Images & Creative (`.png`, `.jpg`, `.svg`, `.gif`, `.psd`, `.kra`)
* `Ctrl` + `+` / `Ctrl` + `-` (or `Ctrl` + Wheel): Zoom In / Zoom Out.
* `=`: Fit image to window.
* `r`: Rotate image 90 degrees clockwise.
* `Shift` + `R` / `R`: Rotate image 90 degrees counter-clockwise.
* `h`: Flip image horizontally (mirror).
* `v`: Flip image vertically (mirror).
* `0` / `Ctrl` + `0`: Reset zoom, pan, rotation, and flip.
* `Ctrl` + `C`: Copy bitmap pixel data directly as `image/png` to clipboard.
* `i`: Toggle EXIF metadata sidebar.

### Markdown & EPUB Readers
* `Shift` + `Mouse Wheel`: Adjust reading column width (`max_text_width`).
* `Shift` + `0`: Reset reading column width to default (820px).
* `Ctrl` + `+` / `Ctrl` + `-`: Increase / Decrease reader font size.
* `Shift` + `=`: Reset font size to default (14px).

### Spreadsheets & Databases (`.csv`, `.xlsx`, `.sqlite`)
* `Ctrl` + `+` / `Ctrl` + `-`: Zoom in / Zoom out table cell font size.
* `Ctrl` + `Mouse Wheel`: Zoom table font size dynamically.
* `←` / `→` / `h` / `l`: Scroll horizontally across columns.

### Source Code & Text
* `Ctrl` + `W`: Toggle soft word wrapping.
* `Ctrl` + `+` / `Ctrl` + `-`: Increase / Decrease font size.
* `Shift` + `=`: Reset font size to default (14px).

### JSON Viewer
* `Ctrl` + `P`: Toggle between Collapsible Tree View and Formatted Raw JSON.
* `Ctrl` + `e`: Expand all tree nodes.
* `Ctrl` + `Shift` + `E`: Collapse all tree nodes.

### Typst (`.typ`)
* `s` / `S`: Toggle between Compiled Rendered PDF and Raw Source Code view.
