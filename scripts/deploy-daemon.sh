#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

TARGET_RELEASE="$PROJECT_ROOT/target/release/kglance"
INSTALL_DEST="/usr/bin/kglance"

echo "==> Building kglance (release mode)..."
cd "$PROJECT_ROOT"
cargo build --release

if [ ! -f "$TARGET_RELEASE" ]; then
    echo "Error: Release binary not found at $TARGET_RELEASE" >&2
    exit 1
fi

echo "==> Installing binary to $INSTALL_DEST (requires sudo)..."
sudo install -m 755 "$TARGET_RELEASE" "$INSTALL_DEST"

echo "==> Restarting kglance user daemon..."
if systemctl --user is-enabled kglance >/dev/null 2>&1 || systemctl --user is-active kglance >/dev/null 2>&1; then
    systemctl --user restart kglance
    echo "✔ systemd service 'kglance.service' restarted successfully."
else
    # Fallback if not running via systemd service
    pkill -f "kglance daemon" || true
    nohup "$INSTALL_DEST" daemon >/dev/null 2>&1 &
    echo "✔ Standalone daemon restarted in background."
fi

echo "==> Done! Kglance daemon is running the latest build."
notify-send "Kglance updated" "Daemon restarted with latest build successfully!" || true
