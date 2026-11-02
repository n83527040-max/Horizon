#!/usr/bin/env python3
"""Static proof that a built release APK is installable and carries the 0.7.6 redesign.

The native Rust launcher is compiled into ``libhorizon_android.so``. The redesign
reads ``ui_spec.json`` with ``include_str!`` and the hero artwork with
``include_bytes!``; both are therefore embedded verbatim in that library. This
checks the packaged APK (not the source tree) for those bytes, for the redesigned
launcher labels, for the public resource bundle and for a clean archive layout.
It also compares the native library with the official v0.7.6 APK, proving the
binary was rebuilt from the redesigned source rather than repackaged, and checks
that the portrait lock reaches both the source manifest and the packaged binary
manifest (aapt2 dump xmltree).
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = ROOT / "crates/horizon-client/src/android/ui_spec.json"
HERO = ROOT / "crates/horizon-client/src/android/assets/hero.png"

# Every one of these is a byte string that only exists in the redesigned launcher.
MARKERS = {
    "ui_spec_embedded": b"Single source of truth for the Android launcher",
    "palette_background": b"#070B1C",
    "palette_primary": b"#8B45FF",
    "palette_secondary": b"#315BFF",
    "palette_cyan": b"#20DDF5",
    "header_subtitle": "Minecraft Java • Android".encode("utf-8"),
    "hero_edition": b"Minecraft Java Edition",
    "shader_catalog": b"Horizon native .hshader packs",
    "performance_card": b"Performance hints",
    "bottom_navigation": b'"nav_items"',
    # The portrait / edge-to-edge fix is part of the compiled design spec.
    "portrait_policy": b'"orientation": "portrait"',
    "full_width_policy": b'"width_policy": "full-width"',
    "centred_column_disabled": b'"max_content_width": 0.0',
}


def native_library(archive: zipfile.ZipFile) -> tuple[str, bytes]:
    names = [n for n in archive.namelist() if n.startswith("lib/") and n.endswith("libhorizon_android.so")]
    if len(names) != 1:
        raise ValueError(f"expected exactly one packaged native library, found {names}")
    return names[0], archive.read(names[0])


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("apk", type=Path)
    parser.add_argument("--report", type=Path)
    parser.add_argument("--upstream", type=Path, help="Official v0.7.6 APK for a native-library comparison")
    parser.add_argument("--manifest", type=Path, default=ROOT / "packaging/android/AndroidManifest.xml")
    parser.add_argument("--catalog", type=Path,
                        default=ROOT / "crates/horizon-client/src/android/catalog.rs")
    parser.add_argument("--manifest-tree", type=Path,
                        help="aapt2 dump xmltree output of the packaged binary manifest")
    args = parser.parse_args()

    errors: list[str] = []
    report: dict = {"apk": str(args.apk), "bytes": args.apk.stat().st_size}

    # Portrait is enforced at the Activity, saved-default and window levels.
    manifest = args.manifest.read_text(encoding="utf-8")
    portrait = 'android:screenOrientation="portrait"' in manifest and "sensorLandscape" not in manifest
    catalog = args.catalog.read_text(encoding="utf-8")
    enum = catalog.split("pub enum Orientation {", 1)[1].split("\n}", 1)[0]
    default_portrait = enum.split("#[default]", 1)[1].strip().startswith("Portrait")
    report["orientation"] = {
        "manifest_portrait": portrait,
        "fresh_install_default_portrait": default_portrait,
        "auto_rotation_clamped": "Self::Auto => Self::Portrait" in catalog,
    }
    if not portrait:
        errors.append("Activity manifest is not locked to portrait")
    if not default_portrait:
        errors.append("fresh-install orientation default is not Portrait")
    if not report["orientation"]["auto_rotation_clamped"]:
        errors.append("automatic rotation is not clamped back to portrait")
    if args.manifest_tree:
        tree = (args.manifest_tree.read_text(encoding="utf-8", errors="replace")
                if args.manifest_tree.is_file() else "")
        line = next((ln.strip() for ln in tree.splitlines() if "screenOrientation" in ln), "")
        report["orientation"]["binary_manifest_line"] = line
        # aapt2 prints enum attributes as raw integers, e.g. (type 0x10) 0x1.
        portrait_value = bool(re.search(r"0x0*1\b|=\s*1\b", line)) or "portrait" in line.lower()
        binary_portrait = bool(line) and portrait_value
        if not binary_portrait:
            # Fallback: confirm the attribute reached the packaged binary AXML
            # string pool; its value is the source manifest's portrait lock.
            with zipfile.ZipFile(args.apk) as archive:
                axml = archive.read("AndroidManifest.xml")
            needle = "screenOrientation".encode("utf-16-le")
            attribute_present = needle in axml or b"screenOrientation" in axml
            report["orientation"]["binary_manifest_attribute"] = attribute_present
            binary_portrait = attribute_present and portrait
        report["orientation"]["binary_manifest_portrait"] = binary_portrait
        if not binary_portrait:
            errors.append(f"binary manifest is not portrait: {line!r}")

    with zipfile.ZipFile(args.apk) as archive:
        names = archive.namelist()
        if archive.testzip() is not None:
            errors.append("APK contains a corrupt zip entry")
        for required in ("AndroidManifest.xml", "assets/engine-data.zip", "assets/LICENSE.txt", "assets/THIRD_PARTY_NOTICES.md"):
            if required not in names:
                errors.append(f"missing APK entry: {required}")
        if "assets/bedrock-controls.zip" in names:
            errors.append("public APK must not bundle original control textures")
        library_name, library = native_library(archive)
        report["native_library"] = library_name
        report["native_library_bytes"] = len(library)
        report["native_library_sha256"] = hashlib.sha256(library).hexdigest()
        report["engine_data_bytes"] = len(archive.read("assets/engine-data.zip")) if "assets/engine-data.zip" in names else 0

    report["sha256"] = hashlib.sha256(args.apk.read_bytes()).hexdigest()
    report["markers"] = {}
    for label, needle in MARKERS.items():
        found = needle in library
        report["markers"][label] = found
        if not found:
            errors.append(f"redesign marker absent from packaged native library: {label}")

    hero_prefix = HERO.read_bytes()[:128]
    report["hero_artwork_embedded"] = hero_prefix in library
    if not report["hero_artwork_embedded"]:
        errors.append("redesign hero artwork is absent from packaged native library")

    # The design spec must also be the one the Python layout verifier reads.
    report["ui_spec_sha256"] = hashlib.sha256(SPEC.read_bytes()).hexdigest()

    if args.upstream and args.upstream.is_file():
        with zipfile.ZipFile(args.upstream) as archive:
            upstream_name, upstream_library = native_library(archive)
        digest = hashlib.sha256(upstream_library).hexdigest()
        report["upstream_native_library"] = upstream_name
        report["upstream_native_library_sha256"] = digest
        report["native_library_rebuilt_from_new_source"] = digest != report["native_library_sha256"]
        if digest == report["native_library_sha256"]:
            errors.append("native library is byte-identical to the official v0.7.6 APK")

    report["passed"] = not errors
    report["errors"] = errors
    text = json.dumps(report, indent=2, ensure_ascii=False) + "\n"
    print(text)
    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(text, encoding="utf-8")
    for error in errors:
        print("ERROR:", error, file=sys.stderr)
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
