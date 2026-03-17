#!/bin/sh
set -eu

APP_ID="io.github.sixesandsevens.cosmical"

AUTOSTART_TARGET="${HOME}/.config/autostart/${APP_ID}.desktop"

if [ -f "$AUTOSTART_TARGET" ]; then
    rm -f "$AUTOSTART_TARGET"
    printf '%s\n' "Autostart disabled."
else
    printf '%s\n' "Autostart was not enabled."
fi
