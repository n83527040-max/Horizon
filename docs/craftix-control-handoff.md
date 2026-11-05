# CRAFTIX controls — handoff for the next agent

**Status: work in progress; do not treat this as a completed control rewrite or
final release.** The default game still uses the earlier touch system. An opt-in
Android path now connects the new core, HUD and an initial editor to the game,
but it has not been verified in actual gameplay and still has important gaps.
Read the latest follow-up in `docs/craftix-controls-replacement.md`. The user
explicitly wants the old system replaced, not merely patched.

## User's current request

Replace the old gameplay HUD and touch/input routing with a new, professional,
round dark-glass / neon-purple system. Back up the old controls first and remove
only the old control code after the replacement is ready; do not remove the Rust
engine, renderer, login, storage or other unrelated working features. Preserve
the new CRAFTIX splash/branding, landscape game in both directions and portrait
Flutter launcher. Keep the hotbar concept and the game's actual inventory/items.

Required gameplay controls: circular joystick with dead zone and smooth visual
return; Sprint, Jump, Sneak, Attack, Interact/Use directly below Attack and
slightly larger; a separate circular Inventory and More. Look/camera touches
must not conflict with other pointers. More must open the custom-controls editor
directly. Top bar needs working Chat, Pause, Settings, Players and supported
camera perspectives, plus actual player/skin information; remove F3, keyboard,
mouse and screenshot from the top bar, not unrelated engine capabilities.

The editor must support custom button creation, deletion, renaming, icons,
key/shortcut binding, position/size/opacity, enabled state and Single/Hold/Toggle;
named layouts with create, select, copy, rename, delete and default; drag
preview; persistent Save and transactional Cancel. Raw key M must really reach
the engine, but must not be represented as a map or mod unless the engine
actually provides that feature. Keep the project general: do not add Litematica
or other mods. Never leave keys held on focus loss, menu changes, resize or exit;
handle joystick + jump + attack and multiple pointers correctly.

After implementation, run available Rust, touch, persistence, Android arm64,
Flutter and regression tests, fix failures, then build an APK with the **existing
CRAFTIX Release key in GitHub Secrets** and compare its public certificate to
the prior release. Do not create a keystore or change/store secrets. The user
will test the APK on their own phone; clearly distinguish automated checks from
actual device tests. Do not describe an incomplete replacement as final.

## What exists now

### Working 0.8.0 candidate and earlier system

* [CRAFTIX 0.8.0 beta](https://github.com/n83527040-max/Horizon/releases/tag/craftix-0.8.0-beta)
  targets commit `fe0f4c6` and contains the **old** touch system, not the new
  replacement. APK SHA-256:
  `d17c19ac8870d69dee40f29ee51c2139b7eb87df004020568b15ed64ecc29318`.
* The user first saw an installation error, then reported 0.8.0 installed and
  opened successfully on their phone and supplied screenshots of the old HUD
  and More editor. That report is not a comprehensive device test. The cause of
  the earlier install error was not established; do not assert it was fixed by
  a signing change. `docs/craftix-apk-install-investigation.md` records checks.
* GitHub Actions compared 0.7.9 and 0.8.0 with the certificate derived from the
  current release secret, for API 28–35: same package `com.craftix.launcher`,
  version codes 31 and 32, and matching public certificate SHA-256
  `77:05:07:A6:EC:48:51:98:66:55:08:29:32:CA:C2:81:61:55:08:0F:D4:73:DB:7E:42:E9:A0:BF:FD:22:DD:2F`.
* The old system is `crates/horizon-client-render/src/touch.rs` and `touch/`
  (`custom.rs`, `editor.rs`, `layout.rs`, `ownership.rs`), with host wiring in
  `crates/horizon-client/src/game.rs`, `game/android_input.rs`, `game/pad.rs`
  and `src/touch_ui.rs`. It has working real engine mappings, a More editor,
  three numbered slots, shared ownership, raw keys and Single/Hold/Toggle, but
  the user rejected its square sprites, overall appearance and editor UX.
  See `docs/craftix-in-game-controls-status.md` for its earlier capabilities;
  its no-phone-test statement predates the user's later report above.
* Its layouts are saved per profile in `horizon-touch.json`. Layout transfer
  uses `craftix-controls/transfer.json` and `slot-1.json`–`slot-3.json`, exposed
  through the Android DocumentsProvider as **Control layouts**, not a native
  picker. Preserve or migrate real user data before replacing these paths.
* Splash/branding: `crates/horizon-client/src/ui_theme.rs` and
  `game/startup_menu.rs`. Launcher/game orientation: Flutter
  `android/app/src/main/AndroidManifest.xml`, `android_main.rs` and
  `android/launcher.rs`. Do not revert them. The Flutter download card work is
  in `flutter/lib/core/theme/craftix_runner_progress.dart`, its PNG asset and
  `flutter/test/craftix_download_state_test.dart`.

### Replacement core and opt-in game path

`crates/horizon-client-render/src/craftix_controls/` contains the new model,
router, editor data model, vector painter, event bridge, migration preview and
initial interactive editor. The Android game integration is in
`crates/horizon-client/src/game/craftix_experimental.rs`; it is **opt-in only**
via `CRAFTIX_EXPERIMENTAL_TOUCH=1` at compile time or in the process environment.
The ordinary game still uses the old controls. The initial editor is wired to
the new router and More only in that experimental path.

The integration and editor need actual gameplay testing, comprehensive UX and
feature completion before the default path changes. In particular, the editor's
Hold/Toggle modes can interact incorrectly with the engine's own toggle-crouch,
toggle-sprint, toggle-attack and toggle-use options: the router latches an input
while the underlying `ToggleKeyMapping` can also latch on press. Resolve this
with shared physical/controller ownership and tests before enabling by default.
Inventory-context custom buttons, global HUD settings, shortcut modifiers,
complete legacy migration, safe-area handling and dense-layout collision checks
also remain open. See `docs/craftix-controls-replacement.md` for details.

## Backup and how to recover

The full 0.8.0 source is in Git at `fe0f4c6` and is also referenced by the
`craftix-0.8.0-beta` release tag. An extra source archive was created at
`backups/craftix-controls-0.8.0-source.tar.gz` during this session, but
`backups/` is intentionally Git-ignored and **is not in the PR**. Do not depend
on that local archive existing in a later checkout. Recreate it from the tag or
commit, e.g.:

```sh
git fetch origin tag craftix-0.8.0-beta
git archive --format=tar.gz --output=/tmp/craftix-controls-0.8.0-source.tar.gz \
  craftix-0.8.0-beta crates/horizon-client-render/src/touch.rs \
  crates/horizon-client-render/src/touch crates/horizon-client/src/game/android_input.rs \
  crates/horizon-client/src/game/pad.rs crates/horizon-client/src/touch_ui.rs
```

The commit/tag also preserve `game.rs` and the rest of the repo. Do not commit
keys, passwords, extracted game assets or a new keystore.

## Verification and CI

* [Run 41922447049](https://github.com/n83527040-max/Horizon/actions/runs/41922447049)
  at `be2f342` passed desktop Rust tests for old `touch::` and new
  `craftix_controls::`, and Android arm64 type-check. It did **not** build an
  APK or run on Android. Later commits before this handoff change docs only.
* [Run 41906796041](https://github.com/n83527040-max/Horizon/actions/runs/41906796041)
  built, signed and statically verified the 0.8.0 candidate at `fe0f4c6`; it ran
  the Flutter analyze/tests and the then-current touch tests. It predates and
  does not validate the new core.
* Locally passed: `python3 tools/check_public_tree.py`, `git diff --check`, and
  `python3 -m unittest discover -s tests -p 'test_android*.py'` (35 tests).
  `tests/test_diagnose_apk_install.py` also passed. This sandbox had no cargo,
  Flutter or ADB; use Actions or a properly provisioned environment.
* The old launcher redesign source regression was narrowed deliberately in
  `tests/test_android_ui_redesign.py`: requested touch/renderer changes are now
  allowed, unrelated core/packaging paths remain protected. The controls-only
  workflow deliberately skips the legacy source/native/APK jobs. Do not report
  skipped jobs as passing.
* `.github/workflows/android-ui-release.yml` controls-only check:
  `gh workflow run android-ui-release.yml --ref <working-branch> -f native_checks=false -f build_apk=false`.
  It tests both cores and type-checks Android without packaging.
* `.github/workflows/flutter-mixed-apk.yml` is a manual release-candidate build
  with signing and certificate checks. Its current tag/filename are hardcoded to
  `craftix-0.8.0-beta` and it deliberately refuses to overwrite that release.
  Before a later candidate, choose a new version/versionCode/tag and keep package
  identity and certificate; do not overwrite an existing APK or claim success
  without an actual successful build, signature verification and SHA-256.
* Emulator update investigation: x86_64 could not install arm64-only APKs and
  a macOS ARM emulator timed out at boot. Neither result validates installation.

## Recommended continuation

1. Implement the new on-screen editor and the host adapter around the new core,
   including IME, data migration and actual engine key-down/up and UI actions.
2. Test input ownership across custom, physical and controller sources; make
   context changes and lifecycle release everything. Test hotbar, small-screen
   geometry, rotation and save/restart without destroying existing data.
3. Switch gameplay drawing/hit routing to the new system, then remove obsolete
   old gameplay control code only after backup and compatibility are verified.
4. Run tests and build a separately versioned, signed candidate. Clearly report
   remaining visual/device limitations and give APK link and SHA-256 only for a
   real verified artifact.
