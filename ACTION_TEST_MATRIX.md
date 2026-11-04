# مصفوفة اختبارات الإجراءات الـ49 — CRAFTIX 0.7.8

المصدر: `tools/verify_action_matrix.py`. **العمود `device` دائماً `NOT TESTED`**؛
نجاح CI يعني فحصاً مصدرياً/وحدوياً/Widget/API/ثنائياً، لا ضغطة زر على هاتف.

| # | Action | عقد JNI | اختبارات Rust | اختبار Dart | LIVE API | ثنائي APK | جهاز |
|---|---|---|---|---|---|---|---|
| 1 | `Page` | PASS | n/a | pages_overflow_test.dart (tab swap) | n/a | PASS | **NOT TESTED** |
| 2 | `Play` | PASS | n/a | native_bridge_test.dart (JNI parity) | n/a | PASS | **NOT TESTED** |
| 3 | `Search` | PASS | PASS | catalog_error_test.dart (retry) | PASS | PASS | **NOT TESTED** |
| 4 | `ShaderSource` | PASS | PASS | catalog_error_test.dart (source tab) | PASS | PASS | **NOT TESTED** |
| 5 | `ModSource` | PASS | PASS | catalog_error_test.dart (source tab) | PASS | PASS | **NOT TESTED** |
| 6 | `Field` | PASS | PASS | accounts_flow_test.dart + catalog_error_test.dart | n/a | PASS | **NOT TESTED** |
| 7 | `FinishNameEdit` | PASS | PASS | accounts_flow_test.dart | n/a | PASS | **NOT TESTED** |
| 8 | `Install` | PASS | PASS | catalog_error_test.dart (install card) | PASS | PASS | **NOT TESTED** |
| 9 | `Import` | PASS | n/a | native_bridge_test.dart (JNI parity) | n/a | PASS | **NOT TESTED** |
| 10 | `OpenFolder` | PASS | n/a | native_bridge_test.dart (JNI parity) | n/a | PASS | **NOT TESTED** |
| 11 | `ToggleSection` | PASS | n/a | pages_overflow_test.dart | n/a | PASS | **NOT TESTED** |
| 12 | `ToggleLocal` | PASS | n/a | widgets_test.dart (installed) | n/a | PASS | **NOT TESTED** |
| 13 | `DeleteLocal` | PASS | n/a | widgets_test.dart (installed) | n/a | PASS | **NOT TESTED** |
| 14 | `ConfirmDeleteLocal` | PASS | n/a | native_bridge_test.dart (JNI parity) | n/a | PASS | **NOT TESTED** |
| 15 | `Next` | PASS | PASS | pages_overflow_test.dart (pagination) | PASS | PASS | **NOT TESTED** |
| 16 | `Previous` | PASS | PASS | pages_overflow_test.dart (pagination) | PASS | PASS | **NOT TESTED** |
| 17 | `Profile` | PASS | n/a | pages_overflow_test.dart (chooser) | n/a | PASS | **NOT TESTED** |
| 18 | `Logs` | PASS | n/a | native_bridge_test.dart (JNI parity) | n/a | PASS | **NOT TESTED** |
| 19 | `RecordPerformance` | PASS | n/a | native_bridge_test.dart (JNI parity) | n/a | PASS | **NOT TESTED** |
| 20 | `ExportPerformance` | PASS | n/a | native_bridge_test.dart (JNI parity) | n/a | PASS | **NOT TESTED** |
| 21 | `ExportDiagnostics` | PASS | n/a | native_bridge_test.dart (JNI parity) | n/a | PASS | **NOT TESTED** |
| 22 | `Account` | PASS | PASS | accounts_flow_test.dart (full sheet) | n/a | PASS | **NOT TESTED** |
| 23 | `SignIn` | PASS | PASS | accounts_flow_test.dart (error stage) | n/a | PASS | **NOT TESTED** |
| 24 | `DeviceSignIn` | PASS | PASS | accounts_flow_test.dart (device code) | n/a | PASS | **NOT TESTED** |
| 25 | `CopyLoginCode` | PASS | n/a | accounts_flow_test.dart (code card) | n/a | PASS | **NOT TESTED** |
| 26 | `CancelLogin` | PASS | n/a | accounts_flow_test.dart (retry/cancel) | n/a | PASS | **NOT TESTED** |
| 27 | `ReviewApp` | PASS | n/a | native_bridge_test.dart (JNI parity) | n/a | PASS | **NOT TESTED** |
| 28 | `Repository` | PASS | n/a | native_bridge_test.dart (JNI parity) | n/a | PASS | **NOT TESTED** |
| 29 | `SignOut` | PASS | n/a | accounts_flow_test.dart (sheet) | n/a | PASS | **NOT TESTED** |
| 30 | `LoginLink` | PASS | n/a | accounts_flow_test.dart (code card) | n/a | PASS | **NOT TESTED** |
| 31 | `CloseLogin` | PASS | n/a | accounts_flow_test.dart (sheet) | n/a | PASS | **NOT TESTED** |
| 32 | `Renderer` | PASS | n/a | pages_overflow_test.dart (chooser) | n/a | PASS | **NOT TESTED** |
| 33 | `RenderScale` | PASS | n/a | pages_overflow_test.dart (chooser) | n/a | PASS | **NOT TESTED** |
| 34 | `ChooseRenderScale` | PASS | n/a | pages_overflow_test.dart (chooser) | n/a | PASS | **NOT TESTED** |
| 35 | `Orientation` | PASS | n/a | pages_overflow_test.dart (chooser) | n/a | PASS | **NOT TESTED** |
| 36 | `InputHost` | PASS | n/a | pages_overflow_test.dart (chooser) | n/a | PASS | **NOT TESTED** |
| 37 | `ChooseInputHost` | PASS | n/a | pages_overflow_test.dart (chooser) | n/a | PASS | **NOT TESTED** |
| 38 | `ChooseOrientation` | PASS | n/a | pages_overflow_test.dart (chooser) | n/a | PASS | **NOT TESTED** |
| 39 | `ChooseRenderer` | PASS | n/a | pages_overflow_test.dart (chooser) | n/a | PASS | **NOT TESTED** |
| 40 | `ChooseProfile` | PASS | n/a | pages_overflow_test.dart (chooser) | n/a | PASS | **NOT TESTED** |
| 41 | `Dismiss` | PASS | n/a | pages_overflow_test.dart | n/a | PASS | **NOT TESTED** |
| 42 | `Immersive` | PASS | n/a | widgets_test.dart (settings) | n/a | PASS | **NOT TESTED** |
| 43 | `Cutout` | PASS | n/a | widgets_test.dart (settings) | n/a | PASS | **NOT TESTED** |
| 44 | `Aspect` | PASS | n/a | pages_overflow_test.dart (chooser) | n/a | PASS | **NOT TESTED** |
| 45 | `ChooseAspect` | PASS | n/a | pages_overflow_test.dart (chooser) | n/a | PASS | **NOT TESTED** |
| 46 | `DisplayHz` | PASS | n/a | pages_overflow_test.dart (chooser) | n/a | PASS | **NOT TESTED** |
| 47 | `ChooseDisplayHz` | PASS | n/a | pages_overflow_test.dart (chooser) | n/a | PASS | **NOT TESTED** |
| 48 | `PerformanceHints` | PASS | n/a | widgets_test.dart (settings) | n/a | PASS | **NOT TESTED** |
| 49 | `GpuProfile` | PASS | n/a | widgets_test.dart (settings) | n/a | PASS | **NOT TESTED** |

## ما تعنيه الأعمدة
- **عقد JNI**: `verify_flutter_native_bridge.py` (25/25) + `check_jni_symbols.py` على `.so` الحقيقية.
- **اختبارات Rust**: `rustc --test bridge_rules.rs` على الحاسب في CI (6 اختبارات فعلية).
- **اختبار Dart**: ملف اختبار حقيقي في `flutter/test` يعمل ضمن `flutter test`.
- **LIVE API**: `test_live_catalog_api.py` يطلب Modrinth وفهرس Horizon فعلياً عبر HTTPS.
- **ثنائي APK**: `verify_mixed_apk.py` + فحص التوقيع والمانيفست والمكتبات.
- **جهاز**: لم يُختبر. لا يُدعى نجاح أي إجراء على هاتف.
