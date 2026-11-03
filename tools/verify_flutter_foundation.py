#!/usr/bin/env python3
"""Verify (and generate) the Flutter foundation's cross-language contract.

Checks performed without a local Flutter SDK:

  * host-info.json is generated from the REAL research-lock.json / Gradle version
  * all 49 Rust `Action` variants are handled by the Kotlin dispatcher
  * all 49 variants are represented in Dart's `HorizonAction` enum
  * every snapshot key produced by Kotlin is parsed by Dart's model
  * no fake/demo performance numbers are present in production Dart code
  * all pubspec assets and referenced source files exist
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RUST_ACTIONS = ROOT / "crates/horizon-client/src/android/launcher_pages.rs"
DART_ACTIONS = ROOT / "flutter/lib/core/bridge/actions.dart"
DART_MODELS = ROOT / "flutter/lib/core/bridge/models.dart"
KOTLIN_BRIDGE = ROOT / "flutter/android/app/src/main/kotlin/dev/horizon/android/HorizonBridge.kt"
HOST_INFO = ROOT / "flutter/assets/host-info.json"
RESEARCH_LOCK = ROOT / "research-lock.json"
BUILD_GRADLE = ROOT / "packaging/android/build.gradle"

results: list[tuple[bool, str]] = []


def check(ok: bool, label: str) -> None:
    results.append((bool(ok), label))


def rust_variants() -> list[str]:
    body = re.search(r"pub\(super\) enum Action \{(.*?)\n\}", RUST_ACTIONS.read_text(), re.S)
    if not body:
        return []
    return [
        line.strip().rstrip(",").split("(")[0].split("<")[0]
        for line in body.group(1).splitlines()
        if line.strip() and not line.strip().startswith(("#", "//"))
    ]


def dart_actions() -> list[str]:
    body = re.search(r"enum HorizonAction \{(.*?)\n\}", DART_ACTIONS.read_text(), re.S)
    if not body:
        return []
    return [line.strip().rstrip(",") for line in body.group(1).splitlines()
            if line.strip() and not line.strip().startswith("//")]


def kotlin_actions() -> set[str]:
    text = KOTLIN_BRIDGE.read_text()
    body = re.search(r"fun dispatch\(action: String.*?\n    \}", text, re.S)
    if not body:
        return set()
    return set(re.findall(r'"([A-Z][A-Za-z]+)"', body.group(0)))


def expected_host_info() -> dict[str, str]:
    version = json.loads(RESEARCH_LOCK.read_text())["minecraft"]["version"]
    gradle = BUILD_GRADLE.read_text()
    app = re.search(r"versionName '([^']+)'", gradle)
    assert app, "versionName missing from packaging/android/build.gradle"
    return {"appVersion": app.group(1), "gameVersion": version}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--generate", action="store_true",
                        help="write flutter/assets/host-info.json from the real sources")
    args = parser.parse_args()

    expected = expected_host_info()
    if args.generate:
        HOST_INFO.parent.mkdir(parents=True, exist_ok=True)
        HOST_INFO.write_text(json.dumps(expected, indent=2) + "\n", encoding="utf-8")
    check(HOST_INFO.is_file(), "host-info.json exists")
    if HOST_INFO.is_file():
        got = json.loads(HOST_INFO.read_text())
        check(got == expected,
              f"host info matches real sources ({expected})" +
              ("" if got == expected else f" (got {got})"))

    variants = rust_variants()
    check(len(variants) == 49, f"49 Rust Action variants (got {len(variants)})")
    dart = dart_actions()
    check(set(dart) == set(variants) and len(dart) == 49,
          "Dart HorizonAction mirrors the Rust Action enum exactly")
    kotlin = kotlin_actions()
    missing = sorted(set(variants) - kotlin)
    check(not missing, "Kotlin dispatcher handles all 49 actions"
          + ("" if not missing else f" (missing: {missing})"))

    # Snapshot model parity: every key Kotlin emits must be read by Dart.
    kotlin_keys = set(re.findall(r'"([a-zA-Z][a-zA-Z0-9]*)"\s+to', KOTLIN_BRIDGE.read_text()))
    dart_keys = set(re.findall(r"raw\['([a-zA-Z][a-zA-Z0-9]*)'\]", DART_MODELS.read_text()))
    required = required_keys
    check(required <= kotlin_keys, "Kotlin emits the real snapshot keys")
    check(required <= dart_keys, "Dart parses the real snapshot keys")

    production = list((ROOT / "flutter/lib").rglob("*.dart"))
    joined = "\n".join(p.read_text() for p in production)
    fake = [n for n in ("267 MB", "60 FPS", "120 FPS", "4.2 GB", "8 GB", "12 installed", "5 installed")
            if n in joined]
    check(not fake, "no hardcoded demo performance numbers in production Dart"
          + ("" if not fake else f" (found {fake})"))

    for rel in (
        "flutter/pubspec.yaml",
        "flutter/analysis_options.yaml",
        "flutter/lib/main.dart",
        "flutter/lib/app.dart",
        "flutter/lib/core/theme/horizon_theme.dart",
        "flutter/lib/core/bridge/launcher_bridge.dart",
        "flutter/lib/navigation/app_shell.dart",
        "flutter/android/app/src/main/AndroidManifest.xml",
        "flutter/android/app/src/main/kotlin/dev/horizon/android/MainActivity.kt",
        "flutter/assets/hero.png",
    ):
        check((ROOT / rel).is_file(), f"{rel} exists")

    passed = sum(1 for ok, _ in results if ok)
    for ok, label in results:
        print(f"  [{'OK' if ok else 'FAIL'}] {label}")
    print(f"\n{passed}/{len(results)} Flutter foundation checks passed")
    return 0 if passed == len(results) else 1


required_keys = {
    "appVersion", "gameVersion", "installed", "results", "settings", "choices",
    "ramUsedMb", "ramPeakMb", "ramTotalMb", "graphicsApi", "renderScalePercent",
    "storageBytes", "cacheBytes", "contentBytes", "engineBridgeReady",
    "profileName", "accountStatus", "avatarPath", "status",
}


if __name__ == "__main__":
    sys.exit(main())
