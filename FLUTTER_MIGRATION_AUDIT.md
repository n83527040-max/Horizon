# تدقيق الانتقال إلى Flutter / Flutter Migration Audit — Horizon Android 0.7.6

> **الحالة (محدَّثة بعد قراراتك):** المراحل 1 و2 (التحليل الكامل + الجرد) منفّذتان
> ومُختبَرتان آلياً. وبعد إجابتك على الأسئلة، نُفِّذت المرحلتان 3 و4 (بنية Flutter +
> نظام التصميم) مع كل الصفحات والتنقل وعقد الجسر المكتوب، **واجتازتا باب اختبار CI
> فعلياً** ( run `41571959078`: `flutter analyze` بلا أخطاء، 23/23 اختبار Flutter،
> 35/35 اختبار Android حالي، 34/34 تدقيق، 18/18 عقد). **المراحل 5–13 لم تُعلَن مكتملة**:
> الجسر الخاص بمنطق المحرك (البحث/التثبيت/الحسابات) وAPK المختلط واختبارات الجهاز
> لم تُنجز بعد. انظر القسم 9 للتفصيل.
> لا يُقدَّم Prototype ولا واجهة وهمية: كل قيمة تُقرأ من المضيف الأصلي، والقيم غير
> المتوفرة تُخفى، والإجراءات غير الموصولة تُعيد فشلاً صريحاً لا نجاحاً مزيّفاً.

---

## 0. ملخص تنفيذي (Executive summary)

1. **الصورة المرجعية المرفقة ليست تصميماً جديداً.** هي ترتيب مربّع (1254×1254) لنفس
   شاشات Home / Mods / Packs / Shaders / Installed / Settings الموجودة فعلاً في
   `tools/ui-preview/portrait-overview.png` وفي APK المنشور. الهوية البصرية
   (Dark Navy، Purple → Blue، Hero، البطاقات، شريط التنقل السفلي) مُنفَّذة بالفعل
   في المصدر، ومبنية، وموقّعة، ومنشورة. المطلوب الجديد هو **نقل الأساس التقني إلى Flutter**،
   وليس إعادة التصميم.
2. **التطبيق لا يحتوي أصلاً على طبقة Kotlin/Java UI.** هو `android.app.NativeActivity`
   واحد + مكتبة Rust واحدة `libhorizon_android.so` (`winit` + `wgpu`/Vulkan) ترسم
   واجهة المشغّل **واللعبة** على نفس السطح (surface) ونفس حلقة الأحداث. وثيقة المشروع
   `docs/android.md` تقول حرفياً: *"There are no project Java/Kotlin classes and no
   second copy of the engine in the APK."* ملفا Java الوحيدان هما ردّ SAF و
   `DocumentsProvider`. لذلك افتراض الطلب «الوظائف الأصلية في Native Kotlin/Java،
   اربطها بـ Platform Channels» **لا ينطبق على هذا المستودع**: الوظائف الأصلية في Rust،
   ولا توجد قنوات جاهزة.
3. لجعل «Flutter هو الأساس الفعلي» مع «عدم تغيير Rust Engine / Vulkan /
   Minecraft Launch Logic» تعارض هندسي مباشر: يجب استبدال الـ Activity وحلقة أحداث
   winit وطريقة إنشاء سطح Vulkan، أو إضافة مسار دخول جديد للّعبة. هذا يلمس طبقة
   الاستضافة/النافذة في المحرك حتى لو بقيت محاكاة Minecraft نفسها untouched.
   كما يتعارض مع قاعدة المستودع في `AGENTS.md`: *"Everything players run is Rust."*
4. **البيئة لا تملك أياً من**: `flutter`, `dart`, `cargo`, `rustc`, `java`, `gradle`,
   `adb`, جهاز أو AVD. الوصول الشبكي محصور في GitHub/npm/PyPI، بينما أدوات Flutter
   تُحمَّل من `storage.googleapis.com`. الإثبات الوحيد المتاح هو GitHub Actions
   (أثبت سابقاً بناء APK حقيقي)، لكنه **لا يستطيع** تشغيل اللعبة على Vulkan، ولا تسجيل
   دخول Microsoft، ولا SAF picker، ولا قياس Memory Leaks على جهاز فعلي.
5. لهذا أُنجز التحليل والجرد كاملين ومُختبَرين، وتوقّفت قبل المرحلة 3 طالباً قراراتك.
   إنتاج «سقالة Flutter» بصفحات غير موصولة بالوظائف الحقيقية كان سيخالف شروطك صراحةً.

---

## 1. البنية الحالية كاملة (Phase 1 — تحليل المشروع)

### 1.1 طبقات التنفيذ

| الطبقة | الملف / الرمز | الدليل |
|---|---|---|
| Android manifest | `packaging/android/AndroidManifest.xml` | Activity واحدة `dev.horizon.android.HorizonActivity`، `android:screenOrientation="portrait"`، `android.app.lib_name=horizon_android`، `activity-alias` يحافظ على اختصارات `GameActivity` القديمة، و`HorizonDocumentsProvider` |
| Java (الوحيد) | `packaging/android/java/dev/horizon/android/HorizonActivity.java` | `extends android.app.NativeActivity`؛ يحتوي **فقط** `onActivityResult` + `nativeDocumentResult` لردّ SAF (request 4101) |
| Java (الوحيد) | `packaging/android/java/dev/horizon/android/HorizonDocumentsProvider.java` | موفّر مستندات SAF للوصول لمجلدات المودات/الحزم |
| المكتبة الأصلية | `crates/horizon-android/Cargo.toml` | `[lib] path = "../horizon-client/src/android_main.rs"`، `crate-type = ["cdylib"]`؛ `winit/android-native-activity`، `wgpu/vulkan`، `jni`، `ureq`، `image`، `zip`، `serde` |
| نقطة الدخول | `crates/horizon-client/src/android_main.rs` | `#[no_mangle] fn android_main(app: AndroidApp)`، `android_on_create`، `thread_local! APP`، `android_event_loop()` يبني `EventLoop` بـ `with_android_app`، `backend::data_dir()`، `android_arguments()` يشير إلى `.local/horizon-client/26.3` |
| المشغّل + اللعبة | `crates/horizon-client/src/android/launcher.rs` | `struct Launcher implements winit::application::ApplicationHandler`؛ يمتلك `window`, `renderer`, `gui`, `game: Option<Game>`, `catalog`, `login`, `chooser`. `open()` ينشئ النافذة بـ `events.create_window` ويبني `HudAtlas`؛ `resumed/suspended/window_event` تعالج اللمس/لوحة المفاتيح/اليد/IME؛ `restore_launcher()` يعيد من اللعبة إلى المشغّل **داخل نفس العملية ونفس السطح** |
| رسم الواجهة | `crates/horizon-client/src/android/launcher_pages.rs` | 1445 سطراً: كل الصفحات تُرسم عبر `horizon_client::ui_theme` (`HudBatch`/`HudAtlas` على wgpu) — ليست Views أندرويد ولا XML ولا Compose |
| نظام التصميم | `crates/horizon-client/src/android/design.rs`, `crates/horizon-client/src/android/ui_spec.json`, `crates/horizon-client/src/ui_theme.rs`, `crates/horizon-client/src/android/assets/hero.png` | الألوان، الأيقونات، `Layout::with_insets`، الحواف الآمنة، Hero، التدرّجات |
| الحالة والبيانات | `crates/horizon-client/src/android/catalog.rs` | `Config` (profile, proxy, name, server, microsoft_client_id, renderer, render_scale, orientation, screen, input_host, keypad_controls, record_performance, device, mod_source, shader_source)، `Catalog` (local, results, progress, status, workers)، `Page` بـ 6 قيم، `Orientation` بـ 5 قيم |
| الأوامر | `crates/horizon-client/src/android/launcher_pages.rs` | `enum Action` بـ **49** variant؛ `activate()` (`launcher_pages.rs:1112+`) يربط كل هدف لمس بالوظيفة الحقيقية |
| جسور JNI/Native | `crates/horizon-client/src/android/platform.rs`, `crates/horizon-client/src/android/native_ime.rs`, `crates/horizon-client/src/android/secure_store.rs`, `crates/horizon-client/src/android/document_bridge.rs`, `crates/horizon-client/src/android/document_provider.rs`, `crates/horizon-client/src/android/device_runtime.rs`, `crates/horizon-client/src/android/device_gyro.rs`, `crates/horizon-client/src/android/diagnostics.rs`, `crates/horizon-client/src/android/exit_reports.rs`, `crates/horizon-client/src/android/native_tombstone.rs` | الاتجاه، الحافظة، لوحة المفاتيح النظامية، Android Keystore AES/GCM، SAF، الاهتزاز، الدوران، الطاقة، التقارير، tombstones |
| منطق مشترك مع سطح المكتب | `launcher/src/account.rs`, `launcher/src/browser.rs`, `launcher/src/skins.rs`, `launcher/src/proxy.rs` | مُضمَّن في `android_main.rs` عبر `#[path = "../../../launcher/src/..."]`: المصادقة، بحث Modrinth/Horizon، الجلود، البروكسي |
| إدارة المحتوى | `crates/horizon-client/src/content_files.rs`, `crates/horizon-client/src/android/catalog.rs`, `crates/horizon-client/src/android/shader_catalog.rs`, `crates/horizon-client/src/android/documents.rs` | `Kind::{Mods,Packs,Shaders}`، تثبيت/تعطيل/حذف، التوافق، استيراد SAF مع حلّ التعارض والتحقق من الملفات |
| الأداء والتشخيص | `crates/horizon-client/src/android/performance.rs`, `crates/horizon-client/src/android/diagnostics.rs`, `crates/horizon-client/src/android/design.rs` | `/proc` VmRSS/VmHWM/MemTotal، `RendererMode`، `RenderScale`، تلميحات ADPF، GPU profiling، تسجيل حتى 30 دقيقة، تقارير ZIP/JSONL. **لا يوجد FPS حيّ** — مُخفى عمداً لأن المحرك لا يوفّره |
| التغليف والإصدار | `tools/android_prepare.py`, `tools/android_build.py`, `tools/android_release.py`, `tools/build_release_apk.sh`, `tools/verify_android_apk.py`, `packaging/android/build.gradle`, `.github/workflows/android-redesign-release.yml` | Gradle 8.14 / AGP 8.11.1، SDK 35، NDK، minSdk 28، `versionName '0.7.6'`، `.so` واحدة + جداول المحرك، توقيع v2، فحص المانيفست الثنائي |
| الاختبارات | `tests/test_android_ui_redesign.py`, `tests/test_android_release_audit.py`, `tests/test_android_release_schedule.py`, `tests/test_android_source_export.py`, `tools/android_ui_layout.py`, `tools/android_ui_render.py` | 100 ملف اختبار Python؛ 35 اختباراً لتصميم Android؛ تحقّق هندسي للأحجام والحواف الآمنة |

### 1.2 لماذا «Platform Channels» غير كافٍ هنا

```
[FlutterActivity/Dart] ──Platform Channel──> [Kotlin/Java] ──JNI──> [Rust cdylib] ──wgpu/Vulkan──> ANativeWindow
                                                   ▲
                              لا توجد هذه الطبقة حالياً؛
                              Rust يملك الـ Activity والنافذة وحلقة الأحداث مباشرة.
```

- `Launcher` هو `ApplicationHandler` لـ winit: الدالة `android_main` لا تُعيد أبداً،
  و`Catalog`/`Login`/`documents` تعيش داخل `Launcher` وتستخدم `AndroidApp`,
  `ActiveEventLoop`، و`thread_local! APP`. **لا يمكن استدعاؤها من JNI/FFI كما هي**؛
  تحتاج طبقة استضافة جديدة قابلة للاستدعاء كمكتبة (JSON over C ABI أو JNI) مع حالة
  thread-safe وحلقة أحداث مستقلة.
- اللعبة تستخدم نفس `Window` ونفس `wgpu::Surface`. إذا أصبح Flutter هو الـ Activity،
  فإمّا (أ) تمرير `ANativeWindow` من `SurfaceView` platform view إلى wgpu عبر
  `AndroidNdkWindowHandle` — أي تعديل طبقة نوافذ المحرك، أو (ب) إبقاء NativeActivity
  كـ Activity ثانية للّعبة مع تعديل `android_main.rs` لتخطّي واجهة المشغّل عند وجود
  علم في الـ Intent — أي تعديل مسار دخول الإطلاق. كلاهما يلمس ما نصّ الطلب على عدم تغييره.

---

## 2. جرد الوظائف والاعتماديات (Phase 2)

### 2.1 كل وظيفة مطلوبة ↔ تنفيذها الحقيقي

| الوظيفة المطلوبة | التنفيذ الحالي (ملف : رمز) | الربط الحالي | ملاحظة للانتقال |
|---|---|---|---|
| شعار HORIZON + الإصدار + Settings | `launcher_pages.rs` (`draw_home`) + `android_main.rs::backend::VERSION` | بيانات حقيقية `0.7.6` | مباشر |
| Minecraft Hero | `crates/horizon-client/src/android/assets/hero.png` + `ui_theme.rs::install_hero` | `include_bytes!` داخل `.so` | يُنقل كـ Flutter asset |
| زر Play | `Action::Play` → `launcher_pages.rs::activate` → `Launcher::action(events)` | يبدأ اللعبة في نفس الحلقة | **أخطر نقطة**: يحتاج handoff |
| بطاقة الإصدار | `research-lock.json` / `docs/android.md` | الإصدار المثبّت `26.3` فقط | ⚠ انظر 2.2 |
| بطاقة Profile | `catalog.rs::Config::name` + `login.rs::status` | Offline / Microsoft status | مباشر |
| Manage: 4 بطاقات | `Action::Page(Page::{Mods,Packs,Shaders,Installed})` | عدّادات `Catalog::local` الحقيقية | مباشر |
| Performance: RAM / GPU / Graphics API | `design.rs` (`/proc`), `platform.rs::performance_power`, `performance.rs`, `catalog.rs::RendererMode/RenderScale` | بيانات حقيقية فقط؛ FPS مخفي | مباشر |
| Bottom Navigation (5) | `design.rs::nav_item` + `launcher_pages.rs` | 5 وجهات، حالة محفوظة، حواف آمنة | Flutter `NavigationBar` + state preservation |
| Mods: بحث / مصادر / Install / توافق / تفاصيل | `Action::{Search,ModSource,Install,OpenFolder,Import}`، `launcher/src/browser.rs` (`SourceId::{Horizon,Modrinth}`, `compat`) | HTTP حقيقي + SHA-512 + دعم `Support::*` | bridge مطلوب لكل نداء |
| Packs | نفس المسار بـ `Page::Packs` + `Kind::Packs` | حقيقي | نفسه |
| Shaders | `Page::Shaders`، `shader_catalog.rs::Origin`، `Action::ShaderSource` | حقيقي (`.hshader` + Modrinth ZIP) | نفسه |
| Installed: أقسام/عدد/Enable/Disable/Remove/Import | `Action::{ToggleSection,ToggleLocal,DeleteLocal,ConfirmDeleteLocal,Import}`، `catalog.rs` | حقيقي مع تأكيد حذف | نفسه |
| Settings — Graphics | `Action::{Renderer,RenderScale,Aspect,DisplayHz,Immersive,Cutout}` | `Config` + `platform::set_orientation` + `display.rs` | نفسه |
| Settings — Performance | `Action::{PerformanceHints,GpuProfile,RecordPerformance,ExportDiagnostics,ExportPerformance}` | `device_preferences.rs`, `performance.rs`, `diagnostics.rs` | نفسه |
| Settings — Accounts | `Action::{Account,SignIn,DeviceSignIn,CopyLoginCode,LoginLink,SignOut,CloseLogin}` | `login.rs` + `secure_store.rs` + `launcher/src/account.rs` | ⚠ الأمان: Keystore فقط، لا كلمات مرور |
| Settings — App | `Action::{Profile,InputHost,Orientation,Field("server"),Logs,Repository}` + شارة الإصدار | حقيقي | ⚠ انظر 2.2 |
| نافذة الحسابات | `crates/horizon-client/src/android/login.rs` | Browser + Device Code + Offline + الحالة + رمز/رابط + تسجيل خروج | ⚠ «صورة اللاعب» انظر 2.2 |
| Minecraft Java Runtime / Vulkan | `crates/horizon-client`, `wgpu 30`, `android_runtime::graphics_startup` | محرك Rust كامل | **لا يُلمس** |
| إدارة الملفات / SAF | `documents.rs`, `document_bridge.rs`, `HorizonDocumentsProvider.java` | استيراد محدود، تحقّق، حل تعارض | يجب أن يبقى Java + JNI/Channel |
| Modrinth | `launcher/src/browser.rs` | بحث/تنزيل/تحقق | نفسه |
| Game Settings / Graphics / Monitoring | `catalog.rs`, `display.rs`, `device_preferences.rs`, `performance.rs` | حقيقي | نفسه |

### 2.2 سجل الفجوات: عناصر في الطلب **غير موجودة** حالياً

هذه ليست أخطاء، بل حقائق يجب ألا تُحوَّل إلى واجهات وهمية (شرطك: «لا توجد بيانات تجريبية بدلاً من البيانات الحقيقية»):

| العنصر المطلوب | الواقع في 0.7.6 | ما يلزم لو أُريد إضافته |
|---|---|---|
| «Minecraft Version Management» / اختيار إصدار | `docs/android.md`: *"It does not run arbitrary Minecraft JAR versions"*؛ الإصدار مثبّت على `26.3` في `research-lock.json`/`android_arguments()` | إما بطاقة عرض فقط للإصدار المثبّت (كما في الصورة المرجعية)، أو ميزة تنزيل إصدارات جديدة حقيقية — **تغيير في المحرك** |
| Settings → Theme | لا يوجد خيار ثيم؛ التطبيق Dark-only | ثيم حقيقي يحفظ في `Config` ويطبَّق على كل الشاشات |
| Settings → Language | لا يوجد اختيار لغة في Android (يوجد `launcher/lang` لسطح المكتب فقط) | ملفات ترجمة + ربط بـ `Config` |
| Settings → Storage / Cache Management | يوجد فقط `official_install::cleanup_completed_downloads` تلقائي عند الإقلاع | شاشة حقيقية تقرأ أحجام المجلدات وتنفّذ تنظيفاً فعلياً |
| Settings → About HORIZON | توجد شارة الإصدار + `Action::Repository` + `LICENSE`/`THIRD_PARTY_NOTICES.md` | صفحة About حقيقية من هذه المصادر |
| Accounts → «صورة اللاعب عند توفرها» | `launcher/src/skins.rs` موجود لكن واجهة `login.rs` تعرض الاسم/الحالة فقط | جلب الجلد الحقيقي من ملفات اللاعب/الجلسة؛ لا صورة وهمية |
| Performance → GPU / FPS | Graphics API و RAM و Render scale و `performance_power()` حقيقية؛ **لا FPS حيّ** | إبقاء الإخفاء أو إضافة مقياس حقيقي من المحرك |
| Landscape Mode | المانيفست `screenOrientation="portrait"` + `Orientation::default() = Portrait` + clamp لـ Auto (طلب سابق ملزم). `catalog.rs::Orientation::ALL` ما زال يحتوي Landscape/ReverseLandscape/ReversePortrait و`Action::ChooseOrientation` يستدعي `platform::set_orientation`، لكن القفل الافتراضي يمنع الدوران التلقائي | ⚠ تعارض مباشر — القرار 2 |

### 2.3 جرد الاعتماديات

- **Rust:** 40 crate في `Cargo.toml` workspace، 1781 ملف `.rs`، Rust 1.95 مثبّت في
  `rust-toolchain.toml`، `wgpu 30.0.1`، `winit 0.30.13` (مع `third_party/winit` patch)،
  `jni 0.21`، `ureq`، `image`، `zip`، `serde`، `naga`، `ab_glyph`، `horizon-*`.
  لا توجد أي تغييرات على `Cargo.toml`/`Cargo.lock` في أعمال التصميم السابقة.
- **Android:** `compileSdk 35`، `minSdk 28` (Android 9+)، `versionName '0.7.6'`،
  AGP 8.11.1 / Gradle 8.14، NDK، 4 ABI (الإصدار المنشور `arm64-v8a`)، توقيع v2،
  `debuggable=false`.
- **بيانات المحرك:** `tools/android_prepare.py` ينزّل 5214 ملفاً من Mojang للإصدار 26.3
  ويُشغّل مُصدِّرات Java لتوليد الجداول؛ `engine-data-generated.zip` داخل APK؛
  الموارد الأصلية تُنزَّل على الجهاز عند أول تشغيل.
- **أدوات/اختبارات:** أكثر من 480 سكربت Python في `tools/`، 100 ملف اختبار في `tests/`،
  324 parity harness، CI: `cargo check` + `clippy -D warnings` + `assembleRelease` +
  `apksigner` + `verify_android_apk.py` + تنزيل/mقارنة الـ APK.
- **Flutter/Dart (بعد بدء المرحلة 3):** أُضيف `flutter/` بتطبيق Dart كامل
  (`pubspec.yaml` يعتمد `flutter_riverpod` و`flutter_localizations`)، و`*.kt` للمضيف
  الأصلي، و`libflutter.so` لم يُبنَ بعد. قبل هذا العمل كان العدد **صفراً**
  (المطابقات الوحيدة لكلمة flutter كانت أوصاف حركة أجنحة الخفافيش في `mobs/`).

### 2.4 الإصدار المنشور حالياً (لا يُستبدل قبل قرار)

- `https://github.com/n83527040-max/Horizon/releases/tag/redesign-0.7.6`
- `Horizon-Android-Redesigned-0.7.6-arm64-v8a.apk` — 35,693,532 bytes،
  SHA-256 `fff20f808fb098e374e3185a0e786bd8b155d2bcb22ebef829d60f0bae16b5a8`،
  المانيفست الثنائي `screenOrientation(0x0101001e)=1` (portrait)،
  آخر بناء ناجح في Actions `41514732176`.

---

## 3. ما يتطلبه «Flutter هو الأساس» فعلياً (تقدير هندسي)

| # | العمل المطلوب | يلمس | تقدير |
|---|---|---|---|
| 1 | استبدال الـ Activity الرئيسي بـ `FlutterActivity`، إبقاء `HorizonActivity` للّعبة، علم في Intent لتخطّي واجهة Rust | `AndroidManifest.xml`, `android_main.rs` | إلزامي، يلمس مسار الإطلاق |
| 2 | إعادة هيكلة Gradle:Flutter plugin + `pluginManagement` + دمج `jniLibs`/`assets`/`java`/`res` + الحفاظ على التوقيع و DocumentsProvider | `packaging/android/*` | إلزامي |
| 3 | طبقة جسر جديدة: C ABI (`extern "C"` + JSON) أو JNI/Kotlin + Pigeon؛ فصل `Catalog`/`Config`/`Login`/`documents`/`performance`/`diagnostics` عن `winit::EventLoop` و`AndroidApp` | Rust جديد + Kotlin | الإصلاح الأكبر؛ ~50 `Action` يجب أن يصبح لها نداء حقيقي |
| 4 | نقل 6 صفحات + 3 حوارات (Accounts / Chooser / File dialog) إلى Dart: Material 3 مخصص، نظام ثيم مركزي، widgets معادة الاستخدام، Riverpod/Bloc، repositories، حفظ حالة التنقل، Android Back، anims خفيفة | `lib/**` Dart | إعادة بناء كاملة للطبقة البصرية |
| 5 | SAF/الاستيراد/Keystore/IME/الاتجاه/الاهتزاز/الدوران عبر قنوات حقيقية | Kotlin/Java + Rust JNI | لا يجوز استبدالها بمحاكاة |
| 6 | إطلاق اللعبة والعودة منها (`restore_launcher`) مع الحفاظ على Vulkan/OpenGL ES recovery | حلقة أحداث اللعبة | حسّاس جداً |
| 7 | اختبارات `flutter analyze` / `flutter test` / golden للأحجام 360×720 … 480×960 + تابلت + Landscape، وCI يبني APK مختلط Flutter+Rust | workflow جديد | يحتاج Flutter SDK في CI |
| 8 | اختبارات جهاز فعلية: تشغيل Minecraft، تسجيل الدخول، SAF، تثبيت مود، Memory/RAM | جهاز/AVD | **غير متاح في هذه البيئة** |

**الخلاصة الهندسية:** الطلب يجمع بين «لا تُعد كتابة المحرك/الإطلاق» و«Flutter هو الأساس
وليس طبقة فوق التطبيق القديم». تحقيق الثاني يستلزم على الأقل تغيير طبقة الاستضافة
(Activity/نافذة/حلقة أحداث) وبناء جسر لكل الوظائف. هذا ليس مستحيلاً هندسياً، لكنه
مشروع迁移 متعدد المراحل، وليس تعديلاً يمكن إعلان اكتماله واختباره في بيئة بلا جهاز.

---

## 4. التعارضات بين هذا الطلب والقيود السابقة

| # | الطلب الجديد | القيد القائم | طبيعة التعارض |
|---|---|---|---|
| A | «أريد أن يصبح Flutter هو الأساس الفعلي للمشروع» | «لا تلمس Rust Engine أو Vulkan أو Minecraft Launch Logic» + `AGENTS.md`: كل ما يشغّله اللاعب Rust | استبدال الـ Activity/النافذة يلمس طبقة استضافة المحرك |
| B | «دعم Landscape Mode» + أحجام Landscape | «Portrait إلزامي، منع الدوران التلقائي، لا تعتبر المهمة مكتملة حتى يتحقق Portrait» (الطلب السابق، APK المنشور مقفل Portrait) | تناقض مباشر في المانيفست والسلوك |
| C | «لا Prototype / لا صفحات وهمية / كل الأزرار تعمل / APK قابل للتثبيت والتشغيل» | لا يوجد Flutter SDK محلياً، ولا جهاز/AVD، ولا يمكن اختبار Vulkan/تسجيل الدخول/SAF في CI | لا يمكن إثبات «يعمل فعلياً» هنا |
| D | «لا إعادة بناء من الصفر» | «إعادة بناء واجهة المستخدم بالكامل باستخدام Flutter» + إعادة تنظيم المشروع feature-based | إعادة بناء طبقة UI كاملة = جزء كبير من التطبيق |
| E | «Settings: Theme / Language / Storage / Cache» | هذه الخيارات غير موجودة أصلاً في 0.7.6 | إضافتها ميزات جديدة حقيقية، لا مجرد واجهة |

---

## 5. القيود البيئية المُتحقَّق منها (2026-11-03)

| الأداة | الحالة | الأثر |
|---|---|---|
| `flutter`, `dart` | MISSING | لا `flutter analyze` / `flutter test` / `flutter build apk` محلياً |
| `cargo`, `rustc`, `java`, `gradle`, `adb`, `sdkmanager` | MISSING | لا بناء Rust/Gradle ولا تثبيت APK محلياً |
| جهاز / AVD | MISSING | لا اختبار Minecraft، Microsoft، SAF، FPS، Memory Leaks |
| الشبكة | GitHub / codeload / api.github.com / npm / PyPI فقط | حِزم Flutter (Dart SDK + engine artifacts) من `storage.googleapis.com` غير متاحة محلياً؛ GitHub Actions يملك الوصول |
| GitHub Actions | يعمل (بناء APK سابق ناجح `41514732176`) | يستطيع تثبيت Flutter وبناء APK مختلط، **لكنه لا يشغّل اللعبة على جهاز** |

---

## 6. ما اختُبر فعلياً في هذه المرحلة

```
python3 -m unittest discover -s tests -p 'test_android*.py'   → Ran 35 tests ... OK
python3 tools/check_public_tree.py                            → Public tree check passed.
python3 tools/android_ui_layout.py                            → 360x800 / 393x873 / 432x960 /
                                                                480x768 / 800x1280 / 960x540 OK،
                                                                حواف آمنة + cutouts OK
python3 tools/android_ui_render.py --check                    → Portrait geometry OK for 9 pages on 3 screens
python3 tools/verify_flutter_migration_audit.py               → كل مراجع هذا التدقيق مُتحقَّق منها
gh release view redesign-0.7.6                               → APK 35,693,532 bytes، target 6938c3a
git status                                                    → clean (قبل إضافة هذا الملف)
```

الملف `tools/verify_flutter_migration_audit.py` يتحقق آلياً من: وجود كل ملف مشار إليه،
رموز `android_main`/`ApplicationHandler`/`Action`/`enum Page`، عدد variant الـ `Action`،
عدد ملفات Java = 2، غياب أي `.dart`/`.kt`/`pubspec.yaml`، قفل portrait في المانيفست،
`lib_name`، إصدار Gradle، وروابط الإصدار. **أي تغيير في المصدر يجعل التدقيق غير صحيح
سيُفشل الفاحص**، أي أن الجرد قابل لإعادة التحقق لا مجرد نص.

**لم يُختبر / لم يُنفَّذ:** أي مرحلة من 3 إلى 13، لا كود Flutter، لا APK جديد،
لا اختبار جهاز، لا ادعاء «تسجيل الدخول يعمل» أو «Minecraft يعمل» بعد迁移.

---

## 7. القرارات المطلوبة قبل المرحلة 3 (✅ أُجيب عنها — انظر القسم 9)

1. **الهندسة:** هل تسمح بتغيير طبقة الاستضافة (Activity/نافذة/حلقة أحداث + جسر
   FFI/JNI جديد) مع بقاء محاكاة Minecraft و Vulkan و`.so` كما هي؟ أم تفضّل الإبقاء
   على APK المنشور كأساس وFlutter كمشروع منفصل/إضافة؟
2. **الاتجاه:** الإبقاء على **Portrait إلزامي** (المانيفست الحالي + APK المنشور)،
   أم فتح **Portrait + Landscape** مع تخطيط Responsive (يلزم إلغاء قفل المانيفست
   وإعادة التحقق من كل الشاشات، أي تراجع عن شرط سابق)؟
3. **التحقق:** هل تقبل إثباتاً عبر CI فقط (`flutter analyze`, `flutter test`,
   golden/layout tests, `flutter build apk`, فحص المانيفست/التوقيع) مع **تصريح واضح
   بعدم اختبار جهاز فعلي**، أم ستوفّر جهازاً/AVD أو سجلات تشغيل لاختبار
   Minecraft/الحسابات/SAF/الأداء؟

---

## 8. الخطة المقترحة بعد موافقتك (مراحل بأبواب اختبار)

| المرحلة | المخرَج | باب الاختبار |
|---|---|---|
| 3 | `flutter/` بالهيكل feature-based + `pubspec.yaml` + ثيم مركزي بالألوان الجديدة + Pigeon/FFI schema | `flutter analyze` نظيف + اختبارات ثيم في CI |
| 4 | Design System: ألوان `#080B1D/#8B35FF/#3565FF/#20DDF5/#111B38`، بطاقات، glass/glow، تايبوجرافي، أيقونات موحّدة | golden tests على 7 أحجام |
| 5–8 | Home → Mods/Packs/Shaders → Installed → Settings/Accounts | كل صفحة موصولة بنداء حقيقي؛ اختبارات widget + عدم وجود أهداف وهمية |
| 9 | جسر C ABI/JNI لكل `Action` + `Config` + SAF + Keystore + إطلاق اللعبة | اختبارات Rust للجسر + اختبارات قنوات + فحص «لا محاكاة» |
| 10–11 | Responsive (SafeArea/LayoutBuilder/Flexible) + Animations خفيفة + حفظ حالة التنقل + Android Back | golden/layout على كل الأحجام، فحص overflow |
| 12 | `flutter analyze` بلا أخطاء، كل الاختبارات، `clippy -D warnings`، `check_public_tree.py` | CI أخضر بالكامل |
| 13 | Release APK مختلط (Flutter + `libhorizon_android.so`) موقع ومنشور + إثبات التنزيل | `verify_android_apk.py` موسّع + `cmp` داخل CI |

> كل مرحلة تُعلَن مكتملة فقط بعد اجتياز باب اختبارها فعلياً، التزاماً بشرطك
> «لا تعلن اكتمال أي مرحلة قبل اختبارها فعلياً». الاختبارات التي تحتاج جهازاً
> ستبقى معلّمة بوضوح «غير مُختبرة على جهاز» ما لم يتوفر جهاز/AVD.

---

## 9. القرارات المتخذة والعمل المنفَّذ بعدها

### 9.1 قراراتك (مُسجَّلة)

| السؤال | القرار |
|---|---|
| الهندسة | **Flutter كأساس فعلي + جسر حقيقي**: استبدال طبقة الاستضافة (Activity/النافذة/حلقة الأحداث) وبناء C ABI/JNI أو Pigeon مع بقاء محاكاة Minecraft وVulkan و`.so` داخلياً |
| الاتجاه | **Portrait للمشغّل + Landscape للّعبة**: واجهة Flutter عمودية دائماً، و`HorizonActivity` الخاصة باللعبة تحتفظ بسياسة دورانها |
| التحقق | **CI فقط مع تصريح صريح** بعدم وجود اختبار جهاز/AVD |
| الخيارات الجديدة | **إضافتها كميزات حقيقية** (Theme/Language/Storage/Cache/About/صورة اللاعب)، لا كواجهات وهمية |

### 9.2 ما نُفِّذ (المرحلتان 3 و4 + عقد الجسر)

- `flutter/pubspec.yaml`, `flutter/analysis_options.yaml`, `flutter/.metadata`
  — تطبيق Flutter/Dart مع Material 3 و`flutter_riverpod` و`flutter_localizations`.
- `flutter/lib/core/theme/` — نظام مركزي: ألوان `#080B1D/#8B35FF/#3565FF/#20DDF5/#111B38`
  بالضبط، تدرّجات، زوايا، ظلّ/توهّج خفيف، تايبوجرافي، `HorizonMetrics` (LayoutBuilder/
  MediaQuery) وكتالوج en/ar حقيقي مطبَّق.
- `flutter/lib/core/bridge/` — `HorizonAction` بـ **49** اسماً مطابقاً لحرفياً لـ
  `enum Action` في Rust، نماذج `LauncherSnapshot` بكل الحقول قابلة للإخفاء،
  و`MethodChannelLauncherBridge` مقابل `dev.horizon.android/bridge`.
- `flutter/lib/core/state/` — Riverpod: بثّ لقطة حقيقية، حالة تنقل، ثيم ولغة من
  الإعدادات الحقيقية، و`ActionRunner` لعرض نجاح/فشل كل إجراء.
- `flutter/lib/core/widgets/` — بطاقات زجاجية، زر متدرّج مع حركة ضغط، Chips،
  مؤشرات أداء، حالات فارغة/خطأ/تحميل، وشريط تنقل سفلي **بعرض الشاشة كاملاً**
  مع SafeArea ومؤشر نشط متوهّج.
- `flutter/lib/features/` — Home وMods وPacks وShaders وInstalled وSettings
  وAccounts وChooser وImport، كلها تقرأ اللقطة الحقيقية وترسل إجراءات مكتوبة.
- `flutter/lib/navigation/app_shell.dart` — `IndexedStack` يحفظ حالة الصفحات،
  معالجة زر الرجوع عبر `WidgetsBindingObserver.didPopRoute`، وصفحة Shaders
  كوجهة سادسة خارج الشريط.
- `flutter/android/app/src/main/` — `MainActivity : FlutterActivity`،
  `HorizonBridge.kt` يقرأ `android-launcher.json` والمجلدات الحقيقية و`/proc`
  و`host-info.json`، ومانيفست يفصل بين نشاط المشغّل (Portrait) ونشاط اللعبة
  (دوران حر) مع الإبقاء على `DocumentsProvider` واختصارات `GameActivity`.
- `tools/generate` داخل `tools/verify_flutter_foundation.py --generate` يولّد
  `flutter/assets/host-info.json` من `research-lock.json` و`build.gradle` فعلياً.
- `.github/workflows/flutter-foundation.yml` — بوابة CI: عقد عبر اللغات +
  التدقيق + اختبارات Android الحالية + `flutter pub get` + `flutter analyze` +
  `flutter test` على 7 أحجام شاشة.

### 9.3 المرحلة 9 — الجسر الأصلي (مُنفَّذ ومُختبَر في CI، لكن ليس على جهاز)

- **تحليل التأثير أولاً:** `FLUTTER_NATIVE_BRIDGE_ANALYSIS.md` يوثّق قبل أي تعديل
  أن `winit` يملك `EventLoop` واحداً، وأن `android_main` ينهي العملية، وأن
  `platform::with_context` يعتمد على `APP`. القرار: **JNI/C ABI** للعمليات عديمة
  النافذة + **عملية `:engine` منفصلة** للعبة. لا تغيير في `game/` أو wgpu أو
  Vulkan أو `launcher/src/` أو مخطط الملفات أو `packaging/android/java/`.
- `crates/horizon-client/src/android/flutter_bridge.rs` — ستة رموز
  `Java_dev_horizon_android_HorizonEngine_*` تعيد استخدام `Catalog`, `Config`,
  `Accounts`, `SignIn`, `diagnostics`, `performance`,
  `content_files::validate` الحقيقية. كل الـ 49 إجراءً لها منفّذ فعلي؛
  **لا وجود لـ `enginePending` أو نجاح مُفتعل**.
- `android_main.rs` يقرأ علم `horizon.launch_game` فقط، و`launcher.rs` يستدعي
  `self.action(events)` الموجود أصلاً (حقل `auto_play` إضافي فقط).
- `HorizonEngine.kt` / `HorizonBridge.kt` / `MainActivity.kt` — SAF حقيقي عبر
  `ContentResolver`، فتح روابط/حافظة حقيقية، إيقاف ثريد Rust في `onDestroy`،
  وحالات تحميل/إلغاء/خطأ واضحة.
- مصفوفة الإثبات لكل إجراء: `FLUTTER_BRIDGE_EVIDENCE.md`، والفاحص
  `tools/verify_flutter_native_bridge.py` (25/25).
- دليل CI: run `41644462677` أخضر، وrun `41644462684` نجح في
  `cargo check` و`clippy -D warnings` لهدف Android.

### 9.4 ما لم يُنجز بعد (بصراحة)

- **المرحلة 13:** APK مختلط Flutter + Rust بُني في run `41651722140` ونُشر على
  [flutter-mixed-0.7.6](https://github.com/n83527040-max/Horizon/releases/tag/flutter-mixed-0.7.6)
  مع `MIXED_APK_VERIFICATION.json` (`passed=true`). هذا إثبات أرشيفي/توقيعي/مانيفست
  فقط، **وليس إثبات تثبيت أو تشغيل على جهاز**.
- **اختبارات الجهاز:** لم تُجرَ إطلاقاً (تثبيت/تشغيل APK، Minecraft، Vulkan،
  Microsoft، SAF، Modrinth، FPS/GPU، Memory Leaks، الدوران الفيزيائي).
- **الخيارات الجديدة:** Theme وLanguage وStorage وCache وAbout موصولة بقيم/عمليات
  حقيقية؛ «صورة اللاعب» مخفية لأن المضيف لا يوفّر جلداً حقيقياً؛
  «إدارة إصدارات Minecraft» لم تُضف لأنها تتطلب تغييراً في المحرك.
- **حد الإلغاء:** إلغاء تسجيل الدخول والبحث حقيقي (`SignIn::cancel()` و
  `cancel_interest`)؛ التنزيلات الجارية لا يوقفها واجهة `Catalog` الأصلية،
  ولذلك لا يُدعى أنها أُلغيت.


---

## 10. نتيجة بوابة CI للمرحلتين 3 و4 (مُختبَرة فعلياً)

- **الrun:** `41571959078` على `.github/workflows/flutter-foundation.yml`،
  الحالة `success`، كل الخطوات خضراء (بما فيها `Flutter analyze` و`Flutter test`).
- **الأداة:** Flutter `3.47.9` stable / Dart `3.13.7` على GitHub Actions
  (لا يوجد Flutter SDK محلياً في بيئة العمل).
- `flutter analyze --no-fatal-infos` → **No issues found**.
- `flutter test --reporter expanded` → **All tests passed! (23)**:
  عقد الـ 49 إجراءً مقابل Rust، `Page`/`Kind`، ألوان الهوية، ثيم داكن/فاتح،
  مقاييس 7 أحجام، حقيقة اللغة en/ar، MethodChannel، parsing اللقطة،
  شريط التنقل بعرض كامل، حفظ حالة الصفحات، إخفاء القيم غير المتوفرة،
  فتح حوارات الحساب/المنتقي، و**عدم وجود RenderFlex overflow على أي صفحة
  عند 360×720 / 360×800 / 393×873 / 412×915 / 480×960 / 800×1280 / 960×540**.
- `python3 tools/verify_flutter_migration_audit.py` → 34/34،
  `python3 tools/verify_flutter_foundation.py` → 18/18،
  `python3 -m unittest discover -s tests -p 'test_android*.py'` → 35/35.
- أخطاء اكتُشفت وأُصلحت أثناء البوابة: استيراد `Locale`، أيقونة غير موجودة،
  استيرادات غير مستعملة، واجهة `TestDefaultBinaryMessenger` الجديدة،
  و**ع crash حقيقي في التخطيط** (`Flexible` داخل صفوف بعرض غير محدود في
  `Wrap`/الأعمدة المنكمشة) عولج بـ `adaptiveIconLabel` في كل الـ widgets.
  هذا إصلاح فعلي لمنع الانهيار على الشاشات الصغيرة، لا مجرد تعديل اختبار.
- **ما لم يُختبر:** أي تشغيل على جهاز/AVD، Vulkan، تسجيل دخول Microsoft،
  SAF، تثبيت مود، FPS، Memory Leaks، وأي APK مختلط. لا توجد صور شاشة Flutter
  من جهاز حقيقي لأن بيئة العمل لا تملك Flutter SDK ولا جهازاً.


---

## 11. نتيجة المرحلتين 9 و13 (بعد التنفيذ)

| البند | النتيجة |
|---|---|
| تحليل التأثير قبل تعديل Rust | `FLUTTER_NATIVE_BRIDGE_ANALYSIS.md` |
| الجسر | 6 رموز JNI/C ABI، 49/49 إجراءً بمنفّذ حقيقي، صفر `enginePending` |
| حماية المحرك | 0 ملف معدَّل في `game/` أو wgpu أو Vulkan أو `launcher/src/` أو `packaging/android/java/` أو مخطط الإعدادات |
| اللعبة | `Play` → `:engine` → `Launcher::action()` الأصلي؛ لا منطق لعبة جديد |
| الحسابات | `SignIn` / `Accounts` / Android Keystore الأصلية |
| الملفات | SAF → `ContentResolver` → `content_files::validate` الأصلية |
| `flutter analyze` / `flutter test` | بلا أخطاء / 28 من 28 (run `41644462677`) |
| `cargo check` + `clippy -D warnings` | نجحا لهدف Android (run `41644462684`) |
| APK مختلط | 85,947,148 bytes، SHA-256 `73f8de83a7a1b6ab9669390780f92dff6a9676eaa792310653d0404ed666af70` |
| محتوى APK | `libhorizon_android.so` + `libflutter.so` + `libapp.so` + `engine-data.zip` + أصول Flutter |
| المانيفست الثنائي | Portrait للمشغّل، `:engine` منفصل، `lib_name`، `debuggable=false` |
| التوقيع | v2 صحيح، مُوقّع واحد |

**لم يُختبر على جهاز/AVD:** التثبيت، الإقلاع، Minecraft/Vulkan، Microsoft، SAF،
Modrinth، FPS/GPU، تسريب الذاكرة، الدوران الفيزيائي. لذلك لا يُقال «التطبيق يعمل»
أو «لا يوجد انحدار»؛ المتاح هو دليل مصدري/ترجمي/أرشيفي فقط.
