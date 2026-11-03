#!/usr/bin/env python3
"""Record the PUBLIC SHA-256 signing certificate fingerprint of a release APK.

Only public certificate data is written. No private key, password or Base64 is
ever read from the repository. The owner's reference fingerprint is compared
and reported; a mismatch is a warning for the final report, not a silent pass.
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

EXPECTED = "77:05:07:A6:EC:48:51:98:66:55:08:29:32:CA:C2:81:61:55:08:0F:D4:73:DB:7E:42:E9:A0:BF:FD:22:DD:2F"


def main() -> int:
    certificate = Path(sys.argv[1])
    output = Path(sys.argv[2])
    text = certificate.read_text(encoding="utf-8", errors="replace")
    # apksigner prints either a colon-separated fingerprint (keytool) or a
    # plain 64-hex digest (verify --print-certs). Accept both, normalise to
    # the colon-separated public form used in the owner's reference.
    match = re.search(r"([0-9A-Fa-f]{2}(?::[0-9A-Fa-f]{2}){31}|[0-9A-Fa-f]{64})", text)
    if match:
        raw = match.group(1).replace(":", "").upper()
        actual = ":".join(raw[i : i + 2] for i in range(0, 64, 2))
    else:
        actual = "NOT FOUND"
    equals = actual == EXPECTED.upper()
    report = {
        "actualSha256Fingerprint": actual,
        "ownerReferenceFingerprint": EXPECTED,
        "matchesOwnerReference": equals,
        "note": "Public certificate only. A mismatch means the owner's reference differs from the supplied keystore; report it, never generate a new key.",
    }
    output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"public certificate SHA-256: {actual}")
    print(f"matches CRAFTIX reference: {equals}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
