# HORIZON 0.7.7 — Flutter + Rust (CI-verified build)

Version Code **29** (upgrade from 0.7.6 / 28). Same `dev.horizon.android`
package and the same development signing identity, so installing over 0.7.6
keeps `android-launcher.json`, `accounts.json`, content folders and the Android
Keystore secrets. `minSdk 28`, `arm64-v8a` only.

## What was fixed (diagnosis first: HORIZON_077_DIAGNOSIS.md)

- **Accounts:** a real offline-name input with the upstream 3–16 rule; real
  `Accounts::play_as_with(..., offline_without_owner=true)` so offline reaches
  the game without Microsoft; multi-account add/select/delete/default;
  Microsoft device-code errors surfaced with their real stage
  (device_code / xbox_live / xsts / minecraft_services / entitlement_profile /
  token_refresh) plus retry; real `account::fresh` refresh operation.
  No fake "Offline name saved" for an empty name.
- **Catalog:** `Page` now sets the real Rust catalog page and browses on open
  (root cause of every page showing "No results"); `catalog.status` is finally
  surfaced and classified (network / api / parse / compatibility / storage /
  auth) with a real Retry; real author, downloads, icon URL, Minecraft versions,
  loaders and compatibility on every card; real version/loader filters;
  compatibility gate before install; real download progress.
- **Localization / RTL:** full Arabic catalog, LTR embedding for Latin/numeric
  fragments, no truncated/mirrored strings, SnackBars float above the bottom nav.
- **Premium motion:** animated nav indicator, staggered entrances, animated
  page transition that preserves IndexedStack state, skeletons, animated
  progress, press glow, animated dialogs/sheets.
- **Engine:** unchanged. No modification to game/, wgpu, winit, Vulkan,
  `launcher/src/`, `packaging/android/java/`, config schema or Cargo manifests.

## Verification performed (see ACTION_TEST_MATRIX.md)

- `flutter analyze` = No issues; 42 Flutter unit/widget tests pass.
- 6 real Rust unit tests (`rustc --test bridge_rules.rs`).
- 24 LIVE API integration checks against Modrinth and the Horizon index.
- `cargo check` + `clippy -D warnings` for the Android Rust target.
- 25/25 JNI contract checks + binary JNI symbol check on `libhorizon_android.so`.
- APK archive, binary manifest, v2 signature, ABI and both native libraries.
- Golden pixel baselines generated in CI (no comparison claim yet).

## NOT tested on a device / AVD

Installing as an upgrade, launching Minecraft, Vulkan/wgpu rendering, Microsoft
OAuth with a real account, Xbox Live / XSTS / Minecraft services with real
tokens, SAF import, Modrinth downloads into the content folders, FPS/GPU,
memory leaks and physical rotation. CI evidence is not a device guarantee.
