# Yazi File Manager Integration

Kglance provides a dedicated plugin for the [Yazi](https://github.com/sxyazi/yazi) terminal file manager: `kglance.yazi`.

When enabled, Yazi provides:
1. **Space key preview**: Press `Space` on any file in Yazi to pop up the Kglance window.
2. **Live Cursor Sync (Hover update)**: As you move your cursor up/down across files in Yazi with `j`/`k`, Kglance automatically and instantly updates its open preview window via DBus without closing or reopening.

---

## 1. Install the Plugin

Clone or copy the `kglance.yazi` folder into your Yazi plugins directory:

```bash
mkdir -p ~/.config/yazi/plugins
cp -r kglance.yazi ~/.config/yazi/plugins/
```

Or clone directly:

```bash
git clone https://github.com/Mintori09/kglance.git /tmp/kglance
cp -r /tmp/kglance/kglance.yazi ~/.config/yazi/plugins/
```

---

## 2. Configure Keybindings

Add the following binding to `~/.config/yazi/keymap.toml`:

```toml
[mgr]
keymap = [
  { on = [ "<Space>" ], run = "plugin kglance", desc = "Preview with Kglance" },
]
```

---

## 3. Enable Cursor Hover Sync (Optional)

To enable live sync when navigating in Yazi while Kglance preview is open, add a hover hook to `~/.config/yazi/init.lua`:

```lua
-- ~/.config/yazi/init.lua
require("kglance"):setup()
```

When configured, moving the cursor in Yazi emits `kglance update <filepath>` via DBus, updating the active preview in sub-5ms latency.
