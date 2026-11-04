#!/usr/bin/env python3
"""Binary-level proof that an APK carries BOTH v1 (JAR) and v2 signatures.

targetSdk >= 30 requires APK Signature Scheme v2, while some OEM package
installers additionally require the legacy v1 (JAR) signature. Checking the
archive bytes is deterministic and avoids parsing apksigner's human-readable
output.

Usage: python3 tools/check_apk_signature_schemes.py <apk>
"""
from __future__ import annotations

import json
import sys
import zipfile
from pathlib import Path

V2_MAGIC = b"APK Sig Block 42"


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: check_apk_signature_schemes.py <apk>", file=sys.stderr)
        return 2
    apk = Path(sys.argv[1])
    with zipfile.ZipFile(apk) as archive:
        names = archive.namelist()
        v1 = [n for n in names if n.startswith("META-INF/")
              and n.endswith((".RSA", ".DSA", ".EC"))]
        manifest = "META-INF/MANIFEST.MF" in names
        signature_files = [n for n in names if n.startswith("META-INF/")
                           and n.endswith(".SF")]
    raw = apk.read_bytes()
    v2 = V2_MAGIC in raw
    report = {
        "apk": str(apk),
        "v1Manifest": manifest,
        "v1SignatureFiles": signature_files,
        "v1SignerFiles": v1,
        "v1Present": manifest and bool(v1) and bool(signature_files),
        "v2SigningBlockPresent": v2,
        "passed": manifest and bool(v1) and bool(signature_files) and v2,
    }
    print(json.dumps(report, indent=2))
    if not report["passed"]:
        print("ERROR: APK must carry BOTH v1 (JAR) and v2 (APK Signature Scheme v2)",
              file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
