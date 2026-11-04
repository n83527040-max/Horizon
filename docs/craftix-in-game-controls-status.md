# CRAFTIX in-game controls — implementation status

This is a work in progress, **not a release sign-off**. The supplied images are
visual references; the current HUD and editor do not yet match them.

## Changes in this work

* The Android game boot screen uses CRAFTIX, a purple glow and a fade driven by
  the existing loading clock. The desktop Horizon screen is unchanged.
* Existing movement and action controls receive Android-only purple rings and
  a pressed-size effect. Painting changes do not alter their saved hitboxes or
  input mappings; the joystick knob still follows the real movement state.
* Opening chat or the auxiliary touch overlay now flushes custom key-release
  events immediately after clearing held touches. Unassigned custom buttons are
  hidden and do not intercept camera touches outside the editor.

## Existing implementation, not newly validated on a device

`touch.rs` tracks fingers independently and maps controls to engine pad states.
`touch/custom.rs` and `game/android_input.rs` implement custom key down/up
edges; `touch/editor.rs` supports key selection, placement, size, opacity and
saving. This existing editor is not the requested three-column screen and does
not yet provide all the requested naming, icons, enable/disable, ordering,
profiles and import/export features. Its existing tests are not a substitute
for testing actual Android input and mods.

Horizon is a Rust Minecraft-compatible client, not a Java runtime. Java Fabric,
Forge and NeoForge mods such as Litematica and Replay Mod cannot run in this
client as Java mods (see `docs/android-direction.md` and `docs/android-0.7.6.md`).
Sending M/G/R key events does not add compatibility with them. Supporting them
would require a separate compatible Java runtime/loader architecture, not just
HUD buttons. Do not present their buttons as working here.

The requested new player/skin/status panel, players button, redesigned hotbar,
custom-controls dialog, complete neon artwork, animations, layout presets and
transfer features remain to be implemented and tested. Existing launcher and
game landscape behavior has not been revalidated in this change.

## Validation

* `tools/check_public_tree.py`: passed.
* Nine Python identity/responsive layout checks: passed; these do not exercise
  this Rust HUD or Android runtime.
* Rust tests and Android performance/content harnesses: not run; Cargo is absent
  in this environment. Added splash and unassigned-key tests are unexecuted.
* Flutter, Android builds, actual-device gameplay, multitouch, focus handling,
  perspective, chat keyboard, persistence, and signature comparison: not run.
* No APK was produced for this incomplete change. The release keystore and
  GitHub Secrets were not changed.
