#!/usr/bin/env bash
set -euo pipefail

BIN="${BIN:-$(which kglance 2>/dev/null || echo kglance)}"
SERVICEMENU_DIR="$HOME/.local/share/kio/servicemenus"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cmd_install() {
    mkdir -p "$SERVICEMENU_DIR"
    cp "$SCRIPT_DIR/data/kglance-rust.desktop" "$SERVICEMENU_DIR/"
    echo "Installed: $SERVICEMENU_DIR/kglance-rust.desktop"
    echo "Restart Dolphin (killall dolphin) or log out to apply."
}

cmd_uninstall() {
    rm -f "$SERVICEMENU_DIR/kglance-rust.desktop"
    echo "Removed: $SERVICEMENU_DIR/kglance-rust.desktop"
    echo "Restart Dolphin (killall dolphin) or log out to apply."
}

case "${1:-help}" in
    install) cmd_install ;;
    uninstall) cmd_uninstall ;;
    *)
        echo "Usage: $0 <install|uninstall>"
        echo ""
        echo "  install   — copy Dolphin KIO service menu (Space key preview)"
        echo "  uninstall — remove it"
        echo ""
        echo "Env: BIN=$BIN  (path to kglance binary)"
        ;;
esac
