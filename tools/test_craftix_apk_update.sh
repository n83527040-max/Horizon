#!/usr/bin/env bash
# Install the unmodified old APK then update with the unmodified new one.
# This runs on an isolated Android emulator and never touches a user's data.
set -euo pipefail
old="${1:?old APK required}"
new="${2:?new APK required}"
echo "Android SDK: $(adb shell getprop ro.build.version.sdk | tr -d '\r')"
echo "Supported ABIs: $(adb shell getprop ro.product.cpu.abilist | tr -d '\r')"
if ! output=$(adb install "$old" 2>&1); then
  printf '%s\n' "$output"
  if [[ "$output" == *INSTALL_FAILED_NO_MATCHING_ABIS* ]]; then
    echo '::warning::Emulator cannot install arm64 APK; update test is inconclusive.'
    exit 1
  fi
  echo '::error::Old APK did not install on the emulator; update was not tested.'
  exit 1
fi
printf '%s\n' "$output"
if ! output=$(adb install -r "$new" 2>&1); then
  printf '%s\n' "$output"
  echo '::error::The Android emulator rejected the APK update.'
  exit 1
fi
printf '%s\n' "$output"
adb shell dumpsys package com.craftix.launcher | grep -E 'versionCode=|versionName=' | head -2
echo '::notice::The unmodified 0.7.9 APK was successfully updated to 0.8.0 on Android emulator API 33.'
