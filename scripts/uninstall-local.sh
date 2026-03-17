#!/bin/sh
set -eu

APP_ID="io.github.sixesandsevens.cosmical"
APP_NAME="cosmi-cal"
ICON_THEME_ROOT="${HOME}/.local/share/icons/hicolor"
ICON_SIZES="32x32 48x48 64x64 128x128 256x256 512x512"
ICON_NAMES="${APP_NAME} ${APP_ID}"

BIN_TARGET="${HOME}/.local/bin/${APP_NAME}"
DESKTOP_TARGET="${HOME}/.local/share/applications/${APP_ID}.desktop"
AUTOSTART_TARGET="${HOME}/.config/autostart/${APP_ID}.desktop"

rm -f "$BIN_TARGET" "$DESKTOP_TARGET" "$AUTOSTART_TARGET"

for size in $ICON_SIZES; do
    ICON_DIR="${ICON_THEME_ROOT}/${size}/apps"
    for icon_name in $ICON_NAMES; do
        rm -f "${ICON_DIR}/${icon_name}.png"
    done
done

SCALABLE_ICON_DIR="${ICON_THEME_ROOT}/scalable/apps"
for icon_name in $ICON_NAMES; do
    rm -f "${SCALABLE_ICON_DIR}/${icon_name}.svg"
done

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${HOME}/.local/share/applications" >/dev/null 2>&1 || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q -t "${HOME}/.local/share/icons/hicolor" >/dev/null 2>&1 || true
fi

printf '%s\n' "Removed local CosmiCal install from ~/.local."
