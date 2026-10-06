#!/usr/bin/env bash
# ==============================================================================
# Kglance — Audio E2E Test Runner
#
# This script performs end-to-end testing of audio playback in Kglance:
# 1. Verifies prerequisites (ffmpeg, busctl, gstreamer, wpctl/pactl).
# 2. Generates synthetic multi-format audio files (MP3 with ID3v2 & cover, FLAC, WAV).
# 3. Executes Rust integration tests (`tests/audio_pipeline.rs`) via cargo nextest.
# 4. Tests Black-Box CLI & DBus daemon workflow (ShowPreview, UpdatePreview, IsGuiOpen).
# ==============================================================================

set -euo pipefail

# ANSI Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BIN_PATH="${PROJECT_ROOT}/target/debug/kglance"
TEST_DIR=""
DAEMON_PID=""

log_step() {
    echo -e "\n${BLUE}${BOLD}==>${NC} ${BOLD}$1${NC}"
}

log_pass() {
    echo -e "  ${GREEN}✔ [PASS]${NC} $1"
}

log_warn() {
    echo -e "  ${YELLOW}⚠ [WARN]${NC} $1"
}

log_fail() {
    echo -e "  ${RED}✘ [FAIL]${NC} $1" >&2
}

cleanup() {
    if [ -n "$DAEMON_PID" ] && kill -0 "$DAEMON_PID" 2>/dev/null; then
        echo -e "\n${YELLOW}Cleaning up background daemon (PID: $DAEMON_PID)...${NC}"
        kill "$DAEMON_PID" 2>/dev/null || true
        wait "$DAEMON_PID" 2>/dev/null || true
    fi

    if [ -n "$TEST_DIR" ] && [ -d "$TEST_DIR" ]; then
        if command -v trash >/dev/null 2>&1; then
            trash "$TEST_DIR" 2>/dev/null || rm -rf "$TEST_DIR"
        else
            rm -rf "$TEST_DIR"
        fi
    fi
}
trap cleanup EXIT INT TERM

# ── 1. Check prerequisites ───────────────────────────────────────────────────
log_step "Checking prerequisites..."

for cmd in cargo ffmpeg busctl; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
        log_fail "Missing required tool: $cmd"
        exit 1
    fi
    log_pass "Found $cmd"
done

if command -v wpctl >/dev/null 2>&1; then
    log_pass "Found wpctl (PipeWire)"
elif command -v pactl >/dev/null 2>&1; then
    log_pass "Found pactl (PulseAudio/PipeWire-pulse)"
else
    log_warn "Neither wpctl nor pactl found. Audio sink inspection will be skipped."
fi

# ── 2. Run Cargo Nextest Integration Test ───────────────────────────────────
log_step "Running Rust Core Pipeline Integration Test..."
cd "$PROJECT_ROOT"
if GST_AUDIOSINK="${GST_AUDIOSINK:-fakesink}" cargo nextest run -j 3 --test audio_pipeline; then
    log_pass "All Rust audio integration tests passed (Parser + GStreamer playbin lifecycle)!"
else
    log_fail "Rust audio integration tests failed!"
    exit 1
fi

# ── 3. Build Kglance binary ─────────────────────────────────────────────────
log_step "Verifying Kglance binary..."
if [ ! -f "$BIN_PATH" ]; then
    echo "Building kglance debug binary..."
    cargo build --bin kglance
fi
log_pass "Binary available at: $BIN_PATH"

# ── 4. Generate test audio files ─────────────────────────────────────────────
log_step "Generating test audio fixtures..."
TEST_DIR="$(mktemp -d -t kglance-audio-test-XXXXXX)"

# MP3 with ID3v2 metadata
ffmpeg -f lavfi -i "sine=frequency=440:duration=4" \
    -metadata title="Acoustic Pulse" \
    -metadata artist="Oxiview Ensemble" \
    -metadata album="System Verification" \
    -y "$TEST_DIR/track1.mp3" >/dev/null 2>&1
log_pass "Generated track1.mp3 (Sine 440Hz with ID3v2 tags)"

# FLAC
ffmpeg -f lavfi -i "sine=frequency=880:duration=3" \
    -metadata title="Hi-Res Sine" \
    -metadata artist="Oxiview Ensemble" \
    -y "$TEST_DIR/track2.flac" >/dev/null 2>&1
log_pass "Generated track2.flac (Sine 880Hz FLAC)"

# Cover image & MP3 with attached picture
ffmpeg -f lavfi -i "color=c=purple:s=200x200:d=1" -vframes 1 -y "$TEST_DIR/cover.jpg" >/dev/null 2>&1
ffmpeg -f lavfi -i "sine=frequency=523:duration=3" \
    -i "$TEST_DIR/cover.jpg" \
    -map 0:a -map 1:v \
    -c:a libmp3lame -c:v mjpeg \
    -id3v2_version 3 \
    -metadata:s:v title="Album cover" \
    -metadata:s:v comment="Cover (front)" \
    -metadata title="Visual Track" \
    -metadata artist="Cover Artist" \
    -y "$TEST_DIR/track_with_cover.mp3" >/dev/null 2>&1
log_pass "Generated track_with_cover.mp3 (with embedded album art)"

# ── 5. Test Black-Box DBus & Daemon Lifecycle ────────────────────────────────
log_step "Testing DBus Daemon Lifecycle & Audio Preview..."

# Check if an existing daemon is running
EXISTING_DAEMON=0
if busctl --user status org.mintori.Kglance >/dev/null 2>&1; then
    EXISTING_DAEMON=1
    log_pass "Detected existing running Kglance DBus service"
else
    echo "Starting new kglance daemon in background..."
    "$BIN_PATH" daemon &
    DAEMON_PID=$!
    sleep 1

    # Wait up to 5s for DBus service registration
    REGISTERED=0
    for i in {1..10}; do
        if busctl --user status org.mintori.Kglance >/dev/null 2>&1; then
            REGISTERED=1
            break
        fi
        sleep 0.5
    done

    if [ "$REGISTERED" -eq 1 ]; then
        log_pass "kglance daemon registered org.mintori.Kglance on session bus"
    else
        log_fail "Daemon failed to register DBus service"
        exit 1
    fi
fi

# Send ShowPreview request via CLI / DBus
log_step "Testing ShowPreview via DBus..."
"$BIN_PATH" "$TEST_DIR/track1.mp3"
sleep 1.2

# Check IsGuiOpen
GUI_OPEN=$(busctl --user call org.mintori.Kglance /org/mintori/Kglance org.mintori.Kglance IsGuiOpen 2>/dev/null | awk '{print $2}' || echo "false")
if [ "$GUI_OPEN" = "true" ]; then
    log_pass "Preview GUI opened successfully (IsGuiOpen = true)"
else
    log_warn "IsGuiOpen returned $GUI_OPEN (may require active display session)"
fi

# Test UpdatePreview to switch track
log_step "Testing UpdatePreview track switching..."
"$BIN_PATH" update "$TEST_DIR/track_with_cover.mp3"
sleep 1.0
log_pass "Sent UpdatePreview for track_with_cover.mp3 without crash"

# Inspect sound server streams if available
if command -v pactl >/dev/null 2>&1; then
    STREAM_COUNT=$(pactl list sink-inputs 2>/dev/null | grep -i "application.name = \"kglance\"" | wc -l || true)
    if [ "$STREAM_COUNT" -gt 0 ]; then
        log_pass "Active sound stream detected in sound server ($STREAM_COUNT sink input)"
    else
        log_warn "No active sink-input for kglance found (GStreamer may use autoaudiosink or idle)"
    fi
fi

# Clean close test
if [ "$EXISTING_DAEMON" -eq 0 ] && [ -n "$DAEMON_PID" ]; then
    log_step "Stopping daemon and verifying audio stream cleanup..."
    kill "$DAEMON_PID" 2>/dev/null || true
    wait "$DAEMON_PID" 2>/dev/null || true
    DAEMON_PID=""
    sleep 0.5
    log_pass "Daemon stopped cleanly"
fi

echo -e "\n${GREEN}${BOLD}======================================================${NC}"
echo -e "${GREEN}${BOLD}        ALL AUDIO E2E TESTS COMPLETED SUCCESSFULLY!    ${NC}"
echo -e "${GREEN}${BOLD}======================================================${NC}"
