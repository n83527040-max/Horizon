## Horizon Android – Redesigned Edition 0.7.6

Real `release` APK built from the redesigned source on `arena/92db30ff-horizon`.
It contains the new mobile-first launcher (dark navy / purple identity, hero
banner, Play button, version + profile cards, Mods / Packs / Shaders /
Installed feature cards, real performance card and the five-destination bottom
navigation). The Rust Minecraft 26.3 engine, Vulkan, accounts, mods, resource
packs, shaders and all existing settings are unchanged.

* Architecture: `arm64-v8a` (Android 9 / API 28+, 64-bit phones and tablets)
* Build type: non-debuggable Gradle `release`, signed and verified with `apksigner`
* Resources: public build; official Minecraft 26.3 assets download on first launch
* Install: allow "install unknown apps" for your browser or file manager, then open the APK
* Verification assets: `APK_VERIFICATION.json`, `release-audit.json`, `badging.txt`, `SHA256SUMS.txt`

`APK_VERIFICATION.json` proves the packaged native library contains the new
`ui_spec.json` palette/layout, the redesigned launcher labels and the hero
artwork, and that it differs from the official v0.7.6 native library.

Updating over an existing installation requires the same signing certificate;
keep this release for future updates. See `REDESIGN.md` for scope and tests.
