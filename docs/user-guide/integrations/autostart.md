# Autostart & Daemon Management

To achieve near-instantaneous (<10ms) preview speeds, Kglance operates best in **Daemon Mode** running in the background.

---

## 1. Systemd User Service (Recommended)

Running Kglance as a systemd user service ensures that the daemon starts on user login and restarts automatically if it ever crashes.

### Create Service File

Create `~/.config/systemd/user/kglance.service`:

```ini
[Unit]
Description=Kglance (Oxiview) File Preview Daemon
Documentation=https://github.com/Mintori09/kglance
PartOf=graphical-session.target
After=graphical-session.target

[Service]
Type=simple
ExecStart=%h/.local/bin/kglance daemon
Restart=on-failure
RestartSec=2
Environment=RUST_LOG=info

[Install]
WantedBy=graphical-session.target
```

### Enable & Start Service

```bash
# Reload systemd daemon
systemctl --user daemon-reload

# Enable and start Kglance daemon immediately
systemctl --user enable --now kglance.service

# Verify status
systemctl --user status kglance.service
```

---

## 2. KDE Plasma Autostart (Desktop Entry)

Alternatively, you can register Kglance in KDE Plasma's System Settings → **Autostart**:

Create `~/.config/autostart/kglance-daemon.desktop`:

```ini
[Desktop Entry]
Type=Application
Name=Kglance Daemon
Comment=High performance file preview daemon
Exec=kglance daemon
Icon=kglance
Terminal=false
Categories=Utility;
X-KDE-autostart-phase=2
```

---

## 3. Managing the Daemon via CLI

* **Check if Daemon is Running**:
  ```bash
  busctl --user status org.mintori.Kglance
  ```

* **Manually Start Daemon in Foreground (for debugging)**:
  ```bash
  RUST_LOG=debug kglance daemon
  ```

* **Kill Running Daemon**:
  ```bash
  pkill -f "kglance daemon"
  ```
