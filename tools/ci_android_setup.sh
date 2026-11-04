#!/usr/bin/env bash
# Pin a deterministic Android SDK for GitHub Actions release builds.
#
# tools/android_build.py selects the numerically newest build-tools and NDK
# directory. The hosted runner ships several of each; pruning to one of each
# keeps the compiler triple, aapt2 and apksigner predictable and frees disk.
set -euo pipefail

SDK="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-/usr/local/lib/android/sdk}}"
echo "ANDROID_HOME=$SDK"
[[ -d "$SDK" ]] || { echo "Android SDK missing: $SDK" >&2; exit 1; }

SDKMANAGER="$(find "$SDK" -type f -name sdkmanager -perm -u+x 2>/dev/null | head -1 || true)"
echo "sdkmanager=${SDKMANAGER:-<not found; using preinstalled packages>}"
if [[ -n "$SDKMANAGER" ]]; then
  yes | "$SDKMANAGER" --licenses >/dev/null 2>&1 || true
  # Upstream builds against NDK 28 and build-tools 35.0.0. Ignore failures:
  # the runner normally already carries compatible versions.
  yes | "$SDKMANAGER" "ndk;28.2.13676358" "build-tools;35.0.0" "platforms;android-35" >/dev/null 2>&1 || true
fi

BUILD_TOOLS="35.0.0"
[[ -x "$SDK/build-tools/$BUILD_TOOLS/aapt2" ]] || { echo "build-tools $BUILD_TOOLS/aapt2 missing" >&2; exit 1; }
[[ -f "$SDK/build-tools/$BUILD_TOOLS/lib/apksigner.jar" ]] || { echo "apksigner.jar missing" >&2; exit 1; }
[[ -d "$SDK/platforms/android-35" ]] || { echo "platform android-35 missing" >&2; exit 1; }

TOOLCHAIN="toolchains/llvm/prebuilt/linux-x86_64/bin"
NDK=""
for candidate in 28.2.13676358 29.0.14206865 30.0.16248370 27.3.13750724; do
  if [[ -x "$SDK/ndk/$candidate/$TOOLCHAIN/aarch64-linux-android28-clang" ]]; then
    NDK="$candidate"
    break
  fi
done
if [[ -z "$NDK" ]]; then
  for directory in "$SDK"/ndk/*; do
    [[ -x "$directory/$TOOLCHAIN/aarch64-linux-android28-clang" ]] || continue
    NDK="$(basename "$directory")"
    break
  done
fi
[[ -n "$NDK" ]] || { echo "No NDK provides aarch64-linux-android28-clang" >&2; exit 1; }

for directory in "$SDK"/build-tools/*; do
  [[ "$(basename "$directory")" == "$BUILD_TOOLS" ]] || rm -rf "$directory" 2>/dev/null || sudo rm -rf "$directory"
done
for directory in "$SDK"/ndk/*; do
  [[ "$(basename "$directory")" == "$NDK" ]] || rm -rf "$directory" 2>/dev/null || sudo rm -rf "$directory"
done

for tool in aarch64-linux-android28-clang aarch64-linux-android28-clang++ llvm-ar llvm-strip llvm-objcopy llvm-readelf; do
  [[ -x "$SDK/ndk/$NDK/$TOOLCHAIN/$tool" ]] || { echo "NDK tool missing: $tool" >&2; exit 1; }
done

echo "Pinned build-tools: $BUILD_TOOLS"
echo "Pinned NDK: $NDK"
echo "Available disk:"
df -h "$GITHUB_WORKSPACE" || true
