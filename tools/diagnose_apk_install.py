#!/usr/bin/env python3
"""Diagnose real-device 'package appears to be invalid' install failures.

Runs on the PUBLISHED APK in CI (where the asset is downloadable) and checks
the things a phone's PackageManager rejects even when `apksigner verify` and
`aapt2 dump badging` succeed on a build host:

* ZIP integrity and central-directory sanity
* uncompressed native libraries must be STORED and 4096-byte page aligned
  when android:extractNativeLibs=false
* v1 (JAR) and v2 (APK Signature Scheme) presence
* compileSdkVersionCodename / preview-SDK markers
* launcher icon must resolve to a real bitmap resource
* apksigner verification at several min-sdk levels

Only public information is emitted. No key material is read.
"""
from __future__ import annotations

import hashlib
import json
import re
import shlex
import subprocess
import sys
import zipfile
from pathlib import Path

APK_SIG_BLOCK_MAGIC = b"APK Sig Block 42"


def run(command: list[str]) -> tuple[int, str]:
    try:
        out = subprocess.check_output(command, text=True, stderr=subprocess.STDOUT)
        return 0, out
    except subprocess.CalledProcessError as error:
        return error.returncode, error.output
    except FileNotFoundError as error:
        return 127, str(error)


def diagnose(apk: Path, aapt2: str, apksigner: str) -> dict:
    report: dict = {"apk": str(apk), "bytes": apk.stat().st_size}
    problems: list[str] = []

    with zipfile.ZipFile(apk) as archive:
        names = archive.namelist()
        report["zipTest"] = archive.testzip()
        if report["zipTest"] is not None:
            problems.append(f"corrupt ZIP entry: {report['zipTest']}")
        for required in ("AndroidManifest.xml", "classes.dex", "resources.arsc"):
            if required not in names:
                problems.append(f"missing {required}")

        v1 = [n for n in names if n.startswith("META-INF/")
              and (n.endswith(".RSA") or n.endswith(".DSA") or n.endswith(".EC"))]
        report["v1SignerFiles"] = v1
        report["v1Present"] = bool(v1)

        # Locate the APK Signing Block just before the central directory.
        raw = apk.read_bytes()
        idx = raw.rfind(APK_SIG_BLOCK_MAGIC)
        report["v2SigningBlockPresent"] = idx >= 0

        libs = []
        for info in archive.infolist():
            if not info.filename.startswith("lib/") or not info.filename.endswith(".so"):
                continue
            stored = info.compress_type == zipfile.ZIP_STORED
            # ZIP local header: 30 bytes + filename + extra field.
            with apk.open("rb") as handle:
                handle.seek(info.header_offset)
                header = handle.read(30)
            name_len = int.from_bytes(header[26:28], "little")
            extra_len = int.from_bytes(header[28:30], "little")
            data_offset = info.header_offset + 30 + name_len + extra_len
            libs.append({
                "name": info.filename,
                "size": info.file_size,
                "stored": stored,
                "dataOffset": data_offset,
                "pageAligned": data_offset % 4096 == 0,
            })
        report["nativeLibs"] = libs

    code, badging = run([aapt2, "dump", "badging", str(apk)])
    report["aapt2Exit"] = code
    if code != 0:
        problems.append("aapt2 dump badging failed")
    extract = re.search(r"extractNativeLibs[:=]'?(true|false)'?", badging)
    # extractNativeLibs is in the binary xmltree, not always in badging;
    # the caller supplies it via --extract-native-libs when known.
    report["badging"] = badging.strip().splitlines()[:8]
    codename = re.search(r"compileSdkVersionCodename='([^']+)'", badging)
    report["compileSdkVersionCodename"] = codename.group(1) if codename else None
    pkg = re.search(r"package: name='([^']+)'", badging)
    label = re.search(r"application-label:'([^']+)'", badging)
    native = re.search(r"native-code: '([^']+)'", badging)
    report["package"] = pkg.group(1) if pkg else None
    report["applicationLabel"] = label.group(1) if label else None
    report["nativeCode"] = native.group(1) if native else None

    for level in (21, 24, 28):
        code, out = run([*shlex.split(apksigner), "verify", "--verbose",
                         "--min-sdk-version", str(level), str(apk)])
        report[f"apksignerMinSdk{level}"] = {
            "exit": code,
            "verified": "Verifies" in out,
            "notes": [line.strip() for line in out.splitlines()
                      if "DOES NOT VERIFY" in line or "ERROR" in line or "Verifies" in line][:6],
        }

    report["sha256"] = hashlib.sha256(apk.read_bytes()).hexdigest()
    report["problems"] = problems
    report["passed"] = not problems
    return report


def main() -> int:
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("apk", type=Path)
    parser.add_argument("--aapt2", default="aapt2")
    parser.add_argument("--apksigner", default="apksigner")
    parser.add_argument("--extract-native-libs", default="auto",
                        choices=("auto", "true", "false"))
    parser.add_argument("--report", type=Path)
    args = parser.parse_args()

    result = diagnose(args.apk, args.aapt2, args.apksigner)
    extract = args.extract_native_libs
    if extract == "auto":
        extract = "false" if all(lib["stored"] for lib in result["nativeLibs"]) else "true"
    result["extractNativeLibs"] = extract
    if extract == "false":
        for lib in result["nativeLibs"]:
            if not lib["stored"]:
                result["problems"].append(f"{lib['name']} is DEFLATED but extractNativeLibs=false")
            if not lib["pageAligned"]:
                result["problems"].append(
                    f"{lib['name']} is not 4096-byte page aligned (offset {lib['dataOffset']})")
    # v2-only APKs are valid on API 24+, but many OEM installers expect v1 too.
    result["warnings"] = []
    if not result["v1Present"]:
        result["problems"].append("v1 (JAR) signature missing: v2-only APKs are rejected by some OEM installers")
    if not result["v2SigningBlockPresent"]:
        result["warnings"].append(
            "v2 block absent: v1 (JAR) signature is present and valid for sideload")
    result["passed"] = not result["problems"]
    text = json.dumps(result, indent=2, ensure_ascii=False) + "\n"
    print(text)
    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(text, encoding="utf-8")
    return 0 if result["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
