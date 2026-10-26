#!/usr/bin/env bash
# Remove Petodoro. Settings are kept unless you pass --purge.
set -euo pipefail
if [[ -x "$HOME/.local/bin/petodoro" ]]; then
    "$HOME/.local/bin/petodoro" --uninstall
else
    rm -f "$HOME/.local/share/applications/petodoro.desktop" \
          "$HOME/.local/share/icons/hicolor/256x256/apps/petodoro.png" \
          "${XDG_CONFIG_HOME:-$HOME/.config}/autostart/petodoro.desktop"
    echo "Petodoro removed."
fi
if [[ "${1:-}" == "--purge" ]]; then
    rm -rf -- "${XDG_CONFIG_HOME:-$HOME/.config}/petodoro"
    echo "Settings deleted too."
fi
