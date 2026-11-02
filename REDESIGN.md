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

## 5. الاختبارات التي نُفّذت فعلياً في هذه البيئة

```text
python3 -m unittest discover -s tests -p 'test_android*.py'   → 28 tests OK
python3 tools/android_ui_layout.py                            → all viewports OK
python3 tools/check_public_tree.py                            → passed
python3 tools/check_public_tree.py --history                   → passed
python3 tools/android_ui_preview.py                           → QA render generated
```

**اختبارات لم تُنفّذ هنا (ببيئة مقيّدة):**

- `cargo test`, `cargo clippy -D warnings`, `cargo build` — لا يوجد `cargo`/`rustc`
  في هذه البيئة، والوصول للشبكة محدود بـ GitHub/npm/PyPI فقط.
- `tools/test_android_diagnostics.py` وأخواتها — تستدعي `cargo` لتشغيل اختبارات Rust
  وتفشل بـ `FileNotFoundError: cargo`، وليس بسبب تغييرات الواجهة.
- اختبارات الأجهزة الحقيقية (تشغيل Minecraft، الحسابات، المودات، الحزم، Shaders،
  الرسومات، الأداء) — تحتاج جهاز Android / AVD و`adb`، غير متوفرة هنا.
- لذلك **لم يتم إنتاج APK Release داخل هذه الجلسة**؛ انظر القسم التالي.

> ملاحظة: `python3 -m unittest discover -s tests` (بدون `-p`) يفشل أيضاً على المصدر
> الأصلي بسبب تضارب اسم `test_basic_feature_parity` بين `tests/` و`tools/`؛
> أمر CI الرسمي في upstream يستخدم `-p 'test_android*.py'`، وهو ما استخدمناه.

---

## 6. كيف تُبنى نسخة APK Release (المسار الحقيقي)

APK لا يمكن إنتاجه هنا لأنه يحتاج: Rust 1.95 + Android SDK 35 + NDK + JDK + Gradle،
بالإضافة إلى جداول المحرك المولّدة privately في `.local/horizon-client/26.3`
(تُنتجها `tools/android_prepare.py` من موارد Mojang الرسمية). مفاتيح التوقيع وملفات
اللعبة و`*.apk` مستثناة من Git عن قصد في `.gitignore`.

على جهاز/Runner فيه الأدوات:

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
python3 tools/android_prepare.py                     # once; needs desktop JDK + Mojang downloads
python3 tools/build_release_apk.sh                   # preflight + tests + release APK
```

أو يدوياً:

```bash
python3 -m unittest discover -s tests -p 'test_android*.py'
python3 tools/android_ui_layout.py
python3 tools/check_public_tree.py --history
python3 tools/android_build.py --abi arm64-v8a --clippy --public
python3 tools/android_release.py --abi arm64-v8a
# النتيجة: .local/android/releases/0.7.6/Horizon-Android-0.7.6-arm64-v8a.apk + SHA256SUMS.txt
```

أضفنا كذلك `.github/workflows/android-ui-release.yml` (`workflow_dispatch` فقط)
لتشغيل فحوصات المصدر و`cargo check`/`clippy` للـ ABI على GitHub Actions،
وجمع APK عند توفّر متطلبات البناء. الملف لا يُفعّل تلقائياً حتى لا يكسر
سياسة upstream الخاصة بتعطيل workflows الإضافية.

---

## 7. معاينة QA (ليست التطبيق)

`tools/ui-preview/index.html` تُولَّد من نفس `ui_spec.json` ونفس حسابات التخطيط،
لفحص المسافات والألوان وشريط التنقل بصرياً دون جهاز Android. هي أداة تحقق داخل
`tools/` ولا تُضمَّن في APK، ولا تحتوي أرقام أداء مزيّفة.

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
