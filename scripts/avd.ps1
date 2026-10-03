# TrustFund — Android emulator (AVD) helper for Windows (PowerShell)
#
# Usage:
#   scripts\avd.ps1 list                 # list available AVDs
#   scripts\avd.ps1 start [name]         # start an AVD (first one if name omitted)
#   scripts\avd.ps1 stop                 # stop running emulator(s)
#
# Requires: ANDROID_HOME (or ANDROID_SDK_ROOT) set, with emulator + platform-tools installed.

param(
  [Parameter(Position = 0)]
  [ValidateSet('list', 'start', 'stop')]
  [string]$Command = 'list',

  [Parameter(Position = 1)]
  [string]$Name
)

$ErrorActionPreference = 'Stop'

$sdk = $env:ANDROID_HOME
if (-not $sdk) { $sdk = $env:ANDROID_SDK_ROOT }
if (-not $sdk) {
  Write-Error "ANDROID_HOME (or ANDROID_SDK_ROOT) is not set. Point it at your Android SDK."
  exit 1
}

$emulator = Join-Path $sdk 'emulator\emulator.exe'
$adb = Join-Path $sdk 'platform-tools\adb.exe'

if (-not (Test-Path $emulator)) {
  Write-Error "emulator not found at $emulator. Install the 'emulator' SDK package."
  exit 1
}

switch ($Command) {
  'list' {
    & $emulator -list-avds
  }
  'start' {
    if (-not $Name) {
      $Name = (& $emulator -list-avds | Select-Object -First 1)
      if (-not $Name) { Write-Error "No AVDs found. Create one in Android Studio's Device Manager."; exit 1 }
      Write-Host "No name given; starting first AVD: $Name"
    }
    Write-Host "Starting emulator '$Name'…"
    # Launch detached so the terminal is free for the Tauri build.
    Start-Process -FilePath $emulator -ArgumentList @('-avd', $Name)
    Write-Host "Waiting for device to come online…"
    & $adb wait-for-device
    Write-Host "Emulator is online. Run: pnpm run android:dev:win"
  }
  'stop' {
    Write-Host "Stopping running emulator(s)…"
    $devices = & $adb devices | Select-String 'emulator-\d+' | ForEach-Object { ($_ -split '\s+')[0] }
    if (-not $devices) { Write-Host "No running emulators."; break }
    foreach ($d in $devices) {
      Write-Host "  killing $d"
      & $adb -s $d emu kill
    }
  }
}
