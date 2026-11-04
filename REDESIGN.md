# إعادة تصميم واجهة Horizon Android 0.7.6 / UI/UX Redesign

> **الحالة:** إعادة تصميم حقيقية لطبقة العرض داخل المشروع الأصلي، وليست Mockup.
> المحرك والأكواد الأصلية (`Rust / Vulkan / winit / الحسابات / المودات / الحزم / الملفات`)
> لم تُحذف ولم تُستبدل.

---

## 1. نقطة البداية (مهمة)

المستودع الذي فُتحت فيه المهمة (`n83527040-max/Horizon`) كان يحتوي على ملف
`README.md` فقط، ولا يوجد فيه أي كود للتطبيق. لذلك تم أولاً استيراد المصدر
العام المشار إليه في الطلب:

- **المصدر:** `Vasysik/horizon-android`
- **الإصدار:** `v0.7.6` (الوسم `7301f6fdd7f470b11317276e7a11b4dc615c137a`)
- تم اختيار وسم `v0.7.6` تحديداً لأن التصميم المرجعي يطلب Badge بالنسخة `0.7.6`
  و`Minecraft Java 26.3`؛ فرع `android` الحالي في upstream هو `0.7.7-dev`.
- استُبعدت صور التوثيق الكبيرة `docs/media` فقط (34 MB) لتقليل حجم المستودع،
  مع الإبقاء على `docs/media/icon-256.png` المطلوب من أداة التغليف.

الالتزام الأول (`Import upstream Horizon Android v0.7.6 as the redesign baseline`)
يحتوي على المشروع كاملاً كما هو، والالتزام الثاني يحتوي على إعادة التصميم فقط،
حتى يمكن مراجعة الفرق بينهما بوضوح.

---

## 2. ما الذي تغيّر (طبقة العرض فقط)

| الملف | التغيير |
|---|---|
| `crates/horizon-client/src/android/design.rs` | **جديد** — نظام التصميم: الألوان، الأنصاف، التخطيط المتجاوب، الأيقونات، البطاقات، شريط التنقل، الحركات، قراءة الذاكرة الحقيقية. |
| `crates/horizon-client/src/android/ui_spec.json` | **جديد** — المصدر الوحيد للقيم البصرية؛ تقرأه Rust وأداة التحقق ومعاينة QA. |
| `crates/horizon-client/src/android/assets/hero.png` | **جديد** — بانوراما Minecraft مولّدة، 512×256 PNG، ~154 KB، مضمّنة في المكتبة الأصلية عبر `include_bytes!`. |
| `crates/horizon-client/src/android/launcher_pages.rs` | إعادة بناء كاملة لكل شاشات الرسم مع **الإبقاء على كل `Action` وكل فرع في `activate()`**. |
| `crates/horizon-client/src/android/launcher.rs` | حقول عرض فقط: `hero_patch`, `hero_ready`, `page_enter` + تسجيل صورة الـ Hero في الـ atlas. لا تغيير في `action()` أو تشغيل اللعبة. |
| `crates/horizon-client/src/ui_theme.rs` | دوال رسم إضافية (`gradient`, `image`, `clip`, `vertices`) ومساحة محجوزة لصورة الـ Hero داخل منطقة الـ glyph atlas. لا تغيير في سلواب سطح المكتب. |
| `crates/horizon-client/src/android/catalog.rs` | دالة قراءة واحدة `searching()` لعرض Skeleton أثناء البحث. |
| `login.rs`, `chooser.rs`, `documents.rs` | توحيد ألوان الحوارات والقوائم المنبثقة مع الهوية الجديدة (بدون تغيير منطقها). |

### الهوية البصرية (`ui_spec.json`)

`#070B1C` خلفية · `#111A35` بطاقات · `#8B45FF` أساسي · `#315BFF` ثانوي ·
`#20DDF5` cyan · نص أبيض · نص ثانوي `#A9B4D0` · زوايا 14–26dp ·
Glassmorphism خفيف · تدرّجات · Glow ناعم · انتقالات 220ms · ضغط 160ms.

### Home

الترتيب مطبّق exactly كما طُلب: Header (شعار / HORIZON / Minecraft Java • Android /
`v0.7.6` Badge / Settings) → Hero Banner بالصورة الحقيقية → زر Play متدرّج →
بطاقتا Version و Profile → أربع بطاقات (Mods, Packs, Shaders, Installed) →
بطاقة Performance → شريط تنقل سفلي ثابت بخمس وجهات.

- **Play** ← `Action::Play` ← `Launcher::action()` ← نفس مسار `game_launch::start` الأصلي.
- **Version** يعرض `horizon_core::VANILLA_VERSION` (`26.3`). اختيار الإصدار **غير معروض**
  لأن المحرك يثبّت نسخة واحدة؛ لم نضف زراً وهمياً.
- **Profile** ← `Action::Account` ← لوحة الحسابات الأصلية (Microsoft / Offline name).
- **Mods / Packs / Shaders / Installed** ← `Action::Page(...)`. صفحة Shaders لا تزال
  موجودة ويمكن الوصول إليها من بطاقتها ومن الـ segmented control في صفحات المحتوى،
  مع إبقاء الشريط السفلي بخمس عناصر فقط كما طُلب.
- **Performance** يقرأ فعلياً:
  - `VmRSS` / `VmHWM` من `/proc/self/status`,
  - `MemTotal` من `/proc/meminfo`,
  - `RendererMode` و `RenderScale` من إعدادات التطبيق,
  - ولا يعرض FPS لأن المحرك لا يوفّر قراءة FPS حيّة داخل اللانشر (يُقاس داخل اللعبة فقط).
  - لا توجد أرقام تجريبية، ولا يوجد إعداد "RAM Allocation" مزيّف لأن 0.7.6 لا يملكه.

### باقي الصفحات

- **Mods / Packs / Shaders:** بحث، استيراد HTTPS `.hmod`، مصادر Horizon/Modrinth،
  فتح مجلد، استيراد ملف، بطاقات نتائج بالاسم والوصف والتوافق الحقيقي
  (`Project::support()` و`game_versions`)، Install/Retry/Installed، Previous/Next،
  Skeleton عند البحث، ورسائل حالة حقيقية.
- **Installed:** تقسيم حسب `Kind::ALL`، عدّاد حقيقي من `Catalog::local`،
  تفعيل/تعطيل `ToggleLocal`، حذف `DeleteLocal` مع نافذة التأكيد الأصلية، استيراد لكل نوع.
- **Settings:** كل إعدادات 0.7.6 الأصلية (الرسومات، الدقة، المظهر، ADPF، GPU profiling،
  التسجيل، التقارير، الملفات، البروفايل، الإدخال، الاتجاه، السيرفر، الحسابات، المعلومات)
  مُجمّعة في أقسام، مع كل `Action` كما كان. إعدادات اللمس التفصيلية تبقى داخل اللعبة
  `Options > Controls` كما هو موثّق؛ لم نخترع شاشة بديلة.

---

## 3. الربط بالوظائف الحقيقية (لا أزرار شكلية)

تمت إضافة اختبار آلي يقارن المصدر قبل/بعد:

- كل متغيرات `enum Action` من 0.7.6 ما زالت موجودة.
- كل فروع `Some(Action::…)` داخل `activate()` ما زالت موجودة.
- لا تغيير في ملفات المحرك/التغليف/الـ manifests.
- لا توجد قيم وهمية (`4.2 GB`, `8 GB`, `12 installed`, `60 FPS`, …) في كود الواجهة.
- لا توجد اعتماديات `Cargo.toml` جديدة.

الاختبارات: `tests/test_android_ui_redesign.py`.

---

## 4. الاستجابة والأمان البصري

`tools/android_ui_layout.py` هو مرآة Python دقيقة لحسابات `design.rs`، ويُتحقق منه على:

| الشاشة | النتيجة |
|---|---|
| 360 × 800 | OK |
| 393 × 873 | OK |
| 432 × 960 | OK |
| 480 × 768 ( tablet portrait ) | OK |
| 800 × 1280 ( tablet ) | OK |
| 960 × 540 ( landscape / foldable ) | OK |
| Safe-area insets 24/16 و48/24 و0/32 | OK |
| 320 × 568 و360 × 640 | OK |

يمنع التحقق: الخروج الأفقي، تداخل البطاقات، اختباء شريط التنقل، تجاوز المحتوى
لمنطقة الشريط، ومشاكل التمرير. التطبيق يحترم `content_rect` و `viewport_bottom`
الخاصين بـ Android من الكود الأصلي، ويضيف العزل العلوي للـ cutout.

---

## 5. الاختبارات والبناء الفعلي على GitHub Actions

أنتجنا **APK Release حقيقي** من هذا المصدر المحدَّث (وليس من النسخة الأصلية)
داخل `.github/workflows/android-redesign-release.yml` على `ubuntu-latest`:
Rust 1.95.0 + target `aarch64-linux-android`، Android SDK 35 /
build-tools 35.0.0 / NDK، JDK 17 (Gradle/apksigner) وJDK 25 (مُصدِّرات Java)،
Gradle 8.14. الـ workflow يعمل بـ `push` على الفرع لأن رمز الـ App الخاص بالجلسة
لا يملك صلاحية `workflow_dispatch`.

**وظيفة `verify` (بوابة قبل البناء):**

```text
git diff --check                                            → OK
python3 tools/check_public_tree.py (+ --history)              → OK
python3 -m unittest discover -s tests -p 'test_android*.py' → 28 tests OK
python3 tools/android_ui_layout.py                          → كل المقاسات + safe areas OK
python3 tools/android_ui_preview.py                         → معاينة QA مولّدة
cmp android-source.yml packaging/android/source-checks.yml  → OK
cargo check  -p horizon-android --target aarch64-linux-android --public → OK
cargo clippy -p horizon-android --target aarch64-linux-android --no-deps -D warnings → OK
```

**وظيفة `release`:**

```text
python3 tools/android_prepare.py --java-home <JDK 25>
  → 5214 ملفاً موثّقاً من Mojang + مُصدِّرات Java → جداول 26.3 المولّدة
python3 tools/build_release_apk.sh  → cargo build --profile release (ThinLTO)
                                     + Gradle assembleRelease + apksigner
tools/verify_android_apk.py         → passed (كل علامات التصميم داخل الـ .so)
tools/check_android_release.py      → passed, debuggable=false, لا أصول لعبة خاصة
aapt2 dump badging                 → dev.horizon.android / 0.7.6 / arm64-v8a
apksigner verify                   → Verifies (APK Signature Scheme v2), signer 1
gh release download + cmp          → الأصل المنشور مطابق بايت-ببايت للـ APK المبني
```

`verify_android_apk.py` يقرأ **داخل ملف APK**: مكتبة
`lib/arm64-v8a/libhorizon_android.so` تحتوي نص `ui_spec.json` كاملاً (الألوان
`#070B1C / #8B45FF / #315BFF / #20DDF5`، `nav_items`)، وعناوين الواجهة الجديدة،
و**بايتات صورة الـ Hero نفسها**؛ كما أن بصمة المكتبة تختلف عن مكتبة الإصدار
الرسمي `v0.7.6`، أي أن APK بُني من السورس المحدَّث لا من النسخة القديمة.

**النتيجة المنشورة:**

- الإصدار: <https://github.com/n83527040-max/Horizon/releases/tag/redesign-0.7.6>
- التحميل المباشر: <https://github.com/n83527040-max/Horizon/releases/download/redesign-0.7.6/Horizon-Android-Redesigned-0.7.6-arm64-v8a.apk>
- الحجم 35,693,532 بايت · SHA‑256 `fff20f808fb098e374e3185a0e786bd8b155d2bcb22ebef829d60f0bae16b5a8`
  · `arm64-v8a` · Android 9 / API 28+ · غير قابل للتصحيح · Portrait
- `APK_VERIFICATION.json` يثبت `binary_manifest_portrait = true`
  (سطر `aapt2 dump xmltree`: `android:screenOrientation(0x0101001e)=1`) ووجود
  سياستي `orientation: portrait` / `width_policy: full-width` داخل الـ `.so`
- مرفقات التحقق: `APK_VERIFICATION.json`, `manifest-tree.txt`, `release-audit.json`,
  `badging.txt`, `SHA256SUMS.txt`, `BUILD_INFO.txt`, `LICENSE.txt`, `THIRD_PARTY_NOTICES.md`
- صور Portrait لكل الصفحات: `tools/ui-preview/index.html` / `portrait-overview.png`
- نسخة احتياطية كـ Actions artifact بنفس الاسم.

**ما لم يُختبر فعلياً:** تشغيل Minecraft على جهاز/AVD، تسجيل الدخول بحساب
Microsoft، تثبيت مود/حزمة/Shader على الجهاز، وقياس FPS/أداء GPU الحقيقي.
هذه تحتاج جهازاً فعلياً و`adb`؛ لم نُضف نتائج Device وهمية. الفحوصات أعلاه
مصدرية/ترجمة/تغليف/توقيع فقط. عند التحديث فوق تثبيت قديم يلزم نفس شهادة
التوقيع، لأن الـ keystore يُولَّد داخل كل Runner.

---

## 6. إعادة إنتاج البناء

```bash
rustup target add aarch64-linux-android
python3 tools/android_prepare.py --java-home <JDK 25>   # موارد Mojang الرسمية، خارج Git
export JAVA_HOME=<JDK 17>; export HORIZON_CLIPPY=1; export HORIZON_JOBS=2
bash tools/build_release_apk.sh
# .local/android/releases/0.7.6/*.apk + SHA256SUMS.txt
```

`tools/ci_android_setup.sh` يثبّت إصدارات SDK/NDK/Gradle على Runner،
`tools/verify_android_apk.py` يتحقق من محتوى APK، و`tools/ci-release-notes.md`
هو نص الإصدار. مفاتيح التوقيع وملفات اللعبة و`*.apk` تبقى خارج Git بحسب
`.gitignore` و`tools/check_public_tree.py`.

---

## 7. معاينة QA (ليست التطبيق)

`tools/ui-preview/index.html` معرض صور لكل الصفحات على المقاسات
`360×800 / 393×873 / 432×960`. تُولَّد الصور عبر `tools/android_ui_render.py`
من نفس `ui_spec.json` ونفس حسابات `tools/android_ui_layout.py` (مرآة `design.rs`)
ومع خطوط المشروع وصورة الـ Hero المضمّنة. هي **رندر تحقق للتخطيط** وليست لقطات
شاشة من جهاز، ولا تُضمَّن في APK، ولا تحتوي أرقام أداء مزيّفة (القيم الحقيقية
تُقرأ وقت التشغيل من `/proc`). الملف `tools/ui-preview/layout-preview.html`
هو فاحص الشاشة الواحدة القديم، و`portrait-overview.png` يجمع الصفحات التسع.

---

## 8. القيود المعروفة / ملاحظات أمانة

- إصدار المصدر `0.7.6` لا يحتوي على: اختيار نسخة Minecraft، إعداد تخصيص RAM،
  FPS حيّ في اللانشر، تجميع `.hnmod` على Android، أو مودات Java Fabric/Forge.
  لم نضف أياً منها كواجهة شكلية.
- صورة الـ Hero مولّدة بالذكاء الاصطناعي (ليست أصول Mojang)، ومضمّنة في
  `libhorizon_android.so` عبر مساحة محجوزة في atlas النصوص: لا تنزيل، لا نسيج GPU
  إضافي، لا تغيير في آلية تحميل Resource Packs.
- الحركات كلها حسابية خفيفة (alpha / انزياح 10dp / Skeleton) ولا تُنشئ خيوطاً
  ولا تلمس حلقة عرض اللعبة； المحرك ي停止 الرسم أثناء اللعب كما كان.
- أي تعديل Backend تم توثيقه أعلاه وهو **إضافي فقط** (getter واحد + دوال atlas +
  حقول عرض). لم تُغيّر آلية التشغيل، الحسابات، الملفات، المودات، الحزم، Shaders،
  Vulkan، أو الإعدادات المحفوظة.

---

## 9. إصلاح الاتجاه العمودي واستغلال العرض الكامل (Portrait / Edge-to-edge)

**سبب المشكلة:** كان `AndroidManifest.xml` يفرض `sensorLandscape`، وكان
`Orientation::default()` هو `Landscape`، لذلك كان `Launcher::open()` يطلب وضعاً
أفقياً. وفوق ذلك كان `design.rs` يحصر كل البطاقات في عمود عرضه
`max_content_width = 560` في وسط الشاشة؛ على شاشة أفقية (~960dp) كان ذلك يترك
حوالي 200dp فراغاً على كل جانب — وهو بالضبط «واجهة Desktop مصغّرة» التي ظهرت.

**ما الذي أُصلح (طبقة العرض + إعدادات النافذة فقط):**

| الملف | الإصلاح |
| --- | --- |
| `packaging/android/AndroidManifest.xml` | `android:screenOrientation="portrait"` بدل `sensorLandscape` |
| `crates/horizon-client/src/android/catalog.rs` | الافتراضي `Portrait` + `launcher_mode()` يمنع `Auto` من الدوران الأفقي؛ كل الخيارات الخمسة باقية |
| `crates/horizon-client/src/android/launcher.rs` | عند إنشاء/استئناف النافذة يُطلب الوضع العمودي؛ خيار المستخدم الصريح في Settings يبقى فعّالاً |
| `crates/horizon-client/src/android/ui_spec.json` | `orientation: portrait`, `width_policy: full-width`, `max_content_width: 0` |
| `crates/horizon-client/src/android/design.rs` | `Layout::with_insets`: عرض كامل + حشوة جانبية متجاوبة + `left/right_inset` للـ cutouts؛ `sheet_rect` / `modal_rect`؛ شريط تنقل من الحافة للحافة مع تعبئة منطقة الإيماءات |
| `crates/horizon-client/src/android/launcher_pages.rs` | تمرير `left_inset` من `content_rect()` إلى التخطيط، وتحديث `chooser.viewport(w,h)` |
| `login.rs`, `chooser.rs`, `documents.rs` | النوافذ المنبثقة صارت ألواحاً عمودية بعرض كامل بدل صناديق `720/440dp` في الوسط |
| `tools/android_ui_layout.py`, `tests/test_android_ui_redesign.py` | فحوص لكل الصفحات التسع: لا فراغات جانبية كبيرة، لا Overflow، الشريط بعرض كامل وفوق safe area |
| `tools/android_ui_render.py` | صور Portrait جديدة لكل الصفحات على المقاسات الثلاثة |

**النتيجة المقاسة (393×873 مثالاً):** `content_x = 16dp`،
`content_w = 361dp` من أصل 393dp (أي 92% من عرض الشاشة؛ الباقي حشوة 16dp على
كل جانب). على 960dp يصبح العرض 892dp بدل 560dp في الوسط. شريط التنقل
= 393dp كاملاً، و`page_bottom <= nav_y` فلا يغطي المحتوى.

`tools/verify_android_apk.py` يتحقق الآن من وجود سياستي
`"orientation": "portrait"` و`"width_policy": "full-width"` **داخل**
`libhorizon_android.so` الموجود في APK، ومن قفل الـ Manifest والافتراضي العمودي؛
و`manifest-tree.txt` المرفق بالإصدار هو تفريغ `aapt2` للـ Manifest الثنائي.

> ملاحظة أمانة: إعداد «Aspect ratio» في الأصل يبقى كما هو (الافتراضي `Native`
> = كامل المساحة). إذا اختار المستخدم نسبة ثابتة يدوياً فقد يظهر letterboxing بحسب
> تصميم الإعداد الأصلي؛ لم نحذفه ولم نغيّر سلوكه.
