# CRAFTIX in-game controls — implementation status

**Device-test candidate, not a final release sign-off.** The supplied images
are the visual target; the present implementation is not pixel-matched and has
not been tested on an Android device.

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
  Hold, Toggle or timed Single Press behavior, set size and opacity, and choose
  Always / Gameplay / Inventory visibility. Position is
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
  after Save/Done; saving a numbered slot or exporting writes that file immediately,
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
  timed single presses, and latched toggles. Distinct custom bindings resolving
  to the same engine key or mouse button, and mapped controller actions, share
  event-order ownership; last-owner release is covered by Rust unit tests.
  Resize and inventory-context transitions clear touches.
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
Mixed physical, built-in, and custom input ownership still needs real-device
integration testing. Multi-touch, backgrounding, rotation, IME, transfer and
persistence after process restart have not been verified on Android hardware.

The underlying engine supports specific mappings and raw keyboard events; a raw
M/R/G key does not provide a map, replay, or any other missing functionality.
No Minecraft Java mod integration is included or implied.

## Verification and release gate

* Local `tools/check_public_tree.py`, `tools/android_ui_layout.py`,
  `tools/android_ui_preview.py`, `git diff --check`, and all 35 Android Python
  tests passed. The old launcher-only regression was updated to protect
  unrelated engine/packaging paths rather than prohibit the requested touch
  engine changes; its comparison starts at the CRAFTIX task baseline.
* GitHub Actions controls-only run [41906693023](https://github.com/n83527040-max/Horizon/actions/runs/41906693023)
  at `382f894` passed desktop touch tests and Android arm64 type-check.
* Candidate run [41906796041](https://github.com/n83527040-max/Horizon/actions/runs/41906796041)
  at `fe0f4c6` passed touch Rust tests, Android Python regressions, Flutter
  analyze/tests, the Android arm64 release build, JNI checks, APK package and
  signature verification, and static install diagnostics. It published the
  prerelease below. The diagnostics do not install or run it.
* [CRAFTIX 0.8.0 beta APK](https://github.com/n83527040-max/Horizon/releases/download/craftix-0.8.0-beta/CRAFTIX-0.8.0-arm64-v8a.apk)
  is 87,431,498 bytes, SHA-256
  `d17c19ac8870d69dee40f29ee51c2139b7eb87df004020568b15ed64ecc29318`.
  CI compared its certificate with CRAFTIX 0.7.8 and verified they match;
  the reference certificate SHA-256 is
  `77:05:07:A6:EC:48:51:98:66:55:08:29:32:CA:C2:81:61:55:08:0F:D4:73:DB:7E:42:E9:A0:BF:FD:22:DD:2F`.
  The existing release secrets were used without changing them.
* Cargo, Flutter and ADB are unavailable locally; no actual device test or
  installation was performed. Test controls, multitouch, keyboard/IME,
  appearance, persistence after restart, transfer and both landscape rotations
  on a phone before treating this candidate as final.
