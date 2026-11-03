# CRAFTIX 0.7.8 — Minecraft Java Launcher for Android

Real signed Release APK built from `arena/92db30ff-horizon`. Two artifacts are
published in this release: `CRAFTIX-Rust-0.7.8-arm64-v8a.apk` (Rust engine only)
and `CRAFTIX-Flutter-Rust-0.7.8-arm64-v8a.apk` (Flutter UI + the same Rust
engine, `libflutter.so`, Dart AOT snapshot and `engine-data.zip`).

## What changed
- Product identity is **CRAFTIX** everywhere user-facing; subtitle
  `Minecraft Java • Android`; tagline `BUILD • PLAY • EXPLORE`; exact palette
  from the supplied reference; real adaptive launcher icon in 5 densities and
  branded splash.
- `applicationId` / namespace are now `com.craftix.launcher` (versionCode 30).
- Home has ONE stable action slot driven by the real engine installer:
  `Download` when `official_install::ready()` is false, a real-progress panel
  while `install_with_progress` runs, then `Play`. No route change, no resize.
- Animated voxel runner tracks real bytes/progress; milestone pickups appear at
  real 25/50/75/100% thresholds; reduced-motion falls back to a static bar.
  Unknown total size is indeterminate — no invented percentage, speed or ETA.
- The Flutter host now stages `assets/engine-data.zip` into `filesDir`, which
  the installer requires; before this, Download could never succeed.
- Release signing uses ONLY the owner's permanent CRAFTIX keystore from the
  four GitHub Actions secrets. No key, Base64, password or private signing
  file is stored in the repository or printed in logs. Debug builds still work
  without release secrets; a release build fails clearly if they are missing.

## Verification performed in CI
`flutter analyze`, `flutter test` (unit/widget/golden generation), real Rust
unit tests (`rustc --test bridge_rules.rs`), `cargo check` + `clippy -D warnings`
for `aarch64-linux-android`, live Modrinth/Horizon API checks, JNI symbol checks
on the packaged `.so`, APK archive/binary-manifest/signature/ABI verification,
SHA256SUMS, and the public SHA-256 certificate fingerprint.

## Migration caveat (important)
Changing `applicationId` from `dev.horizon.android` to `com.craftix.launcher`
AND changing the signing key means Android treats CRAFTIX as a **different app**
from any previously installed HORIZON build. It installs alongside rather than
updating it. There is **no automatic cross-package user-data migration** on
Android; export `android-launcher.json`, `accounts.json` and content folders
manually before uninstalling the old build. No seamless in-place upgrade is
claimed.

## NOT tested on a physical device / AVD
Installing and launching this APK, Minecraft/Vulkan/wgpu runtime, Microsoft
OAuth with real credentials, SAF import, Modrinth downloads on-device, FPS/GPU,
memory leaks and physical rotation. CI evidence is source, compile, unit,
widget, API, archive and signature evidence — not a device guarantee.
