#!/usr/bin/env bash
set -euo pipefail

REPO_OWNER="Mintori09"
REPO_NAME="kglance"
BRANCH="main"
REMOTE_FILE_URL="https://raw.githubusercontent.com/${REPO_OWNER}/${REPO_NAME}/${BRANCH}/data/kglance-rust.desktop"

BIN="${BIN:-$(which kglance 2>/dev/null || echo kglance)}"
SERVICEMENU_DIR="$HOME/.local/share/kio/servicemenus"
TARGET_FILE="$SERVICEMENU_DIR/kglance-rust.desktop"

download_file() {
    local url="$1"
    local dest="$2"

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url" -o "$dest"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$dest" "$url"
    else
        echo "Error: Cần có 'curl' hoặc 'wget' để tải file." >&2
        return 1
    fi
}

cmd_install() {
    mkdir -p "$SERVICEMENU_DIR"

    echo "Downloading kglance-rust.desktop from GitHub..."
    download_file "$REMOTE_FILE_URL" "$TARGET_FILE"

    chmod +x "$TARGET_FILE"

    echo "Installed & set executable: $TARGET_FILE"
    echo "Restart Dolphin (killall dolphin) or log out to apply."
}

cmd_uninstall() {
    rm -f "$TARGET_FILE"
    echo "Removed: $TARGET_FILE"
    echo "Restart Dolphin (killall dolphin) or log out to apply."
}

case "${1:-help}" in
    install) cmd_install ;;
    uninstall) cmd_uninstall ;;
    *)
        echo "Usage: $0 <install|uninstall>"
        echo ""
        echo "  install   — download, set chmod +x & install Dolphin KIO service menu"
        echo "  uninstall — remove it"
        echo ""
        echo "Env: BIN=$BIN  (path to kglance binary)"
        ;;
esac
