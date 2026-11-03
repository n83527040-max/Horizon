# Horizon Flutter host (phases 3–4)

This is the real Flutter/Dart front end for Horizon Android 0.7.6. It is **not**
a mockup: every screen is driven by `LauncherBridge`, whose production
implementation is a `MethodChannel` to `dev.horizon.android/bridge` in
`android/app/src/main/kotlin/dev/horizon/android/HorizonBridge.kt`. Values that
the native host does not expose are hidden rather than replaced with demo data.

## Layout

```
lib/
  main.dart                     portrait launcher entry point
  app.dart                      Material 3 + centralized theme + locale
  core/
    theme/                      palette, typography, radii, responsive metrics, en/ar strings
    bridge/                     typed 49-action contract, snapshot models, MethodChannel client
    state/                      Riverpod providers, action feedback, polling
    widgets/                    reusable cards, gradient button, chips, metrics, bottom nav
  features/                     home / catalog / installed / settings / dialogs
  navigation/app_shell.dart     IndexedStack (state preservation), Android Back, full-width nav
android/app/src/main/           Flutter Activity + native bridge + game NativeActivity
assets/                         hero.png (same embedded art), host-info.json (generated)
test/                           theme, action parity, bridge, widgets, overflow on 7 sizes
```

## Real-data contract

`HorizonBridge.kt` reads the Rust launcher's own `android-launcher.json`, the
three real content folders, `/proc/self/status`, `/proc/meminfo` and
`flutter_assets/host-info.json` (generated from `research-lock.json` and
`packaging/android/build.gradle` by `verify_flutter_foundation.py --generate`).

`tools/verify_flutter_foundation.py` also checks that every one of the 49 Rust
`Action` variants is handled by the Kotlin dispatcher and mirrored in Dart,
that Kotlin's snapshot keys are parsed by Dart, and that no demo performance
numbers exist in production Dart.

## Honest status

- **Phases 3–4, CI-gated (run 41644462677):** `flutter analyze` = No issues;
  28/28 Flutter tests pass, including no-overflow checks on 360x720, 360x800,
  393x873, 412x915, 480x960, 800x1280 and 960x540.
- **Phase 9 implemented (run 41644462677 + 41644462684):** real JNI/C ABI bridge
  `crates/horizon-client/src/android/flutter_bridge.rs` + `HorizonEngine.kt`
  covers all 49 actions through the original `Catalog`, `Config`, `Accounts`,
  `SignIn`, `diagnostics`, `performance` and `content_files::validate`. `Play`
  hands off to the untouched `Launcher::action` in a separate `:engine` process.
  No `enginePending`, no fake success. See `FLUTTER_NATIVE_BRIDGE_ANALYSIS.md`
  and `FLUTTER_BRIDGE_EVIDENCE.md`.
- **Phase 13 built in CI (run 41653704923):** signed mixed APK at
  https://github.com/n83527040-max/Horizon/releases/tag/flutter-mixed-0.7.6
  (85,947,148 bytes) with `libhorizon_android.so`, `libflutter.so`, `libapp.so`,
  `engine-data.zip`, Flutter assets, portrait binary manifest, `:engine`
  process and v2 signature. `MIXED_APK_VERIFICATION.json` reports `passed=true`.
- **NOT device-tested:** installing/launching that APK, Minecraft/Vulkan,
  Microsoft sign-in, SAF import, Modrinth downloads, FPS/GPU, memory leaks and
  physical rotation. CI checked source, compile, contracts, layout, archive,
  manifest and signature — not runtime.

## Verify locally (no Flutter SDK needed)

```sh
python3 tools/verify_flutter_migration_audit.py
python3 tools/verify_flutter_foundation.py
```

With Flutter installed:

```sh
cd flutter && flutter pub get && flutter analyze --no-fatal-infos && flutter test
```
