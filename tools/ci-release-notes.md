## Horizon Android – Redesigned Edition 0.7.6 (Portrait)

Real `release` APK built from the redesigned source on `arena/92db30ff-horizon`.

**Portrait / full-width fix:** the Activity is locked to `portrait` in
`AndroidManifest.xml`, a fresh install defaults to Portrait, and automatic
(full-sensor) rotation can no longer flip the launcher into landscape. Every
page now uses the entire screen width: the old centred 560dp desktop column is
disabled (`max_content_width: 0`), cards adapt responsively with only a small
side padding, cutout insets shift the column instead of centring it, and all
dialogs (Accounts, orientation chooser, file import) are full-width portrait
sheets. The five-destination bottom bar spans 100% of the usable width and
stays above the Android gesture safe area.

It contains the new mobile-first launcher (dark navy / purple identity, hero
banner, Play button, version + profile cards, Mods / Packs / Shaders /
Installed feature cards, real performance card and the fixed bottom
navigation). The Rust Minecraft 26.3 engine, Vulkan, accounts, mods, resource
packs, shaders and all existing settings are unchanged.

* Architecture: `arm64-v8a` (Android 9 / API 28+, 64-bit phones and tablets)
* Build type: non-debuggable Gradle `release`, signed and verified with `apksigner`
* Resources: public build; official Minecraft 26.3 assets download on first launch
* Install: allow "install unknown apps" for your browser or file manager, then open the APK
* Verification assets: `APK_VERIFICATION.json`, `release-audit.json`, `badging.txt`,
  `manifest-tree.txt`, `SHA256SUMS.txt`
* Portrait QA renders of all pages at 360×800 / 393×873 / 432×960:
  `tools/ui-preview/index.html` in the repository

`APK_VERIFICATION.json` proves the packaged native library contains the new
`ui_spec.json` palette/layout, the portrait/full-width policy, the redesigned
launcher labels and the hero artwork, and that it differs from the official
v0.7.6 native library. `manifest-tree.txt` is the binary manifest dumped from
the APK, showing the portrait lock.

Updating over an existing installation requires the same signing certificate;
keep this release for future updates. See `REDESIGN.md` for scope and tests.
