# HORIZON 0.7.7 — تشخيص جذري قبل الإصلاح

> مطلوب صراحةً: «ابدأ أولاً بتشخيص كل مشكلة من Logs وSource Code وAPI Responses،
> ثم حدد السبب الحقيقي، ثم أصلحه واختبره». هذا الملف هو مرحلة التشخيص.
> **لا يُعلن هنا اكتمال أي ميزة.**

## 0. مصدر الأدلة

| الدليل | ما أعطانا |
|---|---|
| 11 لقطة شاشة من جهاز حقيقي (`Screenshot_20261103_1446xx.jpg`) | التطبيق **يعمل فعلاً** على هاتف (هذا يصحّح تصريحي السابق «لم يُختبر على جهاز»)، ويكشف أخطاء فعلية |
| `flutter/lib/features/catalog/catalog_page.dart` | لا يُطلق بحثاً عند فتح الصفحة؛ لا تصنيف للأخطاء؛ بطاقات بلا صور/مطوّر/إصدارات |
| `crates/horizon-client/src/android/flutter_bridge.rs::snapshot()` | يقرأ `e.status` فقط، **ولا يقرأ `e.catalog.status` أبداً** → كل أخطاء الشبكة/API تُخفى |
| `crates/horizon-client/src/android/catalog.rs::Catalog::new()` | `page = Page::Play` افتراضياً، و`browse()` يرجع مبكراً إن لم تكن الصفحة كتالوجاً |
| `launcher/src/browser/modrinth.rs::get()` | رسائل خطأ حقيقية: `No connection to Modrinth`, `Modrinth answered 429`, `wasn't readable` |
| استجابة Modrinth الحقيقية (مُجلَبة أثناء التشخيص) | `GET /v2/search?query=sodium&facets=[["project_type:mod"]]` → `200`, `total_hits=108`, حقل `versions` يحوي `26.3`, `icon_url`, `downloads`, `author`, `categories` (fabric/neoforge/quilt) |
| فهرس Horizon الحقيقي | `https://horizon-rust.github.io/horizon-mods/index.json` → `200`, مشاريع `game:"26.3"` مع `sha256` وتوقيع |
| `launcher/src/account.rs` | `play_as_with(Player::Offline(..))` يرفض اللعب دون حساب Microsoft يملك اللعبة، **إلا** عبر `offline_without_owner=true` الموجود أصلاً |
| `launcher/src/account.rs::Endpoints::live()` | Microsoft OAuth `login.microsoftonline.com/consumers/oauth2/v2.0`، Xbox `user.auth.xboxlive.com`، XSTS `xsts.auth.xboxlive.com`، Minecraft `api.minecraftservices.com`؛ Device Code Flow **بلا Redirect URI** |
| `launcher/src/account.rs::check()` | تحديث `refresh_token` قبل ساعة من انتهاء `minecraft_expires_at`، ثم `owns_game` + `profile` |

## 1. الأخطاء المرصودة على الجهاز (من الصور)

1. **الحسابات:** زر «اسم المستخدم دون اتصال» لا يفتح أي حقل إدخال؛ يظهر Toast «Offline name saved» رغم عدم إدخال اسم. الحالة تبقى `Offline / Sign in`. لا توجد قائمة حسابات ولا تبديل ولا حذف ولا افتراضي.
2. **الكتالوج:** صفحات المودات/الحزم/الشيدرات كلها «لا توجد نتائج بعد» مع SnackBar متكرر «Catalog source saved». لا صور، لا مطوّر، لا إصدارات، لا Loaders.
3. **نصوص مختلطة/مقلوبة:** `items • enable, import or remove content 0`، `MB / 3.7 GB 180`، `MB 232`، `Resource Packs`، `Lighting, shadows and visual effects`، `...Offline / Si` (مقطوع)، وترجمة إنجليزية داخل واجهة عربية.
4. **SnackBar يغطي شريط التنقل السفلي** ويقصّ أسماء التبويبات.
5. **أسهم الترقيم مزدوجة** (`‹ ‹`, `› ›`) لأن النص `'‹'` يُعرض مع أيقونة chevron معاً.
6. **لا توجد حالات تحميل/تقدّم/نجاح/خطأ احترافية** ولا أنيميشن انتقال.
7. بطاقة الأداء تعرض أرقاماً حقيقية لكن باتجاه نص مكسور.

## 2. الأسباب الجذرية (Root Causes)

| # | المشكلة | السبب الحقيقي | الطبقة |
|---|---|---|---|
| R1 | لا نتائج أبداً | `Catalog.page = Page::Play` ولا يُستدعى `browse()` عند فتح صفحة؛ `browse()` يتجاهل الطلب إن لم تكن `page.is_catalog()` | JNI + Dart |
| R2 | لا نتائج حتى بعد اختيار المصدر | `dispatch("ModSource")` يحفظ الإعداد ويستدعي `browse()` لكن `page` ما زال `Play` → لا بحث | JNI |
| R3 | «No results» بدل سبب الخطأ | `snapshot()` يستخدم `e.status` ولا يمرّر `e.catalog.status`؛ لا `errorKind`؛ لا زر إعادة محاولة مصنّف | JNI + Dart |
| R4 | نجاح وهمي للاسم الفارغ | `dispatch("Field", name:'name')` بلا `value` يحفظ `""` ويعيد `ok=true` «Offline name saved» | JNI + Dart |
| R5 |Offline لا يصل للتشغيل | لا يوجد إدخال اسم أصلاً؛ `config.name` فارغ؛ `Accounts::play_as_with` يحتاج `offline_without_owner=true` | JNI + Dart |
| R6 | لا تعدد حسابات/تبديل/حذف | الواجهة تعرض زرّين فقط؛ لا استدعاء لـ `Accounts::offline`, `play_as_with`, `remove_offline`, `remove` | Dart + JNI |
| R7 | أخطاء Microsoft صامتة | `SignInState::Done(Err)` يُخزَّن في `login_message` لكن لا تصنيف مرحلة (device_code/xbox/xsts/minecraft/profile) ولا عرض بارز ولا إعادة محاولة | JNI + Dart |
| R8 | لا Refresh ظاهر | `account::fresh`/`check` موجودان في المحرك لكن لا يوجد إجراء لعرض/تجديد الحالة من Flutter | JNI |
| R9 | نصوص RTL مكسورة | قيم لاتينية/أرقام تُحقن داخل جمل عربية دون عزل اتجاه؛ و24 نصاً ثابتاً بالإنجليزية داخل Dart | Dart |
| R10 | SnackBar فوق الـ nav | `SnackBar` افتراضي `fixed` بلا `behavior: floating` ولا هامش سفلي | Dart |
| R11 | أسهم مزدوجة | `_SmallAction('‹', Icons.chevron_left_rounded, …)` | Dart |
| R12 | بطاقات الكتالوج فقيرة | `snapshot()` يرسل `version` الأول فقط؛ لا `author`, `downloads`, `versions[]`, `loaders[]`, `installable`, `installed` | JNI + Dart |
| R13 | لا فلاتر نسخة/لودر | `Query.filters` لا يُضبط من الجسر (`game_version`, `loader`) | JNI + `catalog.rs` |
| R14 | لا تقدّم تنزيل | `Catalog.progress` خاص ولا يُعرض | `catalog.rs` + JNI |
| R15 | تثبيت غير متوافق | `dispatch("Install")` يستدعي `Catalog::install` الذي يرفض بصمت (`return`) عند عدم التوافق بدل رسالة | JNI |

## 3. ما تبيّنه استجابات الـ API الحقيقية

- Modrinth **لا يعاني مشكلة شبكة**: `200`، `total_hits=108` لـ `sodium`، و`versions` تتضمن `26.3`.
- فهرس Horizon **حيّ** ويحتوي `26.3` مع `sha256` وتوقيع minisign.
- إذن «لا توجد نتائج» ليس عذر شبكة؛ هو **R1/R2/R3**: لم يُطلق البحث، وأي خطأ لاحق يُخفى.
- `modrinth::get()` يرسل User-Agent صحيحاً ويفرق 429/HTTP/transport/JSON — هذه الرسائل يجب أن تصل إلى الواجهة.
- Device Code Flow لا يحتاج Redirect URI: `wait_for_sign_in` يستطلع `/token`؛ Callback هو استطلاع التوكن لا `http://localhost`.
  لذلك «إصلاح Redirect URI» = توثيق المسار الصحيح + عدم إدخال redirect وهمي، لا تغيير بروتوكول Microsoft.

## 4. تحليل التأثير قبل تعديل Rust (شرطك)

**ممنوع لمسه:** `crates/horizon-client/src/game/**`, `horizon-client-render/**`, `horizon-core/**`,
`horizon-world/**`, `horizon-protocol/**`, `horizon-mods/**`, `horizon-shaders/**`,
`launcher/src/**`, `wgpu`, `winit`, Vulkan, منطق Minecraft، مخطط `android-launcher.json`,
`packaging/android/java/**`, `Cargo.toml`/`Cargo.lock`.

**المسموح (طبقة استضافة فقط، إضافي):**
1. `crates/horizon-client/src/android/bridge_rules.rs` (**جديد**): قواعد نقية قابلة للاختبار على الحاسب (`rustc --test`) بلا أي تبعية.
2. `crates/horizon-client/src/android/flutter_bridge.rs`: تمرير `catalog.status`، تصنيف الأخطاء، عمليات `Accounts` الحقيقية، ضبط الصفحة/الفلاتر.
3. `crates/horizon-client/src/android/catalog.rs`: **getters إضافية فقط** (`set_filters`, `progress`, `page`) — لا تغيير في منطق البحث/التنزيل.
4. `flutter/**` و`tools/**` و`.github/workflows/**` وملفات الإصدار.

> كل عمليات الحسابات تستخدم `Accounts::play_as_with(..., offline_without_owner=true)`
> الموجود أصلاً، و`account::fresh` الموجود أصلاً. **لا إعادة كتابة لنظام المصادقة.**

## 5. خطة المراحل (تُنفَّذ بالترتيب، وتُختبر كل مرحلة)

| المرحلة | المحتوى | بوابة الاختبار |
|---|---|---|
| A | `bridge_rules.rs` + اختبارات Rust نقية | `rustc --test` على الحاسب في CI |
| B | JNI: تمرير الحالة/الأخطاء + الحسابات + الكتالوج | `cargo check` + `clippy -D warnings` لهدف Android |
| C | Dart: نماذج/جسر/مزوّدات/حالات الخطأ | `flutter analyze` + `flutter test` |
| D | UI Premium + أنيميشن + RTL/Bidi + تنقل | اختبارات Widget وعدم Overflow على 7 أحجام |
| E | تكامل API حقيقي + فحص رموز JNI في `.so` | سكربت CI يهبط Modrinth/الفهرس + `nm` على المكتبة |
| F | الإصدار 0.7.7 (Version Code 29) | بناء APK + توقيع + مانيفست + ABI + SHA256SUMS |

## 6. ما لا يمكن إثباته في هذه البيئة (يُصرَّح به دائماً)

- تسجيل دخول Microsoft **بمستخدم حقيقي** مقابل خدمات Microsoft (يحتاج حساباً ورموزاً حقيقية).
- Xbox Live / XSTS / Minecraft Services باستجابات حقيقية بمستخدم فعلي.
- `adb install` كتحديث فوق 0.7.6 وعدم فقدان البيانات على جهاز.
- Vulkan/تشغيل Minecraft فعلياً بعد الضغط على Play، FPS/GPU، تسريب الذاكرة، الدوران الفيزيائي.
- Golden pixel comparison: يحتاج خط أساس مولَّداً بـ Flutter SDK على نفس إصدار المحرك؛ سيُولَّد في CI ويُرفع كـ artifact لا يُدعى اختبار مقارنة.
