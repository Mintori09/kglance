# Dolphin File Manager Integration

Kglance integrates natively with **KDE Dolphin** via KDE KIO Service Menus and custom shortcuts. This allows you to press the `Space` key on any file in Dolphin to open an instant QuickLook-style preview.

---

## Automated Setup with `dolphin-setup.sh`

The repository includes a dedicated installer script: [scripts/dolphin-setup.sh](file:///home/mintori/Projects/personal/kglance/scripts/dolphin-setup.sh).

### Quick Install with Space Shortcut

Run the script locally:

```bash
# Assign Space shortcut and install KIO service menu:
./scripts/dolphin-setup.sh install --shortcut
```

Or execute directly from the GitHub repository:

```bash
bash <(curl -s https://raw.githubusercontent.com/Mintori09/kglance/main/scripts/dolphin-setup.sh) install --shortcut
```

The script performs the following tasks:
1. Installs `kglance-rust.desktop` into `~/.local/share/kio/servicemenus/`.
2. Adds the preview action to `~/.local/share/kxmlgui5/dolphin/dolphinui.rc`.
3. Prompts to restart Dolphin to apply the shortcut immediately.

---

## Manual Setup

### 1. Install KIO Desktop File

Create `~/.local/share/kio/servicemenus/kglance-rust.desktop` with the following content:

```ini
[Desktop Entry]
Type=Service
MimeType=application/octet-stream;
Actions=previewWithRust;
X-KDE-Priority=TopLevel
X-KDE-Submenu=Preview

[Desktop Action previewWithRust]
Name=Quick Preview
Icon=kglance
Exec=kglance %U
```

Make the desktop entry executable:

```bash
chmod +x ~/.local/share/kio/servicemenus/kglance-rust.desktop
```

### 2. Configure Shortcut in Dolphin

1. Open **Dolphin**.
2. Go to **Settings** → **Configure Keyboard Shortcuts…** (or press `Ctrl+Shift+,`).
3. In the search box, type `Quick Preview`.
4. Select the action and click **Custom Shortcut**, then press **Space**.
5. Click **Apply** and **OK**.

---

## Uninstallation

To remove the Dolphin integration:

```bash
./scripts/dolphin-setup.sh uninstall
```
