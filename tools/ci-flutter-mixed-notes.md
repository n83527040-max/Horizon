# Horizon Android — Flutter + Rust mixed APK (CI build)

One signed `arm64-v8a` release APK containing:

- the Flutter/Dart launcher (`MainActivity`, FlutterActivity, portrait),
- `libflutter.so` and the Dart AOT snapshot/assets,
- the unchanged real Rust engine `libhorizon_android.so` (wgpu/Vulkan,
  Minecraft Java 26.3, accounts, mods, files),
- `assets/engine-data.zip` from the verified `redesign-0.7.6` release,
- the native `HorizonActivity` in a separate `:engine` process for gameplay,
- the JNI/C ABI bridge (`HorizonEngine`) for all 49 launcher actions.

Artifacts include `MIXED_APK_VERIFICATION.json` (binary manifest, library,
signature and asset checks), `badging.txt`, `manifest-tree.txt`, `signature.txt`
and `SHA256SUMS.txt`.

## Verification performed

- `flutter analyze --no-fatal-infos`, `flutter test --reporter expanded`;
- Python contract verifiers (49/49 actions, no placeholder, JNI parity);
- 35 existing Android regression tests;
- `cargo check` + `clippy -D warnings` for the Android Rust target
  (release workflow for the same commit);
- APK archive, binary manifest, v2 signature and one-signer checks.

## NOT tested on a real device / AVD

Installing and launching this APK, Minecraft runtime and Vulkan/wgpu rendering,
Microsoft sign-in, SAF import, Modrinth downloads, FPS/GPU/performance,
memory leaks and physical rotation. CI-only evidence is not a device guarantee.
