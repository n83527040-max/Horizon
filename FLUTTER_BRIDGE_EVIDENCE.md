# المرحلة 9 — مصفوفة الإثبات لكل إجراء

مصدر هذه المصفوفة: `tools/verify_flutter_native_bridge.py` (فاحص عقد مصدري).
**这不是 إثبات تشغيل على جهاز**: انظر قسم «غير مُختبر» في نهاية الملف.

| # | Action | الطبقة | العملية الحقيقية |
|---|---|---|---|
| 1 | `Page` | UI | navIndexProvider (real tab state) |
| 2 | `Play` | ENGINE | Launcher::action() -> Vulkan/wgpu game launch |
| 3 | `Search` | JNI | Catalog::search -> Modrinth/Horizon HTTP + poll |
| 4 | `ShaderSource` | JNI | Config.shader_source + Config::save + browse |
| 5 | `ModSource` | JNI | Config.mod_source + Config::save + browse |
| 6 | `Field` | JNI | Config/preferences/cache real writes |
| 7 | `FinishNameEdit` | JNI | Config::save (name/server/client id) |
| 8 | `Install` | JNI | Catalog::install -> download + compatibility check |
| 9 | `Import` | SAF | ACTION_OPEN_DOCUMENT -> ContentResolver -> nativeImportStaged |
| 10 | `OpenFolder` | Kotlin | HorizonDocumentsProvider ACTION_VIEW |
| 11 | `ToggleSection` | UI | real Installed section state |
| 12 | `ToggleLocal` | JNI | Catalog::toggle_local -> options.txt / ModList |
| 13 | `DeleteLocal` | JNI | Catalog::pending_delete |
| 14 | `ConfirmDeleteLocal` | JNI | Catalog::remove_local -> real file delete |
| 15 | `Next` | JNI | Catalog::offset + browse |
| 16 | `Previous` | JNI | Catalog::offset + browse |
| 17 | `Profile` | UI | opens real value chooser |
| 18 | `Logs` | JNI | diagnostics::summary(home) |
| 19 | `RecordPerformance` | JNI | Config.record_performance + save |
| 20 | `ExportPerformance` | JNI | performance::export(home) |
| 21 | `ExportDiagnostics` | JNI | diagnostics::export(home) -> ZIP |
| 22 | `Account` | UI | opens real accounts sheet |
| 23 | `SignIn` | JNI | account::SignIn::start_with_client_id (Microsoft device code) |
| 24 | `DeviceSignIn` | JNI | account::SignIn::start_with_client_id (Microsoft device code) |
| 25 | `CopyLoginCode` | Kotlin | ClipboardManager with real user_code |
| 26 | `CancelLogin` | JNI | SignIn::cancel() |
| 27 | `ReviewApp` | Kotlin | ACTION_VIEW review URL |
| 28 | `Repository` | Kotlin | ACTION_VIEW GitHub URL |
| 29 | `SignOut` | JNI | Accounts::remove + save |
| 30 | `LoginLink` | Kotlin | ACTION_VIEW verification_uri?otc=user_code |
| 31 | `CloseLogin` | JNI | SignIn::cancel() |
| 32 | `Renderer` | UI | opens real value chooser |
| 33 | `RenderScale` | UI | opens real value chooser |
| 34 | `ChooseRenderScale` | JNI | RenderScale::parse + Config::save |
| 35 | `Orientation` | UI | opens real value chooser |
| 36 | `InputHost` | UI | opens real value chooser |
| 37 | `ChooseInputHost` | JNI | Config.input_host + save |
| 38 | `ChooseOrientation` | JNI | Config.orientation + save |
| 39 | `ChooseRenderer` | JNI | Config.renderer + save |
| 40 | `ChooseProfile` | JNI | Config.profile + save + refresh |
| 41 | `Dismiss` | UI | closes real dialog |
| 42 | `Immersive` | JNI | Config.screen.immersive + save |
| 43 | `Cutout` | JNI | Config.screen.avoid_cutout + save |
| 44 | `Aspect` | UI | opens real value chooser |
| 45 | `ChooseAspect` | JNI | Config.screen.aspect + save |
| 46 | `DisplayHz` | UI | opens real value chooser |
| 47 | `ChooseDisplayHz` | JNI | Config.device.display_hz + save |
| 48 | `PerformanceHints` | JNI | Config.device.performance_hints + save |
| 49 | `GpuProfile` | JNI | Config.device.gpu_profile + save |

## ما تم إثباته آلياً
- 25/25 فحص عقد: 49 إجراءً، لا `enginePending`،
  تطابق رموز JNI مع Kotlin، تسليم Play إلى `Launcher::action`،
  عملية `:engine` منفصلة، إيقاف ثريد Rust في `onDestroy`،
  وعدم تعديل ملفات المحرك/الحسابات/الملفات المحمية.
- run `41644462677` (أخضر): `flutter analyze` بلا أخطاء، 28/28 اختبار،
  34/34 تدقيق، 18/18 أساس، 25/25 جسر، 35/35 اختبار Android قديم.
- run `41644462684` (أخضر): `cargo check` و`clippy -D warnings`
  لهدف `aarch64-linux-android`، ثم بناء APK Rust مُوقّع.
- run `41665698252` (أخضر): `flutter analyze` نظيف، 42/42 اختبار Flutter،
  6/6 اختبارات Rust نقية، 24/24 فحص API حيّ Modrinth/Horizon،
  25/25 عقد جسر، توليد golden baselines.
- run `41667665339` (أخضر): Rust 0.7.7 — `cargo check` و`clippy -D warnings`
  ثم APK Rust مُوقّع مُنشور. run `41671215363` (أخضر): APK مختلط مُوقّع.
- الإصدار 0.7.7: https://github.com/n83527040-max/Horizon/releases/tag/horizon-0.7.7
  Version Code 29، نفس التوقيع، `SHA256SUMS_RUST.txt` و`SHA256SUMS_FLUTTER.txt`،
  `APK_VERIFICATION.json` و`MIXED_APK_VERIFICATION.json` (`passed=true`).
- `Horizon-Android-Rust-0.7.7-arm64-v8a.apk` — 35,754,228 bytes —
  SHA-256 `58c3ec9c92d144c8d65ed2d0dca9e9f7f49ec3a7148b4732ef9ee31c89004828`.
- `Horizon-Flutter-Rust-0.7.7-arm64-v8a.apk` — 86,503,412 bytes —
  SHA-256 `4e32931b324a905bface8d81908706279a8f7da9d5bc44363e2a46b754d56e64`.

## غير مُختبر على جهاز حقيقي (صريح)
- تشغيل Minecraft وVulkan/wgpu وسطح الرسم بعد تسليم Play.
- تسجيل دخول Microsoft الفعلي مقابل خدمات Microsoft.
- منتقي SAF ونسخ المجلدات/الملفات على جهاز.
- تنزيل Modrinth وتثبيت المودات/الحزم/الشيدرات فعلياً.
- FPS/GPU/الذاكرة وتسريب الذاكرة والدوران الفيزيائي.
- تثبيت وتشغيل APK المختلط فعلياً؛ التحقق أرشيفي/توقيعي فقط.
