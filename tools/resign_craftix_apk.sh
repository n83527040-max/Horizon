#!/usr/bin/env bash
# Re-sign a built APK with the OWNER's permanent CRAFTIX key using BOTH
# v1 (JAR signing) and v2 (APK Signature Scheme v2).
#
# Why: AGP disables v1 for minSdk >= 24, producing v2-only APKs. Some OEM
# package installers reject those with "package appears to be invalid".
# apksigner sign STRIPS pre-existing signatures by default, so re-signing a
# Gradle-signed APK with the SAME key is safe and changes no engine code.
#
# No key material is stored in the repo and no value is printed in logs.
set -euo pipefail
APK="${1:?usage: resign_craftix_apk.sh <apk>}"
: "${CRAFTIX_KEYSTORE_FILE:?CRAFTIX_KEYSTORE_FILE is missing}"
: "${CRAFTIX_STORE_PASSWORD:?CRAFTIX_STORE_PASSWORD is missing}"
: "${CRAFTIX_KEY_ALIAS:?CRAFTIX_KEY_ALIAS is missing}"
: "${CRAFTIX_KEY_PASSWORD:?CRAFTIX_KEY_PASSWORD is missing}"
BT="${ANDROID_HOME:-/usr/local/lib/android/sdk}/build-tools/35.0.0"
JAVA_DIR="${JAVA_HOME:-${JAVA_HOME_17_X64:-}}"
[[ -n "$JAVA_DIR" ]] || { echo "ERROR: JAVA_HOME is not set" >&2; exit 1; }
SIGNER=("$JAVA_DIR/bin/java" -jar "$BT/lib/apksigner.jar")

"${SIGNER[@]}" sign \
  --ks "$CRAFTIX_KEYSTORE_FILE" \
  --ks-pass env:CRAFTIX_STORE_PASSWORD \
  --ks-key-alias "$CRAFTIX_KEY_ALIAS" \
  --key-pass env:CRAFTIX_KEY_PASSWORD \
  --v1-signing-enabled true \
  --v2-signing-enabled true \
  --min-sdk-version 21 \
  "$APK"

# 21 forces the v1 (JAR) signature that OEM installers require. apksigner
# applies v2 as well when possible; v1 alone is still a valid sideload
# signature on every Android version, so a missing v2 is a warning only.
python3 "$(dirname "$0")/check_apk_signature_schemes.py" "$APK"

V21="$("${SIGNER[@]}" verify --verbose --min-sdk-version 21 "$APK" 2>&1)" || {
  printf '%s\n' "$V21" >&2
  echo "ERROR: CRAFTIX APK does not verify at minSdk 21" >&2
  exit 1
}
V28="$("${SIGNER[@]}" verify --verbose --min-sdk-version 28 "$APK" 2>&1)" || true
printf '%s\n%s\n' "$V21" "$V28"
if ! printf '%s' "$V21" | grep -q "Verified using v1 scheme.*: true"; then
  echo "ERROR: CRAFTIX APK is missing the required v1 (JAR) signature" >&2
  exit 1
fi
if ! printf '%s' "$V28" | grep -q "Verified using v2 scheme.*: true"; then
  echo "NOTE: v2 scheme not present; v1 (JAR) signature is used for compatibility"
fi
echo "CRAFTIX APK signed with the permanent owner key; v1 (JAR) verified."
