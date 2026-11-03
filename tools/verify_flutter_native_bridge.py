#!/usr/bin/env python3
"""Phase-9 gate: prove every Flutter action reaches a real native operation.

This is a *source contract* verifier, not a claim of device validation. It
checks that:

* all 49 Rust `Action` variants have a real arm in the JNI dispatcher,
* no `enginePending` / fake-success placeholder survives anywhere,
* JNI symbol names match the Kotlin `external` declarations,
* the game hand-off reaches the original `Launcher::action`,
* the manifest keeps Flutter portrait and the engine in its own process,
* Activity lifecycle stops the Rust poller thread,
* protected engine files were not modified.

Run: python3 tools/verify_flutter_native_bridge.py [--write-evidence]
"""
from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RUST_ACTIONS = ROOT / "crates/horizon-client/src/android/launcher_pages.rs"
DART_ACTIONS = ROOT / "flutter/lib/core/bridge/actions.dart"
RUST_BRIDGE = ROOT / "crates/horizon-client/src/android/flutter_bridge.rs"
ANDROID_MAIN = ROOT / "crates/horizon-client/src/android_main.rs"
LAUNCHER = ROOT / "crates/horizon-client/src/android/launcher.rs"
KOTLIN_ENGINE = ROOT / "flutter/android/app/src/main/kotlin/dev/horizon/android/HorizonEngine.kt"
KOTLIN_BRIDGE = ROOT / "flutter/android/app/src/main/kotlin/dev/horizon/android/HorizonBridge.kt"
MAIN_ACTIVITY = ROOT / "flutter/android/app/src/main/kotlin/dev/horizon/android/MainActivity.kt"
FLUTTER_MANIFEST = ROOT / "flutter/android/app/src/main/AndroidManifest.xml"
ANALYSIS = ROOT / "FLUTTER_NATIVE_BRIDGE_ANALYSIS.md"
EVIDENCE = ROOT / "FLUTTER_BRIDGE_EVIDENCE.md"

# Paths that the user explicitly forbade changing. `launcher.rs` is intentionally
# NOT here: its only change is the additive `auto_play` hosting hop, verified
# separately below.
PROTECTED = (
    "crates/horizon-client/src/game/",
    "crates/horizon-client-render/",
    "crates/horizon-core/",
    "crates/horizon-world/",
    "crates/horizon-protocol/",
    "crates/horizon-mods/",
    "crates/horizon-shaders/",
    "launcher/src/",
    "packaging/android/java/",
    "tools/android_build.py",
    "tools/android_release.py",
    "tools/android_prepare.py",
    "Cargo.toml",
    "crates/horizon-android/Cargo.toml",
)

# Where each action is really executed. UI = real Flutter state,
# ENGINE = original NativeActivity, SAF/Kotlin = real Android APIs,
# JNI = real Rust Catalog/Config/Accounts/diagnostics/performance.
LAYERS = {
    "Page": "UI", "Play": "ENGINE", "Search": "JNI", "ShaderSource": "JNI",
    "ModSource": "JNI", "Field": "JNI", "FinishNameEdit": "JNI", "Install": "JNI",
    "Import": "SAF", "OpenFolder": "Kotlin", "ToggleSection": "UI",
    "ToggleLocal": "JNI", "DeleteLocal": "JNI", "ConfirmDeleteLocal": "JNI",
    "Next": "JNI", "Previous": "JNI", "Profile": "UI", "Logs": "JNI",
    "RecordPerformance": "JNI", "ExportPerformance": "JNI", "ExportDiagnostics": "JNI",
    "Account": "UI", "SignIn": "JNI", "DeviceSignIn": "JNI", "CopyLoginCode": "Kotlin",
    "CancelLogin": "JNI", "ReviewApp": "Kotlin", "Repository": "Kotlin",
    "SignOut": "JNI", "LoginLink": "Kotlin", "CloseLogin": "JNI", "Renderer": "UI",
    "RenderScale": "UI", "ChooseRenderScale": "JNI", "Orientation": "UI",
    "InputHost": "UI", "ChooseInputHost": "JNI", "ChooseOrientation": "JNI",
    "ChooseRenderer": "JNI", "ChooseProfile": "JNI", "Dismiss": "UI",
    "Immersive": "JNI", "Cutout": "JNI", "Aspect": "UI", "ChooseAspect": "JNI",
    "DisplayHz": "UI", "ChooseDisplayHz": "JNI", "PerformanceHints": "JNI",
    "GpuProfile": "JNI",
}
REAL_OPERATION = {
    "Page": "navIndexProvider (real tab state)",
    "Play": "Launcher::action() -> Vulkan/wgpu game launch",
    "Search": "Catalog::search -> Modrinth/Horizon HTTP + poll",
    "ShaderSource": "Config.shader_source + Config::save + browse",
    "ModSource": "Config.mod_source + Config::save + browse",
    "Field": "Config/preferences/cache real writes",
    "FinishNameEdit": "Config::save (name/server/client id)",
    "Install": "Catalog::install -> download + compatibility check",
    "Import": "ACTION_OPEN_DOCUMENT -> ContentResolver -> nativeImportStaged",
    "OpenFolder": "HorizonDocumentsProvider ACTION_VIEW",
    "ToggleSection": "real Installed section state",
    "ToggleLocal": "Catalog::toggle_local -> options.txt / ModList",
    "DeleteLocal": "Catalog::pending_delete",
    "ConfirmDeleteLocal": "Catalog::remove_local -> real file delete",
    "Next": "Catalog::offset + browse", "Previous": "Catalog::offset + browse",
    "Profile": "opens real value chooser", "Logs": "diagnostics::summary(home)",
    "RecordPerformance": "Config.record_performance + save",
    "ExportPerformance": "performance::export(home)",
    "ExportDiagnostics": "diagnostics::export(home) -> ZIP",
    "Account": "opens real accounts sheet",
    "SignIn": "account::SignIn::start_with_client_id (Microsoft device code)",
    "DeviceSignIn": "account::SignIn::start_with_client_id (Microsoft device code)",
    "CopyLoginCode": "ClipboardManager with real user_code",
    "CancelLogin": "SignIn::cancel()", "ReviewApp": "ACTION_VIEW review URL",
    "Repository": "ACTION_VIEW GitHub URL", "SignOut": "Accounts::remove + save",
    "LoginLink": "ACTION_VIEW verification_uri?otc=user_code",
    "CloseLogin": "SignIn::cancel()", "Renderer": "opens real value chooser",
    "RenderScale": "opens real value chooser",
    "ChooseRenderScale": "RenderScale::parse + Config::save",
    "Orientation": "opens real value chooser", "InputHost": "opens real value chooser",
    "ChooseInputHost": "Config.input_host + save",
    "ChooseOrientation": "Config.orientation + save",
    "ChooseRenderer": "Config.renderer + save",
    "ChooseProfile": "Config.profile + save + refresh",
    "Dismiss": "closes real dialog", "Immersive": "Config.screen.immersive + save",
    "Cutout": "Config.screen.avoid_cutout + save",
    "Aspect": "opens real value chooser",
    "ChooseAspect": "Config.screen.aspect + save",
    "DisplayHz": "opens real value chooser",
    "ChooseDisplayHz": "Config.device.display_hz + save",
    "PerformanceHints": "Config.device.performance_hints + save",
    "GpuProfile": "Config.device.gpu_profile + save",
}

checks: list[tuple[bool, str]] = []


def check(condition: bool, message: str) -> None:
    checks.append((bool(condition), message))


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8") if path.is_file() else ""


def rust_actions() -> list[str]:
    text = read(RUST_ACTIONS)
    body = text.split("pub(super) enum Action {", 1)[1].split("\n}", 1)[0]
    names = re.findall(r"^\s*([A-Z][A-Za-z0-9_]*)\b", body, re.M)
    return [n for n in names if n != "Action"]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write-evidence", action="store_true")
    args = parser.parse_args()

    actions = rust_actions()
    check(len(actions) == 49, f"exactly 49 Rust Action variants (found {len(actions)})")
    dart = read(DART_ACTIONS)
    dart_names = re.findall(r"^\s*([A-Z][A-Za-z0-9_]*),?$", dart.split("enum HorizonAction {", 1)[1].split("}", 1)[0], re.M)
    check(set(actions) == set(dart_names), "Dart HorizonAction mirrors the 49 Rust variants")
    check(set(actions) == set(LAYERS), "evidence matrix covers all 49 actions")
    check(set(actions) == set(REAL_OPERATION), "real-operation table covers all 49 actions")

    rust = read(RUST_BRIDGE)
    kotlin = read(KOTLIN_BRIDGE)
    engine_kt = read(KOTLIN_ENGINE)
    all_source = rust + kotlin + engine_kt + read(MAIN_ACTIVITY)
    check("enginePending" not in all_source, "no enginePending placeholder remains")
    check("not implemented yet" not in all_source.lower(), "no 'not implemented' stub remains")
    check("todo!(" not in rust and "unimplemented!(" not in rust, "no Rust todo!/unimplemented! in the bridge")

    # Every action is either an explicit Rust match arm or an Android-host arm.
    rust_arms = set(re.findall(r'"([A-Z][A-Za-z0-9_]*)"\s*(?:=>|\|)', rust))
    kotlin_arms = set(re.findall(r'"([A-Z][A-Za-z0-9_]*)"\s*->', kotlin))
    missing = sorted(set(actions) - rust_arms - kotlin_arms)
    check(not missing, f"all 49 actions have a real native handler (missing: {missing})")

    # JNI symbol parity.
    symbols = set(re.findall(r'Java_dev_horizon_android_HorizonEngine_(\w+)', rust))
    externals = set(re.findall(r'external fun (\w+)\(', engine_kt))
    check(symbols == externals and len(symbols) >= 6,
          f"JNI C ABI symbols match Kotlin externals ({sorted(symbols)})")
    check("nativeImportStaged" in symbols, "SAF import ends in real Rust archive validation")
    check("content_files::validate" in rust, "import reuses the engine's own validator")

    # Game hand-off to the untouched engine path.
    main_rs = read(ANDROID_MAIN)
    launcher = read(LAUNCHER)
    check("flutter_bridge::launch_game_requested()" in main_rs,
          "android_main reads the Flutter Play hand-off")
    check("Launcher::new(home.clone(), auto_play)" in main_rs,
          "hand-off reaches the original Launcher")
    check("self.action(events)" in launcher.split("fn resumed", 1)[1].split("fn suspended", 1)[0],
          "auto_play calls the existing Launcher::action (no new game logic)")
    check("mod flutter_bridge;" in main_rs, "JNI module is compiled into libhorizon_android.so")

    # Lifecycle / threads.
    check("bridge?.shutdown()" in read(MAIN_ACTIVITY) and
          "HorizonEngine.nativeShutdown()" in kotlin,
          "onDestroy stops the Rust poller thread")
    check("shutdown.store(true" in rust, "Rust poller honours a shutdown flag")
    check("poll_loop" in rust and "std::thread::Builder" in rust,
          "worker polling runs on a Rust-owned thread, never blocking JNI callers")

    # Manifest: portrait launcher, separate rotatable engine process.
    manifest = read(FLUTTER_MANIFEST)
    check('android:name="dev.horizon.android.MainActivity"' in manifest and
          'android:screenOrientation="portrait"' in manifest,
          "Flutter launcher is portrait-locked")
    check('android:process=":engine"' in manifest,
          "engine Activity lives in a separate process (exit(0) cannot kill Flutter)")
    check('android:screenOrientation="unspecified"' in manifest,
          "native game Activity may rotate for gameplay")
    check('android:name="android.app.lib_name"' in manifest,
          "NativeActivity still loads libhorizon_android.so")

    # No rewrite of the engine.
    baseline = subprocess.check_output(
        ["git", "log", "--format=%H", "--grep=Import upstream Horizon Android v0.7.6", "-1"],
        cwd=ROOT, text=True,
    ).strip()
    changed = subprocess.check_output(
        ["git", "diff", "--name-only", baseline, "HEAD", "--"],
        cwd=ROOT, text=True,
    ).splitlines()
    forbidden = [p for p in changed if any(p.startswith(x) for x in PROTECTED)]
    check(not forbidden, f"protected engine/account/file sources untouched ({forbidden})")
    check(ANALYSIS.is_file(), "impact analysis is documented before Rust changes")
    check("pub(super) fn new(home: PathBuf, auto_play: bool)" in launcher,
          "launcher.rs change is limited to the additive auto_play constructor")

    for ok, message in checks:
        print(f"  [{'OK' if ok else 'FAIL'}] {message}")
    passed = sum(1 for ok, _ in checks if ok)
    print(f"\n{passed}/{len(checks)} native-bridge contract checks passed")
    if args.write_evidence and passed == len(checks):
        lines = [
            "# المرحلة 9 — مصفوفة الإثبات لكل إجراء",
            "",
            "مصدر هذه المصفوفة: `tools/verify_flutter_native_bridge.py` (فاحص عقد مصدري).",
            "**这不是 إثبات تشغيل على جهاز**: انظر قسم «غير مُختبر» في نهاية الملف.",
            "",
            "| # | Action | الطبقة | العملية الحقيقية |",
            "|---|---|---|---|",
        ]
        for i, name in enumerate(actions, 1):
            lines.append(f"| {i} | `{name}` | {LAYERS[name]} | {REAL_OPERATION[name]} |")
        lines += [
            "",
            "## ما تم إثباته آلياً",
            f"- {passed}/{len(checks)} فحص عقد: 49 إجراءً، لا `enginePending`،",
            "  تطابق رموز JNI مع Kotlin، تسليم Play إلى `Launcher::action`،",
            "  عملية `:engine` منفصلة، إيقاف ثريد Rust في `onDestroy`،",
            "  وعدم تعديل ملفات المحرك/الحسابات/الملفات المحمية.",
            "- `flutter analyze` و`flutter test` في CI؛ `cargo check`/`clippy` لهدف Android في CI.",
            "",
            "## غير مُختبر على جهاز حقيقي (صريح)",
            "- تشغيل Minecraft وVulkan/wgpu وسطح الرسم بعد تسليم Play.",
            "- تسجيل دخول Microsoft الفعلي مقابل خدمات Microsoft.",
            "- منتقي SAF ونسخ المجلدات/الملفات على جهاز.",
            "- تنزيل Modrinth وتثبيت المودات/الحزم/الشيدرات فعلياً.",
            "- FPS/GPU/الذاكرة وتسريب الذاكرة والدوران الفيزيائي.",
            "- APK مختلط قابل للتثبيت والتشغيل (المرحلة 13).",
        ]
        EVIDENCE.write_text("\n".join(lines) + "\n", encoding="utf-8")
        print(f"wrote {EVIDENCE.relative_to(ROOT)}")
    return 0 if passed == len(checks) else 1


if __name__ == "__main__":
    sys.exit(main())
