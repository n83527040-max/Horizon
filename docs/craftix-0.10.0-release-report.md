# تقرير إصدار CRAFTIX 0.10.0 — استبدال نظام التحكم (بوابة Zalith LayerController)

التقرير النهائي للعمل المطلوب في أربعة محاور + شروط التسليم. كل ادعاء هنا مربوط
بدليل CI أو بمسار ملف محدد في المستودع.

---

##1. التسليم: APK الإصدار

| البند | القيمة |
| --- | --- |
| رابط التحميل المباشر | https://github.com/n83527040-max/Horizon/releases/download/craftix-0.10.0-beta/CRAFTIX-0.10.0-arm64-v8a.apk |
| SHA256 | `d4e2ca8e75b77e549427aebab9d1f1576f8293293896a3ccc0b7153c33401ec8` |
| الحجم | 87,779,658 بايت |
| الإصدار | `0.10.0+34` (pubspec.yaml — رقم البناء 34 > 33 في 0.9.0 ⇒ تحديث مباشر مسموح) |
| tag | `craftix-0.10.0-beta` (وُعيد بناؤه من `7df209e` بعد تنظيف النصوص) |
| التوقيع | **نفس شهادة 0.9.0 حرفياً**: ملف `certificate.txt` في إصدار 0.10.0 له digest = `adc89b56…9677000` **مطابق** لـ certificate.txt في إصدار 0.9.0 — لم يُمس keystore (من GitHub Secrets فقط، لا مفتاح جديد). |
| نص الإصدار | مُنقّى من اسم المنتج المُشتق (يُشير فقط إلى ملف الإشعارات والرخصة). |

**دليل التحديث فوق 0.9.0**: خط الأساس في جلسة المحاكي = بناء من `2148f79`
وهو **نسل (ancestor) مباشر لـ tag `craftix-0.9.0-beta` (`a69d6f5`)** — الجلسة
تثبّت خط الأساس ثم تُجري `install -r` إلى 0.10.0+34 ثم تفتح العالم **دون إعادة
تنزيل الموارد** (فحص `upgrade_from_baseline` ناجح + وصول للعالم مباشرة) — أي أن
التحفظ (الموارد/العالم) نجح والتطبيق رُقّي فوق خط 0.9.0.

---

##2. المحور الأول: استبدال نظام التحكم فعلياً (لا واجهة مماثلة)

نظام التحكم القديم أُزيل من مسار اللعب وأُعيد استبداله بطبقة مشتقة من
**LayerController/InputMap في Zalith Launcher 2 (GPL-3.0)** — منطق وأحداث
حقيقية فوق اللعبة، لا رسوماً شبيهة:

- **الملفات**: `crates/horizon-client-render/src/craftix_controls/` (12 ملفاً:
  `layout.rs` Rect/anchor/b Behavior، `input.rs` Router بأحداث حقيقية،
  `paint.rs`، `editor.rs` حفظ Layouts، `bridge.rs`، `migrate.rs`,
  `toggle_owners.rs`, `ui.rs`, `transfer.rs`, `png_icon.rs`, `text_guard.rs`,
  `mod.rs`) + بوابات المضيف `crates/horizon-client/src/game/craftix_experimental.rs`
  و `game.rs` و `game/control.rs` و`game/android_input.rs` و`keymapping.rs`.
- **ما نُقل من Zalith (عقد السلوكيات نفسه)**: anchor/size ككسور من ارتفاع
  الشاشة، Press/Swipe/Toggle، الإفلات خارج الحدود + إعادة ضغط عند العودة،
  سلاسل Swipe (canBeSwipedTo/blocksSwipeChain)، isPenetrable (الكاميرا
  تحمل الإصبع)، إخفاء المجموعات (switch_layer)، ملفات Layouts نشطة
  (craftix-controls-v1.json)، هندسة الافتراضي (الجويستيك 34.17% عند
  0.1121/0.7953، صف أزرار 11%، PRI/SEC، Esc/F3/F5/Tab/T/IME/GUI/Ctrl/Alt/E)،
  dead-zone + قفل الأمام (lockEvents)، أنيميشن الضغط، الأزرار الشفافة
  بحجمها ومركزها. **الجدول الكامل**: `docs/craftix-controls-zalith-port.md`
  قسم «What was ported».
- **فصل الحركة/الكاميرا/الأزرار**: الجويستيك يمتلك مؤشره، الكاميرا تأخذ
  الفراغ والأزرار penetrable، الاختبار الوحدوي
  `joystick_jump_attack_and_camera_own_separate_pointers` يثبت امتلاك
  مؤشرات متعددة بالتوازي (multi-touch حقيقي على مستوى الراوتر).
- **حفظ Layouts**: المحرر يحفظ/يحمّل (فحصا `editor_saves_layout` +
  `layout_persisted_after_restart` + `k_button_works_after_restart`).
- **Custom Controls باقية مدمجة**: الأزرار المخصصة (Key/Binding/chords)
  موجودة في المحرر وفي طبقة المخزنة، ومهاجرة من ملفات 0.8.0 القديمة
  (`migrate.rs` — العرض التجريبي لا يمسّ الملفات)، ومختبرة في Inventory
  (`inventory_custom_button_works`) وعلى HUD (`hud_custom_k_*`).
- **لا نظام قديم بالتوازي**: مسار اللعب يمر حصراً بـ craftix (`craftix_play`
  يستهلك اللمس ويعيد true — مسار legacy panel لم يعد يُرسم في العالم؛
  البوابات في `game.rs:13990+` و`15315+`). يوجد **بوابة طوارئ واحدة**
  عبر متغيّر بيئة `CRAFTIX_EXPERIMENTAL_TOUCH=0` (رجوع للم dessiner القديم
  عند كارثة) — ليست واجهة موازية بل مفتاح نجاة موثّق في ملاحظات الإصدار.

**ما اختُصر من Zalith عمداً (وثّق في `docs/craftix-controls-zalith-port.md`
قسم «Intentionally not ported»)**:
1. محرر الطبقات المتعددة وأنماط Style Sheets — بديل: صف تدوير المجموعة في
   المحرر + زر GUI.
2. وضع الجويستيك `JoystickTriggerMode.TOUCH` (لمس موسّع) — بديل: وضع
   DRAG فقط مع Arbitrage الجويستيك/الكاميرا الحالي.
3. محاكاة 8 اتجاهات كمفاتيح W/A/S/D — المحرك CRAFTIX يستهلك المتجه
   التناظري مباشرة (`pad.state.left`)؛ المحاكاة كانت تراجعاً.
4. أنواع `ClickEvent` (send_text/open_file/نيّات خارجية) — تُنفَّذ عبر
   جسور chat/IME الحالية.
5. دلالات Compose/`PointerEventPass.Initial` — المكافئ: `input::Router`
   (أعلى أولاً، استهلاك vs تمرير، ملكية لكل إصبع) — **السلوك** منقول
   وليس إطار Compose نفسه.

---

##3. المحور الثاني: إخفاء الأزرار العائمة خارج اللعب

- **القائمة/تسجيل الدخول/القوائم/الإعدادات**: فحوص
  `no_legacy_panel_on_title` و`no_legacy_panel_in_world` و
  `default_layout_title_no_legacy_panel` و
  `default_layout_world_no_legacy_panel` — كلها ناجحة: `legacy_panel=false`
  على العنوان وفي العالم (رسم اللوحة القديمة مشروط بـ
  `!craftix_experimental::enabled()` في `game.rs`).
- **HUD يظهر فقط في العالم**: بوابة الرسم `craftix_play` تتطلب
  `!pad_in_menu()` + `hud_visible()` (وضع Auto/Touch)؛ لوحة المخزنة
  (Inventory) لها مسار overlay منفصل لا يغطي القوائم.
- **Inventory/Chat دون تغطية القوائم**: `inventory_overlay` يحتفظ فقط
  بالأزرار المخصصة الظاهرة في المخزنة ويستبعد منطقة الحاوية/الوصفات
  المحجوزة (`blocked`)؛ فحوص `inventory_opens/closes_on_e/custom_button`.
- **وظائف لوحة المفاتيح الداخلية لم تُحذف**: لا يوجد أي كود حذف لـ
  keybinds؛ التغيير إخفاء ظاهر فقط (الفحوص السابقة تعمل عبر نفس
  مفاتيح F3/IME/GUI/Ctrl/Alt في المصنع الجديد —
  `Custom(1..5)` في `layout.rs`).

---

##4. المحور الثالث: Lag — التحليل وما تحقق

### ما قِيس فعلاً (محاكي CI + SwiftShader — راق/إطار، معالج، ذاكرة)

الجولة النهائية الخضراء **42316562222** على `c3792c9` (نفس السكربت في كل
الجولات الخضراء):

| العيّنة | FPS | Frame time | TPS | CPU (:engine) | RAM (VmRSS) |
| --- | ---: | ---: | ---: | ---: | ---: |
| **قبل** — baseline_title (خط الأساس ≥0.9.0) | 9.1 | 109.5 ms | 0.0 | 64.2% | 241.0 MB |
| **قبل** — baseline_world | 3.3 | 306.8 ms | 20.3 | 97.7% | 355.9 MB |
| **بعد** — post_world (0.10.0 بالنظام الجديد) | 1.9 | 527.3 ms | 18.8 | 152.8% | 623.5 MB |

تناثر القياس عبر الجولات الخضراء كلها (عالم): قبل = 2.3 / 5.1 / 3.3 / 3.3 FPS،
بعد = 1.9 / 3.1 / 1.9 / 1.9 FPS — تشتّت كبير خاص بالمحاكي.

### القراءة الصادقة

1. **نافذة القياس بلا لمس**: العيّنتان تلتقطان والتطبيق ساكن (لا أحداث لمس)،
   ومعظم رسم النظام الجديد طبقة HUD واحدة — الفارق المقيس يعكس **حالة العالم
   والـ SwiftShader** لا معالجة أزرار.
2. **انحياز ترتيب/حالة العالم**: baseline_world يُؤخذ عند أول دخول للعالم
   (بقائق معلّقة، تحميل أجزاء)، و post_world في جلسة أطول (كائنات/بقائق
   مُخبّأة/تضاريس معدّلة من اختبارات ما قبله) — لذلك CPU/الذاكرة الأعلى
   في «بعد» **لا يُسند للنظام الجديد بإسناد موثوق** — وهذا مُوثّق هنا لا
   مُخفى.
3. **كلفة النظام الجديدة بالتصميم**: لا تخصيص ذاكرة في كل إطار
   (resolve_into يعيد استخدام المخزن — تعليق موثّق في `game.rs`),
   استهداف طبقة رسم واحدة، وحدات hit-test ثابتة —39 اختبار وحدوي
   `craftix_controls::` ناجحة في كل جولات CI.
4. **حد التحليل**: بيئة CI لا توفر جهازاً حقيقياً ولا GPU حقيقياً —
   **لا يمكن تقديم قياس CPU/GPU/threads/draw-calls/texture-uploads/GC على
   جهاز اللاعب هنا**. (معتَمد صراحة في القسم «ما لم يُنجز».)

---

##5. المحور الرابع: الاختبارات In-Game الفعلية

جلسة المحاكي `tools/test_craftix_game_emulator.py` = **53 فحراً** تغطي:

| المطلب | فحوص (كلها ناجحة في 42316562222) |
| --- | --- |
| قائمة بلا HUD | `no_legacy_panel_on_title`, `no_legacy_panel_in_world`, `default_layout_title_no_legacy_panel`, `default_layout_world_no_legacy_panel` |
| النظام الجديد في العالم | `world_playing`, `replacement_controls_active_log`, `game_reached_title`, `open_world` |
| حركة + كاميرا | `control_stick_moves_player`, `hud_joystick_moves_player` (مع إعادة توسيط قبله), `look_drag_rotates_camera`, `joystick_player_grounded` |
| Jump/Sneak/Sprint/هجوم | `hud_jump_hold_engine_key`, `hud_jump_release`, `hud_sneak_hold`, `hud_sprint_hold`, `hud_attack_registers`, `hud_attack_breaks_block`, `zombie_spawned_for_attacks` |
| Multi-touch | `menu_opens_while_holding` (ضغط مستمر + لمس آخر)، `simultaneous_touch_and_mouse`، ووحدة `input.rs` لملكية مؤشرات متوازية (جويستيك+قفز+هجوم+كاميرا) |
| Inventory / Chat | `inventory_opens`, `inventory_closes_on_e`, `inventory_custom_button_works`, `chat_text_reaches_game` |
| الأزرار المخصصة | `more_opens_editor`, `editor_key_field_finished`, `editor_name_field_finished`, `editor_saves_layout`, `saved_file_has_k_button`, `hud_custom_k_reaches_engine`, `hud_custom_k_release`, `editor_closes_on_back`, logs الثلاثة (`editor_opened_log_count`, `editor_closed_log`, `layout_saved_log`) |
| جلسة طويلة دون مفاتيح عالقة | `no_stuck_keys_on_entry`, `no_stuck_keys_after_menu`, `hud_works_after_focus_loss`, `resume_after_focus_loss`, `hud_jump_release`, `hud_sprint_hold` |
| الافتراضي المستطيل | `default_layout_jump_works`, `default_layout_f3_toggles_debug` |
| بناء/تحديث | `install`, `first_run_install`, `upgrade_from_baseline`, `baseline_launch`, `walk_arena_ready` |
| مقاييس | `Session.perf`: baseline_title/baseline_world/post_world (FPS + frame_ms + TPS + CPU + RAM) |

**اختبارات الوحدة/التكامل** (CI أخضر في نفس الجولات):
`controls-desktop-tests` (39 اختبار `craftix_controls::` + keymapping +
sandbox craftix — الأحداث تصل runtime حقيقي) و`controls-android-check`.

**الجولات الخضراء (أدلة)**:

| Run | SHA | النتيجة |
| --- | --- | --- |
| 42301763824 | `bf54bc6` | SUCCESS — الجلسة كاملة، 0 فشل |
| 42306835737 | `fa9e7a4` | SUCCESS — الجلسة كاملة + مقاييس، 0 فشل |
| **42316562222** | `c3792c9` | **SUCCESS — الجلسة كاملة + المقاييس النهائية، 0 فشل** (الدليل الأساسي) |
| 42316864626 | `7df209e` | SUCCESS — بناء الإصدار النهائي |

الـAPK المُسلَّم بُني من `7df209e` — فرقه عن `c3792c9` (الذي اختُبر):
تعليقات + إعادة تسمية اسم تخطيط مرئي إلى `Default` + نص ملاحظات الإصدار —
**صفر تغيير سلوكي** (لا يمس الاختبارات).

---

##6. ما لم يُنجز / ما اختُصر (بصراحة)

1. **قياسات على جهاز حقيقي لم تُنفَّذ**: لا جهاز Android مادي متاح في بيئة
   التسليم — كل أرقام الأداء أعلاه من محاكي x86_64 بـ SwiftShader (رسم
   برمجي على المعالج). ادعاء «إصلاح Lag على جهاز اللاعب» **غير مثبت
   بالأرقام الحقيقية** — المُثبَّت: (أ) لا انحدار وظيفي، (ب) كلفة تصميمية
   منخفضة للنظام الجديد، (ج) أرقام المحاكي مع انحياز ترتيبها الموثّق.
2. **تفكيك CPU/GPU/threads/draw-calls/رفع نسيج/GC على مستوى المحرك لم
   يُقدَّم**: أدوات القياس هذه تتطلب جهازاً حقيقياً أو trace فعلي
   (Perfetto/GPU renderdoc) — خارج موارد هذه الجلسة.
3. **انحياز العيّنة المذكور** (حالة العالم المتراكمة بين baseline وpost)
   — لم يُعالَج بإعادة تصميم الجلسة (كلفة جولة كاملة إضافية + خطورة
   تشوّه)؛ عُرض بدل إخفائه.
4. **تعضيد جهاز لمس ثانٍ / شاشات مختلفة**: الاختبار على مقاس نافذة واحد
   (1080×2400) — أبعاد أخرى مغطاة بالاختبارات الوحدوية (aspect fractions)
   لا باختبار جهاز.
5. **محرر Zalith متعدد الطبقات وأنماطه البصرية لم يُنقل** (بديل موثّق —
   أنظر المحور1) + وضع TOUCH للجويستيك + محاكاة 8 اتجاهات + ClickEvent
   الخارجية + Compose — كما هو موثّق حججاً في ملف الـport.
6. **جلسة «طويلة» = قرابة ساعة** في CI (دخول عالم + اختبارات + تركيز/عودة
   + تحرير + إعادة تشغيل) — ليست اختبار تحمّل لعدة ساعات ليلية.
7. **ملاحظة سياسة**: `CONTRIBUTING.md` يمنع GPL كسياسة عامة؛ المنفذ
   بأمر مباشر مع الالتزام بـ GPL-3.0 (إشعارات، إشعار عدم رسمية، بلا اسم،
   ملفات رخصة) — والتعارض موثّق صراحة في `docs/craftix-controls-zalith-port.md`.

---

##7. امتثال GPL-3.0

- `THIRD_PARTY_NOTICES.md` — قسم مشتق من Zalith Launcher 2 + إشعار أن
  CRAFTIX تعديل غير رسمي وغير تابع/مُصدَّق (GPL 7(b)/7(c)).
- `third_party/zalith-layer-controller/LICENSE` — نص GPL-3.0 الكامل.
- `third_party/zalith-layer-controller/UPSTREAM-README.md` — نص README
  الأصلي.
- `docs/craftix-controls-zalith-port.md` — إشعارات حقوق النقل + سجل
  النقل/العدم نقل/الانحرافات + ملاحظة سياسة CONTRIBUTING.
- **لا اسم Zalith في المنتج**: صفر نتائج في `crates/ flutter/ tests/ tools/
  .github/` بعد التنظيف (اسم التخطيط الافتراضي صار `Default`، تعليقات
  الشيفرة صار «upstream»، ملاحظات الإصدار المنشورة مُعاد كتابتها). الاسم
  موجود فقط في التوثيق + ملف الإشعارات + مجلد الرخصة (كما ينص التعليق).

---

##8. خلاصة حالة الطلب

| # | المطلب | الحالة |
| --- | --- | --- |
| 1 | استبدال حقيقي بنظام Zalith (InputMap/LayerController/لمس حقيقي/تحرك/layouts/multi-touch/فصل) + Custom Controls مدمجة | ✅ منفَّذ ومُختبَر — ما اختُصر موثّق |
| 2 | إخفاء خارج اللعب + HUD في العالم + Inventory/Chat دون تغطية + بلا حذف لوحة مفاتيح | ✅ فحوص مخصصة ناجحة |
| 3 | تحليل Lag + ضمان عدم كلفة النظام الجديد | ⚠️ جزئي: تحاليل وأرقام محاكي + ضمان تصميمي؛ **قياس جهاز حقيقي لم يُنجز** (معتَمد) |
| 4 | اختبارات in-game فعلية + قياسات قبل/بعد | ✅ 53 فحراً + جولات خضراء متعددة + جدول FPS/frame_ms/TPS/CPU/RAM |
| 5 | APK مُحدَّث فوق 0.9.0 + رابط + تقرير شامل بما لم يُنجز | ✅ هذا التقرير + الرابط والـSHA256 أعلاه |
