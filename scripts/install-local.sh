#!/bin/sh
set -eu

APP_ID="io.github.sixesandsevens.cosmical"
APP_NAME="cosmi-cal"

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPO_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)

BIN_DIR="${HOME}/.local/bin"
APP_DIR="${HOME}/.local/share/applications"
ICON_DIR="${HOME}/.local/share/icons/hicolor/256x256/apps"

BIN_SOURCE="${REPO_ROOT}/target/release/${APP_NAME}"
BIN_TARGET="${BIN_DIR}/${APP_NAME}"
DESKTOP_SOURCE="${REPO_ROOT}/packaging/${APP_ID}.desktop"
DESKTOP_TARGET="${APP_DIR}/${APP_ID}.desktop"
ICON_SOURCE="${REPO_ROOT}/resources/icons/hicolor/scalable/apps/icon.svg"
ICON_PNG_SOURCE="${REPO_ROOT}/resources/icons/hicolor/256x256/apps/${APP_NAME}.png"
ICON_TARGET="${ICON_DIR}/${APP_NAME}.png"

mkdir -p "$BIN_DIR" "$APP_DIR" "$ICON_DIR"

if [ ! -x "$BIN_SOURCE" ]; then
    printf '%s\n' "Missing release binary: ${BIN_SOURCE}" >&2
    printf '%s\n' "Build it first with: cargo build --release" >&2
    exit 1
fi

printf '%s\n' "Installing binary to ${BIN_TARGET}..."
install -m 0755 "$BIN_SOURCE" "$BIN_TARGET"

printf '%s\n' "Installing desktop entry to ${DESKTOP_TARGET}..."
install -m 0644 "$DESKTOP_SOURCE" "$DESKTOP_TARGET"

if [ -f "$ICON_PNG_SOURCE" ]; then
    printf '%s\n' "Installing packaged icon to ${ICON_TARGET}..."
    install -m 0644 "$ICON_PNG_SOURCE" "$ICON_TARGET"
elif command -v rsvg-convert >/dev/null 2>&1; then
    printf '%s\n' "Rendering icon to ${ICON_TARGET}..."
    rsvg-convert -w 256 -h 256 "$ICON_SOURCE" -o "$ICON_TARGET"
else
    printf '%s\n' "No packaged PNG icon and rsvg-convert not found; the launcher may appear without an icon until one is installed." >&2
fi

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${HOME}/.local/share/applications" >/dev/null 2>&1 || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q -t "${HOME}/.local/share/icons/hicolor" >/dev/null 2>&1 || true
fi

printf '%s\n' ""
printf '%s\n' "Local install complete."
printf '%s\n' "Binary: ${BIN_TARGET}"
printf '%s\n' "Desktop entry: ${DESKTOP_TARGET}"
printf '%s\n' "Icon: ${ICON_TARGET}"
printf '%s\n' "If the launcher icon does not appear immediately, try logging out and back in or refreshing your app launcher."
