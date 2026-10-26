#!/usr/bin/env bash
# Wrap a (universal) macOS binary into Petodoro.app and zip it. Run on macOS.
#   scripts/build-macos-app.sh path/to/petodoro 0.3.0 [outdir]
set -euo pipefail
bin="$1"
version="$2"
out="${3:-.}"
work="$(mktemp -d)"
trap 'rm -rf -- "$work"' EXIT

app="$work/Petodoro.app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$bin" "$app/Contents/MacOS/petodoro"
chmod 755 "$app/Contents/MacOS/petodoro"

# Icon: render the mascot, then build an .icns with the system tools.
"$bin" --icon "$work/icon.png"
iconset="$work/petodoro.iconset"
mkdir -p "$iconset"
for size in 16 32 128 256; do
    sips -z "$size" "$size" "$work/icon.png" --out "$iconset/icon_${size}x${size}.png" >/dev/null
    double=$((size * 2))
    sips -z "$double" "$double" "$work/icon.png" --out "$iconset/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$iconset" -o "$app/Contents/Resources/petodoro.icns"

cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleName</key><string>Petodoro</string>
<key>CFBundleDisplayName</key><string>Petodoro</string>
<key>CFBundleIdentifier</key><string>com.petodoro</string>
<key>CFBundleExecutable</key><string>petodoro</string>
<key>CFBundleIconFile</key><string>petodoro</string>
<key>CFBundleVersion</key><string>$version</string>
<key>CFBundleShortVersionString</key><string>$version</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>LSMinimumSystemVersion</key><string>11.0</string>
<key>LSUIElement</key><true/>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
PLIST

# Ad-hoc signature so Apple Silicon will run it at all.
codesign --force --deep --sign - "$app" >/dev/null 2>&1 || true
(cd "$work" && ditto -c -k --keepParent Petodoro.app petodoro-macos.zip)
mv "$work/petodoro-macos.zip" "$out/"
echo "$out/petodoro-macos.zip"
