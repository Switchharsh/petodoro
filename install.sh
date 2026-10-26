#!/usr/bin/env bash
# Build (from source) or take the bundled binary (release tarball), then
# install Petodoro for the current user and start it. No sudo needed.
set -euo pipefail
cd "$(dirname "$0")"
say() { printf '\033[1;35m==>\033[0m %s\n' "$*"; }

if [[ -f Cargo.toml ]]; then
    if ! command -v cargo >/dev/null 2>&1 && [[ -x "$HOME/.cargo/bin/cargo" ]]; then
        export PATH="$HOME/.cargo/bin:$PATH"
    fi
    if ! command -v cargo >/dev/null 2>&1; then
        say "Installing Rust (only needed once, to build) from https://rustup.rs ..."
        if ! command -v curl >/dev/null 2>&1; then
            echo "curl is missing: run 'sudo apt install curl' and try again." >&2
            exit 1
        fi
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
        # shellcheck disable=SC1091
        source "$HOME/.cargo/env"
    fi
    say "Building (the first build takes a minute or two)..."
    cargo build --release --locked
    BIN=target/release/petodoro
else
    BIN=./petodoro
fi

"$BIN" --install
case ":$PATH:" in
    *":$HOME/.local/bin:"*) ;;
    *) say "Tip: log out and back in so the 'petodoro' command is on your PATH." ;;
esac
