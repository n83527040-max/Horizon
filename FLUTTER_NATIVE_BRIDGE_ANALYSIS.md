# المرحلة 9 — تحليل التأثير قبل تعديل Rust

> مطلوب صراحةً: «إذا ظهر أن ربط وظيفة معينة يتطلب تغييراً في Rust Engine أو
> Vulkan، توقف وحلل التأثير قبل تعديلها». هذا الملف هو نتيجة ذلك التوقف.
> لا يُعلن هنا اكتمال المرحلة 9 أو 13.

## 1. ما تبيّنه قراءة المصدر (حقائق لا تخمين)

| الحقيقة | الدليل في المصدر | الأثر على الجسر |
|---|---|---|
| التطبيق الأصلي `NativeActivity` واحد | `packaging/android/AndroidManifest.xml` + `HorizonActivity.java extends NativeActivity` | لا توجد طبقة Java/Kotlin تستقبل Platform Channels أصلاً |
| المكتبة `libhorizon_android.so` هي `cdylib` جذرها `android_main.rs` | `crates/horizon-android/Cargo.toml` `[lib] path = ../horizon-client/src/android_main.rs` | أي تابع JNI/C ABI يُضاف هنا يصل إلى نفس المكتبة، دون مكتبة ثانية |
| `winit` يملك `EventLoop` واحداً لكل عملية، و`APP` هو `thread_local` | `android_main.rs`: `APP`, `android_event_loop()`, `android_main(app)` | استدعاء `Launcher::action` من ثريد JNI عشوائي **غير آمن**: يحتاج `ActiveEventLoop` ونفس الـ thread |
| `android_main` ينهي العملية بـ `std::process::exit(0)` | نهاية `android_main.rs` | لو عاشFlutter وNativeActivity في نفس العملية، أي خروج من المحرك يقتل واجهة Flutter |
| `platform::with_context` يعتمد على `APP` | `platform.rs::with_context` | clipboard / فتح الروابط / منتقي المستندات تعمل فقط داخل عملية `NativeActivity` |
| سطح Vulkan/wgpu و`Renderer` مملوكان لـ `Launcher` | `launcher.rs` (`renderer`, `window`, `game`) | لا يجوز نقل ملكية السطح إلى Flutter ولا مشاركته بين عمليتين |
| `HorizonDocumentsProvider` و`document_bridge` JNI حقيقيان | `documents.rs`, `document_bridge.rs`, `document_provider.rs` | SAF يجب أن يمر عبر `ContentResolver` الحقيقي، لا مسارات ملفات مُخمّنة |
| `Config` = ملف `android-launcher.json`؛ الحسابات = `Accounts::load()` + Android Keystore | `catalog.rs::Config::load/save`, `secure_store.rs`, `launcher/src/account.rs` | توافق الملفات/الإعدادات/الحسابات القديمة = قراءة/كتابة هذه الملفات نفسها |
| `Catalog` لا يحتاج نافذة ولا `ActiveEventLoop` | `catalog.rs` (`new`, `refresh`, `search`, `install`, `toggle_local`, `remove_local`, `poll`) | **كل** منطق المودات/البحث/التنزيل/التثبيت قابل للتنفيذ من JNI فعلياً |
| `account::SignIn` يعمل على ثريد مستقل بلا واجهة | `launcher/src/account.rs` (`SignIn::start_with_client_id`, `state()`, `cancel()`) | تسجيل الدخول Microsoft قابل للتنفيذ من JNI، وFlutter يعرض الكود/الرابط فقط |
| `ANativeActivity_onCreate` هو نقطة دخول منفصلة عن JNI | `documents.rs` يستخدم `#[unsafe(export_name = "Java_dev_horizon_android_HorizonActivity_nativeDocumentResult")]` | يمكن أن يتعايش تابع JNI ونقطة دخول NativeActivity في `.so` واحدة |

## 2. القرار المعماري (لماذا JNI + عملية محرك منفصلة، وليس Pigeon وحده)

- **Pigeon** يولّد واجهات Dart/Java/Kotlin فقط؛ لا يصل إلى Rust. لذلك الحدّ الحقيقي مع Rust هو **JNI/C ABI** (`#[unsafe(export_name = "Java_dev_horizon_android_HorizonEngine_…")]`). عقد الـ 49 إجراءً يبقى MethodChannel مكتوباً ومُختبَراً (`actions_parity_test.dart`)، وداخل Kotlin يُترجم إلى JNI.
- **العمليتان:**
  - العملية الرئيسية: `MainActivity` (FlutterActivity) + Kotlin + `HorizonEngine` JNI. تنفّذ كل شيء لا يحتاج سطح الرسم: الإعدادات، الحسابات، البحث/التنزيل/التثبيت، الملفات، الأداء، SAF.
  - العملية `:engine`: `HorizonActivity` (NativeActivity الأصلي) يبقى كما هو، ويُستدعى **فقط** لتشغيل Minecraft/Vulkan أو لأي أمر يحتاج `ActiveEventLoop`. `android:process=":engine"` يمنع `std::process::exit(0)` من قتل Flutter.
- هذه ليست «واجهة فوق وظائف ناقصة»: كل إجراء إمّا ينفّذ العملية الحقيقية في Rust/Java ويُعيد نتيجتها، أو يُعيد `needsActivity` صريحاً ويُسلَّم إلى المحرك الحقيقي، أو يُعيد فشلاً صريحاً. **لا توجد قيمة نجاح مُفتعلة**.

## 3. ما الذي يتغيّر في Rust، وما الذي لا يتغيّر أبداً

**لا يتغيّر (محمي):**
`crates/horizon-client/src/game/**`، `crates/horizon-client-render/**`،
`crates/horizon-core/**`، `crates/horizon-world/**`، `crates/horizon-protocol/**`،
`crates/horizon-mods/**`، `crates/horizon-shaders/**`، `launcher/src/**`،
`winit`/`wgpu`، سطح Vulkan، `content_files`، `account`،
مخطط `android-launcher.json`، أسماء مجلدات المحتوى، `packaging/android/java/**`.

**يتغيّر (طبقة استضافة فقط، إضافي):**
1. `android_main.rs`: سطر `#[path = "android/flutter_bridge.rs"] mod flutter_bridge;` + قراءة أمر `horizon.command` عند الإقلاع (لا يلمس `android_main` الأصلي إلا بفرع مبكر).
2. ملف جديد `android/flutter_bridge.rs`: توابع JNI/C ABI تعيد استخدام `Catalog`/`Config`/`Accounts`/`SignIn`/`diagnostics`/`performance` الحقيقية.
3. `launcher.rs`: علم `auto_play` واحد فقط؛ في `resumed()` يستدعي `self.action(events)` الموجود أصلاً. **لا تغيير في منطق `action()` ولا في اللعبة.**
4. `packaging/android/AndroidManifest.xml`: إضافة `MainActivity` و`android:process=":engine"` فقط.

> الخلاصة: لا توجد وظيفة في القائمة التالية تتطلب تعديل Rust Engine أو Vulkan أو
> winit. لو ظهر عكس ذلك أثناء التنفيذ، يُوقف العمل ويُسجَّل هنا قبل أي تعديل.

## 4. جدول الـ 49 إجراءً: من ينفّذها فعلياً

الرموز: `JNI` = Rust حقيقي عبر C ABI · `Kotlin` = Android حقيقي (Context/SAF/Intent) ·
`ENGINE` = عملية NativeActivity الحقيقية · `UI` = حالة Flutter نفسها (لا بيانات مخترعة).

| # | Action | المنفّذ | العملية الحقيقية |
|---|---|---|---|
| 1 | Page | UI | `navIndexProvider` (حالة تنقّل حقيقية) |
| 2 | Play | ENGINE | `Launcher::action()` الأصلي → Vulkan/wgpu/جلسة/تثبيت Minecraft |
| 3 | Search | JNI | `Catalog::search` → Modrinth/Horizon عبر الشبكة + `poll` |
| 4 | ShaderSource | JNI | `Config.shader_source` + `save` + `browse` |
| 5 | ModSource | JNI | `Config.mod_source` + `save` + `browse` |
| 6 | Field | JNI/Kotlin | `name`/`server`/`search`/theme/language/clearCache حقيقية |
| 7 | FinishNameEdit | JNI | `Config::save` للاسم/السيرفر |
| 8 | Install | JNI | `Catalog::install` → تنزيل + فحص توافق + كتابة الملف |
| 9 | Import | Kotlin→JNI | `ACTION_OPEN_DOCUMENT` → `ContentResolver` → `nativeImportStaged` |
| 10 | OpenFolder | Kotlin | `HorizonDocumentsProvider` عبر `ACTION_VIEW` |
| 11 | ToggleSection | UI | حالة أقسام «Installed» الحقيقية |
| 12 | ToggleLocal | JNI | `Catalog::toggle_local` → `options.txt` / `ModList` |
| 13 | DeleteLocal | JNI | `Catalog::pending_delete` |
| 14 | ConfirmDeleteLocal | JNI | `Catalog::remove_local` → حذف ملف حقيقي |
| 15 | Next / 16 Previous | JNI | `Catalog::offset` + `browse` (صفحة نتائج حقيقية) |
| 17 | Profile | UI | فتح منتقي القيم الحقيقية |
| 18 | Logs | JNI | `diagnostics::summary(home)` |
| 19 | RecordPerformance | JNI | `Config.record_performance` + `save` |
| 20 | ExportPerformance | JNI | `performance::export(home)` |
| 21 | ExportDiagnostics | JNI | `diagnostics::export(home)` → ZIP حقيقي |
| 22 | Account | UI | فتح حوار الحسابات |
| 23 | SignIn / 24 DeviceSignIn | JNI | `account::SignIn::start_with_client_id` (Microsoft device code) |
| 25 | CopyLoginCode | Kotlin | نسخ `user_code` الحقيقي إلى الحافظة |
| 26 | CancelLogin / 30 CloseLogin | JNI | `SignIn::cancel()` |
| 27 | ReviewApp | Kotlin | فتح رابط مراجعة التطبيق الحقيقي |
| 28 | Repository | Kotlin | فتح مستودع GitHub الحقيقي |
| 29 | SignOut | JNI | `Accounts::remove(uuid, store())` + `save` |
| 31 | LoginLink | Kotlin | فتح `verification_uri?otc=<user_code>` |
| 32 | Renderer / 39 ChooseRenderer | JNI | `Config.renderer` + `save` |
| 33 | RenderScale / 34 ChooseRenderScale | JNI | `RenderScale::parse` + `Config::save` |
| 35 | Orientation / 38 ChooseOrientation | JNI | `Config.orientation` + `save` (-game يتولى الدوران) |
| 36 | InputHost / 37 ChooseInputHost | JNI | `Config.input_host` + `save` |
| 40 | ChooseProfile | JNI | `Config.profile` + `save` |
| 41 | Dismiss | UI | إغلاق الحوار |
| 42 | Immersive / 43 Cutout | JNI | `Config.screen.*` + `save` |
| 44 | Aspect / 45 ChooseAspect | JNI | `Config.screen.aspect` + `save` |
| 46 | DisplayHz / 47 ChooseDisplayHz | JNI | `Config.device.display_hz` + `save` |
| 47 | PerformanceHints / 48 GpuProfile | JNI | `Config.device.*` + `save` |

(ترقيم الجدول توضيحي؛ number الفعلي هو `HorizonAction` في Dart و`enum Action` في Rust،
ويتحقق منه `test/actions_parity_test.dart`.)

## 5. حدود الإثبات في هذه البيئة

لا يوجد محلياً: Flutter SDK، cargo/rustc، Android SDK/NDK، JDK، Gradle، جهاز أو AVD.
الإثبات المتاح: `flutter analyze`/`flutter test` في CI،
`cargo check`/`clippy` لهدف Android في CI، فاحص عقد بيثوني يثبت أن كل إجراء له
منفّذ حقيقي ولا يوجد `enginePending`، فحص توقيع/مانيفست/مكتبات APK،
واختبارات وحدة للتخطيط. **غير متاح**: تشغيل Minecraft، Vulkan، تسجيل دخول Microsoft
فعلي، SAF على جهاز، FPS، تسريب الذاكرة، الدوران الفيزيائي. كل ذلك يُدرج صراحةً في
قائمة «غير مُختبر» ولا يُعلَن عنه نجاح.
