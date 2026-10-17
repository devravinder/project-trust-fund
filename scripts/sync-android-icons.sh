#!/usr/bin/env bash
# Sync TrustFund's generated Android launcher icons into the generated Android
# project, replacing whatever Tauri scaffolded (which may be Tauri's defaults).
#
# Run after `tauri android init` (or any time the logo changes) on the machine
# that has src-tauri/gen/android:
#   pnpm tauri icon public/logo.svg   # regenerate from the logo first
#   bash scripts/sync-android-icons.sh
#
# Then rebuild the APK: pnpm run android:build:apk

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/src-tauri/icons/android"
DST="$ROOT/src-tauri/gen/android/app/src/main/res"

if [ ! -d "$SRC" ]; then
  echo "Source icons not found at $SRC. Run: pnpm tauri icon public/logo.svg" >&2
  exit 1
fi
if [ ! -d "$DST" ]; then
  echo "Android project not found at $DST. Run: pnpm run android:init" >&2
  exit 1
fi

echo "Copying launcher icons: $SRC -> $DST"
cp -rv "$SRC"/mipmap-* "$DST"/ 2>/dev/null || cp -r "$SRC"/mipmap-* "$DST"/
# Also copy the adaptive-icon background/foreground resources if present.
if [ -d "$SRC/values" ]; then
  mkdir -p "$DST/values"
  cp -r "$SRC/values/." "$DST/values/"
fi

echo "Done. Rebuild the APK: pnpm run android:build:apk"
