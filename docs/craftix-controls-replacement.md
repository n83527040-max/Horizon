# New CRAFTIX controls — replacement in progress

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
