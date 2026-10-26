#!/usr/bin/env bash
# One-line installer for Linux and macOS:
#   curl -fsSL https://raw.githubusercontent.com/Switchharsh/petodoro/HEAD/get.sh | bash
# Downloads the latest release for this machine (or builds from source if
# there is none), installs it for the current user, and starts your pet.
set -euo pipefail
REPO="Switchharsh/petodoro"
BASE="https://github.com/$REPO/releases/latest/download"
tmp="$(mktemp -d)"
trap 'rm -rf -- "$tmp"' EXIT

case "$(uname -s)-$(uname -m)" in
    Linux-x86_64) asset="petodoro-x86_64-linux.tar.gz" ;;
    Linux-aarch64 | Linux-arm64) asset="petodoro-arm64-linux.tar.gz" ;;
    Darwin-*) asset="petodoro-macos.zip" ;;
    *) asset="" ;;
esac

if [[ -n "$asset" ]] && curl -fsSL -o "$tmp/$asset" "$BASE/$asset"; then
    echo "==> Downloaded $asset"
    if [[ "$asset" == *.zip ]]; then
        ditto -x -k "$tmp/$asset" "$tmp"
        xattr -dr com.apple.quarantine "$tmp/Petodoro.app" 2>/dev/null || true
        "$tmp/Petodoro.app/Contents/MacOS/petodoro" --install
    else
        tar -xzf "$tmp/$asset" -C "$tmp"
        "$tmp/petodoro/petodoro" --install
    fi
else
    echo "==> No prebuilt release for this machine; building from source."
    if ! command -v git >/dev/null 2>&1; then
        echo "Please install git first (Linux: sudo apt install git)." >&2
        exit 1
    fi
    git clone --depth 1 "https://github.com/$REPO" "$tmp/src"
    bash "$tmp/src/install.sh"
fi
