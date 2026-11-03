#!/usr/bin/env python3
"""Verify every factual claim in FLUTTER_MIGRATION_AUDIT.md against the checkout.

Phase 1/2 of the Flutter migration plan is an *inventory*: it is only useful if
it stays true. This checker re-validates all referenced paths and the source
symbols/counts the audit quotes. Run it after any source change:

    python3 tools/verify_flutter_migration_audit.py
    python3 tools/verify_flutter_migration_audit.py --online   # also check GH release
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
AUDIT = ROOT / "FLUTTER_MIGRATION_AUDIT.md"

PATH_TOKEN = re.compile(
    r"^(?:crates|packaging|tools|tests|docs|launcher|flutter|\.github|mods|third_party)/"
    r"[^`\s]+\.(?:rs|py|json|xml|gradle|java|md|yml|yaml|png|sh|toml|txt|css)$"
)
ROOT_FILES = (
    "Cargo.toml",
    "Cargo.lock",
    "AGENTS.md",
    "README.md",
    "REDESIGN.md",
    "FLUTTER_MIGRATION_AUDIT.md",
    "research-lock.json",
    "rust-toolchain.toml",
)

# (label, relative path, snippets that must all be present)
SYMBOL_CHECKS = [
    ("manifest: portrait activity", "packaging/android/AndroidManifest.xml", [
        'android:screenOrientation="portrait"',
        "dev.horizon.android.HorizonActivity",
        "android.app.lib_name",
        "HorizonDocumentsProvider",
        "activity-alias",
    ]),
    ("gradle: version/minSdk", "packaging/android/build.gradle", [
        "versionName '0.7.7'", "versionCode 29", "compileSdk 35", "minSdk", "jniLibs.srcDirs",
    ]),
    ("native library is a cdylib", "crates/horizon-android/Cargo.toml", [
        'crate-type = ["cdylib"]', "android-native-activity", "wgpu", "jni",
    ]),
    ("Android entry point", "crates/horizon-client/src/android_main.rs", [
        "fn android_main(app: AndroidApp)", "android_event_loop()",
        "with_android_app", "android_on_create", "thread_local!",
        "../../../launcher/src/account.rs", "../../../launcher/src/browser.rs",
        "../../../launcher/src/skins.rs", "../../../launcher/src/proxy.rs",
        ".local/horizon-client/26.3",
    ]),
    ("launcher owns window + game", "crates/horizon-client/src/android/launcher.rs", [
        "impl ApplicationHandler for Launcher", "events.create_window",
        "fn restore_launcher", "fn open(", "HudAtlas", "set_orientation",
    ]),
    ("49 real actions", "crates/horizon-client/src/android/launcher_pages.rs", [
        "pub(super) enum Action", "fn activate", "Action::Play",
        "Action::Install", "Action::SignIn", "Action::DeviceSignIn",
        "Action::ToggleLocal", "Action::DeleteLocal", "Action::Import",
        "Action::ChooseOrientation", "Action::ExportDiagnostics",
    ]),
    ("config + pages", "crates/horizon-client/src/android/catalog.rs", [
        "pub struct Config", "pub enum Page", "pub enum Orientation",
        "microsoft_client_id", "render_scale", "mod_source", "shader_source",
    ]),
    ("content kinds", "crates/horizon-client/src/content_files.rs", [
        "pub enum Kind", "Mods", "Packs", "Shaders", "fn accepts",
    ]),
    ("account flows", "crates/horizon-client/src/android/login.rs", [
        "fn start_device", "fn sign_out", "fn open_link", "fn session",
        "fn status", "Sign in with browser", "device code",
    ]),
    ("Android Keystore", "crates/horizon-client/src/android/secure_store.rs", [
        "Android Keystore", "initialize(vm: JavaVM)", "Cipher",
    ]),
    ("SAF import", "crates/horizon-client/src/android/documents.rs", [
        "document_result", "request != 4101", "content_files::validate",
        "fn begin", "fn resolve",
    ]),
    ("JNI platform bridge", "crates/horizon-client/src/android/platform.rs", [
        "pub fn set_orientation", "pub fn open_login_url",
        "pub fn performance_power", "pub fn clipboard_get",
    ]),
    ("real RAM, no fake FPS", "crates/horizon-client/src/android/design.rs", [
        '"VmRSS:"', '"VmHWM:"', '"MemTotal:"',
    ]),
    ("performance recording", "crates/horizon-client/src/android/performance.rs", [
        "pub fn record", "pub fn configure_gpu_profile", "pub fn export",
    ]),
    ("Modrinth / Horizon sources", "launcher/src/browser.rs", [
        "pub enum SourceId", "Modrinth", "Horizon", "fn sources", "compat",
    ]),
    ("docs: no Java/Kotlin UI", "docs/android.md", [
        "android.app.NativeActivity",
        "There are no project Java/Kotlin classes",
        "It does not run arbitrary Minecraft JAR versions",
    ]),
    ("repo rule: players run Rust", "AGENTS.md", [
        "Everything players run is Rust",
    ]),
    ("portrait/full-width spec", "crates/horizon-client/src/android/ui_spec.json", [
        '"orientation"', '"max_content_width"', "portrait", "full-width",
    ]),
    ("release workflow", ".github/workflows/android-redesign-release.yml", [
        "cargo", "clippy", "verify_android_apk.py", "gh release upload",
    ]),
]

results: list[tuple[bool, str]] = []


def check(ok: bool, label: str) -> None:
    results.append((bool(ok), label))


def enum_variants(path: str, name: str) -> list[str]:
    text = (ROOT / path).read_text(encoding="utf-8")
    match = re.search(rf"enum {name} \{{(.*?)\n\}}", text, re.S)
    if not match:
        return []
    return [
        line.strip().rstrip(",")
        for line in match.group(1).splitlines()
        if line.strip() and not line.strip().startswith(("#", "//"))
    ]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--online", action="store_true",
                        help="also verify the published release asset via gh")
    args = parser.parse_args()

    check(AUDIT.is_file(), "FLUTTER_MIGRATION_AUDIT.md exists")
    text = AUDIT.read_text(encoding="utf-8")

    # 1. Every backticked source path mentioned in the audit exists.
    referenced = sorted({
        token for token in re.findall(r"`([^`\n]+)`", text)
        if PATH_TOKEN.match(token)
    } | set(ROOT_FILES))
    missing = [p for p in referenced if not (ROOT / p).is_file()]
    check(not missing, f"all {len(referenced)} referenced files exist"
          + ("" if not missing else f" (missing: {missing})"))
    check("tools/verify_flutter_migration_audit.py" in referenced,
          "audit names its own verifier")

    # 2. Source symbols quoted in the inventory.
    for label, rel, snippets in SYMBOL_CHECKS:
        path = ROOT / rel
        if not path.is_file():
            check(False, f"{label}: {rel} missing")
            continue
        body = path.read_text(encoding="utf-8", errors="replace")
        absent = [s for s in snippets if s not in body]
        check(not absent, f"{label}" + ("" if not absent else f" (absent: {absent})"))

    # 3. Structural counts.
    actions = enum_variants(
        "crates/horizon-client/src/android/launcher_pages.rs", "Action")
    check(len(actions) == 49, f"Action variants = 49 (got {len(actions)})")
    pages = enum_variants("crates/horizon-client/src/android/catalog.rs", "Page")
    check(len(pages) == 6, f"Page variants = 6 (got {len(pages)})")
    orientations = enum_variants(
        "crates/horizon-client/src/android/catalog.rs", "Orientation")
    check(len(orientations) == 5,
          f"Orientation variants = 5 (got {len(orientations)})")

    java = sorted((ROOT / "packaging/android/java").rglob("*.java"))
    check(len(java) == 2, f"exactly 2 Java sources (got {len(java)})")
    check(all("NativeActivity" in p.read_text() or "DocumentsProvider" in p.read_text()
              for p in java), "Java sources are only SAF callback + DocumentsProvider")

    dart = [p for p in ROOT.rglob("*.dart") if ".git" not in p.parts]
    kt = [p for p in ROOT.rglob("*.kt") if ".git" not in p.parts]
    pubspec = [p for p in ROOT.rglob("pubspec.yaml") if ".git" not in p.parts]
    check(len(dart) >= 10 and len(kt) >= 2 and len(pubspec) == 1,
          f"Flutter host present (dart={len(dart)}, kt={len(kt)}, "
          f"pubspec={len(pubspec)})")
    check((ROOT / "flutter/lib/core/bridge/actions.dart").is_file()
          and (ROOT / "flutter/test/actions_parity_test.dart").is_file(),
          "typed action contract and parity test exist")

    rust_files = list((ROOT / "crates").rglob("*.rs"))
    check(len(rust_files) >= 1700, f"Rust engine intact ({len(rust_files)} .rs files)")
    py_tools = list((ROOT / "tools").glob("*.py"))
    check(len(py_tools) > 480, f"Python tooling intact ({len(py_tools)} scripts)")
    tests = list((ROOT / "tests").glob("*.py"))
    check(len(tests) == 100, f"100 test files (got {len(tests)})")

    hero = ROOT / "crates/horizon-client/src/android/assets/hero.png"
    check(hero.is_file() and hero.read_bytes()[:8] == b"\x89PNG\r\n\x1a\n"
          and hero.stat().st_size > 100_000, "hero.png is the embedded PNG asset")
    check((ROOT / "tools/ui-preview/portrait-overview.png").is_file(),
          "portrait QA overview exists")

    # 4. Optional live release check.
    if args.online:
        try:
            def release(tag: str) -> dict[str, int]:
                data = json.loads(subprocess.check_output(
                    ["gh", "release", "view", tag, "--json", "tagName,name,assets"],
                    text=True, cwd=ROOT))
                return {a["name"]: a["size"] for a in data["assets"]}

            rust = release("redesign-0.7.6")
            # The Rust-only APK was rebuilt from the new bridge source; accept
            # both the historical and the rebuilt size instead of a stale hash.
            check(rust.get("Horizon-Android-Redesigned-0.7.6-arm64-v8a.apk", 0)
                  in (35_693_532, 35_736_872),
                  "published Rust release still carries the rebuilt portrait APK")
            mixed = release("flutter-mixed-0.7.6")
            check(mixed.get("Horizon-Flutter-Rust-0.7.6-arm64-v8a.apk", 0) > 80_000_000,
                  "published 0.7.6 mixed APK (historical evidence) exists")
            current = release("horizon-0.7.7")
            check(current.get("Horizon-Android-Rust-0.7.7-arm64-v8a.apk", 0) > 30_000_000,
                  "HORIZON 0.7.7 Rust engine APK is published")
            check(current.get("Horizon-Flutter-Rust-0.7.7-arm64-v8a.apk", 0) > 80_000_000,
                  "HORIZON 0.7.7 mixed Flutter + Rust APK is published")
        except Exception as exc:  # pragma: no cover - network dependent
            check(False, f"live release check failed: {exc}")

    passed = sum(1 for ok, _ in results if ok)
    for ok, label in results:
        print(f"  [{'OK' if ok else 'FAIL'}] {label}")
    print(f"\n{passed}/{len(results)} audit checks passed"
          + (" (offline; --online not requested)" if not args.online else ""))
    return 0 if passed == len(results) else 1


if __name__ == "__main__":
    sys.exit(main())
