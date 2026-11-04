# CRAFTIX / HORIZON — وثيقة تسليم كاملة (Handoff)

> **لأي وكيل/مطوّر يكمل من هنا:** اقرأ هذا الملف كاملاً قبل أول تعديل.
> آخر commit وقت كتابة الوثيقة: `7db3652` على الفرع
> `arena/92db30ff-horizon`. الـ PR إلى `main` مطلوب (هذه المهمة).
> اللغة: شرح عربي + مصطلحات/أوامر إنجليزية كما هي في الكود.

---

## 0. القاعدة الذهبية (أهم شيء في المشروع كله) 🔐

**ممنوع نهائياً إنشاء مفتاح توقيع جديد أو تغيير التوقيع في أي إصدار أو تحديث قادم.**

كل إصدار (Release) يجب أن يُوقَّع بـ **نفس مفتاح CRAFTIX Release** المخزّن في
**GitHub Actions repository secrets** بهذه الأسماء بالضبط:

| Secret | الاستعمال |
|---|---|
| `CRAFTIX_KEYSTORE_BASE64` | keystore بصيغة Base64؛ يُفكّ إلى `$RUNNER_TEMP/craftix-release.jks` |
| `CRAFTIX_STORE_PASSWORD` | كلمة مرور الـ keystore (تُمرَّر لـ apksigner عبر `env:`) |
| `CRAFTIX_KEY_ALIAS` | اسم الـ alias داخل الـ keystore |
| `CRAFTIX_KEY_PASSWORD` | كلمة مرور المفتاح الخاص |

**بصمة الشهادة العامة (SHA-256) التي يجب أن تبقى مطابقة دائماً:**
```
77:05:07:A6:EC:48:51:98:66:55:08:29:32:CA:C2:81:61:55:08:0F:D4:73:DB:7E:42:E9:A0:BF:FD:22:DD:2F
```
(الـ hex بدون نقطتين: `770507a6ec4851986655082932cac2816155080fd473db7e42e9a0bffd22dd2f`)

قواعد ملزمة:
1. لا تضع `.jks` / `.keystore` / Base64 / كلمات المرور / `signing.properties`
   داخل المستودع أو في issue أو PR أو سجلات CI. `.gitignore` يغطيها.
2. لا تطبع قيم الأسرار في اللوجات. استخدم `env:CRAFTIX_*` مع apksigner،
   و`-storepass:env` مع keytool، ولا تستخدم `set -x` حولها.
3. بناء الـ **release يفشل بوضوح** إذا غاب أي secret
   (`packaging/android/build.gradle` يرمي `GradleException`؛
   `tools/android_build.py` يتحقق من المتغيرات الأربعة + وجود الملف؛
   `tools/ci_craftix_signing.sh` يتحقق من الأربعة).
   بناء الـ **debug يعمل بدون أسرار** (debug keystore محلي في `.local/`).
4. لا يوجد «fallback صامت» لتوقيع debug ولا توليد مفتاح تلقائي أبداً.
5. الملف المؤقت يُحذف في خطوة `if: always()` في كلا الـ workflows.
6. بعد كل بناء: قارن بصمة الشهادة العامة بالقيمة أعلاه
   (`tools/ci_record_certificate.py` → `CERT_FINGERPRINT.json`).
7. إن فُقد المفتاح أو اختلفت البصمة: **قف وأبلغ المالك**؛ لا تُنشئ مفتاحاً جديداً.
8. سياسة المفتاح مستقلة عن تغيير `applicationId`: كلاهما حدث مرة واحدة ولا يُعاد.

كيف تتحقق سريعاً من التوقيع في أي APK جديد:
```bash
"$JAVA_HOME/bin/java" -jar "$ANDROID_HOME/build-tools/35.0.0/lib/apksigner.jar" \
  verify --verbose --print-certs YOUR.apk
# Signer #1 certificate SHA-256 digest يجب أن يساوي 770507a6...dd2f
python3 tools/check_apk_signature_schemes.py YOUR.apk   # يجب أن يمرّ بـ v1 + v2
```

---

## 1. هوية المستودع والحقائق الأساسية

| العنصر | القيمة |
|---|---|
| المستودع | `https://github.com/n83527040-max/Horizon` (خاصّ بالمالك، public) |
| فرع العمل | `arena/92db30ff-horizon` — **ثابت، لا تبدّله ولا تنشئ غيره** |
| `main` | `68fc23c` → `a92b3c2` (أضاف الحزمة) → `f32d063` (حذفها). شجرة `main` لا تحتوي إلا `README.md` |
| merge-base | `68fc23c33960dbfbb36ce6f16add41ccfba1d7c6` |
| عدد commits على الفرع | 48 منذ الأساس |
| الأصل (upstream) | `Vasysik/horizon-android`، الوسم `v0.7.6`، commit `7301f6fdd7f470b11317276e7a11b4dc615c137a` |
| أساس الاستيراد | commit `6c790ee` (نسخة كاملة من v0.7.6 عدا `docs/media` الكبيرة) |
| لعبة الهدف | Minecraft Java **26.3** (مثبّت في `research-lock.json`) |
| معرّف التطبيق الحالي | **`com.craftix.launcher`** |
| الاسم التجاري | **CRAFTIX** (كان HORIZON حتى 0.7.7) |
| الإصدار الحالي | `0.7.8` / `versionCode 30` (pubspec: `0.7.8+30`) |
| النسخة السابقة | HORIZON 0.7.7 / versionCode 29، ثم 0.7.6 / 28 |
| الحزمة التنفيذية | `CRAFTIX_Implementation_Package.zip` كانت على `main` ثم حُذفت؛ نُسخة للقراءة في `/tmp/craftix/` داخل sandbox (غير مُضافة للفرع) |
| أدوات مفقودة محلياً | لا `flutter`/`dart`/`cargo`/`rustc`/`java`/`gradle`/`adb`/SDK/NDK/جهاز. كل اختبارات البناء في **CI فقط** |

**ملفات الوثائق الموجودة (اقرأها عند الحاجة):**
`README.md`, `REDESIGN.md`, `FLUTTER_MIGRATION_AUDIT.md`,
`FLUTTER_NATIVE_BRIDGE_ANALYSIS.md`, `FLUTTER_BRIDGE_EVIDENCE.md`,
`HORIZON_077_DIAGNOSIS.md`, `ACTION_TEST_MATRIX.md`,
`CRAFTIX_MIGRATION_ANALYSIS.md`, `CRAFTIX_TEST_REPORT.md`,
`tools/ci-craftix-078-notes.md`, `AGENTS.md`, `CLAUDE.md`.

---

## 2. الخط الزمني: شنو درنا بالترتيب (48 commit)

| المرحلة | Commits | النتيجة |
|---|---|---|
| A. استيراد + إعادة تصميم Rust UI | `6c790ee`…`5d4d6cb` | واجهة 0.7.6 كاملة، Portrait/full-width، APK موقّع `redesign-0.7.6`، 27 صورة QA |
| B. تدقيق + جرد قبل Flutter | `d0e12d7` | `FLUTTER_MIGRATION_AUDIT.md` + فاحص 34/34؛ تبيّن أن التطبيق NativeActivity واحد بلا طبقة Java/Kotlin |
| C. أساس Flutter (مرحلتان 3–4) | `5c81885`…`74e5625` | `flutter/` كامل: Material 3، Riverpod، ثيم، en/ar، 49 إجراءً مكتوباً، 7 أحجام شاشة، CI أخضر |
| D. المرحلة 9 — الجسر الأصلي | `fe81b60` | 7 رموز JNI/C ABI، 49/49 إجراءً حقيقياً، `Play` → `Launcher::action` في `:engine`، SAF، حالة/إلغاء/أخطاء |
| E. المرحلة 13 — APK مختلط | `f557b42`…`cf83052` | `flutter-mixed-apk.yml`، `libflutter.so` + `libhorizon_android.so` في APK واحد، `flutter-mixed-0.7.6` |
| F. تشخيص وإصلاح 0.7.7 | `0ec3f84`…`1c23047` | `HORIZON_077_DIAGNOSIS.md`: أخطاء الحسابات/الكتالوج/RTL/الحركة + اختبارات حقيقية، إصدار `horizon-0.7.7` (v29) |
| G. **CRAFTIX 0.7.8** | `04b30df`…`cc9d8ad` | العلامة، `applicationId`، أيقونة adaptive، Home Download/Progress/Play حقيقي، عدّاء، توقيع دائم، إصدار `craftix-0.7.8` (v30) |
| H. **إصلاح فشل التثبيت على الهاتف** | `ab5087b`…`7db3652` | تشخيص → v2-only هو السبب → إعادة توقيع v1+v2 بنفس المفتاح + بوابات ثنائية |

---

## 3. المعمارية بالتفصيل (مهم جداً قبل أي تعديل)

### 3.1 محرك Rust (لا تلمسه)
- الـ `cdylib` هو `crates/horizon-android`، لكن **جذر المكتبة هو
  `crates/horizon-client/src/android_main.rs`** (`[lib] path = ../horizon-client/src/android_main.rs`).
- `android_main(app: AndroidApp)` يملك:
  - `APP` = `thread_local! RefCell<Option<AndroidApp>>`،
  - **حلقة `winit::EventLoop` واحدة لكل عملية**،
  - `std::process::exit(0)` في النهاية،
  - سطح `wgpu`/Vulkan واحد، `Launcher` → `game::android_game()` عند Play.
- `platform::with_context()` يعتمد على `APP` → يعمل **فقط داخل عملية NativeActivity**.
- لذلك: **أي استدعاء JNI عشوائي داخل حلقة المحرك غير آمن**، وأي خروج للمحرك في نفس
  عملية Flutter يقتلها. هذا هو سبب قرار `:engine`.

**محمي (ممنوع التعديل):** `crates/horizon-client/src/game/**`،
`crates/horizon-client-render/**`, `horizon-core/**`, `horizon-world/**`,
`horizon-protocol/**`, `horizon-mods/**`, `horizon-shaders/**`, `launcher/src/**`,
`winit`, `wgpu`, Vulkan، `content_files`، `account`، مخطط `android-launcher.json`،
`packaging/android/java/**`, `Cargo.toml`/`Cargo.lock`.
الفاحص `tests/test_android_ui_redesign.py::test_engine_and_packaging_are_untouched`
و`tools/verify_flutter_native_bridge.py` يثبتان ذلك (مع استثناء موثّق واحد:
`tools/android_build.py` لحارس التوقيع فقط).

### 3.2 العمليتان
```
[عملية com.craftix.launcher]         [عملية com.craftix.launcher:engine]
 MainActivity : FlutterActivity  --Intent(horizon.launch_game=true)-->  HorizonActivity : NativeActivity
   Dart UI / Riverpod                      android_main() / winit / wgpu / Vulkan
   MethodChannel dev.horizon.android/bridge  ANativeActivity_onCreate
   HorizonEngine (JNI)                    std::process::exit(0) لا يقتل Flutter
```
- المانيفست: `android:screenOrientation="portrait"` للمشغّل،
  `screenOrientation="unspecified"` + `android:process=":engine"` للعبة.
- `<meta-data android:name="flutterEmbedding" android:value="2" />` إلزامي
  (بدونه يرفض `flutter build apk` بـ «deleted Android v1 embedding»).
- `<application android:name="${applicationName}">` يُحلّ إلى `android.app.Application`
  بواسطة Flutter Gradle plugin (مثبت في `MIXED_manifest-tree.txt`).

### 3.3 طبقة JNI/C ABI
الكائن `dev.horizon.android.HorizonEngine` في
`flutter/android/app/src/main/kotlin/dev/horizon/android/HorizonEngine.kt`
يصرّح 7 دوال `external`، و`crates/horizon-client/src/android/flutter_bridge.rs`
يصدّر الرموز المقابلة:

| الرمز | الوظيفة الحقيقية |
|---|---|
| `nativeInit(home, cache)` | `backend::initialize`، `secure_store::initialize(vm)`، `configure_*_cache_dir`، `Catalog::new`، تشغيل خيط `poll_loop` |
| `nativeSnapshot()` | لقطة JSON كاملة من ملفات/`/proc`/`Catalog`/`Accounts`/`official_install::ready` |
| `nativeDispatch(action, argsJson)` | منفّذ الـ 49 إجراءً (انظر `ACTION_TEST_MATRIX.md`) |
| `nativeCancel()` | إلغاء تسجيل الدخول/اهتمام البحث |
| `nativeShutdown()` | إيقاف خيط Rust قبل موت الـ Activity |
| `nativeImportStaged(kind, staged, name)` | `content_files::validate` + `horizon_mods::Package::read` + `horizon_shader_api::check` |
| `nativeInstallStart()` | **جديد:** `official_install::install_with_progress` الحقيقي |

**لماذا بقيت الحزمة البرمجية `dev.horizon.android` رغم تغيير `applicationId`؟**
أسماء رموز JNI مرتبطة باسم فئة Java/Kotlin لا بـ `applicationId`؛ تغييرها يكسر الربط
مع `.so`. الأسماء الداخلية ليست واجهة للمستخدم → العلامة التجارية تغيّرت فقط.
`tools/check_jni_symbols.py` يفحص الرموز داخل `.so` الحقيقية.

**ملفات الجسر:** `flutter_bridge.rs` (JNI), `bridge_rules.rs` (قواعد نقية قابلة
للاختبار بـ `rustc --test` على الحاسب، 7 اختبارات), `catalog.rs` (getters إضافية
فقط: `progress()`, `filters()`, `page()`, `set_filters()`).

### 3.4 طبقة Android/Kotlin
- `MainActivity.kt`: `FlutterActivity`، ينشئ `MethodChannel`،
  `onActivityResult` لـ SAF، **`onDestroy` → `HorizonBridge.shutdown()`**.
- `HorizonBridge.kt`:
  - `ensureEngineData()` — **إصلاح جذري**: ينسخ `assets/engine-data.zip` إلى
    `filesDir/engine-data.zip` عند الإقلاع؛ لولاه لَفشل التنزيل دائماً لأن
    `android_main` كان ينسخه فقط في `:engine`.
  - يترجم `snapshot`/`dispatch`/`cancel`/`installStart` إلى JNI على
    `single-thread executor`، ويعيد النتيجة على الـ main thread.
  - SAF: `ACTION_OPEN_DOCUMENT` → `ContentResolver` → ملف مؤقت في `cacheDir`
    بحدّ 256 MiB → `nativeImportStaged`.
  - يفتح الروابط/الحافظة، ويرسل أحداث `nativeEvent` إلى Dart (نتائج الاستيراد،
    أكواد تسجيل الدخول، أخطاء إطلاق اللعبة).
- Java (من upstream، لم تُلمس): `HorizonActivity.java` (SAF callback)،
  `HorizonDocumentsProvider.java` (SAF provider، سلطته
  `${applicationId}.documents`).

### 3.5 طبقة Flutter/Dart
```
flutter/lib/main.dart              Portrait + SystemUiOverlayStyle
flutter/lib/app.dart               MaterialApp(title:'CRAFTIX'), ar/en, ThemeMode
core/theme/horizon_theme.dart      لوحة CRAFTIX + HorizonMetrics (LayoutBuilder/MediaQuery)
core/theme/horizon_strings.dart     كتالوج en/ar كامل + native() لترجمة رسائل Rust
core/theme/horizon_motion.dart     StaggeredEntrance, Skeleton, AnimatedPageFade,
                                  PremiumPressable, ltr() لعزل الاتجاه
core/theme/craftix_runner_progress.dart  شريط تقدّم العدّاء (تفصيله §3.7)
core/bridge/actions.dart           49 HorizonAction + Page/Kind/Source/SettingKeys
core/bridge/models.dart            LauncherSnapshot/InstalledItem/CatalogProject/
                                  BridgeResult/LauncherAccount (كلها nullable عند الغياب)
core/bridge/launcher_bridge.dart   MethodChannel + events + cancel() + installStart()
core/state/providers.dart          snapshotProvider (poll 4s), navIndexProvider,
                                  nativeEventsProvider, ActionRunner
core/widgets/                     HorizonCard, GradientButton, Chips, BottomNav,
                                  EmptyState/Error/AsyncSnapshotView
features/home|mods|packs|shaders|installed|settings|dialogs
navigation/app_shell.dart          IndexedStack يحفظ الحالة + AnimatedPageFade +
                                  didPopRoute + SnackBar عائم + مؤشر تقدّم
```
لا توجد بيانات تجريبية في `lib/` (فاحص يمنع `"267 MB"`, `"60 FPS"`, …).

### 3.6 الـ 49 إجراءً وحالة التنزيل/التثبيت
- كل إجراء له منفّذ حقيقي؛ **لا `enginePending` ولا نجاح وهمي**.
  الجدول الكامل: `ACTION_TEST_MATRIX.md` + `FLUTTER_BRIDGE_EVIDENCE.md`.
- Microsoft: `account::SignIn::start_with_client_id` (Device Code Flow **بلا
  Redirect URI**؛ الـ callback هو استطلاع `/token`)، تصنيف مراحل
  `device_code` / `xbox_live` / `xsts` / `minecraft_services` /
  `entitlement_profile` / `token_refresh` عبر `bridge_rules::auth_stage`.
- Offline: `Accounts::play_as_with(..., offline_without_owner=true)` الموجود أصلاً،
  مع التحقق `valid_offline_name` (3–16 حرفاً، ASCII).
- الكتالوج: `Catalog::search` → Modrinth/فهرس Horizon الحقيقيين،
  `classify_error` يفرّق network/api/parse/compatibility/storage/auth،
  و`canRetry` + زر إعادة محاولة. فلاتر `game_version` و`loader` مُبيّضة.
- التثبيت: `Catalog::install` يتحقق التوافق ويُحمّل بـ SHA-512/SHA-1؛
  الحالات المعروضة: Compatible / Incompatible / Browse only / Installed / فشل.

### 3.7 آلة حالة الشاشة الرئيسية (أهم ميزة جديدة في 0.7.8)
```
official_install::ready(home) == false  →  زر Download (+ وصف الإصدار 26.3)
        │ nativeInstallStart() → thread يستدعي install_with_progress
        ▼
installActive == true  →  لوحة تقدّم في نفس الموضع (بلا تنقّل/تغيّر حجم)
   stage/done/total/speed/ETA/milestone من عيّنات حقيقية
   الحجم مجهول => indeterminate، ولا نسبة مئوية مُخترعة أبداً
        ▼
ready == true  →  زر Play في نفس الموضع → Intent إلى :engine → Launcher::action
```
- الحقول في اللقطة: `installReady`, `installActive`, `installStage`,
  `installDone`, `installTotal`, `installPercent`, `installSpeedBps`,
  `installEtaSeconds`, `installMilestone`, `installError`.
- القواعد الحسابية في `bridge_rules.rs`: `download_percent`,
  `transfer_speed` (عيّنتان حقيقيتان، ≥250ms), `eta_seconds`, `milestone`؛
  لها اختبارات `rustc --test`.
- `_ActionSlot` في `home_page.dart` ارتفاعه ثابت (`_slotHeight = 178`) مع
  `AnimatedSwitcher` → لا قفزة تخطيط.
- `CraftixRunnerProgress`: عدّاء voxel يتحرك بـ `AnimatedPositioned` مدفوعاً
  بالتقدّم الحقيقي، pickup عند 25/50/75/100%، احتفال عند الاكتمال،
  **ودعم `MediaQuery.disableAnimations`** (شريط ثابت). بلا `Timer.periodic` ولا حلقات متكررة.
- **حد معروف:** `install_with_progress` لا يملك خطاف إلغاء → لا يوجد زر Cancel
  للتنزيل (لم نضف زراً شكلياً). موثّق في التقرير.

### 3.8 الملفات على الجهاز (توافق قديم محفوظ)
`filesDir/android-launcher.json` (Config), `accounts.json` + `account-secrets/`
(Android Keystore), `flutter-preferences.json` (theme/language),
`engine-data.zip`, `.local/horizon-client/26.3/`, مجلدات
`horizon-mods` / `resourcepacks` / `shaderpacks`.
تغيير `applicationId` + المفتاح ⇒ **لا ترقية في المكان ولا نقل بيانات تلقائي**؛
النسخ اليدوي فقط (موضّح في `CRAFTIX_MIGRATION_ANALYSIS.md`).

---

## 4. العلامة التجارية والهجرة (0.7.8)
- `CRAFTIX` في كل نص مرئي، `Minecraft Java • Android`،
  `BUILD • PLAY • EXPLORE`، لوحة:
  `#070B1D` / `#111B38` / `#8B35F5` / `#315BFF` / `#22D3EE` /
  `#F7F8FF` / `#A8B1CC`.
- أيقونة adaptive حقيقية: `tools/generate_craftix_icons.py` من
  `tools/craftix-icon-source.png` يُنتج 5 كثافات (`mipmap-mdpi…xxxhdpi`)
  + `mipmap-anydpi-v26/ic_launcher.xml` + foreground + لون الخلفية،
  في **شجرتَي** `flutter/android/app/src/main/res` و`packaging/android/res`.
- splash: `drawable/launch_background.xml` بخلفية navy + الأيقونة.
- `applicationId`/`namespace` = `com.craftix.launcher` في
  `packaging/android/build.gradle` و`tools/prepare_mixed_flutter.py`.
- **تحذير:** لا يوجد OAuth redirect URI ولا `google-services.json` ولا App Links
  ولا Firebase في المشروع؛ Device Code Flow لا يحتاجها. تم توثيق الفحص.

---

## 5. نظام البناء والإصدار (ورشة العمل كاملة)

### 5.1 APK الخاص بـ Rust فقط
- `packaging/android/build.gradle`: AGP 8.11.1، `compileSdk 35`،
  `minSdk 28`, `targetSdk 35`, `versionCode 30`, `versionName '0.7.8'`,
  `namespace/applicationId com.craftix.launcher`,
  `signingConfigs.craftix` من متغيرات البيئة الأربعة،
  `release.signingConfig = craftix` (وإلا `GradleException`).
  `useLegacyPackaging = true` + `extractNativeLibs="true"` في مانيفست packaging.
- `tools/build_release_apk.sh` → فحوصات → `tools/android_release.py` →
  `tools/android_build.py` (`cargo build --target aarch64-linux-android`,
  Gradle 8.14 `assembleRelease`) → `.local/android/apk/arm64-v8a/horizon-android-public.apk`.
- `tools/android_build.py`: **للـ release يتطلب أسرار CRAFTIX ويمنع توليد
  debug key**؛ للـ debug يولّد `.local/android/debug.keystore`.
- `tools/ci_android_setup.sh`: يثبّت build-tools 35.0.0 + NDK 28.
- `tools/android_prepare.py`: غالباً يعطي **`PREPARE=fallback`** (جداول
  v0.7.6 الرسمية المولّدة) لأن `26.3.json` يعيد sha1 مختلفاً؛ موثّق وغير مخفي،
  و`assets/engine-data.zip` حقيقي (~9 MB).

### 5.2 APK المختلط (Flutter + Rust)
- `.github/workflows/flutter-mixed-apk.yml`:
  1. Flutter stable + JDK 17 + SDK،
  2. فكّ CRAFTIX keystore،
  3. فحوصات العقد + `rustc --test bridge_rules.rs` + 35 اختبار،
  4. `flutter pub get`, `flutter analyze --no-fatal-infos`, `flutter test`,
  5. **ينتظر** أصل `CRAFTIX-Rust-0.7.8-arm64-v8a.apk` في الإصدار (حتى 70×30s)،
     ينزّله ويستخرج `libhorizon_android.so` + `engine-data.zip`،
     ويتحقق رموز JNI بـ `tools/check_jni_symbols.py`،
  6. `tools/prepare_mixed_flutter.py`: `flutter create` لتوليد scaffold Gradle
     (إصدارات AGP/Kotlin مطابقة للأداة)، ثم يرقع `applicationId`/`namespace`/
     `versionCode`/`versionName`، ويضيف `src/main/assets` و`src/main/jniLibs`،
     وينسخ ملفات Java الأصلية، ولا ي overwrite المانيفست/Kotlin/الموارد،
  7. `flutter build apk --release --target-platform android-arm64`,
  8. إعادة التوقيع §5.4، التحقق §5.5، النشر.

### 5.3 سلسلة التوقيع (لا تُغيّر)
```
secrets.CRAFTIX_KEYSTORE_BASE64 ──base64 -d──> $RUNNER_TEMP/craftix-release.jks
        │  (خارج المستودع، umask 077، يُحذف في always())
        ├── tools/ci_craftix_signing.sh : keytool -storepass:env → بصمة عامة فقط
        ├── packaging/android/build.gradle : signingConfigs.craftix (Gradle)
        ├── tools/prepare_mixed_flutter.py : signingConfigs.create("craftix")
        └── tools/resign_craftix_apk.sh : apksigner sign
                --ks-pass env:CRAFTIX_STORE_PASSWORD
                --key-pass env:CRAFTIX_KEY_PASSWORD
                --v1-signing-enabled true --v2-signing-enabled true
                --min-sdk-version 21
```
`tools/ci_record_certificate.py` يقرأ `apksigner verify --print-certs`،
يقبل الـ 64-hex أو الشكل بنقطتين، ويكتب `CERT_FINGERPRINT.json` مع
`matchesOwnerReference`.

### 5.4 لماذا نُعيد التوقيع (قصة خطأ الجهاز)
**المرض:** لقطة من هاتف حقيقي: «لم يتم تثبيت التطبيق لأن الحزمة تبدو غير صالحة»
مع ظهور أيقونة CRAFTIX ⇒ المانيفست قُرئ، الفشل في التحقق من التوقيع.
**التشخيص:** `tools/diagnose_apk_install.py` على الـ APK المنشور
(run `41718389434`) أثبت: ZIP/مانيفست/مكتبات محاذاة 4K سليمة، لكن
`v1Present=false` و`v2 only`، و`apksigner verify --min-sdk-version 21`
يفشل بـ `Missing META-INF/MANIFEST.MF`. AGP يعطّل JAR v1 عند `minSdk>=24`،
ومثبّتات بعض الشركات ترفض v2-only.
**الدواء:** إعادة التوقيع بـ **نفس المفتاح** مع v1+v2 (`apksigner sign` يستبدل
التوقيعات السابقة افتراضياً). لا مفتاح جديد، لا تغيير بصمة.
**ال booster:** `tools/check_apk_signature_schemes.py` فحص ثنائي يلزم
`META-INF/*.RSA` + `APK Sig Block 42`؛ `verify_mixed_apk.py` يفشل إن غاب أحدهما؛
`diagnose_apk_install.py` بوابة في كلا الـ workflows.
**النتيجة:** كلا APK الآن `v1Present=true`, `v2SigningBlockPresent=true`,
`apksigner verify` ينجح عند 21/24/28، `passed=true, problems=[]`
(run `41724975391` + تشخيص مستقل `41726421227`).

### 5.5 بوابات التحقق (كلها لازم تبقى خضراء)
| الأداة | النتيجة المتوقعة |
|---|---|
| `python3 tools/verify_craftix_release.py` | **48/48** |
| `python3 tools/verify_flutter_foundation.py` | **18/18** |
| `python3 tools/verify_flutter_native_bridge.py` | **28/28** |
| `python3 tools/verify_flutter_migration_audit.py --online` | **40/40** |
| `python3 tools/verify_action_matrix.py` | **10/10** |
| `python3 -m unittest discover -s tests -p 'test_android*.py'` | **35/35 OK** |
| `python3 tools/check_public_tree.py` (+`--history`) | passed |
| `python3 tools/android_ui_layout.py` / `android_ui_render.py --check` | OK |
| `rustc --test crates/horizon-client/src/android/bridge_rules.rs` | **7 tests** |
| `flutter analyze --no-fatal-infos` | **No issues found** |
| `flutter test --reporter expanded` | **57 +1 skipped** |
| golden generation (`--update-goldens`) | baseline مولَّد (ليس مقارنة بكسلية) |
| `cargo check` + `clippy -D warnings` (aarch64-linux-android) | نجحا |
| `tools/test_live_catalog_api.py` | Modrinth/فهرس Horizon حيّان |
| `tools/check_jni_symbols.py` على `.so` | كل الرموز موجودة |
| `tools/verify_mixed_apk.py` + `diagnose_apk_install.py` | `passed=true` |

### 5.6 الـ workflows و محفّزاتها
| Workflow | الدور |
|---|---|
| `flutter-foundation.yml` | بوابة سريعة: عقود + Python + Rust rules + analyze + test + goldens |
| `android-redesign-release.yml` | يبني APK الـ Rust-only ويوقّعه وينشره على `craftix-0.7.8` |
| `flutter-mixed-apk.yml` | يبني APK المختلط معتمداً على أصل الـ Rust المنشور |
| `apk-install-diagnose.yml` | ينزّل الـ APK المنشور ويشخّصه بلا إعادة بناء |
| `android-source.yml` / `android-ui-release.yml` | تاريخية (مقارنة `packaging/android/source-checks.yml`، QA قديم) |

**ملاحظة تشغيلية:** `tools/ci-craftix-078-notes.md` داخل `paths` الخاصة بـ
`android-redesign-release.yml` ⇒ أي تعديل على ذلك الملف يعيد بناء الـ Rust APK
(~15 دقيقة) ويغيّر الـ SHA. لذلك حدّثنا نصوص الإصدار على GitHub بـ
`gh release edit --notes-file` بدل تعديل الملف. **مصدر الحقيقة للأرقام هو
`SHA256SUMS_*.txt` و`CRAFTIX_TEST_REPORT.md` لا ذلك الملف.**

---

## 6. الإصدار المنشور [`craftix-0.7.8`](https://github.com/n83527040-max/Horizon/releases/tag/craftix-0.7.8)

| الأصل | الحجم | SHA-256 |
|---|---|---|
| [`CRAFTIX-Flutter-Rust-0.7.8-arm64-v8a.apk`](https://github.com/n83527040-max/Horizon/releases/download/craftix-0.7.8/CRAFTIX-Flutter-Rust-0.7.8-arm64-v8a.apk) | 87,046,381 B | `873cf57b0333053f8202d15f24068fb1fcc10d13e3f1b0b23eef51c7bcc6781e` |
| [`CRAFTIX-Rust-0.7.8-arm64-v8a.apk`](https://github.com/n83527040-max/Horizon/releases/download/craftix-0.7.8/CRAFTIX-Rust-0.7.8-arm64-v8a.apk) | 36,284,309 B | `19c874f9cef26af75b4517d5594ab05f098cf08b12f4335a1f7f44fa86747088` |

- `MIXED_APK_VERIFICATION.json`: `passed=true`, `errors=[]`,
  `com.craftix.launcher` / v30 / 0.7.8، Portrait ثنائي، `:engine`،
  `debuggable=false`، v1+v2، مُوقّع واحد، ABI `arm64-v8a`.
- محتوى مختلط: `libhorizon_android.so` + `libflutter.so` + `libapp.so` +
  `assets/engine-data.zip` + `assets/flutter_assets/*`.
- أصول إضافية: `SHA256SUMS_RUST.txt`, `SHA256SUMS_FLUTTER.txt`,
  `CERT_FINGERPRINT.json`, `MIXED_CERT_FINGERPRINT.json`, `certificate.txt`,
  `MIXED_certificate.txt`, `MIXED_signature.txt`, `badging.txt`,
  `MIXED_badging.txt`, `manifest-tree.txt`, `MIXED_manifest-tree.txt`,
  `APK_VERIFICATION.json`, `release-audit.json`,
  `INSTALL_DIAGNOSTICS.json`, `MIXED_INSTALL_DIAGNOSTICS.json`, `BUILD_INFO.txt`.
- إصدارات تاريخية (للمقارنة فقط): `redesign-0.7.6`, `flutter-mixed-0.7.6`,
  `horizon-0.7.7`. **لا تخلط بينها وبين CRAFTIX 0.7.8.**

### بوابات CI الخضراء الموثّقة
`41644462677`, `41644462684` (0.7.7 verify/release)،
`41653704923`, `41654860781` (0.7.6 mixed)،
`41698930063`, `41703484804`, `41707815054`, `41707815327` (0.7.8)،
`41710991357`, `41710991450`, `41719778413`, `41719778241` (توقيع)،
`41718389434` (تشخيص العلة), `41724975391` (الإصلاح),
`41726421227` (تشخيص مستقل بعد الإصلاح), `41723266063`.

---

## 7. شنو تْأكّد فعلياً / شنو ما زال ما تْأكّدش

**مُثبت مصدرياً/وحدوياً/ثنائياً (CI):**
الـ 49 إجراءً، تدفّقات Microsoft/Offline (اختبارات منفصلة)،
البحث الحقيقي وتصنيف الأخطاء، التنزيل/التثبيت عبر `official_install`،
`Play` → `Launcher::action`، SAF → `content_files::validate`،
التوقيع v1+v2 بنفس المفتاح، المانيفست الثنائي، ABI، الشهادة،
التخطيط بلا overflow على 360×720 … 960×540، الترجمة RTL.

**غير مُختبر على جهاز/AVD — صرّح به دائماً ولا تدّعِ نجاحه:**
تثبيت/تشغيل APK، Minecraft/Vulkan/wgpu، تسجيل دخول Microsoft/Xbox/XSTS/
Minecraft Services بمستخدم حقيقي، SAF والتنزيل من Modrinth على الجهاز،
FPS/GPU/الذاكرة، تسريب الذاكرة، الدوران الفيزيائي، إلغاء التنزيل،
نقل بيانات المستخدم بين معرّفي الحزمة. آخر لقطة من المستخدم أثبتت فشل تثبيت
→ شُخّص وأُصلح في CI لكن **لم نُثبت التثبيت على هاتفه**؛ طلبنا إعادة التنزيل
ومقارنة الـ SHA.

---

## 8. القواعد التي يجب أن يكمل بها الوكيل التالي (من كل الطلبات السابقة)

1. **نفس مفتاح CRAFTIX دائماً** (§0) — أهم قاعدة على الإطلاق.
2. ممنوع إعادة كتابة Rust Engine / Vulkan / wgpu / winit / منطق Minecraft،
   وحذف/تعطيل أي وظيفة، وتغيير نظام الحسابات/المودات/الملفات/الإعدادات،
   وكسر توافق الملفات القديمة. أي تغيير في المحرك ⇒ **قف + حلّل الأثر + وثّقه** أولاً.
3. ممنوع Mock/Demo data، أزرار شكلية، نجاح وهمي، إخفاء الأخطاء برسالة
   «لا توجد نتائج»، أو اعتبار `ok=true` من JNI دليلاً دون أثر على الملفات/المحرك.
4. لا تُعلن «مكتمل» أو «يعمل» قبل اختبار فعلي؛ افصل دائماً بين أدلة CI وأدلة الجهاز.
5. حافظ على فصل Flutter ↔ Rust، وعلى `:engine` المنفصل،
   وعلى Portrait للمشغّل / دوران حر للعبة.
6. بيانات حقيقية فقط؛ أخفِ القيمة إن لم يوفّرها المضيف (لا تخترع RAM/FPS/نسخة).
7. كل زر يجب أن يعمل؛ كل صفحة كاملة؛ دعم 360dp فأكثر؛ SafeArea؛ لا overflow.
8. شغّل الفحوصات كلها بعد أي تعديل، وانشر تقريراً صادقاً بما مرّ وما لم يُختبر.
9. لا تضع أسراراً/كلمات مرور/Base64 في المستودع أو اللوجات أو الدردشة.
10. لا تُبدّل الفرع: `arena/92db30ff-horizon` فقط؛ الدفع إليه فقط؛
    أي PR منه فقط.

---

## 9. كيف تكمل عملياً (Checklist للإصدار القادم)

1. ارفع الإصدار في **كل** المواضع معاً:
   - `packaging/android/build.gradle` → `versionCode`, `versionName`
   - `flutter/pubspec.yaml` → `version: X.Y.Z+N`
   - `tools/prepare_mixed_flutter.py` → ثوابت `versionCode`/`versionName` في `patch_gradle()`
   - `tools/verify_mixed_apk.py` → توقّع `versionCode`/`versionName`
   - `tools/verify_flutter_migration_audit.py` → فحص version/minSdk + أسماء أصول الإصدار
   - `flutter/assets/host-info.json` بـ `python3 tools/verify_flutter_foundation.py --generate`
   - `tools/ci-craftix-078-notes.md` + وثائق `CRAFTIX_*`
2. **لا تغيّر** أسماء الأسرار ولا `signingConfigs.craftix` ولا
   `resign_craftix_apk.sh` ولا بصمة `77:05:...`.
3. ادفع → راقب `flutter-foundation.yml`، ثم `android-redesign-release.yml`،
   ثم `flutter-mixed-apk.yml` (ينتظر أصل الـ Rust)، ثم `apk-install-diagnose.yml`.
4. تحقق: `MIXED_APK_VERIFICATION.json`, `CERT_FINGERPRINT.json`
   (`matchesOwnerReference: true`), `INSTALL_DIAGNOSTICS.json`
   (`v1Present && v2SigningBlockPresent && passed`), و`cmp` التنزيل.
5. حدّث `CRAFTIX_TEST_REPORT.md` و`ACTION_TEST_MATRIX.md`
   بـ `python3 tools/verify_action_matrix.py --write`.
6. صرّح دائماً بما لم يُختبر على الجهاز.

**مهام مقترحة متبقية (اختيارية، بترتيب الأولوية):**
- اختبار فعلي على جهاز/AVD (تثبيت، Play، Vulkan، Microsoft، SAF، Modrinth).
- خطاف إلغاء حقيقي للتنزيل (يحتاج تعديلاً في `official_install` ⇒ تحليل أثر أولاً).
- صورة اللاعب/الجلد الحقيقية، وإدارة إصدارات Minecraft (تحتاج موافقة صريحة
  لأنها تمس المحرك).
- نقل بيانات يدوي بين `dev.horizon.android` و`com.craftix.launcher` عبر SAF.
- تقليل إعادة البناء التوثيقية (أضف `- '!**/*.md'` إلى `paths` في
  `android-redesign-release.yml` — سيُشغّل الـ workflow مرة أخيرة ثم يتوقف).
- إزالة الأرقام القديمة من `tools/ci-craftix-078-notes.md` لصالح
  «راجع SHA256SUMS».

---

## 10. خريطة الملفات (مرجع سريع)

| المسار | الدور |
|---|---|
| `crates/horizon-client/src/android_main.rs` | جذر `libhorizon_android.so`، `APP`، `auto_play`، `#![recursion_limit="512"]` |
| `crates/horizon-client/src/android/flutter_bridge.rs` | JNI: snapshot/dispatch/accounts/install/import |
| `crates/horizon-client/src/android/bridge_rules.rs` | قواعد نقية + 7 اختبارات (`rustc --test`) |
| `crates/horizon-client/src/android/catalog.rs` | بحث/تثبيت/فلاتر حقيقية (getters مضافة فقط) |
| `crates/horizon-client/src/android/launcher.rs` | `Launcher::new(home, auto_play)` → `action()` الأصلي |
| `crates/horizon-client/src/official_install.rs` | `ready()` + `install_with_progress()` + `InstallProgress/InstallStage` |
| `flutter/android/app/src/main/AndroidManifest.xml` | Flutter launcher + `:engine` + provider + embedding v2 |
| `flutter/android/app/src/main/kotlin/.../{MainActivity,HorizonBridge,HorizonEngine}.kt` | مضيف Android + JNI |
| `flutter/android/app/src/main/res/**` | أيقونات mipmap/adaptive + styles + splash |
| `flutter/lib/**` | واجهة Flutter كاملة (§3.5) |
| `flutter/test/**` | 57 اختباراً + golden generation + helpers/fixtures |
| `packaging/android/{build.gradle,AndroidManifest.xml,java/**,res/**}` | APK الـ Rust-only |
| `tools/ci_craftix_signing.sh` | فكّ keystore من الأسرار إلى RUNNER_TEMP |
| `tools/resign_craftix_apk.sh` | إعادة التوقيع v1+v2 بنفس المفتاح |
| `tools/check_apk_signature_schemes.py` | فحص ثنائي v1+v2 |
| `tools/ci_record_certificate.py` | بصمة الشهادة العامة + مقارنة المرجع |
| `tools/diagnose_apk_install.py` | تشخيص «الحزمة غير صالحة» |
| `tools/prepare_mixed_flutter.py` | scaffold Gradle + رقع الهوية + staging payload |
| `tools/verify_craftix_release.py` | 48 فحصاً للهوية/التوقيع/المثبّت/العدّاء |
| `tools/verify_mixed_apk.py`, `verify_android_apk.py`, `check_android_release.py` | تحقق APK |
| `tools/verify_flutter_{foundation,native_bridge,migration_audit}.py`, `verify_action_matrix.py` | عقود + مصفوفة 49 |
| `tools/build_release_apk.sh`, `android_release.py`, `android_build.py`, `ci_android_setup.sh`, `android_prepare.py` | سلسلة بناء Rust |
| `.github/workflows/*.yml` | 6 بوابات CI (§5.6) |
| `CRAFTIX_TEST_REPORT.md`, `CRAFTIX_MIGRATION_ANALYSIS.md`, `CRAFTIX_HANDOFF.md` | التقارير |
| `ACTION_TEST_MATRIX.md`, `FLUTTER_BRIDGE_EVIDENCE.md`, `FLUTTER_NATIVE_BRIDGE_ANALYSIS.md`, `HORIZON_077_DIAGNOSIS.md` | أدلة المراحل السابقة |

---

## 11. خلاصة بجملة واحدة للوكيل التالي

> المشروع الآن **CRAFTIX 0.7.8**: واجهة Flutter احترافية + محرك Rust/Vulkan كما هو،
> 49 إجراءً موصولاً بـ JNI/C ABI، تنزيل/Play حقيقي على الشاشة الرئيسية،
> APK مختلط موقّع بـ **v1+v2 بنفس مفتاح CRAFTIX من أسرار GitHub Actions**
> (بصمة `77:05:...:2F`). كل شيء مُثبت في CI؛ **الجهاز لم يُختبر بعد**.
> القاعدة رقم 1: **لا مفتاح جديد، لا تغيير توقيع، لا أسرار في المستودع** — أبداً.
