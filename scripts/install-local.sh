#!/bin/sh
set -eu

APP_ID="io.github.sixesandsevens.cosmical"
APP_NAME="cosmi-cal"

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPO_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)

BIN_DIR="${HOME}/.local/bin"
APP_DIR="${HOME}/.local/share/applications"
ICON_THEME_ROOT="${HOME}/.local/share/icons/hicolor"
ICON_NAMES="${APP_NAME} ${APP_ID}"

BIN_SOURCE="${REPO_ROOT}/target/release/${APP_NAME}"
BIN_TARGET="${BIN_DIR}/${APP_NAME}"
DESKTOP_SOURCE="${REPO_ROOT}/packaging/${APP_ID}.desktop"
DESKTOP_TARGET="${APP_DIR}/${APP_ID}.desktop"
ICON_SOURCE="${REPO_ROOT}/resources/icons/hicolor/scalable/apps/icon.svg"
ICON_TARGET="${ICON_THEME_ROOT}/256x256/apps/${APP_NAME}.png"
ICON_SIZES="32x32 48x48 64x64 128x128 256x256 512x512"

if ! command -v cargo >/dev/null 2>&1; then
    printf '%s\n' "Cargo was not found. Install Rust/Cargo, then rerun this script." >&2
    printf '%s\n' "On Pop!_OS/Ubuntu: sudo apt install cargo" >&2
    printf '%s\n' "Or with rustup: https://rustup.rs" >&2
    exit 1
fi

mkdir -p "$BIN_DIR" "$APP_DIR"

printf '%s\n' "Building CosmiCal..."
cargo build --manifest-path "${REPO_ROOT}/Cargo.toml" --release

printf '%s\n' "Installing binary to ${BIN_TARGET}..."
install -m 0755 "$BIN_SOURCE" "$BIN_TARGET"

printf '%s\n' "Installing desktop entry to ${DESKTOP_TARGET}..."
sed "s|^Exec=.*|Exec=${BIN_TARGET}|" "$DESKTOP_SOURCE" > "$DESKTOP_TARGET"
chmod 0644 "$DESKTOP_TARGET"

ICON_INSTALLED=0

for size in $ICON_SIZES; do
    ICON_PNG_SOURCE="${REPO_ROOT}/resources/icons/hicolor/${size}/apps/${APP_NAME}.png"
    ICON_DIR="${ICON_THEME_ROOT}/${size}/apps"

    if [ -f "$ICON_PNG_SOURCE" ]; then
        mkdir -p "$ICON_DIR"
        for icon_name in $ICON_NAMES; do
            SIZE_ICON_TARGET="${ICON_DIR}/${icon_name}.png"
            printf '%s\n' "Installing ${size} icon to ${SIZE_ICON_TARGET}..."
            install -m 0644 "$ICON_PNG_SOURCE" "$SIZE_ICON_TARGET"
        done
        ICON_INSTALLED=1
    fi
done

SCALABLE_ICON_SOURCE="${REPO_ROOT}/resources/icons/hicolor/scalable/apps/${APP_NAME}.svg"
if [ ! -f "$SCALABLE_ICON_SOURCE" ] && [ -f "$ICON_SOURCE" ]; then
    SCALABLE_ICON_SOURCE="$ICON_SOURCE"
fi

if [ -f "$SCALABLE_ICON_SOURCE" ] && grep -Eq '<(path|rect|circle|ellipse|polygon|polyline|line|image|g|defs|use)\b' "$SCALABLE_ICON_SOURCE"; then
    SCALABLE_ICON_DIR="${ICON_THEME_ROOT}/scalable/apps"
    mkdir -p "$SCALABLE_ICON_DIR"
    for icon_name in $ICON_NAMES; do
        SCALABLE_ICON_TARGET="${SCALABLE_ICON_DIR}/${icon_name}.svg"
        printf '%s\n' "Installing scalable icon to ${SCALABLE_ICON_TARGET}..."
        install -m 0644 "$SCALABLE_ICON_SOURCE" "$SCALABLE_ICON_TARGET"
    done
    ICON_INSTALLED=1
elif [ "$ICON_INSTALLED" -eq 0 ] && command -v rsvg-convert >/dev/null 2>&1; then
    ICON_DIR="${ICON_THEME_ROOT}/256x256/apps"
    mkdir -p "$ICON_DIR"
    for icon_name in $ICON_NAMES; do
        RENDERED_ICON_TARGET="${ICON_DIR}/${icon_name}.png"
        printf '%s\n' "Rendering icon to ${RENDERED_ICON_TARGET}..."
        rsvg-convert -w 256 -h 256 "$ICON_SOURCE" -o "$RENDERED_ICON_TARGET"
    done
    ICON_INSTALLED=1
fi

if [ "$ICON_INSTALLED" -eq 0 ]; then
    printf '%s\n' "No packaged icons were found and rsvg-convert is unavailable; the launcher may appear without an icon until one is installed." >&2
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
