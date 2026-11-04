# CRAFTIX in-game controls — implementation status

**Work in progress, not a release sign-off. No new APK was built.** The supplied
images are the visual target; the present implementation is not pixel-matched.

## Implemented in source

* Android boot uses CRAFTIX with a purple glow and timed fade. Existing game
  controls keep their engine input wiring; Android-only purple rings and a
  eased pressed-size effect and a subtle idle pulse do not alter their hitboxes.
  The Android Standard preset uses a reference-inspired landscape arrangement;
  existing saved placements are not automatically overwritten.
* A More button near the hotbar opens a modal, three-column custom-button
  manager; the older auxiliary keyboard remains accessible. The manager can
  add or delete a button, assign an engine action or supported physical key,
  edit its name with the Android IME, cycle icons, enable/disable it, select
  Hold or Toggle behavior, set size and opacity, and choose Always / Gameplay / Inventory visibility. Position is
  edited in the existing drag editor; select Custom in its toolbar to return.
  Onscreen slots can be reordered using that editor's Order controls.
* Save commits the settings, Cancel restores the original editor snapshot.
  Unassigned, disabled, fully transparent and context-hidden custom buttons
  do not capture gameplay touches. Inventory visibility requires the existing
  inventory-controls setting to be enabled. The key picker lists supported
  keys, but a key without a corresponding game action does not create one.
* Layouts has standard and left-handed presets; three numbered persistent slots
  can be saved and loaded, and a separately confirmed reset removes custom
  buttons. Presets and loaded layouts are applied to the active settings only
  after Save/Done; saving a named slot or exporting writes that file immediately,
  even if the editor is later cancelled.
* Layout transfer is bounded versioned JSON, containing only control data. On
  Android, Layouts > Export writes `transfer.json` in the current profile's
  **Control layouts** Android Documents provider folder. Copy it with a Files
  app. To import, replace `transfer.json`, select Import and then Save/Done.
  The three numbered slots use the same folder. This is not a system picker.
* Hotbar borders have a purple overlay over the original engine's nine slots,
  selection and real items. The More button moves upward if the left-hand
  offhand or attack gauge would overlap it. A compact card uses the real player
  name, health and armor while retaining original survival bars. In multiplayer
  a top-row Tab button holds the actual engine player-list action. It is not
  shown for a single-player world. The card draws the available player skin's
  face and hat from the atlas, or Steve if unavailable; integrated offline
  skins are registered in that same atlas. This path needs device validation.
* Neon line segments use one quad each rather than a quad per pixel; this bounds
  the geometry cost without claiming a measured frame-rate improvement.
* Per-finger custom key ownership provides down/up events, shared-key holds,
  and latched toggles. Resize and inventory-context transitions clear touches.
  The auxiliary overlay no longer swallows the end of an already-owned HUD
  touch merely because the finger moved over it. Transitions clear held keys.
* There are Rust tests for editor interactions, keys, context visibility,
  multitouch holds, drag positioning, layout serialization, old settings and
  reset/cancel, plus geometry tests. Tests and code are not a device sign-off.

## Remaining work and limitations

The manager is an initial dark-purple three-column implementation, not yet a
pixel-faithful reproduction of the references; text localization, icon artwork,
and a polished one-screen placement workflow remain incomplete. Chamfered glass,
sliders and a basic opening fade are implemented, but have not been visually
validated. The HUD layout, glow, icons, animation, skin portrait, full player
info, configurable top-row players button, and responsive collision handling
still need further work and visual/device validation. The fixed
More/Tab buttons are not yet individually movable/resizable. Inventory-only
buttons require enabling inventory controls. Layout slots are numbered, not
user-named, and transfer uses the Files provider rather than a native picker.

The underlying engine supports specific mappings and raw keyboard events; a raw
M/R/G key does not provide a map, replay, or any other missing functionality.
No Minecraft Java mod integration is included or implied.

## Verification and release gate

* `tools/check_public_tree.py` and nine Python identity/responsive checks passed
  locally. They do not test Rust interaction or Android.
* GitHub Actions controls-only run 41900991959 at commit `95832ad` passed the
  Rust touch tests and Android arm64 type-check. Run 41902824058 at `62550d8`
  passed desktop tests; its Android job was cancelled when the newer run started,
  not reported as a success. Run 41903433421 at `c334e02` is in progress.
  These jobs do not build or run an APK. Changes after those commits require
  their own check.
* Cargo, Flutter and ADB are unavailable locally. No actual device tests have
  been performed for each button, multitouch, keyboard/IME, appearance,
  persistence after process restart, transfer, orientation, or performance.
  Android Landscape and launcher Portrait still require device verification.
* No APK or signing operation was produced for these changes, and no release
  signing key or GitHub Secret was changed. A release requires completion,
  real Android tests, an APK built with the existing CRAFTIX Release key, and
  a certificate comparison. Previous 0.7.9 does not contain this work.
