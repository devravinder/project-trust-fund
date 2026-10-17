# Sync TrustFund's generated Android launcher icons into the generated Android
# project, replacing whatever Tauri scaffolded. Run after `tauri android init`:
#   pnpm tauri icon public\logo.svg
#   powershell -ExecutionPolicy Bypass -File scripts\sync-android-icons.ps1
# Then: pnpm run android:build:apk

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$src = Join-Path $root 'src-tauri\icons\android'
$dst = Join-Path $root 'src-tauri\gen\android\app\src\main\res'

if (-not (Test-Path $src)) {
  Write-Error "Source icons not found at $src. Run: pnpm tauri icon public\logo.svg"
  exit 1
}
if (-not (Test-Path $dst)) {
  Write-Error "Android project not found at $dst. Run: pnpm run android:init"
  exit 1
}

Write-Host "Copying launcher icons: $src -> $dst"
Get-ChildItem $src -Directory -Filter 'mipmap-*' | ForEach-Object {
  Copy-Item $_.FullName -Destination $dst -Recurse -Force
}
if (Test-Path (Join-Path $src 'values')) {
  Copy-Item (Join-Path $src 'values\*') -Destination (Join-Path $dst 'values') -Recurse -Force
}
Write-Host "Done. Rebuild the APK: pnpm run android:build:apk"
