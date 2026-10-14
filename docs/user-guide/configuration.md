# Configuration Guide

Kglance stores its configuration file in JSON format at:

```text
~/.config/kglance/config.json
```

If the file does not exist, Kglance automatically creates it on first startup with default values. You can override the configuration directory using the `KGLANCE_CONFIG_DIR` environment variable.

---

## Configuration File Schema

Below is a complete `config.json` configuration example with default values explained:

```json
{
  "ui": {
    "theme": "Dark",
    "font_size": 14.0,
    "font_family": "Noto Sans",
    "font_family_mono": "Fira Code",
    "epub_font_family": "Noto Serif",
    "epub_reading_mode": "single_chapter",
    "max_text_width": 820.0,
    "default_width": 1024,
    "default_height": 768,
    "min_width": 800,
    "min_height": 600,
    "prefer_mermaid_cli": false,
    "word_wrap": false,
    "json_tree_view": true,
    "ruby_display_mode": "stacked"
  },
  "cache": {
    "max_memory_mb": 512,
    "max_disk_cache_mb": 1024
  },
  "scroll": {
    "smooth_scroll_enabled": true,
    "friction": 1.8,
    "spring_stiffness": 180.0
  },
  "selection_menu": {
    "enabled": true,
    "custom_actions": [
      {
        "name": "Translate VI",
        "command": "trans -b :vi",
        "popup": true,
        "toast": false,
        "stream": false
      }
    ]
  }
}
```

---

## Detailed Options

### `ui` Section

| Key | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `theme` | `string` | `"Auto"` | Color theme. Options: `"Auto"` (detects system dark/light mode), `"Dark"`, `"Light"`, `"Nord"`. |
| `font_size` | `number` | `14.0` | Base UI and content font size in points/pixels. |
| `font_family` | `string \| null` | `null` | Custom font family name for UI and prose text. When `null`, uses system default. |
| `font_family_mono` | `string \| null` | `null` | Custom font family for monospace code previews. |
| `epub_font_family` | `string \| null` | `null` | Dedicated typography font family for EPUB reader views (e.g. `"Noto Serif"`). |
| `epub_reading_mode` | `string` | `"single_chapter"` | EPUB reader layout mode. Options: `"single_chapter"` (chapter-by-chapter reading with `←`/`→`), `"continuous"` (seamless vertical scroll across all chapters). |
| `max_text_width` | `number \| null` | `820.0` | Maximum container width (in pixels) for centered reading in Markdown/EPUB. Set to `null` to fill window width. |
| `default_width` | `integer` | `1024` | Initial window width when first opened. |
| `default_height` | `integer` | `768` | Initial window height when first opened. |
| `min_width` | `integer` | `800` | Minimum allowed window width constraint. |
| `min_height` | `integer` | `600` | Minimum allowed window height constraint. |
| `prefer_mermaid_cli` | `boolean` | `false` | When `true`, delegates Mermaid diagram rendering to the external `mmdc` CLI tool. |
| `word_wrap` | `boolean` | `false` | Default soft-wrap setting for text and code editors (`Ctrl+W` toggles this per session). |
| `json_tree_view` | `boolean` | `true` | When `true`, opens `.json` files in the interactive Collapsible Tree View rather than formatted raw text. |
| `ruby_display_mode` | `string` | `"stacked"` | Rendering style for Furigana/Ruby text. Options: `"stacked"` (ruby annotations placed directly above base text), `"tooltip"` (hover popup), `"parentheses"` (inline `Base(Ruby)`), `"hidden"` (hides ruby text). |

### `cache` Section

| Key | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `max_memory_mb` | `integer` | `512` | Maximum memory limit (in MB) allocated for parsed ASTs, rendered PDF pages, and cached thumbnails before automatic LRU eviction. |
| `max_disk_cache_mb` | `integer` | `1024` | Maximum disk cache limit (in MB) for temporary parsed files, converted PDFs, and thumbnails. |

### `scroll` Section

| Key | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `smooth_scroll_enabled` | `boolean` | `true` | Enables physics-based smooth scrolling and kinetic fling across all scrollable viewers. |
| `friction` | `number` | `1.8` | Kinetic inertia deceleration friction. |
| `spring_stiffness` | `number` | `180.0` | Stiffness parameter for the spring physics animation. |

### `selection_menu` Section

| Key | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `enabled` | `boolean` | `true` | Enables the floating toolbar menu when text is selected across PDF, Markdown, EPUB, and Source Code viewers. |
| `custom_actions` | `array` | `[]` | List of custom shell actions available in the selection menu. |

#### Custom Action Schema:
* `name`: Label displayed on the action button in the selection menu.
* `command`: Shell command to execute (selected text is passed via stdin and `$KGLANCE_TEXT`).
* `popup`: When `true`, renders the command output in an overlay popup modal inside Kglance.
* `toast`: When `true`, shows a brief notification toast upon command completion.
* `stream`: When `true`, streams command stdout in real-time into the popup modal (useful for AI CLI or long-running commands).

See the [Custom Actions Guide](custom-actions.md) for full documentation on environment variables, file context (`$KGLANCE_FILE`), and practical examples.

---

## Automatic Theme Detection

When `ui.theme` is set to `"Auto"`:
1. Under **KDE Plasma 6**, Kglance queries `kreadconfig6 --group General --key ColorScheme` to match the current Breeze or custom theme.
2. Under **GNOME / Freedesktop**, it queries `gsettings get org.gnome.desktop.interface color-scheme`.
3. If neither returns light theme markers, it safely defaults to `Dark`.
