#!/usr/bin/env bash
# Decode the OWNER's permanent CRAFTIX release keystore at build time.
#
# Values come ONLY from GitHub Actions repository secrets (never from the repo,
# never printed). The decoded keystore lives in $RUNNER_TEMP, outside source.
# Only the PUBLIC SHA-256 certificate fingerprint is printed for verification.
set -euo pipefail

: "${CRAFTIX_KEYSTORE_BASE64:?CRAFTIX_KEYSTORE_BASE64 secret is missing}"
: "${CRAFTIX_STORE_PASSWORD:?CRAFTIX_STORE_PASSWORD secret is missing}"
: "${CRAFTIX_KEY_ALIAS:?CRAFTIX_KEY_ALIAS secret is missing}"
: "${CRAFTIX_KEY_PASSWORD:?CRAFTIX_KEY_PASSWORD secret is missing}"
: "${RUNNER_TEMP:?RUNNER_TEMP is not set}"

KEY="$RUNNER_TEMP/craftix-release.jks"
umask 077
printf '%s' "$CRAFTIX_KEYSTORE_BASE64" | base64 -d > "$KEY"
if [[ ! -s "$KEY" ]]; then
  echo "ERROR: decoded CRAFTIX keystore is empty" >&2
  exit 1
fi

JAVA_HOME_DIR="${JAVA_HOME:-${JAVA_HOME_17_X64:-}}"
[[ -n "$JAVA_HOME_DIR" ]] || { echo "ERROR: JAVA_HOME is not set" >&2; exit 1; }
# -storepass:env avoids putting the password on the command line. Only the
# PUBLIC certificate fingerprint is emitted; no private key data is printed.
"$JAVA_HOME_DIR/bin/keytool" -list -keystore "$KEY" \
  -storepass:env CRAFTIX_STORE_PASSWORD -alias "$CRAFTIX_KEY_ALIAS" \
  | grep -i 'SHA-256' | head -1 || true

echo "CRAFTIX_KEYSTORE_FILE=$KEY" >> "$GITHUB_ENV"
echo "CRAFTIX release keystore decoded outside the repository (path only)."
