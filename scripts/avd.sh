#!/usr/bin/env bash
# TrustFund — Android emulator (AVD) helper for Linux
#
# Usage:
#   scripts/avd.sh list              # list available AVDs
#   scripts/avd.sh start [name]      # start an AVD (first one if name omitted)
#   scripts/avd.sh stop              # stop running emulator(s)
#
# Requires: ANDROID_HOME (or ANDROID_SDK_ROOT) set, with emulator + platform-tools installed.

set -euo pipefail

# Resolve SDK: env vars first, then the default install location.
SDK="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-}}"
if [ -z "$SDK" ] || [ ! -d "$SDK" ]; then
  if [ -d "$HOME/Android/Sdk" ]; then
    SDK="$HOME/Android/Sdk"
  fi
fi
if [ -z "$SDK" ] || [ ! -d "$SDK" ]; then
  echo "Android SDK not found. Set ANDROID_HOME (or ANDROID_SDK_ROOT), or install it to ~/Android/Sdk." >&2
  exit 1
fi

EMULATOR="$SDK/emulator/emulator"
ADB="$SDK/platform-tools/adb"

if [ ! -x "$EMULATOR" ]; then
  echo "emulator not found at $EMULATOR. Install the 'emulator' SDK package." >&2
  exit 1
fi

CMD="${1:-list}"
NAME="${2:-}"

case "$CMD" in
  list)
    "$EMULATOR" -list-avds
    ;;
  start)
    if [ -z "$NAME" ]; then
      NAME="$("$EMULATOR" -list-avds | head -n 1)"
      if [ -z "$NAME" ]; then
        echo "No AVDs found. Create one in Android Studio's Device Manager." >&2
        exit 1
      fi
      echo "No name given; starting first AVD: $NAME"
    fi
    echo "Starting emulator '$NAME'…"
    # Launch detached so the terminal is free for the Tauri build.
    nohup "$EMULATOR" -avd "$NAME" >/dev/null 2>&1 &
    echo "Waiting for device to come online…"
    "$ADB" wait-for-device
    echo "Emulator is online. Run: pnpm run android:dev:linux"
    ;;
  stop)
    echo "Stopping running emulator(s)…"
    DEVICES="$("$ADB" devices | grep -oE 'emulator-[0-9]+' || true)"
    if [ -z "$DEVICES" ]; then
      echo "No running emulators."
      exit 0
    fi
    for d in $DEVICES; do
      echo "  killing $d"
      "$ADB" -s "$d" emu kill
    done
    ;;
  *)
    echo "Unknown command: $CMD (use list|start|stop)" >&2
    exit 1
    ;;
esac
