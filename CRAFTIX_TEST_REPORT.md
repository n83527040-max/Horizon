# CRAFTIX 0.7.8 — تقرير التنفيذ والاختبارات

> مصدر التنفيذ: `CRAFTIX_Implementation_Package.zip` (من فرع `main`)،
> SHA-256 `c523b236239d42195c3bc0ddfa49a0a8522e6e9f239c364807025e6c6309fc69`.
> قُرئ كاملاً: المواصفة (11 قسماً)، رسالة التسليم العربية، وصورتا المرجع.
> **هذا تقرير اختبارات، وليس إدعاءً بتشغيل على جهاز فعلي.**

## 1. ترتيب التنفيذ (القسم 9 من المواصفة)

| # | المرحلة | الحالة |
|---|---|---|
| 1 | فحص المستودع + تقرير أساس | ✅ `CRAFTIX_MIGRATION_ANALYSIS.md` §1 |
| 2 | تحليل `applicationId` + التدقيق/التوقيع | ✅ `CRAFTIX_MIGRATION_ANALYSIS.md` §2–4 |
| 3 | العلامة/الlabel/الأيقونة/splash/about | ✅ منفّذ ومُختبَر بـ 45 فحصاً |
| 4 | Shell ثابت + insets + تنقّل + استجابة | ✅ `resizeToAvoidBottomInset:false`، مؤشر متحرك، احترام تقليل الحركة، 7 أحجام |
| 5 | Home: Download → تقدّم حقيقي → Play | ✅ `nativeInstallStart` + `official_install::install_with_progress` |
| 6 | شريط العدّاء المتحرك + reduced motion | ✅ `craftix_runner_progress.dart` |
| 7 | اختبارات/إصلاحات الحسابات | ✅ اختبارات 0.7.7 قائمة + اختبارات جديدة (Microsoft/Offline منفصلان) |
| 8 | تدقيق الكتالوج/التثبيت | ✅ اختبارات 0.7.7 قائمة + اختبار API حيّ + مصفوفة 49 |
| 9 | توقيع GitHub Actions بأسرار CRAFTIX | ✅ منفّذ؛ خطوة فكّ المفتاح نجحت في CI |
| 10 | بناء + اختبارات + تحقق + Release | ✅ مكتمل (انظر §5) |

## 2. ما نُفِّذ فعلياً (ملفات)

| الغرض | الملفات |
|---|---|
| الهوية والألوان | `flutter/lib/core/theme/horizon_strings.dart`, `horizon_theme.dart`, `app.dart`, `pubspec.yaml`, `flutter/assets/host-info.json` |
| الأيقونة/splash | `tools/generate_craftix_icons.py` + 30 أصلاً `mipmap-*` و`mipmap-anydpi-v26` في شجرتَي res، `launch_background.xml` |
| معرّف الحزمة والإصدار | `packaging/android/build.gradle` (`com.craftix.launcher`, 0.7.8/30)، `tools/prepare_mixed_flutter.py`، المانيفستات |
| آلة حالة التنزيل | `crates/horizon-client/src/android/flutter_bridge.rs` (`nativeInstallStart` + حقول اللقطة)، `HorizonEngine.kt`, `HorizonBridge.kt` (`ensureEngineData`), `launcher_bridge.dart`, `models.dart`, `home_page.dart` |
| القواعد النقية | `crates/horizon-client/src/android/bridge_rules.rs` (`download_percent`, `transfer_speed`, `eta_seconds`, `milestone`) |
| العدّاء/الحركة | `flutter/lib/core/theme/craftix_runner_progress.dart`, `horizon_motion.dart`, `navigation/app_shell.dart` |
| التوقيع الدائم | `tools/ci_craftix_signing.sh`, `tools/ci_record_certificate.py`, `packaging/android/build.gradle`, `tools/android_build.py` (حارس إضافي فقط)، الـ workflows، `.gitignore` |
| الاختبارات/الفواحص | `flutter/test/craftix_download_state_test.dart`, `craftix_branding_test.dart`, `tools/verify_craftix_release.py`, تحديث `check_jni_symbols.py`, `verify_mixed_apk.py`, `verify_flutter_migration_audit.py`, `tests/test_android_ui_redesign.py` |
| التوثيق/الإصدار | `CRAFTIX_MIGRATION_ANALYSIS.md`, `tools/ci-craftix-078-notes.md`, `README.md`, `ACTION_TEST_MATRIX.md` |

## 3. الاختبارات ونتائجها

| الاختبار | النتيجة |
|---|---|
| `python3 tools/verify_craftix_release.py` | **45/45** |
| `python3 tools/verify_flutter_foundation.py` | **18/18** |
| `python3 tools/verify_flutter_native_bridge.py` | **28/28** |
| `python3 tools/verify_flutter_migration_audit.py --online` | **38/38** (قبل نشر 0.7.8) |
| `python3 tools/verify_action_matrix.py` | **10/10** |
| `python3 -m unittest discover -s tests -p 'test_android*.py'` | **35/35 OK** |
| `python3 tools/check_public_tree.py` | passed |
| `python3 tools/android_ui_layout.py` | OK لكل الأحجام/الـ cutouts |
| `python3 tools/android_ui_render.py --check` | Portrait geometry OK لـ 9 صفحات × 3 شاشات |
| `git diff --check` | OK |
| `rustc --test bridge_rules.rs` | **محلياً غير متاح**؛ في CI: نجح (ضمن `41699947997` و`41699947955`) |
| `flutter analyze --no-fatal-infos` | **No issues found** (run `41699947997`) |
| `flutter test --reporter expanded` | **57 اختباراً ناجحاً + 1 skipped** (run `41699947997`) |
| `flutter test --dart-define=GENERATE_GOLDENS=true --update-goldens` | baseline مولَّد (1 test) — **ليس مقارنة بكسلية** |
| `cargo check -p horizon-android --target aarch64-linux-android` | **نجح** (run `41699947929`) |
| `cargo clippy -D warnings` | **نجح** (run `41699947929`) |
| `tools/test_live_catalog_api.py` | Modrinth/فهرس Horizon حيّان (ضمن بوابة CI) |
| `tools/check_jni_symbols.py` على `.so` | يُشغَّل في CI على المكتبة المُعبَّأة |
| APK: archive/binary manifest/signature/ABI | 🔄 قيد التنفيذ في الـ workflows |
| جهاز/AVD فعلي | **لم يُختبر** — صراحًة |

## 4. ما يعمل فعلياً مقابل ما لم يُثبت

**مثبت مصدرياً/وحدوياً/ثنائياً (لا يعني ضغطة زر على هاتف):**
- الـ 49 إجراءً لها منفّذ JNI/Kotlin/Rust حقيقي؛ لا `enginePending` ولا نجاح وهمي.
- Microsoft Device Code / Browser وOffline: تدفّقات `account::SignIn` و`Accounts`
  الأصلية + اختبارات Widget منفصلة لكل مسار (`accounts_flow_test.dart`).
- الكتالوج: `Catalog::search` الحقيقي مع تصنيف أخطاء network/api/parse/comaptibility
  وزر إعادة محاولة؛ استجابات Modrinth الحقيقية مُختبرة بسكربت API.
- التنزيل/التثبيت: `official_install::install_with_progress` الحقيقي، تقدّم/مرحلة/سرعة/ETA
  مشتقة من عيّنات فعلية، ولا نسبة مئوية عند حجم غير معروف.
- `Play` يُسلَّم إلى `Launcher::action` في عملية `:engine` المنفصلة.
- SAF: `ContentResolver` → `nativeImportStaged` → `content_files::validate`.
- التوقيع: فكّ Base64 من أسرار CRAFTIX الأربعة إلى `RUNNER_TEMP` فقط، وحذفه في `always()`.

**غير مُختبر على جهاز/AVD (صريح، لا يُدعى نجاحه):**
تثبيت/تشغيل APK، Minecraft/Vulkan/wgpu/Xbox Live/XSTS/Minecraft Services بمستخدم
حقيقي، SAF على جهاز، تنزيل Modrinth على الجهاز، FPS/GPU/الذاكرة، تسريب الذاكرة،
الدوران الفيزيائي، إلغاء التنزيل (لا يملك `install_with_progress` خطاف إلغاء،
لذلك لا يُعرض زر Cancel)، ونقل بيانات المستخدم بين معرّفي الحزمة.

## 5. الإصدار [`craftix-0.7.8`](https://github.com/n83527040-max/Horizon/releases/tag/craftix-0.7.8)

| الأصل | الحجم | SHA-256 |
|---|---|---|
| `CRAFTIX-Flutter-Rust-0.7.8-arm64-v8a.apk` | 87,035,648 bytes | `d4087676e2518e70e2d472fe6aa546982a46e3973e75a5f469d8e218f3d46bff` |
| `CRAFTIX-Rust-0.7.8-arm64-v8a.apk` | 36,271,536 bytes | `febfc11c5bacc4caf80a114ea85a270d31af0e747b00e5b5b7c4247568069670` |

- `SHA256SUMS_RUST.txt` و`SHA256SUMS_FLUTTER.txt` منشوران، وتم التحقق من تطابق
  التنزيل مع الملف المبني داخل CI (`cmp`).
- **بصمة الشهادة العامة (SHA-256):**
  `77:05:07:A6:EC:48:51:98:66:55:08:29:32:CA:C2:81:61:55:08:0F:D4:73:DB:7E:42:E9:A0:BF:FD:22:DD:2F`
  — **مطابقة تماماً** لمرجع المالك في الحزمة (`CERT_FINGERPRINT.json`
  و`MIXED_CERT_FINGERPRINT.json` كلاهما `matchesOwnerReference: true`).
  هذا دليل على استخدام نفس مفتاح CRAFTIX Release وليس مفتاحاً جديداً.
- `MIXED_APK_VERIFICATION.json`: `passed=true`, `errors=[]`،
  `com.craftix.launcher` / `versionCode=30` / `versionName=0.7.8`،
  Portrait ثنائي، `:engine` منفصل، `debuggable=false`،
  توقيع v2 بمُوقّع واحد، وكل من `libhorizon_android.so` و`libflutter.so`
  و`libapp.so` و`engine-data.zip` وأصول Flutter موجودة.
- `APK_VERIFICATION.json` و`release-audit.json` و`badging.txt`
  و`manifest-tree.txt` و`certificate.txt` منشورة للأثر.
- بوابات الإصدار الخضراء على HEAD `ea7703b5`:
  `41707815284` (foundation)، `41707815054` (Rust APK)،
  `41707815327` (mixed APK).

### ملاحظات أمان/بناء صريحة
- لا ملف `.jks`/`.keystore`/Base64/كلمات مرور داخل المستودع؛ `git ls-files`
  يثبت ذلك (45/45 في `verify_craftix_release.py`). المفتاح فُكّ إلى
  `$RUNNER_TEMP/craftix-release.jks` ثم حُذف في خطوة `if: always()`.
- جداول محرك Minecraft جاءت من مسار `PREPARE=fallback` (جداول الإصدار
  الرسمي v0.7.6 المولّدة) لأن التحقق الكامل من `26.3.json` أعاد sha1 مختلفاً؛
  هذا موثّق ولا يُخفى، والـ APK يحتوي `engine-data.zip` الحقيقي.
- لا يُدعى أي اختبار على جهاز: راجع §4.
