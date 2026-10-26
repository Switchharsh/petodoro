#!/usr/bin/env bash
# Remove everything install.sh put in place. Your settings are kept unless
# you pass --purge.
set -euo pipefail

PREFIX="${PREFIX:-$HOME/.local}"
CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}"

pkill -x petodoro 2>/dev/null || true
rm -f "$PREFIX/bin/petodoro" \
      "$PREFIX/share/applications/petodoro.desktop" \
      "$PREFIX/share/icons/hicolor/256x256/apps/petodoro.png" \
      "$CONFIG/autostart/petodoro.desktop"
if [[ "${1:-}" == "--purge" ]]; then
    rm -rf "$CONFIG/petodoro"
fi
echo "Petodoro removed. Bye bye~"
