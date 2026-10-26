#!/usr/bin/env bash
# Build (from source) or copy (from a release tarball) and install Petodoro
# for the current user. No sudo needed.
#
#   ./install.sh        interactive
#   ./install.sh -y     accept all defaults (install Rust if missing,
#                       start on login, launch now)
set -euo pipefail
cd "$(dirname "$0")"

PREFIX="${PREFIX:-$HOME/.local}"
YES=0
[[ "${1:-}" == "-y" || "${1:-}" == "--yes" ]] && YES=1

ask() { # ask "question" -> 0 for yes (default), 1 for no
    [[ $YES == 1 ]] && return 0
    local a
    read -rp "$1 [Y/n] " a || true
    [[ ! "$a" =~ ^[nN] ]]
}

say() { printf '\033[1;35m==>\033[0m %s\n' "$*"; }

if [[ -f Cargo.toml ]]; then
    # 1. Rust toolchain (only needed to build).
    if ! command -v cargo >/dev/null 2>&1 && [[ -x "$HOME/.cargo/bin/cargo" ]]; then
        export PATH="$HOME/.cargo/bin:$PATH"
    fi
    if ! command -v cargo >/dev/null 2>&1; then
        say "Rust is not installed. It is only needed once, to build Petodoro."
        if ask "Install Rust now with rustup (https://rustup.rs, into ~/.cargo)?"; then
            if ! command -v curl >/dev/null 2>&1; then
                echo "curl is missing: run 'sudo apt install curl' and try again." >&2
                exit 1
            fi
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
            # shellcheck disable=SC1091
            source "$HOME/.cargo/env"
        else
            echo "Install Rust with 'sudo apt install cargo' or from https://rustup.rs, then re-run." >&2
            exit 1
        fi
    fi

    # 2. Build. Pure Rust: no -dev packages needed.
    say "Building (the first build takes a minute or two)..."
    cargo build --release --locked
    SRC=target/release/petodoro
else
    # Running from a prebuilt release tarball.
    SRC=./petodoro
fi

# 3. Install binary, icon and menu entry.
BIN="$PREFIX/bin/petodoro"
ICON_DIR="$PREFIX/share/icons/hicolor/256x256/apps"
APPS_DIR="$PREFIX/share/applications"
say "Installing to $BIN"
install -Dm755 "$SRC" "$BIN"
mkdir -p "$ICON_DIR" "$APPS_DIR"
"$BIN" --icon "$ICON_DIR/petodoro.png"
cat > "$APPS_DIR/petodoro.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Petodoro
GenericName=Desktop Pet Timer
Comment=Floating desktop pet with a pomodoro timer and stopwatch
Exec="$BIN"
Icon=petodoro
Terminal=false
Categories=Utility;Clock;
Keywords=pomodoro;timer;stopwatch;pet;focus;
EOF
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database -q "$APPS_DIR" || true

case ":$PATH:" in
    *":$PREFIX/bin:"*) ;;
    *) say "Note: $PREFIX/bin is not on your PATH yet (it will be after you log out and in)." ;;
esac

# 4. Autostart and launch.
if ask "Start Petodoro automatically when you log in?"; then
    "$BIN" --autostart on
fi
if ask "Launch Petodoro now?"; then
    nohup "$BIN" >/dev/null 2>&1 &
    say "Your pet should be in the bottom-right corner. Right-click it for the menu!"
fi
say "Done. Find Petodoro in the app menu, or run: petodoro --help"
