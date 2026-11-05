#!/usr/bin/env bash
set -euo pipefail

EPUB_PATH="/home/mintori/Documents/Efficient Node_ js_ A Beyond-The-Basics Guide -- Samer Buna -- 2025 -- O'Reilly Media, Incorporated -- isbn13 9781098145194 -- 65b5aab642b8157b28fa78792d9db892 -- Anna’s Archive.epub"
OUTPUT_DIR="screenshots"
mkdir -p "$OUTPUT_DIR"

echo "[*] Cleaning up existing screenshots in '$OUTPUT_DIR'..."
if command -v trash >/dev/null 2>&1; then
    find "$OUTPUT_DIR" -type f -name "*.png" -exec trash {} + 2>/dev/null || true
else
    find "$OUTPUT_DIR" -type f -name "*.png" -delete 2>/dev/null || true
fi

if [ ! -f "$EPUB_PATH" ]; then
    echo "[-] EPUB file does not exist: $EPUB_PATH" >&2
    exit 1
fi

echo "[*] Ensuring project binary is up to date (building if needed)..."
just build

echo "[*] Launching kglance preview..."
cargo run --bin kglance -- --standalone "$EPUB_PATH" &
APP_PID=$!

cleanup() {
    echo "[*] Cleaning up kglance process (PID: $APP_PID)..."
    kill "$APP_PID" 2>/dev/null || true
    pkill -P "$APP_PID" 2>/dev/null || true
}
trap cleanup EXIT

echo "[*] Waiting for kglance window to appear (timeout: 60s)..."
WINDOW_ID=""
for _ in {1..120}; do
    # Check if the process died prematurely
    if ! kill -0 "$APP_PID" 2>/dev/null; then
        echo "[-] kglance process exited unexpectedly!" >&2
        exit 1
    fi

    # Find the newest window matching kglance
    WINDOW_ID=$(kdotool search --class "kglance" 2>/dev/null | tail -n 1 || true)
    if [ -n "$WINDOW_ID" ]; then
        break
    fi
    sleep 0.5
done

if [ -z "$WINDOW_ID" ]; then
    echo "[-] Failed to find kglance window after 60 seconds!" >&2
    exit 1
fi

echo "[+] Found kglance window (ID: $WINDOW_ID). Activating window..."
kdotool windowactivate "$WINDOW_ID"
sleep 1.0

# Linux input-event-codes: KEY_HOME = 102, KEY_PAGEDOWN = 109
KEY_HOME=102
KEY_PAGEDOWN=109
TOTAL_SHOTS=10

echo "[*] Sending Home key to scroll to the top of the file..."
ydotool key "${KEY_HOME}:1" "${KEY_HOME}:0"
sleep 1.0

echo "[*] Starting scroll and capture cycle for $TOTAL_SHOTS screenshots..."
for shot in $(seq 1 "$TOTAL_SHOTS"); do
    if [ "$shot" -eq 1 ]; then
        echo "[-] Shot 1: Capturing the initial top page..."
    else
        echo "[-] Shot $shot: Scrolling 1 time PageDown..."
        ydotool key "${KEY_PAGEDOWN}:1" "${KEY_PAGEDOWN}:0"
        sleep 0.4
    fi

    # Allow UI time to render changes
    sleep 0.5

    # Ensure kglance remains the active window
    kdotool windowactivate "$WINDOW_ID"
    sleep 0.2

    TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    SHOT_FILE="${OUTPUT_DIR}/epub_scroll_shot_${shot}_${TIMESTAMP}.png"
    echo "[+] Capturing active window -> $SHOT_FILE..."
    spectacle -a -b -n -o "$SHOT_FILE"
    sleep 0.5
done

echo "[✔] Successfully captured $TOTAL_SHOTS screenshots into '$OUTPUT_DIR'."
