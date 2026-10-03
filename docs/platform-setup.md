# TrustFund — Platform Setup (Linux & Android)

Concrete setup for running on Linux (dev) and building the Android APK from
Linux or Windows. Desktop Windows is covered in the README.

> **Rust toolchain note:** these flows require **rustup** (to add Android/Linux
> targets). If `rustup` is missing, install it from https://rustup.rs. A
> standalone (non-rustup) Rust install cannot add cross-compilation targets.

---

## 1. Linux — desktop dev mode

### System packages
Tauri v2 needs the WebKitGTK + build toolchain. On Debian/Ubuntu:

```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  build-essential curl wget file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  pkg-config
```

Fedora:

```bash
sudo dnf install -y webkit2gtk4.1-devel openssl-devel curl wget file \
  libappindicator-gtk3-devel librsvg2-devel gcc gcc-c++ make
```

Arch:

```bash
sudo pacman -S --needed webkit2gtk-4.1 base-devel curl wget file openssl \
  libayatana-appindicator librsvg
```

> libSQL compiles bundled SQLite (C). `build-essential` / `gcc` covers the C
> compiler on Linux — no extra step like Windows' MSVC wrapper is needed.

### Run
```bash
pnpm install
pnpm run tauri:dev:linux
```

### Build desktop package
```bash
pnpm run tauri:build:linux          # .deb / .rpm / .AppImage (per distro)
```

---

## 2. Android — build the APK (Linux or Windows)

### 2.1 Prerequisites (both OSes)

1. **JDK 17** (Temurin/OpenJDK 17 recommended).
2. **Android SDK + NDK** — easiest via **Android Studio** (install the SDK,
   Platform-Tools, and a recent **NDK** from SDK Manager), or the command-line
   tools.
3. **Rust Android targets** (requires rustup):
   ```bash
   rustup target add aarch64-linux-android armv7-linux-androideabi \
     i686-linux-android x86_64-linux-android
   ```

### 2.2 Environment variables

Point Tauri at the SDK/NDK/JDK. Use the real paths on your machine.

**Linux (bash, add to ~/.bashrc):**
```bash
export JAVA_HOME="/usr/lib/jvm/java-17-openjdk"
export ANDROID_HOME="$HOME/Android/Sdk"
export NDK_HOME="$ANDROID_HOME/ndk/<ndk-version>"   # e.g. 27.1.12297006
export PATH="$ANDROID_HOME/platform-tools:$PATH"
```

**Windows (PowerShell, user env vars):**
```powershell
setx JAVA_HOME "C:\Program Files\Eclipse Adoptium\jdk-17"
setx ANDROID_HOME "$env:LOCALAPPDATA\Android\Sdk"
setx NDK_HOME "$env:LOCALAPPDATA\Android\Sdk\ndk\<ndk-version>"
# Add %ANDROID_HOME%\platform-tools to PATH
```
On Windows, Android builds still need the **MSVC C++ toolchain** for host-side
build steps, so run Android scripts through the MSVC wrapper (the `:win`
scripts below already do this).

### 2.3 One-time: initialize the Android project

This generates `src-tauri/gen/android` (committed to the repo).

```bash
# Linux
pnpm run android:init

# Windows
pnpm run android:init:win
```

### 2.4 Manage the emulator (AVD) from the command line

You need an emulator (AVD) created once. The easiest way to **create** one is
Android Studio → **Virtual Device Manager** (pick e.g. Pixel 7 + API 34). You
do not need to write any code there — it's just the AVD creator.

Once an AVD exists, start/stop it from the terminal with the helper scripts
(no need to reopen Android Studio):

```bash
# Linux
pnpm run avd:list:linux                 # list AVDs
pnpm run avd:start:linux                # start the first AVD
pnpm run avd:start:linux -- Pixel_7     # start a named AVD
pnpm run avd:stop:linux                 # stop running emulator(s)
```

```powershell
# Windows
pnpm run avd:list:win                   # list AVDs
pnpm run avd:start:win                  # start the first AVD
pnpm run avd:start:win -- Pixel_7       # start a named AVD
pnpm run avd:stop:win                   # stop running emulator(s)
```

`avd:start` launches the emulator detached and waits until the device is
online, so the terminal is free for the build. Verify with `adb devices`.

> These scripts use `ANDROID_HOME` (or `ANDROID_SDK_ROOT`) to find
> `emulator` and `adb`. Make sure that env var is set (section 2.2).

### 2.5 Run on a device/emulator (dev)

Connect a device (USB debugging on) or start an emulator (above), then:

```bash
# Linux
pnpm run android:dev:linux

# Windows
pnpm run android:dev:win
```

### 2.6 Build the APK

```bash
# Linux
pnpm run android:build:apk:linux

# Windows
pnpm run android:build:apk:win
```

Output APK (debug/unsigned) is under:
```
src-tauri/gen/android/app/build/outputs/apk/
```

### 2.7 Signing (for sharing the APK)

A **debug** APK installs on your own device for testing. To share a stable APK
with friends, sign a **release** APK:

1. Create a keystore (once):
   ```bash
   keytool -genkey -v -keystore trustfund.jks -keyalg RSA -keysize 2048 \
     -validity 10000 -alias trustfund
   ```
2. Configure signing in `src-tauri/gen/android/app/build.gradle.kts` (or a
   `keystore.properties`) per Tauri's Android signing docs.
3. Build the signed release APK:
   ```bash
   pnpm tauri android build --apk
   ```
4. Share the resulting `.apk`; friends enable "install from unknown sources"
   and install it directly (no Play Store, free).

> Keep `trustfund.jks` and its passwords **out of git** (already covered by the
> credential/secret ignore patterns — do not commit keystores).

---

## 3. Android — camera permission (QR scanning)

The "Connect by scanning QR" feature uses the device camera. After
`android:init`, add the camera permission to
`src-tauri/gen/android/app/src/main/AndroidManifest.xml`:

```xml
<uses-permission android:name="android.permission.CAMERA" />
<uses-feature android:name="android.hardware.camera" android:required="false" />
```

On desktop (no camera), use the manual-entry fallback in the Connect dialog.

## 4. libSQL on Android — note

The libSQL embedded replica includes native (C/Rust) code. It compiles for
Android via the NDK and the Rust Android targets configured above. This is the
one area to validate first on a real device/emulator (the Android spike noted
in `architecture.md`). If the embedded replica misbehaves on Android, the
fallback is the libSQL **remote** client (online-only, no local replica).

---

## 5. Quick reference — scripts

| Script | Platform | Purpose |
|---|---|---|
| `tauri:dev:linux` | Linux | Desktop dev |
| `tauri:build:linux` | Linux | Desktop package |
| `android:init` / `android:init:win` | Linux / Win | One-time Android project init |
| `avd:list:linux` / `avd:list:win` | Linux / Win | List emulators (AVDs) |
| `avd:start:linux` / `avd:start:win` | Linux / Win | Start an emulator (optional name) |
| `avd:stop:linux` / `avd:stop:win` | Linux / Win | Stop running emulator(s) |
| `android:dev:linux` / `android:dev:win` | Linux / Win | Run on device/emulator |
| `android:build:apk:linux` / `android:build:apk:win` | Linux / Win | Build APK |
| `tauri:dev:win` / `tauri:build:win` | Windows | Desktop dev / package |
