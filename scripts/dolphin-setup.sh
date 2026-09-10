#!/usr/bin/env bash
set -euo pipefail

REPO_OWNER="Mintori09"
REPO_NAME="kglance"
BRANCH="main"
REMOTE_FILE_URL="https://raw.githubusercontent.com/${REPO_OWNER}/${REPO_NAME}/${BRANCH}/data/kglance-rust.desktop"

BIN="${BIN:-$(which kglance 2>/dev/null || echo kglance)}"
SERVICEMENU_DIR="$HOME/.local/share/kio/servicemenus"
TARGET_FILE="$SERVICEMENU_DIR/kglance-rust.desktop"
DOLPHIN_UI_DIR="$HOME/.local/share/kxmlgui5/dolphin"
DOLPHIN_UI_FILE="$DOLPHIN_UI_DIR/dolphinui.rc"
ACTION_NAME="servicemenu_kglance-rust.desktop::previewWithRust"

download_file() {
    local url="$1"
    local dest="$2"

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url" -o "$dest"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$dest" "$url"
    else
        echo "Error: Need 'curl' or 'wget' to download file." >&2
        return 1
    fi
}

configure_shortcut() {
    local shortcut="$1"
    mkdir -p "$DOLPHIN_UI_DIR"

    # Terminate running Dolphin instances so configuration is not overwritten
    killall dolphin >/dev/null 2>&1 || true

    if [ ! -f "$DOLPHIN_UI_FILE" ]; then
        cat <<EOF >"$DOLPHIN_UI_FILE"
<?xml version='1.0'?>
<!DOCTYPE gui SYSTEM 'kpartgui.dtd'>
<gui name="dolphin" version="1">
 <ActionProperties scheme="Default">
  <Action name="${ACTION_NAME}" shortcut="${shortcut}"/>
 </ActionProperties>
</gui>
EOF
    else
        # Remove any existing binding for this action to avoid duplicates
        sed -i "/${ACTION_NAME}/d" "$DOLPHIN_UI_FILE"

        if grep -q "</ActionProperties>" "$DOLPHIN_UI_FILE"; then
            sed -i "s|.*</ActionProperties>|  <Action name=\"${ACTION_NAME}\" shortcut=\"${shortcut}\"/>\n&|" "$DOLPHIN_UI_FILE"
        else
            sed -i "s|</gui>| <ActionProperties scheme=\"Default\">\n  <Action name=\"${ACTION_NAME}\" shortcut=\"${shortcut}\"/>\n </ActionProperties>\n</gui>|" "$DOLPHIN_UI_FILE"
        fi
    fi

    echo "Configured Dolphin shortcut '$shortcut' for Kglance in $DOLPHIN_UI_FILE"
}

remove_shortcut() {
    if [ -f "$DOLPHIN_UI_FILE" ]; then
        killall dolphin >/dev/null 2>&1 || true
        sed -i "/${ACTION_NAME}/d" "$DOLPHIN_UI_FILE"
        echo "Removed shortcut configuration from $DOLPHIN_UI_FILE"
    fi
}

cmd_install() {
    local shortcut=""
    local set_shortcut=0

    while [ $# -gt 0 ]; do
        case "$1" in
            --shortcut)
                set_shortcut=1
                if [ $# -gt 1 ] && [[ "$2" != --* ]]; then
                    shortcut="$2"
                    shift
                else
                    shortcut="Space"
                fi
                ;;
            *)
                ;;
        esac
        shift
    done

    mkdir -p "$SERVICEMENU_DIR"

    if ! command -v "$BIN" >/dev/null 2>&1; then
        echo "Warning: '$BIN' was not found in your PATH." >&2
        echo "         Make sure to build or install kglance before using the shortcut or context menu." >&2
    fi

    echo "Downloading kglance-rust.desktop from GitHub..."
    download_file "$REMOTE_FILE_URL" "$TARGET_FILE"

    chmod +x "$TARGET_FILE"

    echo "Installed & set executable: $TARGET_FILE"

    if [ "$set_shortcut" -eq 1 ]; then
        configure_shortcut "$shortcut"
    fi

    echo "Restart Dolphin (killall dolphin) or log out to apply."
}

cmd_uninstall() {
    rm -f "$TARGET_FILE"
    echo "Removed: $TARGET_FILE"

    remove_shortcut

    echo "Restart Dolphin (killall dolphin) or log out to apply."
}

action="${1:-help}"
[ $# -gt 0 ] && shift

case "$action" in
    install) cmd_install "$@" ;;
    uninstall) cmd_uninstall ;;
    *)
        echo "Usage: $0 install [--shortcut [KEY]] | uninstall"
        echo ""
        echo "  install [--shortcut [KEY]] — download, set chmod +x & install Dolphin KIO service menu"
        echo "                               If --shortcut is specified, assigns shortcut (default: Space)"
        echo "  uninstall                  — remove desktop entry and shortcut"
        echo ""
        echo "Env: BIN=$BIN  (path to kglance binary)"
        ;;
esac
