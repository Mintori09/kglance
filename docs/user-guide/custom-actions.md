# Custom Actions Guide (Selection Menu)

Kglance features an interactive floating **Selection Menu** that appears whenever you select text in document viewers (Markdown, Source Code/Text, PDF, and EPUB).

Beyond standard operations like copying plain text or HTML, Kglance allows you to define arbitrary shell commands (**Custom Actions**) to process the selected text and display results either in an interactive popup modal, as a notification toast, or silently in the background.

---

## 1. Quick Overview

When you click a custom action button in the selection menu:
1. Kglance spawns the configured shell command using `sh -c "<command>"`.
2. The selected text is piped directly to the command's standard input (`stdin`).
3. The selected text is also exposed via the `$KGLANCE_TEXT` environment variable (safely truncated to 16 KB to protect process environment limits).
4. Full file context (file path, directory, name, stem, extension) is provided through dedicated environment variables.
5. The working directory (`cwd`) of the spawned process defaults to the file's parent directory (`$KGLANCE_FILE_DIR`).

---

## 2. Environment Variables & Process Context

Every custom action process is invoked with the following environment variables:

| Variable | Description | Example |
| :--- | :--- | :--- |
| `KGLANCE_TEXT` | Selected text (up to 16 KB UTF-8). Larger selections are still delivered via `stdin`. | `"fn main() { ... }"` |
| `KGLANCE_FILE` | Absolute path to the currently previewed file (if applicable). | `"/home/user/notes/todo.md"` |
| `KGLANCE_FILE_DIR` | Directory containing the previewed file. | `"/home/user/notes"` |
| `KGLANCE_FILE_NAME` | File name including extension. | `"todo.md"` |
| `KGLANCE_FILE_STEM` | Base file name without extension. | `"todo"` |
| `KGLANCE_FILE_EXT` | Lowercased file extension. | `"md"` |

### Working Directory (`cwd`)
- If previewing a file on disk whose parent directory exists, the command executes with `cwd` set to `KGLANCE_FILE_DIR`.
- If no file is associated (or the parent directory is inaccessible), `cwd` falls back to the user's home directory (`$HOME`), or `/tmp`.

---

## 3. Configuration Schema

Custom actions are configured in `~/.config/kglance/config.json` under the `"selection_menu"` section:

```json
{
  "selection_menu": {
    "enabled": true,
    "custom_actions": [
      {
        "name": "Action Label",
        "command": "shell command here",
        "popup": true,
        "toast": false,
        "stream": false
      }
    ]
  }
}
```

### Action Properties

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `name` | `string` | *(Required)* | Label displayed on the action button in the floating menu. |
| `command` | `string` | *(Required)* | Shell command string executed via `sh -c`. |
| `popup` | `boolean` | `true` | When `true`, opens an overlay modal card inside Kglance displaying the command's stdout rendered as Markdown. |
| `toast` | `boolean` | `false` | When `true`, displays a desktop-style toast notification in the bottom corner upon command exit. |
| `stream` | `boolean` | `false` | When `true` (and `popup: true`), streams output in real-time as chunks arrive rather than waiting for command exit. Ideal for LLMs / AI CLI tools. |

> [!NOTE]
> Custom actions can also be managed visually without manually editing JSON: open Kglance Settings (`Ctrl+,` or `,`), navigate to the **Previews** tab, and manage actions under **Selection Toolbar**.

---

## 4. Practical Examples

### Example 1: Translation (Google Translate CLI)
Translates selected text into Vietnamese using `translate-shell` (`trans`):
```json
{
  "name": "Translate VI",
  "command": "trans -b :vi",
  "popup": true,
  "toast": false,
  "stream": false
}
```

### Example 2: Streaming AI Assistant (Ollama / LLM)
Ask a local LLM to explain or summarize selected code/text in real time:
```json
{
  "name": "Explain Code",
  "command": "ollama run qwen2.5:coder \"Explain the following code concisely:\n$(cat)\"",
  "popup": true,
  "toast": false,
  "stream": true
}
```

### Example 3: Word & Character Count
Show word count statistics in a quick popup:
```json
{
  "name": "Word Count",
  "command": "echo \"### Selection Stats\n- Characters: $(wc -m)\n- Words: $(wc -w)\n- Lines: $(wc -l)\"",
  "popup": true,
  "toast": false,
  "stream": false
}
```

### Example 4: Search Selection on Web
Open the selected query in your default browser (runs silently with a confirmation toast):
```json
{
  "name": "Search DuckDuckGo",
  "command": "xdg-open \"https://duckduckgo.com/?q=$(python3 -c 'import urllib.parse, sys; print(urllib.parse.quote(sys.stdin.read().strip()))')\"",
  "popup": false,
  "toast": true,
  "stream": false
}
```

### Example 5: Using File Context
Run Git blame or view diff for the current file:
```json
{
  "name": "Git Status",
  "command": "echo \"### File: $KGLANCE_FILE_NAME\n\`\`\`\n$(git status -s)\n\`\`\`\"",
  "popup": true,
  "toast": false,
  "stream": false
}
```

---

## 5. Security & Execution Notes

- **Execution Shell**: Commands run with user permissions via `sh -c`. Avoid running untrusted command scripts.
- **Execution Limits**: Non-streaming commands have a timeout safeguard (`30 seconds`).
- **Markdown Rendering**: Popup modals automatically render command stdout as GitHub Flavored Markdown (including syntax-highlighted code blocks, tables, and lists).
