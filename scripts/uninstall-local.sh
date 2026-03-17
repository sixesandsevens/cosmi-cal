#!/bin/sh
set -eu

APP_ID="io.github.sixesandsevens.cosmical"
APP_NAME="cosmi-cal"

BIN_TARGET="${HOME}/.local/bin/${APP_NAME}"
DESKTOP_TARGET="${HOME}/.local/share/applications/${APP_ID}.desktop"
ICON_TARGET="${HOME}/.local/share/icons/hicolor/256x256/apps/${APP_ID}.png"

rm -f "$BIN_TARGET" "$DESKTOP_TARGET" "$ICON_TARGET"

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${HOME}/.local/share/applications" >/dev/null 2>&1 || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q -t "${HOME}/.local/share/icons/hicolor" >/dev/null 2>&1 || true
fi

printf '%s\n' "Removed local CosmiCal install from ~/.local."
