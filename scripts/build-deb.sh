#!/usr/bin/env bash
# Package a built petodoro binary as a .deb (double-click to install on Mint).
#   scripts/build-deb.sh target/release/petodoro 0.3.0 [outdir]
# Set DEB_ARCH to override the architecture (default: this machine's).
set -euo pipefail
bin="$1"
version="$2"
out="${3:-.}"
arch="${DEB_ARCH:-$(dpkg --print-architecture)}"
root="$(mktemp -d)"
trap 'rm -rf -- "$root"' EXIT

install -Dm755 "$bin" "$root/usr/bin/petodoro"
mkdir -p "$root/usr/share/icons/hicolor/256x256/apps" "$root/usr/share/applications" \
         "$root/usr/share/doc/petodoro" "$root/DEBIAN"
"$bin" --icon "$root/usr/share/icons/hicolor/256x256/apps/petodoro.png"
cp "$(dirname "$0")/../LICENSE" "$root/usr/share/doc/petodoro/copyright"
cat > "$root/usr/share/applications/petodoro.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Petodoro
GenericName=Desktop Pet Timer
Comment=Floating desktop pet with a pomodoro timer and stopwatch
Exec=petodoro
Icon=petodoro
Terminal=false
Categories=Utility;Clock;
Keywords=pomodoro;timer;stopwatch;pet;focus;
DESKTOP
size_kb="$(du -sk "$root/usr" | cut -f1)"
# The newest glibc symbol the binary uses decides the minimum libc6.
glibc="$(objdump -T "$bin" 2>/dev/null | grep -o 'GLIBC_[0-9.]*' | sort -uV | tail -1 | cut -d_ -f2 || true)"
glibc="${glibc:-2.34}"
cat > "$root/DEBIAN/control" <<CONTROL
Package: petodoro
Version: $version
Architecture: $arch
Maintainer: Petodoro contributors <petodoro@users.noreply.github.com>
Installed-Size: $size_kb
Depends: libc6 (>= $glibc)
Recommends: libnotify-bin, pulseaudio-utils | pipewire-bin
Section: utils
Priority: optional
Homepage: https://github.com/Switchharsh/petodoro
Description: floating desktop pet with a pomodoro timer and stopwatch
 A tiny always-on-top pet (anime girls, cats, a ghost, a robot...) that you
 can drag anywhere. It keeps you company with a pomodoro timer and a
 stopwatch, talks, gets dizzy, eats snacks and wanders around.
 Uses about 5 MB of RAM and almost no CPU.
CONTROL
deb="$out/petodoro_${version}_${arch}.deb"
dpkg-deb --root-owner-group --build "$root" "$deb" >/dev/null
echo "$deb"
