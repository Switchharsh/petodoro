#!/usr/bin/env bash
# Install the APK on a running emulator, show the pet, poke it, and save
# screenshots (used by CI to check the Android build really works).
#   scripts/android-smoke.sh petodoro-android.apk out-dir
set -euxo pipefail
apk="$1"
out="$2"
pkg=io.github.switchharsh.petodoro
mkdir -p "$out"
# Save a screenshot, and also print a small JPEG of it (base64) plus the
# app's state into the log, so it can be checked without the artifact.
shot() {
    adb exec-out screencap -p > "$out/$1.png"
    set +x
    echo "STATE $1: pid=$(adb shell pidof "$pkg" | tr -d '\r') overlays=$(adb shell dumpsys window windows | grep -c "$pkg")"
    if command -v ffmpeg >/dev/null; then
        ffmpeg -loglevel error -y -i "$out/$1.png" -vf scale=200:-1 -q:v 18 "$out/$1.jpg"
        echo "BEGIN-SHOT $1"
        base64 -w 0 "$out/$1.jpg"; echo
        echo "END-SHOT $1"
    fi
    set -x
}

adb install -r "$apk"
adb shell appops set "$pkg" SYSTEM_ALERT_WINDOW allow
adb shell pm grant "$pkg" android.permission.POST_NOTIFICATIONS || true
adb shell am start -n "$pkg/.MainActivity"
sleep 8
shot 1-setup-screen
adb shell input keyevent KEYCODE_HOME
sleep 6
shot 2-pet-on-home-screen

# Same placement maths as the app: bottom-right corner of the screen.
read -r W H < <(adb shell wm size | tr -d '\r' | awk -F'[: x]+' '/Physical/{print $3, $4}')
dpi=$(adb shell wm density | tr -d '\r' | awk -F': ' '/Physical/{print $2}')
read -r bx by px py < <(awk -v W="$W" -v H="$H" -v dpi="$dpi" 'BEGIN {
  d = dpi / 160; ui = d / 1.6; if (ui < 1) ui = 1; s = 1.5 * ui
  w = int(120 * s + 0.999); h = int(160 * s + 0.999)
  x0 = W - w - 60; y0 = (H - int(48 * d)) - h - 40
  printf "%d %d %d %d\n", x0 + 60 * s, y0 + 24 * s, x0 + 60 * s, y0 + 125 * s }')
echo "screen ${W}x${H} @${dpi}dpi, bubble at $bx,$by, pet at $px,$py"

adb shell input tap "$bx" "$by"        # start the focus timer
sleep 3
shot 3-timer-started
adb shell input swipe "$px" "$py" "$px" "$py" 1500   # long press: menu
sleep 3
shot 4-menu
adb shell input keyevent KEYCODE_BACK
tx=$((px > 350 ? px - 300 : 50)); ty=$((py > 650 ? py - 600 : 50))
adb shell input swipe "$px" "$py" "$tx" "$ty" 800   # drag the pet
sleep 3
shot 5-dragged
adb shell pidof "$pkg" > "$out/pid.txt" || echo "NOT RUNNING" > "$out/pid.txt"
adb logcat -d -t 400 > "$out/logcat.txt" || true
