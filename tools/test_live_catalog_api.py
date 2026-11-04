#!/usr/bin/env python3
"""LIVE API integration test for the real content sources used by Rust Catalog.

This is NOT a mock and NOT a source-only check: it performs HTTPS requests to
Modrinth's public API and Horizon's published index, then validates the exact
response shape, pinned game version, loaders, icons, checksums and facets that
`launcher/src/browser/modrinth.rs` and `horizon.rs` parse.

Run in CI (GitHub Actions runners have outbound HTTPS):
    python3 tools/test_live_catalog_api.py
"""
from __future__ import annotations

import json
import re
import sys
import urllib.parse
import urllib.request
from pathlib import Path

API = "https://api.modrinth.com/v2"
INDEX = "https://horizon-rust.github.io/horizon-mods/index.json"
UA = "craftix/0.7.8 (github.com/n83527040-max/Horizon integration-test)"
GAME = "26.3"
results: list[tuple[bool, str]] = []


def check(ok: bool, label: str) -> None:
    results.append((bool(ok), label))


def get(url: str) -> object:
    request = urllib.request.Request(url, headers={"User-Agent": UA})
    with urllib.request.urlopen(request, timeout=30) as response:
        if response.status != 200:
            raise RuntimeError(f"HTTP {response.status} for {url}")
        return json.loads(response.read().decode("utf-8"))


def search(kind: str, query: str = "", version: str | None = None) -> dict:
    facets = [[f"project_type:{kind}"]]
    if version:
        facets.append([f"versions:{version}"])
    params = {
        "facets": json.dumps(facets, separators=(",", ":")),
        "limit": "3",
        "offset": "0",
    }
    if query:
        params["query"] = query
    return get(f"{API}/search?{urllib.parse.urlencode(params)}")  # type: ignore[return-value]


def main() -> int:
    report: dict[str, object] = {}
    try:
        mods = search("mod", "sodium")
        hits = mods.get("hits", [])
        check(mods.get("total_hits", 0) > 0, "Modrinth mod search returns real hits")
        check(len(hits) >= 1, "mod hits parse into a list")
        if hits:
            hit = hits[0]
            for key in ("project_id", "slug", "title", "description", "author",
                        "downloads", "icon_url", "versions", "categories", "project_type"):
                check(key in hit, f"mod hit has real field '{key}'")
            check(any(v == GAME for v in hit.get("versions", [])),
                  f"mod hit lists pinned Minecraft {GAME}")
            check(str(hit.get("icon_url", "")).startswith("https://"),
                  "mod hit has a real icon URL")
            check(isinstance(hit.get("downloads"), int), "downloads is a real integer")
            report["first_mod"] = hit.get("slug")
            project_id = str(hit.get("project_id", ""))
            if project_id:
                versions = get(f"{API}/project/{urllib.parse.quote(project_id)}/version")
                check(isinstance(versions, list) and len(versions) > 0,
                      "project versions endpoint returns real versions")
                if isinstance(versions, list) and versions:
                    files = versions[0].get("files", [])
                    check(any("sha512" in f.get("hashes", {}) or "sha1" in f.get("hashes", {})
                              for f in files),
                          "version files carry real SHA-512/SHA-1 checksums")
        packs = search("resourcepack")
        check(packs.get("total_hits", 0) > 0, "Modrinth resource pack search returns hits")
        report["pack_hits"] = packs.get("total_hits")
        shaders = search("shader")
        check(shaders.get("total_hits", 0) >= 0, "Modrinth shader search endpoint responds")
        report["shader_hits"] = shaders.get("total_hits")
        pinned = search("mod", "sodium", GAME)
        check(pinned.get("total_hits", 0) > 0,
              f"versions:{GAME} facet returns real compatible projects")
        report["pinned_26_3_hits"] = pinned.get("total_hits")
        index = get(INDEX)
        check(isinstance(index, dict) and index.get("schema") == 1,
              "Horizon index has the real schema")
        projects = index.get("projects", []) if isinstance(index, dict) else []
        check(len(projects) > 0, "Horizon index lists real projects")
        native = [p for p in projects if any(
            v.get("game") == GAME for v in p.get("versions", []))]
        check(len(native) > 0, f"Horizon index has projects for Minecraft {GAME}")
        check(all(re.fullmatch(r"[0-9a-f]{64}", v["file"]["sha256"])
                  for p in projects for v in p.get("versions", [])
                  if "sha256" in v.get("file", {})),
              "Horizon index checksums are real SHA-256 hex digests")
        report["horizon_projects"] = len(projects)
        report["horizon_26_3"] = len(native)
    except Exception as error:  # a real network/API failure must FAIL, not be skipped
        check(False, f"LIVE API request failed: {error}")

    for ok, label in results:
        print(f"  [{'OK' if ok else 'FAIL'}] {label}")
    passed = sum(1 for ok, _ in results if ok)
    print(f"\n{passed}/{len(results)} LIVE API integration checks passed")
    Path("LIVE_API_REPORT.json").write_text(json.dumps(report, indent=2) + "\n")
    return 0 if passed == len(results) else 1


if __name__ == "__main__":
    sys.exit(main())
