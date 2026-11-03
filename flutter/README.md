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

- **Completed and CI-gated (run 41571959078, all steps green):** phases 3
  (Flutter architecture) and 4 (design system), plus all-page layouts,
  navigation, and the typed bridge contract. `flutter analyze` = No issues;
  23/23 Flutter tests pass, including no-overflow checks on 360x720, 360x800,
  393x873, 412x915, 480x960, 800x1280 and 960x540.
- **Not yet bridged:** engine-domain actions (Modrinth/Horizon search, install,
  account sign-in, enable/disable, diagnostics, game play). The Kotlin
  dispatcher returns an explicit, visible failure for those — never fake success.
  That is phase 9.
- **Not yet built:** the combined Flutter + `libhorizon_android.so` Release APK
  and on-device validation. That is phase 13. This tree has no APK.
- **Device checks** (Minecraft launch, Microsoft sign-in, SAF import, memory,
  FPS) have **not** been run: no AVD/device is available in this environment.

## Verify locally (no Flutter SDK needed)

```sh
python3 tools/verify_flutter_migration_audit.py
python3 tools/verify_flutter_foundation.py
```

With Flutter installed:

```sh
cd flutter && flutter pub get && flutter analyze --no-fatal-infos && flutter test
```
