#!/usr/bin/env bash
# One-line installer:
#   curl -fsSL https://raw.githubusercontent.com/Switchharsh/petodoro/HEAD/get.sh | bash
# Downloads the latest prebuilt release (or builds from source if there is
# none), installs it for the current user, and starts your pet.
set -euo pipefail
REPO="Switchharsh/petodoro"
tmp="$(mktemp -d)"
trap 'rm -rf -- "$tmp"' EXIT

url=""
if [[ "$(uname -m)" == "x86_64" ]]; then
    url="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" 2>/dev/null \
        | grep -o '"browser_download_url": *"[^"]*x86_64-linux.tar.gz"' | cut -d'"' -f4 || true)"
fi

if [[ -n "$url" ]]; then
    echo "==> Downloading $url"
    curl -fsSL "$url" | tar -xz -C "$tmp"
    "$tmp/petodoro/petodoro" --install
else
    echo "==> No prebuilt release for this machine yet; building from source."
    if ! command -v git >/dev/null 2>&1; then
        echo "Please install git first:  sudo apt install git" >&2
        exit 1
    fi
    git clone --depth 1 "https://github.com/$REPO" "$tmp/src"
    bash "$tmp/src/install.sh"
fi
