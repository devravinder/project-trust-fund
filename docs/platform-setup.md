# TrustFund — Platform Setup (Linux & Android)

Setup for running on Linux (dev) and building the Android APK. Desktop Windows
is covered in the README.

> **Data layer is pure Rust** (JSON file + Turso over HTTP) — no native SQLite,
> so **no C compiler / MSVC / NDK is needed for the data layer**. The only
> native-toolchain needs are Tauri's general requirements (WebKitGTK on Linux,
> the Android SDK/NDK + rustup for Android builds).

> **Rust toolchain note:** Android/Linux cross-target builds require **rustup**
> (to add targets). A standalone (non-rustup) Rust install cannot add targets.
> Install rustup from <https://rustup.rs>.

---

## 1. Linux — desktop dev mode

### System packages

Tauri v2 needs WebKitGTK + a build toolchain. On Debian/Ubuntu:

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

### Run / build

```bash
pnpm install
pnpm run tauri:dev            # desktop dev (hot reload)
pnpm run tauri:build          # .deb / .rpm / .AppImage
```

The desktop/dev and build scripts are the same on every OS now (no MSVC wrapper
needed since the data layer is pure Rust).

---

## 2. Android — build the APK

### 2.1 Prerequisites

1. **rustup** + Android targets:

   ```bash
   rustup target add aarch64-linux-android armv7-linux-androideabi \
     i686-linux-android x86_64-linux-android
   ```

2. **JDK 17** (Temurin/OpenJDK 17).
3. **Android SDK + NDK** — easiest via **Android Studio** (install SDK,
   Platform-Tools, and a recent **NDK**).

### 2.2 Environment variables

**Linux (bash, ~/.bashrc):**

```bash
export JAVA_HOME="/usr/lib/jvm/java-17-openjdk"
export ANDROID_HOME="$HOME/Android/Sdk"
export NDK_HOME="$ANDROID_HOME/ndk/<ndk-version>"
export PATH="$ANDROID_HOME/platform-tools:$PATH"
```

**Windows (PowerShell, user env vars):**

```powershell
setx JAVA_HOME "C:\Program Files\Eclipse Adoptium\jdk-17"
setx ANDROID_HOME "$env:LOCALAPPDATA\Android\Sdk"
setx NDK_HOME "$env:LOCALAPPDATA\Android\Sdk\ndk\<ndk-version>"
# Add %ANDROID_HOME%\platform-tools to PATH, then restart your terminal/VS Code.
```

### 2.3 One-time: initialize the Android project

Generates `src-tauri/gen/android`.

```bash
pnpm run android:init
```

### 2.4 Manage the emulator (AVD) from the command line

Create an AVD once in Android Studio → **Virtual Device Manager** (e.g. a Pixel
image). Then start/stop it from the terminal (AVD scripts stay per-OS because
they use PowerShell vs bash):

```bash
# Linux
pnpm run avd:list:linux
pnpm run avd:start:linux                 # first AVD
pnpm run avd:start:linux -- Pixel_7      # named AVD
pnpm run avd:stop:linux
```

```powershell
# Windows
pnpm run avd:list:win
pnpm run avd:start:win
pnpm run avd:start:win -- Pixel_3a_API_36_17_x86_64
pnpm run avd:stop:win
```

`avd:start` launches detached and waits for the device to come online, then the
terminal is free. Verify with `adb devices`. The scripts resolve the SDK from
`ANDROID_HOME` / `ANDROID_SDK_ROOT` (reading the persisted value, so they work in
VS Code even if the terminal predates the env var), falling back to the default
install path.

### 2.5 Run on a device/emulator (dev)

Connect a device (USB debugging) or start an emulator, then:

```bash
pnpm run android:dev
```

### 2.6 Build the APK

```bash
pnpm run android:build:apk
```

Output (debug/unsigned): `src-tauri/gen/android/app/build/outputs/apk/`

### 2.7 Signing (to share a release APK)

```bash
keytool -genkey -v -keystore trustfund.jks -keyalg RSA -keysize 2048 \
  -validity 10000 -alias trustfund
```

Configure signing in `src-tauri/gen/android/app/build.gradle.kts` (or a
`keystore.properties`) per Tauri's Android signing docs, then
`pnpm run android:build:apk`. Share the `.apk`; friends enable "install from
unknown sources". **Keep the keystore out of git.**

or

1. set path

APKSIGNER=~/Android/Sdk/build-tools/35.0.0/apksigner

1. Use the Android debug keystore
   - normally exists here: ~/.android/debug.keystore

2. Sign the APK

   ```bash
     $APKSIGNER sign \

    --ks ~/.android/debug.keystore \
    --ks-key-alias androiddebugkey \
    --ks-pass pass:android \
    --key-pass pass:android \
    --out app-universal-signed.apk \
    app-universal-release-unsigned.apk

   ```

---

## 3. Android — camera permission (QR scanning)

"Connect by scanning QR" uses the camera. After `android:init`, add to
`src-tauri/gen/android/app/src/main/AndroidManifest.xml`:

```xml
<uses-permission android:name="android.permission.CAMERA" />
<uses-feature android:name="android.hardware.camera" android:required="false" />
```

On desktop (no camera), use the manual-entry fallback in the Connect dialog.

---

## 4. Known build constraint (JFrog mirror)

`tauri build` / `tauri android build` enforce that the Rust `tauri` crate and
the npm `@tauri-apps/*` packages share the same major.minor. In the current
environment the JFrog npm mirror caps `@tauri-apps/api` at **2.11.1** while the
Rust crates resolve to **2.12** (the mirrored Rust `tauri` 2.11.x is a broken
publish). Until the mirror carries the 2.12 npm packages, the CLI bundling step
is blocked.

Workaround for a desktop binary without the CLI check:

```bash
pnpm run build:binary        # frontend + cargo build --release
```

→ `src-tauri/target/release/trust-fund.exe`. Dev (`tauri dev`) only warns.

To unblock full `tauri build` / APK bundling: ask the JFrog admin to mirror
`@tauri-apps/api`, `@tauri-apps/cli`, `@tauri-apps/plugin-store` **2.12.x** (or a
non-corrupt Rust `tauri` 2.11.x).

---

## 5. Quick reference — scripts

| Script | Purpose |
| --- | --- |
| `tauri:dev` | Desktop dev (hot reload) |
| `tauri:build` | Desktop package (installer) |
| `build:binary` | Desktop release binary (bypasses CLI version check) |
| `android:init` | One-time Android project init |
| `android:dev` | Run on device/emulator |
| `android:build:apk` | Build APK |
| `avd:list:win` / `avd:list:linux` | List emulators |
| `avd:start:win` / `avd:start:linux` | Start an emulator (optional name) |
| `avd:stop:win` / `avd:stop:linux` | Stop running emulator(s) |
