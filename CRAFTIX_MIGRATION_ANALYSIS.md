# CRAFTIX — تحليل الهجرة وتقرير الأساس (المرحلتان 1 و2 من المواصفة)

> ملف التنفيذ: `CRAFTIX_Implementation_Package.zip` (SHA-256
> `c523b236239d42195c3bc0ddfa49a0a8522e6e9f239c364807025e6c6309fc69`) — قُرئ كاملاً:
> `CONTENTS.txt`، `MESSAGE_TO_CODING_AGENT_AR.txt`، `CRAFTIX_IMPLEMENTATION_SPEC.md`
> (11 قسماً)، وصورتا `assets/CRAFTIX_UI_Visual_Reference.png` و
> `assets/CRAFTIX_App_Icon_Reference.png`.

## 1. تقرير الأساس قبل أي تعديل (commit `1c23047`، إصدار 0.7.7 / versionCode 29)

| الفحص | النتيجة |
|---|---|
| `git status` | نظيف، الفرع `arena/92db30ff-horizon` |
| `verify_flutter_migration_audit.py` | 34/34 |
| `verify_flutter_foundation.py` | 18/18 |
| `verify_flutter_native_bridge.py` | 25/25 |
| `verify_action_matrix.py` | 10/10 |
| `python3 -m unittest discover -s tests -p 'test_android*.py'` | 35/35 OK |
| `check_public_tree.py` | passed |
| `android_ui_layout.py` | OK لكل الأحجام/الـ cutouts |
| `android_ui_render.py --check` | Portrait geometry OK لـ 9 صفحات × 3 شاشات (بعد تثبيت Pillow) |
| `rustc --test bridge_rules.rs` | **غير متاح محلياً** (لا `rustc` في sandbox)؛ يعمل في CI (موجود في `flutter-foundation.yml`) |
| `flutter analyze` / `flutter test` | **غير متاح محلياً** (لا Flutter SDK)؛ يعمل في CI |

إصدارات منشورة قبل هذا العمل: `redesign-0.7.6`، `flutter-mixed-0.7.6`،
`horizon-0.7.7` (يحتوي APK لـ Rust-only و APK مختلط، موقّعين بمفتاح التطوير
`CN=Horizon Android Development`).

## 2. تحليل تغيير `applicationId` → `com.craftix.launcher`

### ما يعتمد على معرّف الحزمة فعلياً (فحص مصدري)

| الموضع | الحالي | الأثر عند التغيير | القرار |
|---|---|---|---|
| `packaging/android/build.gradle` | `namespace 'dev.horizon.android'`، `applicationId` من `-PappId` أو `dev.horizon.android` | يتغير معرّف التطبيق و namespace | يُحدَّث إلى `com.craftix.launcher` |
| `flutter/android/app/src/main/AndroidManifest.xml` | `${applicationId}.documents` لسلطة الـ DocumentsProvider | تصبح `com.craftix.launcher.documents` | تلقائي عبر placeholder، لا تغيير يدوي |
| `tools/prepare_mixed_flutter.py` | يرقع `applicationId`/`namespace` إلى `dev.horizon.android` | يجب أن يرقعهما إلى `com.craftix.launcher` | يُحدَّث |
| Kotlin/Java classes | الحزمة `dev.horizon.android` (`MainActivity`, `HorizonBridge`, `HorizonEngine`, `HorizonActivity`, `HorizonDocumentsProvider`) | **لا علاقة لها بـ applicationId**؛ لكنها مرتبطة بأسماء رموز JNI | **تبقى كما هي** |
| رموز JNI/C ABI | `Java_dev_horizon_android_HorizonEngine_*` و`Java_dev_horizon_android_HorizonDocumentsProvider_*` و`Java_dev_horizon_android_HorizonActivity_*` | لو غُيّرت الحزمة البرمجية لانكسر الربط مع `.so` | لا تُغيّر — موثّق أدناه |
| `android.app.lib_name` | `horizon_android` | لا يتأثر | يبقى |
| نشاط `HorizonActivity` | `android:process=":engine"` | اسم العملية يصبح `com.craftix.launcher:engine` | تلقائي، لا كود يعتمد على الاسم الحرفي |
| OAuth / Redirect URI | Device Code Flow في `launcher/src/account.rs` بلا Redirect URI | لا يوجد `google-services.json` ولا App Links ولا deep links ولا Firebase في المستودع | **لا يوجد ما يُحدَّث**؛ الـ callback هو استطلاع `/token` |
| أذونات | INTERNET / VIBRATE / WRITE_EXTERNAL_STORAGE | لا تتأثر | تبقى |

### تحذير الهجرة (مهم، مطابق للمواصفة)

- تغيير `applicationId` يجعل Android يتعامل مع CRAFTIX كتطبيق **مختلف** عن HORIZON:
  يُثبَّت إلى جانبه ولا يُحدِّثه. تغيير مفتاح التوقيع يزيد هذا الأثر.
- **لا توجد ترقية في المكان ولا نقل بيانات تلقائي** بين معرّفي حزمة مختلفين على Android
  (sandbox لكل تطبيق؛ لا يمكن قراءة `filesDir` للتطبيق القديم بدون صلاحيات نظام).
- لذلك: لا أعد بتحديث دون فقدان بيانات؛ أوصي بتثبيت نظيف + نسخ يدوي
  لـ `android-launcher.json` و`accounts.json` ومجلدات المحتوى عبر SAF/الملفات.
  تُوثَّق هذه القيود في Release Notes بدل ادعاء ترقية سلسة.
- الأسماء البرمجية (`dev.horizon.android.*`) تبقى داخلية لحماية 49 إجراءً ورموز JNI؛
  العلامة التجارية المرئية للمستخدم تصبح CRAFTIX بالكامل.

## 3. تحليل التوقيع (السياسة الدائمة)

- الأسرار الموجودة في GitHub Actions (من لقطة المالك، ولم تُقرأ قيمها):
  `CRAFTIX_KEYSTORE_BASE64`, `CRAFTIX_STORE_PASSWORD`, `CRAFTIX_KEY_ALIAS`,
  `CRAFTIX_KEY_PASSWORD`.
- `gh secret list` يُعيد 403 لهذا الـ integration token → لا يمكن سرد الأسماء برمجياً،
  لكن الـ workflows تستخدمها مباشرة عبر `${{ secrets.* }}`.
- الوضع الحالي: `tools/android_build.py` يولّد `debug.keystore`
  (`CN=Horizon Android Development`) ويوقّع به **حتى إصدارات release** — يجب إيقافه للـ release.
- الخطة: فكّ Base64 إلى `$RUNNER_TEMP/craftix-release.jks` (خارج المستودع)، تمرير
  المسار/كلمات المرور عبر متغيرات بيئة إلى Gradle فقط، حذف الملف في `always()`،
  عدم طباعة أي قيمة، جعل بناء الـ release يفشل بوضوح عند غياب الأسرار،
  والتحقق من بصمة الشهادة العامة (`SHA-256`) ومقارنتها بالمرجع
  `77:05:07:A6:EC:48:51:98:66:55:08:29:32:CA:C2:81:61:55:08:0F:D4:73:DB:7E:42:E9:A0:BF:FD:22:DD:2F`.
- `.gitignore` يُوسَّع صراحةً لـ `*.jks`, `*.keystore`, `signing.properties`,
  `*.base64`, `craftix-release.*`. لا ملف توقيع خاص داخل المستودع إطلاقاً.

## 4. فحص تأثير المحرك (قبل تعديل Rust)

المطلوب الجديد الوحيد الذي يمس Rust هو **حالة تنزيل حقيقية** للشاشة الرئيسية.
الفحص: `horizon_client::official_install::ready(home)` و
`install_with_progress(home, Fn(InstallProgress) + Sync)` موجودان أصلاً ويستخدمان في
`launcher.rs::action()`. إذن يمكن استدعاؤهما من طبقة الجسر JNI **دون لمس**
`game/` أو `wgpu` أو `winit` أو Vulkan أو منطق Minecraft أو `launcher/src/`.

**شرط مكتشف حقيقي:** `official_install.rs:446` يقرأ `home/engine-data.zip`؛
و`android_main.rs` هو من ينسخه من أصول APK إلى `filesDir` — لكنه يعمل فقط في عملية
`:engine`. لذلك يجب على مضيف Flutter نسخ `assets/engine-data.zip` إلى `filesDir`
عند الإقلاع، وإلا فشل التنزيل عند أول ضغطة Download. هذا إصلاح جذري موثّق،
لا تغيير في المحرك.

**حدّ الإلغاء:** `install_with_progress` لا يملك خطاف إلغاء (mutex واحد
`INSTALLATION` بلا AtomicBool). لذلك لا يُعرض زر Cancel للتنزيل؛ يُصرَّح بذلك
في التقرير بدل إضافة زر شكلي — مطابق للمواصفة «Provide Cancel only if cancellation
is safe and implemented».

## 5. ترتيب التنفيذ المعتمد (من القسم 9 من المواصفة)

1. ✅ فحص + تقرير أساس (هذا الملف).
2. ✅ تحليل الهجرة والتوقيع (هذا الملف).
3. العلامة التجارية: نصوص، label، أيقونة adaptive بكل الكثافات، splash، about/settings.
4. Shell ثابت: safe areas/insets، تنقل، استجابة 360dp، مؤشر متحرك.
5. آلة حالة Home: Download → تقدّم حقيقي → Play، بلا مغادرة الصفحة.
6. شريط تقدّم العدّاء (runner) مدفوع بالتقدّم الحقيقي + دعم تقليل الحركة.
7. اختبارات/إصلاحات مسارات الحسابات (Microsoft وOffline كل على حدة).
8. تدقيق الكتالوج/التثبيت وحالات الخطأ الحقيقية.
9. توقيع GitHub Actions بأسرار CRAFTIX الأربعة.
10. ✅ بناء + اختبارات + تحقق التوقيع/المانيفست/ABI + تقرير نهائي + Release `craftix-0.7.8`.
