# New CRAFTIX controls — replacement in progress

> **Current status (newer than the historical entries below):** an opt-in
> Android game path now connects the new HUD, input bridge and an initial
> interactive editor. It is not the default. It has not been run end-to-end on
> Android and does not yet meet all requirements; no new APK was produced.
> See the follow-up section at the end before using the older status notes.

The user confirmed that CRAFTIX 0.8.0 eventually installed and opened on their
phone, and provided screenshots of the existing HUD and editor. Their new
request supersedes the earlier instruction to preserve the old touch system:
the replacement should be a new implementation, not a visual patch.

## Backup

The complete 0.8.0 source is permanently available at Git commit `fe0f4c6`
(which remains in the current branch history). An additional source-only
archive of the relevant touch modules and game integration was made at
`backups/craftix-controls-0.8.0-source.tar.gz` (SHA-256
`6722037b4a67ca4c7139357ad3f468242f726020e93c6c04790f42c450b2f442`).
The repository intentionally ignores `backups/`; the original files can also be
restored from the recorded Git commit. No private keys are in the backup.

## Implemented independently so far

`crates/horizon-client-render/src/craftix_controls/` contains an isolated new
model for circular controls, validated named layouts and transactional editing,
real key tokens, Hold/Toggle/Single input ownership, joystick dead zone and
sensitivity, camera drag events, and a vector-based purple glass painter.
It has unit tests for multiple simultaneous pointers, shared bindings, input
release, named layout persistence, touch geometry, and bounded rendering.
These are **new components**, not a claim that they have been integrated.

## Still required before switching away from the old system

* Connect the new router to real engine input and release paths, including
  physical/controller overlap, menus, inventory, focus, rotation and shutdown.
* Build the new on-screen editor: named layout creation, editing, duplication,
  deletion, button properties, position preview, save/cancel, and IME.
* Integrate the painter with gameplay and hotbar, player status, actual
  Chat/Pause/Settings/Inventory/Players/perspective, More, and remove the old
  touch UI and old hit routing only after replacement and migration are safe.
* Handle safe areas and responsive collision with hotbar and controls on small
  landscape screens, and verify real touch targets and visuals on a phone.
* Migrate or preserve existing saved controls and layouts without data loss;
  integrate import/export and handle shortcuts the engine genuinely supports.
* Run integration and Android/Flutter tests, build and verify a signed APK using
  the existing release secrets, and compare its public certificate. No new APK
  has been built for this replacement. The 0.8.0 beta is the old system.

Until these are implemented, the old system remains active deliberately so a
partially built replacement cannot break the player's working installation.

## Automated checks so far

* [CI run 41922447049](https://github.com/n83527040-max/Horizon/actions/runs/41922447049)
  at `be2f342` passed the existing and new Rust touch-model tests, including
  layout and drag tests, and Android arm64 type-check. No APK was built or run.
* `tools/check_public_tree.py`, `git diff --check` and 35 Android Python tests
  passed locally. Cargo, Flutter and ADB are unavailable on the local machine.
* The user verified that 0.8.0 installed and displayed the old controls; this
  does not test the new components, which are not active in the game yet.

## Follow-up work on `arena/8fe2b017-horizon`

* `bridge.rs` converts router events to the existing engine dispatch token
  vocabulary with first/last ownership for multiple actions on one binding.
  `game/android_input.rs` now exposes that same real dispatch as a separate
  method. **The game does not call the new bridge yet**, so this does not
  establish end-to-end input delivery.
* `migrate.rs` previews a non-destructive conversion from the currently loaded
  old `Settings` into three named layouts. It keeps the source unchanged,
  preserves custom key/action, visibility and toggle data where supported,
  disables unbound placeholders and reports unsupported behaviors and separate
  numbered slots. **The preview is not wired into persistence or UI**, and is
  not a lossless migration of all legacy settings; do not delete the source.
* Inventory-specific visibility and gesture rejection were added to the new
  model; a new save operation uses unique temporary files so a stale temp from
  a crash cannot block later saves. These capabilities are still not live.
* [CI run 42032750552](https://github.com/n83527040-max/Horizon/actions/runs/42032750552)
  passed desktop Rust tests and Android arm64 type-check for commit `6783d54`.
  A follow-up engine-dispatch refactor is being checked separately. No new APK
  has been built or tested on Android.
* [CI run 42033939455](https://github.com/n83527040-max/Horizon/actions/runs/42033939455)
  passed desktop Rust tests and Android arm64 type-check for `647551c`, including
  the dispatch refactor and binding validation. APK and Android device execution
  were not part of this run. Python Android regression suite: 35 passed;
  signature-diagnosis unit test: 1 passed.

## Follow-up: opt-in Android host integration

The current code adds `game/craftix_experimental.rs`. With
`CRAFTIX_EXPERIMENTAL_TOUCH=1` supplied **at Rust compilation** (or as a
process environment variable), gameplay touches are routed through the new
router and bridge into the existing engine input dispatcher. Its joystick is
merged into controller movement; Look updates view rotation; the new painter
draws the gameplay HUD. Hotbar clicks are reserved before camera capture, and
a viewport-resolved layout moves controls away from the hotbar. Focus loss,
menus and viewport changes clear router ownership. The default remains old.

`craftix_controls/ui.rs` adds an initial Buttons / Layouts / Position editor,
with native Android text input, key names or existing engine bindings, behavior,
icon, enabled state, size and opacity, named layouts, drag, reset, Save and
Cancel. More opens this editor in the experimental path. Saving writes a
separate `craftix-controls-v1.json` only after validation; invalid existing new
files are not overwritten. A preview migrates `horizon-touch.json` and up to
three old slot files without changing them. Imported old modes and unsupported
settings are **not** a complete lossless migration; warnings are logged.

Remaining before release includes: verify runtime/engine event ordering and
multitouch on an actual game path (including controller/keyboard overlap),
validate editor rendering and IME on Android, comprehensive settings and
migration/slot coverage, shortcuts/modifiers, safe areas and dense layouts,
menu/inventory context behavior and focus/rotation, and full Android/Flutter
regression and signed APK. The old renderer/touch code is still active by
default and has not been removed. The editor is an initial implementation, not
a claim that all requested functionality or UX is finished. No phone or
emulator gameplay has been tested. The 0.8.0 APK is still the previous system.

[CI run 42046306563](https://github.com/n83527040-max/Horizon/actions/runs/42046306563)
passed Rust touch tests and Android arm64 type-check at `58b096a`. Its editor →
router → bridge test is not actual gameplay. No APK was built in that run.

### Verification and a known blocking interaction

[CI run 42049238782](https://github.com/n83527040-max/Horizon/actions/runs/42049238782)
passed the full source checks and Android clippy/type-check for both arm64 and
armv7 at `14fac0b`; APK and controls-only jobs were skipped. Earlier
[CI run 42046306563](https://github.com/n83527040-max/Horizon/actions/runs/42046306563)
passed the touch suites and arm64 type-check. These checks do not run a game.
The first full check failed because `sdkmanager` was missing from PATH; after
using the existing SDK setup script, it exposed two clippy errors that were
fixed. No Android device/gameplay test or new APK/signing verification occurred.

### Follow-up: separate toggle ownership and a real WASM guest

The opt-in Android path now separates the touch-owned held level from the
engine's own toggle state for sneak, sprint, attack and use. It aggregates
resolved inputs before merging them, and local and remote handlers consume
only effective transitions for the replacement touch source. Physical mouse
presses while touch is active still enter the normal mouse handler, so duplicate
clicks and claimed mod-item use require explicit integration coverage. The old
dispatch remains the default, and its keypad event ordering is preserved. This
is not a claim that every combination of real hardware, controller, mod,
lifecycle and gameplay is verified.

On arm64 Android the default Rust feature set includes the WASM mod runtime;
legacy armv7 builds explicitly disable it. The replacement delivers raw key
hooks before keypad consumption, but still is not a complete synthetic
`winit::KeyboardInput`. No claim is made for Java/Fabric/Forge mods.

[CI run 42055251200](https://github.com/n83527040-max/Horizon/actions/runs/42055251200)
passed desktop tests, including the shared toggle-ownership path and a real
sandboxed WASM guest reached by K/A touch-router and bridge events, and
Android arm64 type-check. The WASM test calls the runtime as a modeled forwarding
step: it does **not** instantiate Android `Game`, a device or full gameplay.
The test is not a general guarantee of third-party mod compatibility. APK and
native clippy jobs were skipped. No new APK or device test is available.

**Do not enable the new path by default yet.** See the newer follow-up below:
some features have since been implemented, but device/game integration and
important overlapping input cases remain unverified.

Final checks in this iteration: [run 42056034148](https://github.com/n83527040-max/Horizon/actions/runs/42056034148)
passed the controls suite (including the two-button K ownership test with a
real WASM guest) and Android arm64 type-check on `a7f8b09`.
[Run 42057288839](https://github.com/n83527040-max/Horizon/actions/runs/42057288839)
passed source checks and Android Clippy/type-check for arm64 and armv7 at
`7195fe7`. Neither run builds an APK or executes on a device. The intervening
commits changed documentation only.

### Newer follow-up: shortcuts, icons, transfers and inventory overlay

The experimental editor accepts Ctrl/Shift/Alt chords, and the bridge owns
individual keys, presses modifiers first, releases the main key first and
releases remaining keys in reverse acquisition order on teardown. The guest
test observes separately registered modifier and main-key events; it does not
prove that a mod's binding system has native chord support. The dispatch still
is not a complete `WindowEvent`, and `ModifiersChanged` is not synthesized.

The editor's Icons page imports a PNG by filename from the Android Documents
Provider's **Control layouts** folder. Users first copy the file there using
Files; this is not a native picker. PNG is bounded and embedded as a 16x16 RGBA
thumbnail in the layout, so JSON transfer carries the icon. Import/export
buttons use `Control layouts/transfer-v1.json`; import remains staged until
Save, while Cancel preserves the original. Existing invalid saved files are
not overwritten. Both mechanisms need device/IME and rendering verification.

Custom buttons can select Always, Gameplay or Inventory visibility. The
experimental inventory path draws/hit-tests custom buttons only outside a
conservatively protected container/recipe region and clears input on context
changes. It does not support arbitrary menu/custom contexts, and collisions
with every modded/extended screen are not established. `mod_key` intentionally
uses inventory browser events when applicable and otherwise suppresses mod key
downs while a screen is open; inventory buttons must not be described as
general-purpose WASM mod shortcuts.

The editor text guard suppresses paired IME key presses/releases and preserves
releases for previously held hardware keys. Actual Android IME, multi-device
ordering and background/focus transitions remain untested. Physical mouse
overlap and claimed mod-item side effects are still unresolved. No device
test or new signed APK has occurred; preserve the old default path.

Verification of this follow-up: [controls run 42062963063](https://github.com/n83527040-max/Horizon/actions/runs/42062963063)
passed the Rust controls and sandbox guest tests (including ordered modifier
and main-key events), persistence/transfer/PNG tests, and Android arm64
type-check at `c847c05`. [Android run 42064813399](https://github.com/n83527040-max/Horizon/actions/runs/42064813399)
passed source checks plus Clippy/type-check on arm64 and armv7 at `857edf0`.
The preceding run 42064067572 failed a Clippy `nonminimal_bool` check; it was
fixed and rerun. The checks do not run the Android game or build a release APK.

### Delivery iteration: default path, input ownership and device testing

**The replacement is now the default.** `craftix_experimental::enabled()`
returns true unless `CRAFTIX_EXPERIMENTAL_TOUCH=0` is set (environment or
compile time), which is the rollback switch to the legacy touch system. The
legacy renderer/touch code stays in the tree as that fallback; gameplay HUD,
inventory overlay, movement merge and the More editor all run through the
replacement when it is active. Menus still use the shared legacy touch strip
(a documented limitation, not a claim of menu-context parity).

**Touch/mouse ownership.** The replacement HUD's visibility follows the
*configured* input preference (`Auto`/`Touch` show it, an explicit
`KeyboardMouse`/`Controller` preference hides it), not whichever device was
used last: moving or clicking a physical mouse no longer tears down held touch
buttons, and a touchscreen tap no longer hides the HUD beside mouse input. The
auxiliary physical-pointer path (`android_aux_mouse`) now hit-tests the
replacement layout in gameplay instead of hidden legacy regions, so a mouse
click either presses a visible replacement button (keypad mode) or stays with
the game/mods; a press already owned by the HUD still receives its release.
HUD buttons continue to dispatch through router -> bridge ->
`android_dispatch_craftix_inputs` with the existing edge aggregation, so one
press produces one engine edge.

**Mod input.** The engine supports sandboxed WASM `.hmod` mods on Android;
Java/Fabric/Forge mods are not supported by this client and are not claimed.
Raw custom keys (`Action::Key`, e.g. a K button) reach `mod_key`/native key
events from the replacement dispatch; `action:*` tokens drive the engine's own
mappings. The CI WASM guest test still proves Router -> Bridge -> `Mods::key`
ordering for K/A and modifier chords; it does not run a full Android `Game`.

**Emulator game test.** `tools/test_craftix_game_emulator.py` (dispatched via
`android-ui-release.yml` with `game_test=true`) builds a non-public x86_64
test APK (control socket compiled in, signed with the existing certificate),
boots an API 33 emulator and drives the real `HorizonActivity`: first-run
resource install, world entry over the control socket, real `adb input` touch
on Jump/Sprint/Sneak/Attack/Joystick, the More editor creating and saving a
custom K button through the Android IME, inventory-overlay taps, a menu opened
while a touch is held, engine-level mouse alongside a held HUD button, chat
text injection, background/foreground focus loss and restart persistence, with
screenshots and logcat uploaded as artifacts. Results are recorded in that
run's `report.json`; the test APK is never published.
