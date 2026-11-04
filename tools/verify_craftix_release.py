#!/usr/bin/env python3
"""CRAFTIX release gate: branding, package ID, icon, signing policy, installer.

Source-level contract checks (NOT a claim of device validation). It also fails
if private signing material or secret VALUES are ever committed to the repo.
Run: python3 tools/verify_craftix_release.py [--report PATH]
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SECRETS = (
    "CRAFTIX_KEYSTORE_BASE64",
    "CRAFTIX_STORE_PASSWORD",
    "CRAFTIX_KEY_ALIAS",
    "CRAFTIX_KEY_PASSWORD",
)
EXPECTED_CERT = "77:05:07:A6:EC:48:51:98:66:55:08:29:32:CA:C2:81:61:55:08:0F:D4:73:DB:7E:42:E9:A0:BF:FD:22:DD:2F"

checks: list[tuple[bool, str]] = []


def check(ok: bool, message: str) -> None:
    checks.append((bool(ok), message))


def read(rel: str) -> str:
    path = ROOT / rel
    return path.read_text(encoding="utf-8") if path.is_file() else ""


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", type=Path)
    args = parser.parse_args()

    strings = read("flutter/lib/core/theme/horizon_strings.dart")
    check(strings.count("'appName': 'CRAFTIX'") == 2,
          "CRAFTIX app name in both en and ar catalogs")
    check("'appSubtitle': 'Minecraft Java • Android'" in strings,
          "subtitle is Minecraft Java • Android")
    check("BUILD • PLAY • EXPLORE" in strings, "BUILD • PLAY • EXPLORE tagline")
    check("HORIZON" not in strings.replace("horizon", ""),
          "no HORIZON branding in user-facing strings")

    gradle = read("packaging/android/build.gradle")
    checker = read("tools/prepare_mixed_flutter.py")
    check("namespace 'com.craftix.launcher'" in gradle and
          "'com.craftix.launcher'" in gradle,
          "Rust-only Gradle uses com.craftix.launcher")
    check("com.craftix.launcher" in checker,
          "Flutter Gradle patch uses com.craftix.launcher")
    check("versionCode 30" in gradle and "versionName '0.7.8'" in gradle,
          "versionCode 30 / versionName 0.7.8")
    check(read("flutter/pubspec.yaml").count("0.7.8+30") == 1,
          "Flutter pubspec version 0.7.8+30")
    check(json.loads(read("flutter/assets/host-info.json"))["appVersion"] == "0.7.8",
          "host-info.json generated from the real Gradle version")

    for manifest in ("flutter/android/app/src/main/AndroidManifest.xml",
                     "packaging/android/AndroidManifest.xml"):
        text = read(manifest)
        check('android:label="CRAFTIX"' in text, f"CRAFTIX app label in {manifest}")
        check("@mipmap/ic_launcher" in text, f"adaptive mipmap icon in {manifest}")

    for tree in ("flutter/android/app/src/main/res", "packaging/android/res"):
        densities = list((ROOT / tree).glob("mipmap-*/ic_launcher.png"))
        foregrounds = list((ROOT / tree).glob("mipmap-*/ic_launcher_foreground.png"))
        check(len(densities) == 5, f"5 legacy icon densities in {tree}")
        check(len(foregrounds) == 5, f"5 adaptive foreground densities in {tree}")
        check((ROOT / tree / "mipmap-anydpi-v26/ic_launcher.xml").is_file(),
              f"adaptive icon XML in {tree}")
        check("ic_launcher_background" in read(f"{tree}/values/ic_launcher_background.xml"),
              f"adaptive background colour in {tree}")

    # --- signing policy -----------------------------------------------------
    workflows = {
        p.name: p.read_text(encoding="utf-8")
        for p in (ROOT / ".github/workflows").glob("*.yml")
    }
    joined = "\n".join(workflows.values())
    for name in SECRETS:
        check(f"secrets.{name}" in joined, f"workflow references {name}")
    check("CRAFTIX_KEYSTORE_FILE" in gradle and "CRAFTIX_KEYSTORE_FILE" in checker,
          "Gradle reads the decoded keystore path from env, not the repo")
    check("throw new GradleException" in gradle,
          "release build fails clearly when CRAFTIX secrets are missing")
    check("signingConfig craftixKeystore ? signingConfigs.craftix : null" in gradle,
          "release never silently falls back to debug signing")
    resign = read("tools/resign_craftix_apk.sh")
    check("--v1-signing-enabled true" in resign and "--v2-signing-enabled true" in resign,
          "apksigner re-signs with v1 (JAR) + v2 using the SAME CRAFTIX key")
    check("CRAFTIX_KEYSTORE_FILE" in resign and "env:CRAFTIX_STORE_PASSWORD" in resign,
          "re-sign reads the permanent key from secrets/env, never from the repo")
    check(joined.count("tools/resign_craftix_apk.sh") >= 2,
          "both release workflows re-sign the final APK with v1+v2")
    check("No fallback debug" in gradle or "never fall back" in read("tools/android_build.py"),
          "debug fallback explicitly forbidden")
    check(EXPECTED_CERT in joined or EXPECTED_CERT in read("CRAFTIX_MIGRATION_ANALYSIS.md"),
          "expected public certificate fingerprint recorded")
    check("CRAFTIX_KEYSTORE_BASE64" not in gradle and
          "CRAFTIX_KEYSTORE_BASE64" not in read("flutter/android/app/src/main/kotlin/dev/horizon/android/HorizonEngine.kt"),
          "no keystore Base64 in Gradle/Dart/Kotlin source")

    # No private signing material is tracked by git.
    tracked = subprocess.check_output(
        ["git", "ls-files"], cwd=ROOT, text=True).splitlines()
    private = [p for p in tracked if re.search(r"\.(jks|keystore|base64)$|signing\.properties", p)]
    check(not private, f"no private signing material committed ({private})")
    gitignore = read(".gitignore")
    check(all(token in gitignore for token in ("*.jks", "*.keystore", "*.base64", "signing.properties")),
          ".gitignore covers keystores and signing property files")

    # --- real installer -----------------------------------------------------
    rust = read("crates/horizon-client/src/android/flutter_bridge.rs")
    engine_kt = read("flutter/android/app/src/main/kotlin/dev/horizon/android/HorizonEngine.kt")
    bridge_kt = read("flutter/android/app/src/main/kotlin/dev/horizon/android/HorizonBridge.kt")
    check("nativeInstallStart" in rust and "external fun nativeInstallStart" in engine_kt,
          "nativeInstallStart JNI symbol matches Kotlin")
    check("install_with_progress" in rust,
          "installer uses the engine's real official_install::install_with_progress")
    check("official_install::ready" in rust,
          "Download/Play state comes from the real readiness check")
    check("ensureEngineData" in bridge_kt,
          "Flutter host stages engine-data.zip before Download can run")
    check("bridge_rules::download_percent" not in rust and
          "rules::download_percent" in rust,
          "progress percentage comes from tested pure rules")
    check("fn download_percent" in read("crates/horizon-client/src/android/bridge_rules.rs"),
          "download percent rule has host-runnable Rust unit tests")

    dart_home = read("flutter/lib/features/home/home_page.dart")
    runner = read("flutter/lib/core/theme/craftix_runner_progress.dart")
    check("_ActionSlot" in dart_home and "_slotHeight" in dart_home,
          "Home has one fixed-size Download/Progress/Play action slot")
    check("installStart" in dart_home, "Download calls the real installStart bridge")
    check("disableAnimations" in runner, "runner respects reduced motion")
    check("AnimatedPositioned" in runner and "LinearProgressIndicator" in runner,
          "runner position is driven by real progress with indeterminate fallback")
    check(not re.search(r"Timer\.periodic|AnimationController\([^)]*repeat|\.repeat\(", runner),
          "no repeating timer animation (battery)")
    check(read("flutter/test/craftix_download_state_test.dart").count("testWidgets") >= 6,
          "real widget tests for all download states")

    results = [{"check": m, "passed": ok} for ok, m in checks]
    passed = sum(1 for ok, _ in checks if ok)
    report = {
        "passed": passed == len(checks),
        "checks": len(checks),
        "results": results,
        "expectedCertificateSha256": EXPECTED_CERT,
        "scope": "Source contract only; device/AVD execution is NOT tested here.",
    }
    print(json.dumps(report, indent=2, ensure_ascii=False))
    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"\n{passed}/{len(checks)} CRAFTIX release checks passed")
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
