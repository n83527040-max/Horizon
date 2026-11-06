# CRAFTIX control layer — Zalith Launcher 2 LayerController port

Status: implemented in `crates/horizon-client-render/src/craftix_controls/`
(plus its Android host glue in `crates/horizon-client/src/game/`).
Upstream: [ZalithLauncher/ZalithLauncher2](https://github.com/ZalithLauncher/ZalithLauncher2),
`LayerController/` and `InputMap/` modules, GPL-3.0, Copyright (C) 2025-2026
MovTery and contributors (license copy: `third_party/zalith-layer-controller/LICENSE`).

This document records exactly what was ported, what was deliberately not
ported and why, and every deviation — so the modification can be audited
instead of taken on trust. The modified program keeps the name **CRAFTIX**
(`com.craftix.launcher`); it is an unofficial modification and is not
affiliated with the Zalith Launcher project.

> Policy note: `CONTRIBUTING.md` ("Only original code, or code whose
> license allows it") lists GPL among the licenses that are off limits.
> This port was carried out under an explicit instruction to study
> Zalith Launcher 2, port the compatible parts, and comply with GPL-3.0
> (copyright notices, unofficial-modification marking, no use of the
> Zalith name, license files added). If that standing policy is meant to
> hold, the port must be reverted and the control layer rewritten from an
> allowed-license source; the notice above and this file make the
> derivation discoverable either way.

## What was ported (same contract, Rust implementation)

| Zalith concept | Where it landed |
| --- | --- |
| `ButtonPosition` 0..10000 anchor in *available* space | `layout::Rect.anchor` (0..1 fractions) |
| `ButtonSize` percentage of screen height (w/h) | `layout::Rect.size` (fractions of height) |
| `VisibilityType` always / in_game / in_menu | `layout::Visibility` (Always/Gameplay/Inventory) plus the host gate that draws the HUD only while playing |
| `InteractionBehavior` Press / Swipable / Toggle | `layout::Behavior` Hold/Swipe/Toggle (plus the pre-existing `Single` pulse, kept for old layouts) |
| `releaseOnOutOfBounds`, re-press on return | `input::Router::moved` (OOB release → camera takes the pointer → fresh press on re-entry) |
| Swipe chains (`canBeSwipedTo`, `blocksSwipeChain`) | `input::Router` button chain: swipe-in only into Swipe widgets, Toggle blocks the chain |
| `isPenetrable` (camera keeps the pointer) | `Button.penetrable` → `Role::Button.look` emits `Event::Look` while held |
| `switch_layer` / layer hide | `Action::Group(u16)` + `Layout.hidden_groups` + host toggle (the default "GUI" button collapses the top bar) |
| `ControlManager` layouts as JSON files, active layout | existing `editor::Layouts` (`craftix-controls-v1.json`, named slots) |
| Default layout geometry (5 layers, joystick 34.17 % height at (0.1121, 0.7953), 11 %-high button row, PRI/SEC 21.5×13.8 %, anchors for Esc/F3/F5/Tab/T/IME/GUI/Ctrl/Alt/E) | `layout::Layout::default` mapped onto CRAFTIX actions |
| Joystick dead zone, forward-lock (`lockEvents`) | `Layout.joystick_dead_zone`, `Router` north-flick lock that holds Sprint until the stick is touched again |
| Widget press animation | existing `paint::Animator` (unchanged) |
| Flat translucent button style with pressed brightening and centred label | `paint::paint_rect_button` |

## Intentionally not ported, with the reason

1. **Multi-layer editor UI and per-layer styles.** Zalith's editor manages
   named layers, per-layer hide flags and style sheets (light/dark corner
   radii, alpha). CRAFTIX's in-game editor keeps one flat button list; the
   data model carries a `group` id and the runtime `hidden_groups` set
   (toggleable through a widget's `Action::Group`), but layer creation and
   style sheets were not ported because they would duplicate the existing
   per-button opacity/label controls without a functional gain for the
   CRAFTIX default layout. **Alternative shipped:** the group cycle row in
   the editor's Buttons page plus the "GUI" toggle button.
2. **`JoystickTriggerMode.TOUCH`.** Only DRAG semantics were ported
   (finger must start on the stick); TOUCH mode enlarges the touchable
   region so a nearby tap steals movement — CRAFTIX's fixed-base analog
   stick with its existing hotbar/camera arbitration already covers the
   interaction, and a second trigger mode would need new hit-priority
   rules across the router. **Alternative:** the existing `Router`
   arbitration (one joystick pointer, camera owns empty space) is kept
   unchanged.
3. **8-directional key emulation (InputMap → GLFW keys W/A/S/D).**
   Zalith must translate the stick into Java `Keyboard` events because it
   drives an unmodified Minecraft through LWJGL. The CRAFTIX engine
   consumes analog stick vectors natively (`pad.state.left`), so the port
   keeps the analog `Event::Move` pipeline and only borrows the dead-zone
   and lock contract. Emulating 8-way keys here would be a regression.
4. **`ClickEvent` types `send_text` / `open_file` / external intents.**
   CRAFTIX dispatches through its existing `Action` → token bridge
   (`action:key.*`, physical keys, chords, bindings). Text sending and
   file intents belong to the launcher UI, not the in-world HUD, and are
   reachable through the chat/IME paths already present.
5. **Compose/`PointerEventPass.Initial` capture semantics.** The Rust HUD
   is drawn by `HudBatch`, not Compose; the equivalent contract
   (topmost-first hit testing, consume vs pass-through, per-pointer
   ownership) is implemented in `input::Router` and the host's touch
   dispatch. The *observable* behaviour follows the upstream
   `TouchProcessor`/`PointerEventBus` rules listed above.

## Deviations (deliberate, engine-driven)

- **Joystick dead zone 0.25 instead of 0.5.** Upstream's 0.5 guards
  8-way key emulation; with native analog vectors it would make the stick
  feel heavy. Documented in `layout::Layout::default`.
- **Anchor geometry coexists with legacy circles.** Layouts written
  before the port (`circle` field) keep their exact hit boxes and are
  still painted as circles; new widgets and the new default use `rect`.
  Nothing is silently rewritten on load.
- **`Single` behaviour stays.** Upstream has no edge-pulse behaviour;
  existing CRAFTIX layouts use it for inventory/more-style taps.
- **Names.** No source file, product name, package or string shown to
  players uses "Zalith"; upstream names appear only in this
  documentation, the notices file and the license folder.

## Verification

- Unit tests: `craftix_controls::` tests cover OOB release + re-press,
  swipe chains + Toggle blocking, penetrable look-through, the forward
  lock, hidden-group hit filtering, editor round-trips and the bridge's
  edge ordering (`cargo test -p horizon-client-render --lib
  craftix_controls::`).
- In-game: see the emulator session recorded with the 0.10.0 build
  (main menu without floating buttons, world HUD, multi-touch
  move+camera+attack, inventory/chat, custom buttons, long-session key
  release, FPS/RAM before-after).

## Load path on boot (observed behaviour)

`Controls::default()` resolves `craftix-controls-v1.json` as follows:

- **File present and valid** — its layouts are used as-is.
- **File missing (`NotFound`)** — the host runs `migrate::preview` over the
  legacy `horizon-touch.json` settings (or `Settings::default()` when that
  file is absent too). Legacy-mapped buttons (Jump, Attack, joystick, …)
  keep their **old circle positions** (e.g. Jump at 0.94/0.5); buttons with
  no legacy equivalent (F3, IME, GUI, Ctrl, Alt, More) keep the
  `Layout::default()` rectangle anchors. Nothing is written back until the
  player saves.
- **File present but invalid** — `Layouts::default()` (the full
  rectangular Zalith-style HUD) is activated **in memory only**;
  `load_error` is recorded, which blocks Save in the editor so the player's
  file is never silently replaced.

Consequence for tests: removing the file does **not** exercise the shipped
rectangular default; an empty-but-present schema
(`{"schema":1,"active":0,"items":[]}`) is the stable way to select
`Layout::default()` on a device.
