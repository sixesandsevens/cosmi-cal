#!/bin/sh
set -eu

APP_ID="io.github.sixesandsevens.cosmical"
APP_NAME="cosmi-cal"

SOURCE_DESKTOP="${HOME}/.local/share/applications/${APP_ID}.desktop"
AUTOSTART_DIR="${HOME}/.config/autostart"
AUTOSTART_TARGET="${AUTOSTART_DIR}/${APP_ID}.desktop"
BIN_TARGET="${HOME}/.local/bin/${APP_NAME}"

if [ ! -f "$SOURCE_DESKTOP" ]; then
    printf '%s\n' "CosmiCal is not installed locally yet. Run ./scripts/install-local.sh first." >&2
    exit 1
fi

if [ ! -x "$BIN_TARGET" ]; then
    printf '%s\n' "CosmiCal binary was not found at ${BIN_TARGET}. Run ./scripts/install-local.sh first." >&2
    exit 1
fi

mkdir -p "$AUTOSTART_DIR"

printf '%s\n' "Enabling CosmiCal autostart at ${AUTOSTART_TARGET}..."
sed "s|^Exec=.*|Exec=${BIN_TARGET}|" "$SOURCE_DESKTOP" > "$AUTOSTART_TARGET"
chmod 0644 "$AUTOSTART_TARGET"

printf '%s\n' "Autostart enabled."
