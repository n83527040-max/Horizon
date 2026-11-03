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

"${SIGNER[@]}" verify --verbose --min-sdk-version 21 "$APK"
echo "CRAFTIX APK re-signed with v1 (JAR) + v2 using the permanent owner key."
