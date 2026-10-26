#!/usr/bin/env bash
# Install the APK on a running emulator, show the pet, poke it, and save
# screenshots (used by CI to check the Android build really works).
#   scripts/android-smoke.sh petodoro-android.apk out-dir
set -euxo pipefail
apk="$1"
out="$2"
pkg=io.github.switchharsh.petodoro
mkdir -p "$out"
shot() { adb exec-out screencap -p > "$out/$1.png"; }

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
adb shell input swipe "$px" "$py" "$((px - 300))" "$((py - 500))" 800   # drag the pet
sleep 3
shot 5-dragged
adb shell pidof "$pkg" > "$out/pid.txt" || echo "NOT RUNNING" > "$out/pid.txt"
adb logcat -d -t 400 > "$out/logcat.txt" || true
