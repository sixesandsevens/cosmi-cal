#!/bin/sh
set -eu

APP_ID="io.github.sixesandsevens.cosmical"

SOURCE_DESKTOP="${HOME}/.local/share/applications/${APP_ID}.desktop"
AUTOSTART_DIR="${HOME}/.config/autostart"
AUTOSTART_TARGET="${AUTOSTART_DIR}/${APP_ID}.desktop"

if [ ! -f "$SOURCE_DESKTOP" ]; then
    printf '%s\n' "CosmiCal is not installed locally yet. Run ./scripts/install-local.sh first." >&2
    exit 1
fi

mkdir -p "$AUTOSTART_DIR"

printf '%s\n' "Enabling CosmiCal autostart at ${AUTOSTART_TARGET}..."
install -m 0644 "$SOURCE_DESKTOP" "$AUTOSTART_TARGET"

printf '%s\n' "Autostart enabled."
