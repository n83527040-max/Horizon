# CRAFTIX APK update investigation

The user reports that Android says the 0.8.0 package appears invalid when
updating their installed 0.7.9. This is not a diagnosis of a signing mismatch.

On 2026-11-04, the user supplied screenshots showing package
`com.craftix.launcher`, installed 0.7.9 (version code 31), and the beginning
`40825e97da19e5cf940b81` of the downloaded 0.7.9 APK SHA-256. That prefix
matches the GitHub 0.7.9 release asset digest; the screenshot does not show the
full digest or independently verify the installed binary's signer.

GitHub Actions [run 41917755262](https://github.com/n83527040-max/Horizon/actions/runs/41917755262)
downloaded the published 0.7.9 and 0.8.0 APKs and compared their package names,
version codes, and public signing certificates across Android API levels 28–35.
The certificates match each other and the public certificate derived from the
current GitHub Secrets release key. Version 0.8.0 has version code 32, above
31. Static archive and apksigner checks passed. No key or secret was changed.

The 0.8.0 release asset SHA-256 is
`d17c19ac8870d69dee40f29ee51c2139b7eb87df004020568b15ed64ecc29318`.
The checksum of the file downloaded on the user's phone has not yet been
provided. The phone model and Android version are also unknown.

An x86_64 Android emulator could not install the old arm64-only APK, so its
update result is inconclusive. A macOS ARM64 emulator did not finish booting;
it did not reach APK installation. Neither outcome proves that the update
works or fails on the user's device. Further diagnosis needs the new file's
complete checksum, the phone/Android version, and, if possible, the precise
PackageManager error (for example from `adb install -r` without uninstalling).
Do not instruct the user to uninstall their existing app and risk its data.
