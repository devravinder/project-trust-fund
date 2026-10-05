# TrustFund — Android emulator (AVD) helper for Windows (PowerShell)
#
# Usage:
#   scripts\avd.ps1 list                 # list available AVDs
#   scripts\avd.ps1 start [name]         # start an AVD (first one if name omitted)
#   scripts\avd.ps1 stop                 # stop running emulator(s)
#
# Requires: ANDROID_HOME (or ANDROID_SDK_ROOT) set, with emulator + platform-tools installed.

$ErrorActionPreference = 'Stop'

# Parse arguments from the automatic $args array. pnpm forwards a literal "--"
# separator; drop it and any empty tokens. Remaining: [command] [name].
$tokens = @($args | Where-Object { $_ -and $_ -ne '--' })
$Command = if ($tokens.Count -ge 1) { [string]$tokens[0] } else { 'list' }
$Name = if ($tokens.Count -ge 2) { [string]$tokens[1] } else { '' }

if ($Command -notin @('list', 'start', 'stop')) {
  Write-Error "Unknown command '$Command'. Use: list | start [name] | stop"
  exit 1
}

# Resolve the Android SDK path robustly, in order:
#   1. process env (ANDROID_HOME / ANDROID_SDK_ROOT)
#   2. persisted User/Machine env (reads registry, so it works even if this
#      terminal was launched before the var was set — common in VS Code)
#   3. the default install location
function Resolve-Sdk {
  foreach ($v in @($env:ANDROID_HOME, $env:ANDROID_SDK_ROOT)) {
    if ($v -and (Test-Path $v)) { return $v }
  }
  foreach ($scope in @('User', 'Machine')) {
    foreach ($name in @('ANDROID_HOME', 'ANDROID_SDK_ROOT')) {
      $v = [Environment]::GetEnvironmentVariable($name, $scope)
      if ($v -and (Test-Path $v)) { return $v }
    }
  }
  $default = Join-Path $env:LOCALAPPDATA 'Android\Sdk'
  if (Test-Path $default) { return $default }
  return $null
}

$sdk = Resolve-Sdk
if (-not $sdk) {
  Write-Error "Android SDK not found. Set ANDROID_HOME (or ANDROID_SDK_ROOT), or install the SDK to %LOCALAPPDATA%\Android\Sdk. If you just set it, restart VS Code so the terminal picks it up."
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
