# WhatsApp Lite

A small, open-source Tauri 2 desktop wrapper that opens `https://web.whatsapp.com` in the operating system's WebView. There is no custom messaging backend and no custom chat UI.

> **Status:** The Rust project type-checks and an optimized Ubuntu 24.04 Linux build produced a Debian package (2,614,346 bytes / about 2.49 MiB). The graphical app was not launched, and no Windows or macOS build or runtime test was performed. See [Validation and known limitations](#validation-and-known-limitations) before treating it as a release build.

## Implemented

- Direct WhatsApp Web window in the native WebView, with a persistent (non-incognito) WebView profile.
- System tray menu: Open WhatsApp, Minimize to tray, Launch at login, Desktop notifications, and Quit. Tray left-click restores; double-click toggles visibility. Window close hides to tray.
- Global show/hide shortcut: **Ctrl+Alt+W** on Windows/Linux; **Cmd+Option+W** on macOS.
- Optional launch-at-login checkbox, implemented with Tauri's autostart plugin. Autostart launches hidden to tray.
- Last window outer position and size, plus the notification toggle, stored as small JSON preferences in the OS app config directory. No chat content or session data is written by the Rust app.
- Native notification adapter for the page's `Notification` API using Tauri's notification plugin. Notification activation attempts to restore/focus the window.
- The WebView theme is left at the system default (`theme: None`) so the platform WebView follows the OS appearance. WhatsApp's own theme choice remains controlled by WhatsApp Web.
- Navigation is limited to `https://web.whatsapp.com`; popup windows are denied. The remote capability grants WhatsApp Web only notification APIs and the three window methods needed for notification-click restore.

## Requirements

- Rust stable **1.90 or newer** (the current Tauri plugin workspace requires this baseline).
- Node.js 20+ and npm.
- Platform prerequisites from the [Tauri 2 setup guide](https://v2.tauri.app/start/prerequisites/).

### Linux (Ubuntu 22.04+ / Debian family)

Install Tauri's native dependencies (WebKitGTK **4.1**, not the older 4.0 package name in some older guides):

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

Other distributions need their equivalent WebKitGTK 4.1, GTK, OpenSSL, app-indicator, SVG, and build-tool packages. Use the linked Tauri prerequisite guide for Fedora, Arch, and other supported distributions.

### macOS 12+

Install Xcode Command Line Tools (`xcode-select --install`) or Xcode. Signing and notarization are not configured here; unsigned local builds may prompt for approval.

### Windows 10/11

Install Rust, Microsoft C++ Build Tools with **Desktop development with C++**, and the Microsoft Edge WebView2 Runtime. The configured MSI uses the WebView2 bootstrapper if the runtime is missing. MSI packaging can also require the Windows VBScript optional feature.

## Run in development

From this directory:

```bash
npm install
npm run check:config
npm run test:bridge
npm run dev
```

On Linux, `npm run dev` also needs a graphical session and the native packages listed above. The application window itself is remote; `frontend/index.html` exists as the minimal local asset root required by Tauri's packaging pipeline and is not the WhatsApp UI.

## Build installers

Build on each target operating system for the most reliable packages:

```bash
npm run build
```

Configured bundle targets are MSI (Windows), DMG (macOS), DEB, and RPM (Linux). Explicit examples:

```bash
# Windows (run on Windows)
npm run tauri build -- --bundles msi

# macOS (run on macOS)
npm run tauri build -- --bundles dmg

# Linux (run on Linux; one or both formats)
npm run tauri build -- --bundles deb,rpm
```

Cross-compiling GUI bundles between operating systems is not promised. Code signing, macOS notarization, Linux AppImage, and a CI release pipeline are not configured. Windows WebView2, macOS WebKit, and Linux WebKitGTK use different native runtimes.

## Use

1. Start the app. On first use, WhatsApp Web presents its normal QR sign-in flow.
2. The session is held in the WebView's persistent app profile. It is local to this OS user and app identity; clearing the app's WebView data or reinstalling with a changed identifier may require sign-in again.
3. Close the window to keep the process and session alive in the tray. Use the tray menu's **Quit** to exit fully.
4. Use **Launch at login** and **Desktop notifications** in the tray menu to opt in/out.

WhatsApp is a third-party service. Account access and feature availability remain subject to WhatsApp's own terms and current web-client behavior.

## Privacy and security notes

- No app server, analytics, telemetry, message database, or auto-update mechanism is included.
- WhatsApp messages and session material remain in the native WebView's own profile. App preferences contain only geometry and the notification enabled flag.
- `csp` is deliberately unset: WhatsApp Web depends on a changing set of remote scripts, media, and styles. The main-frame navigation callback restricts top-level navigation to the exact WhatsApp Web origin, and the remote Tauri capability is restricted to that same origin. This is not a security guarantee against compromise of WhatsApp Web or the system WebView.
- Notifications can expose sender/message text on the OS notification surface. Disable **Desktop notifications** in the tray if that is not desired. The plugin may request the OS notification permission when WhatsApp first asks to show one.

## Validation and known limitations

- Verified here: `cargo check` passed on Ubuntu 24.04 with Rust 1.99; `cargo fmt --check`, the static Tauri configuration checks, JavaScript syntax check, and notification bridge mock tests passed. `npx tauri build --bundles deb` produced `WhatsApp Lite_0.1.0_amd64.deb` (2,614,346 bytes, approximately 2.49 MiB). The app was not launched because this sandbox has no desktop display, so live sign-in, tray behavior, autostart, and OS notifications were not runtime-tested.
- Windows/MSI and macOS/DMG builds were not attempted. The Linux package build is not evidence of those target builds or runtime behavior.
- The notification bridge adapts main-frame `Notification` constructor calls. WhatsApp Web can change notification implementation, and notifications emitted directly from a service worker may bypass this page-level adapter or vary by WebView/OS. Validate foreground/background behavior and notification click actions on each target OS before release.
- Native notification presentation and permission prompts are OS-controlled. Tauri documents Windows notification delivery as supported for **installed apps**; development-mode notification appearance can differ. Linux desktops need a working notification daemon/AppIndicator implementation.
- OS-theme synchronization means using the WebView's system theme preference; the app does not force WhatsApp's in-product light/dark preference.
- The PRD's RAM and startup targets, and installer size on Windows/macOS, are **not measured or claimed**. The one Linux `.deb` package output was measured above; it does not imply comparable sizes or app memory/startup performance on other platforms. The release profile requests size-oriented optimizations, but the system WebView and WhatsApp Web payload must be benchmarked on real devices.
- Multiple monitor removal can leave a previously saved window position off-screen; move the window back with the OS window manager or reset `preferences.json` in the app config directory.

## Project layout

```text
frontend/index.html                    Minimal build asset (not the app UI)
src-tauri/Cargo.toml                    Rust dependencies and release profile
src-tauri/tauri.conf.json               Tauri 2 app and bundle config
src-tauri/capabilities/whatsapp-remote.json
src-tauri/src/lib.rs                    Tray, shortcut, autostart, prefs, window lifecycle
src-tauri/src/notification_bridge.js    Remote Web API -> native notification adapter
src-tauri/src/main.rs                  Desktop entrypoint
src-tauri/icons/                        App and tray icon files
scripts/check-config.mjs                Static project invariants
scripts/test-notification-bridge.mjs     Mock tests for notification forwarding
scripts/make-icons.py                    PNG/ICO/ICNS generator (requires Pillow)
```
