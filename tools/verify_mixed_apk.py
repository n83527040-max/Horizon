#!/usr/bin/env python3
"""Static proof for the phase-13 mixed Flutter + Rust release APK.

This verifies the *packaged APK*, not the source tree: both native libraries,
Flutter assets, the Rust engine data, the binary manifest (portrait launcher,
separate rotatable engine process), the signature and the release flags.

It does NOT claim the app runs on a device. See FLUTTER_BRIDGE_EVIDENCE.md.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import shlex
import subprocess
import sys
import zipfile
from pathlib import Path


def run(command: list[str]) -> str:
    return subprocess.check_output(command, text=True, stderr=subprocess.STDOUT)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("apk", type=Path)
    parser.add_argument("--report", type=Path)
    parser.add_argument("--aapt2", default="aapt2")
    parser.add_argument("--apksigner", default="apksigner",
                        help="apksigner command; may be 'java -jar <path>/apksigner.jar'")
    args = parser.parse_args()

    report: dict[str, object] = {"apk": str(args.apk), "bytes": args.apk.stat().st_size}
    errors: list[str] = []

    with zipfile.ZipFile(args.apk) as archive:
        names = archive.namelist()
        report["entries"] = len(names)
        rust = [n for n in names if n.endswith("libhorizon_android.so")]
        flutter = [n for n in names if n.endswith("libflutter.so")]
        engine = [n for n in names if n.endswith("engine-data.zip")]
        dart_assets = [n for n in names if "assets/flutter_assets/" in n]
        report["rustLibraries"] = rust
        report["flutterLibraries"] = flutter
        report["engineData"] = engine
        report["flutterAssetCount"] = len(dart_assets)
        if len(rust) != 1:
            errors.append(f"expected one libhorizon_android.so, found {rust}")
        if len(flutter) != 1:
            errors.append(f"expected one libflutter.so, found {flutter}")
        if len(engine) != 1:
            errors.append(f"expected assets/engine-data.zip, found {engine}")
        if len(dart_assets) < 3:
            errors.append("Flutter assets (Dart snapshot/fonts) are missing")
        if rust:
            data = archive.read(rust[0])
            report["rustLibraryBytes"] = len(data)
            report["rustLibrarySha256"] = hashlib.sha256(data).hexdigest()

    badging = run([args.aapt2, "dump", "badging", str(args.apk)])
    report["badging"] = badging.strip().splitlines()[:4]
    if "name='com.craftix.launcher'" not in badging:
        errors.append("package name is not com.craftix.launcher")
    if "versionCode='30'" not in badging or "versionName='0.7.8'" not in badging:
        errors.append("versionCode/versionName are not 30 / 0.7.8")
    if "application-label:'CRAFTIX'" not in badging:
        errors.append("CRAFTIX app label is missing from the binary manifest")
    if not re.search(r"(?:sdkVersion|minSdkVersion):'28'", badging):
        errors.append("minSdk is not 28")

    tree = run([args.aapt2, "dump", "xmltree", "--file", "AndroidManifest.xml", str(args.apk)])
    report["binary_manifest_portrait"] = "screenOrientation(0x0101001e)=1" in tree
    report["engine_process_present"] = ":engine" in tree
    report["debuggable"] = "application-debuggable" in badging
    report["native_activity_meta"] = "android.app.lib_name" in tree
    if not report["binary_manifest_portrait"]:
        errors.append("binary manifest has no portrait launcher Activity")
    if not report["engine_process_present"]:
        errors.append("engine Activity is not isolated in the :engine process")
    if report["debuggable"]:
        errors.append("release APK is debuggable")
    if not report["native_activity_meta"]:
        errors.append("NativeActivity lib_name meta-data is missing")

    signature = run([*shlex.split(args.apksigner), "verify", "--verbose",
                     "--print-certs", str(args.apk)])
    report["signature_v2"] = "Verified using v2 scheme (APK Signature Scheme v2): true" in signature
    report["signer_count"] = re.search(r"Number of signers: (\d+)", signature).group(1) \
        if re.search(r"Number of signers: (\d+)", signature) else "0"
    if not report["signature_v2"]:
        errors.append("APK is not signed with v2")
    if report["signer_count"] != "1":
        errors.append(f"unexpected signer count: {report['signer_count']}")

    public_cert = re.search(
        r"Signer #1 certificate SHA-256 digest: ([0-9a-fA-F]{64})", signature)
    report["signerCertificateSha256"] = public_cert.group(1) if public_cert else None
    if not public_cert:
        errors.append("public signer certificate fingerprint could not be read")
    else:
        raw = public_cert.group(1).upper()
        colon = ":".join(raw[i : i + 2] for i in range(0, 64, 2))
        report["signerCertificateSha256ColonForm"] = colon
        report["matchesOwnerReference"] = (
            colon == "77:05:07:A6:EC:48:51:98:66:55:08:29:32:CA:C2:81:61:55:08:0F:D4:73:DB:7E:42:E9:A0:BF:FD:22:DD:2F"
        )

    report["sha256"] = hashlib.sha256(args.apk.read_bytes()).hexdigest()
    report["passed"] = not errors
    report["errors"] = errors
    report["scope"] = "Archive, binary manifest and signature only; no device/AVD execution."
    text = json.dumps(report, indent=2) + "\n"
    print(text)
    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(text, encoding="utf-8")
    for error in errors:
        print("ERROR:", error, file=sys.stderr)
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
