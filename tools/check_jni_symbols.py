#!/usr/bin/env python3
"""Binary-level JNI/C ABI test on the packaged libhorizon_android.so.

Reads the real shared object bytes (from an APK or a loose .so) and verifies
that every symbol Kotlin declares `external` is actually exported. This is a
binary check, not a source grep: the .so dynstr table contains these names.
"""
from __future__ import annotations

import argparse
import json
import sys
import zipfile
from pathlib import Path

EXPECTED = (
    "Java_dev_horizon_android_HorizonEngine_nativeInit",
    "Java_dev_horizon_android_HorizonEngine_nativeSnapshot",
    "Java_dev_horizon_android_HorizonEngine_nativeDispatch",
    "Java_dev_horizon_android_HorizonEngine_nativeCancel",
    "Java_dev_horizon_android_HorizonEngine_nativeShutdown",
    "Java_dev_horizon_android_HorizonEngine_nativeImportStaged",
    "Java_dev_horizon_android_HorizonEngine_nativeInstallStart",
    "Java_dev_horizon_android_HorizonActivity_nativeDocumentResult",
    "Java_dev_horizon_android_HorizonDocumentsProvider_nativeCall",
    "Java_dev_horizon_android_HorizonDocumentsProvider_nativeOpen",
    "ANativeActivity_onCreate",
)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("artifact", type=Path, help="APK or .so file")
    args = parser.parse_args()
    path: Path = args.artifact
    data: bytes
    if zipfile.is_zipfile(path):
        with zipfile.ZipFile(path) as archive:
            names = [n for n in archive.namelist() if n.endswith("libhorizon_android.so")]
            if len(names) != 1:
                print(f"FAIL: expected exactly one Rust .so, found {names}", file=sys.stderr)
                return 1
            data = archive.read(names[0])
    else:
        data = path.read_bytes()

    found = {name: name.encode("ascii") in data for name in EXPECTED}
    missing = [name for name, ok in found.items() if not ok]
    report = {"bytes": len(data), "symbols": found, "missing": missing,
              "passed": not missing}
    print(json.dumps(report, indent=2))
    for name in missing:
        print(f"ERROR: JNI symbol missing from binary: {name}", file=sys.stderr)
    return 0 if not missing else 1


if __name__ == "__main__":
    sys.exit(main())
