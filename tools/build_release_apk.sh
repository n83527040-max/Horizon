#!/usr/bin/env bash
# Build a Horizon Android 0.7.6 release APK after the redesign checks pass.
#
# This is the real release path; it uses the project's existing Rust/Gradle
# toolchain (tools/android_release.py -> tools/android_build.py). It never
# bypasses the engine, the signing config or the generated-table allowlist.
#
# Requirements (not available inside the Arena sandbox):
#   Rust 1.95 + Android targets, Android SDK 35 / build-tools / NDK,
#   JDK 17, Gradle, and prepared engine tables from `tools/android_prepare.py`.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

fail() { printf 'ERROR: %s\n' "$*" >&2; exit 1; }
need() { command -v "$1" >/dev/null 2>&1 || fail "missing tool: $1"; }

need python3
need cargo
need rustup
need java
need git

: "${ANDROID_HOME:=${ANDROID_SDK_ROOT:-}}"
if [[ -z "$ANDROID_HOME" ]]; then
  fail "set ANDROID_HOME (Android SDK 35, build-tools 35.0.0, NDK 28)"
fi
[[ -d "$ANDROID_HOME/build-tools" ]] || fail "Android build-tools not found in $ANDROID_HOME"
[[ -d "$ANDROID_HOME/ndk" ]] || fail "Android NDK not found in $ANDROID_HOME"

VERSION="$(python3 - <<'PY'
import re, pathlib
text = pathlib.Path('packaging/android/build.gradle').read_text()
print(re.search(r"versionName '([^']+)'", text).group(1))
PY
)"
TABLES=".local/horizon-client/26.3"
[[ -f "$TABLES/horizon-resources.json" ]] || \
  fail "missing generated engine tables; run: python3 tools/android_prepare.py"
[[ -f "$TABLES/worldgen/blocks/structure-blocks.json" ]] || \
  fail "missing generated structure transforms; run: python3 tools/android_prepare.py again"

echo "== Horizon Android $VERSION redesign: source checks =="
python3 -m unittest discover -s tests -p 'test_android*.py'
python3 tools/android_ui_layout.py
python3 tools/check_public_tree.py --history
git diff --check

ABIS=(arm64-v8a)
if [[ "${1:-}" == "--all-abis" ]]; then
  ABIS=(arm64-v8a armeabi-v7a x86 x86_64)
fi

echo "== Native type-check / clippy (Android target) =="
for ABI in "${ABIS[@]}"; do
  python3 tools/android_build.py --abi "$ABI" --clippy --public \
    --sdk "$ANDROID_HOME" --java-home "${JAVA_HOME:-$(dirname "$(dirname "$(readlink -f "$(command -v java)")")")}"
done

echo "== Release APK =="
python3 tools/android_release.py --abi "${ABIS[@]}"

echo "== Result =="
find ".local/android/releases/$VERSION" -maxdepth 1 -type f -printf '  %f\n' | sort
printf '\nAPKs stay outside Git (*.apk is ignored). Keep the signing key for upgrades.\n'
