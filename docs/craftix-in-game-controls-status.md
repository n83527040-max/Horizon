# CRAFTIX in-game controls — implementation status

**Work in progress, not a release sign-off. No APK has been built for this work.**
The images supplied by the user are the target, not proof that the current HUD
or editor matches them.

## Implemented in source

* Android's boot screen uses CRAFTIX, a purple glow and a timed fade. Desktop
  retains its existing identity. Android-only rings and a pressed-size effect
  were added to the existing movement/action HUD; hitboxes are not animated.
* Existing per-finger engine actions and custom key down/up logic are reused.
  Opening chat or the auxiliary overlay flushes held custom key releases.
  Unbound and disabled buttons are not drawn and do not capture play touches.
* In the existing control editor a custom button can have a name, one of six
  icon choices, a binding, enabled state, order, location, size and opacity.
  In-game and title-screen Android editors both route names to the system IME.
  Editor changes are transactional: Done saves; Cancel restores the snapshot.
* Standard and left-handed control presets are available in Layouts. The
  presets reset built-in layouts and visibility but keep custom buttons and
  unrelated preferences. They must still be saved with Done.
* Layouts can be transferred as bounded, versioned JSON with only control data.
  On Android, Layouts > Export writes `transfer.json` in the current profile's
  **Control layouts** document-provider folder. Copy that file using Android's
  Files app; to import, replace `transfer.json` in that folder and select
  Layouts > Import, then Done. Import validates before changing settings; it
  does not overwrite gameplay or graphics preferences. This path is not a system
  file picker and has not been tried on a phone.
* Legacy custom buttons default to enabled and text-only when loaded. New
  source tests cover metadata, presets, cancellation, export/import, limits,
  multitouch key release, unassigned and disabled buttons.

## Important limitations

The editor is not yet the requested three-column design. The complete neon HUD
artwork and animation, redesigned nine-slot hotbar, player/skin/status panel,
players button, more menu and full visual placement from the supplied images
are not implemented here. Existing engine functions must not be represented as
newly verified on a device. Launcher orientation and absence of camera/input
conflicts require real-device tests.

Horizon is a Rust Minecraft-compatible client, not a Java runtime. Java Fabric,
Forge and NeoForge mods such as Litematica and Replay Mod cannot run as Java
mods here (see `docs/android-direction.md` and `docs/android-0.7.6.md`). Sending
M/G/R events does not add compatibility. A separate compatible Java
runtime/loader architecture would be needed; do not claim mod support.

## Validation

* `tools/check_public_tree.py`: passed locally.
* Nine Python identity/responsive checks: passed locally. They do not exercise
  the Rust HUD or Android runtime.
* GitHub Actions run 41890858043 at commit `49759c4`: the desktop Rust touch
  tests and Android arm64 type-check jobs passed independently. Its overall
  result was failure because the legacy source-regression suite rejects changes
  to engine files; those legacy tests predate this HUD task. The follow-up
  dispatch-only controls mode separates the controls checks from that suite.
* Local Cargo, Flutter and ADB are unavailable. No physical Android tests of
  rendering, multitouch, IME, profile transfer, process restart or orientation
  have been performed. Tests cannot establish visual fidelity or release safety.
* No APK, signing operation, signing key or GitHub Secret was created or changed
  for these changes. A final release requires a complete implementation, a
  signed build with the existing CRAFTIX Release key, certificate comparison,
  and actual Android-device testing.
