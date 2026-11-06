#!/usr/bin/env python3
"""Compare two published APKs and the release key without exposing secrets.

This is a static check, not an installation on a device.  The reference
certificate is supplied as the public SHA-256 fingerprint of the key.
"""
import argparse
import hashlib
import json
import re
import shlex
import subprocess
from pathlib import Path


def run(*args):
    return subprocess.run(args, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                          text=True, check=True).stdout


def inspect(path, aapt2, signer):
    badging = run(aapt2, 'dump', 'badging', str(path))
    package = re.search(r"package: name='([^']+)' versionCode='([^']+)' versionName='([^']*)'", badging)
    if not package:
        raise ValueError(f'Cannot read package/version from {path.name}')
    signatures = {}
    for api in (28, 29, 30, 31, 32, 33, 34, 35):
        output = run(*signer, 'verify', '--verbose', '--print-certs', '--min-sdk-version', str(api),
                     '--max-sdk-version', str(api), str(path))
        cert = re.search(r'Signer #1 certificate SHA-256 digest: ([0-9a-fA-F]{64})', output)
        count = re.search(r'Number of signers: (\d+)', output)
        if not cert or not count or count.group(1) != '1':
            raise ValueError(f'Invalid signer on API {api} for {path.name}')
        signatures[api] = cert.group(1).lower()
    return {'file': path.name, 'bytes': path.stat().st_size,
            'sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
            'package': package.group(1), 'versionCode': int(package.group(2)),
            'versionName': package.group(3), 'signatures': signatures}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('old', type=Path)
    parser.add_argument('new', type=Path)
    parser.add_argument('--aapt2', required=True)
    parser.add_argument('--apksigner', required=True)
    parser.add_argument('--key-fingerprint', required=True, help='Public SHA-256 certificate fingerprint')
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    old = inspect(args.old, args.aapt2, shlex.split(args.apksigner))
    new = inspect(args.new, args.aapt2, shlex.split(args.apksigner))
    key = re.sub(r'[^0-9a-fA-F]', '', args.key_fingerprint).lower()
    errors = []
    if len(key) != 64:
        errors.append('Invalid release key certificate fingerprint')
    if old['package'] != new['package']:
        errors.append('Package names differ')
    if new['versionCode'] <= old['versionCode']:
        errors.append('New versionCode is not higher than installed candidate')
    for api in old['signatures']:
        if old['signatures'][api] != new['signatures'][api]:
            errors.append(f'Certificates differ on API {api}')
        if new['signatures'][api] != key:
            errors.append(f'New certificate differs from release secret on API {api}')
    result = {'old': old, 'new': new, 'releaseKeyCertificate': key, 'errors': errors,
              'passed': not errors, 'scope': 'Static APK checks only; no device installation.'}
    args.report.write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(result, indent=2))
    for line in [f"Old {old['versionName']} code {old['versionCode']} package {old['package']}",
                 f"Old APK SHA-256: {old.get('sha256', 'n/a')}",
                 f"New {new['versionName']} code {new['versionCode']} package {new['package']}",
                 f"New APK SHA-256: {new.get('sha256', 'n/a')}",
                 f"Release certificate SHA-256: {key}",
                 f"Certificate/API checks: {'PASS' if not errors else 'FAIL'}; actual device install not tested"]:
        print(f'::notice::{line}')
    if errors:
        raise SystemExit('; '.join(errors))


if __name__ == '__main__':
    main()
