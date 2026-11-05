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

**Do not enable the new path by default yet:** the new router implements
Hold/Toggle itself, while `mapped_key` and `play_mouse`/`remote_mouse` also
apply the engine's optional toggle-crouch, toggle-sprint, toggle-attack and
toggle-use mappings. Those combined latches can disagree or stick. Correctly
resolve source ownership and test all combinations, including controller and
hardware, before claiming these modes work or deleting the old path. Other
known gaps are listed in the follow-up above.
