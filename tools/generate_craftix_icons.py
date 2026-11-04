#!/usr/bin/env python3
"""Generate real CRAFTIX launcher icon assets from the supplied icon reference.

Produces legacy mipmap PNGs, round icons, adaptive-icon foreground layers in all
required densities, an adaptive-icon XML and a dark-navy background colour.
The source is the *cropped icon* from CRAFTIX_Implementation_Package.zip, not
the full poster. Deterministic: re-running produces identical bytes.

Usage: python3 tools/generate_craftix_icons.py [source.png]
"""
from __future__ import annotations

import sys
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
SOURCE = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "tools/craftix-icon-source.png"
NAVY = (7, 11, 29, 255)  # #070B1D

# launcher icon size / adaptive foreground canvas size (dp at mdpi).
DENSITIES = {
    "mdpi": 1.0,
    "hdpi": 1.5,
    "xhdpi": 2.0,
    "xxhdpi": 3.0,
    "xxxhdpi": 4.0,
}

RES_TREES = (
    ROOT / "flutter/android/app/src/main/res",
    ROOT / "packaging/android/res",
)


def square(source: Image.Image, size: int) -> Image.Image:
    """Fit the icon into a square with a small breathing margin."""
    canvas = Image.new("RGBA", (size, size), NAVY)
    icon = source.convert("RGBA")
    usable = int(size * 0.86)
    icon.thumbnail((usable, usable), Image.Resampling.LANCZOS)
    canvas.alpha_composite(icon, ((size - icon.width) // 2, (size - icon.height) // 2))
    return canvas


def adaptive_foreground(source: Image.Image, size: int) -> Image.Image:
    """108dp canvas; artwork stays inside the central 66dp safe zone."""
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    icon = source.convert("RGBA")
    safe = int(size * 66 / 108)
    icon.thumbnail((safe, safe), Image.Resampling.LANCZOS)
    canvas.alpha_composite(icon, ((size - icon.width) // 2, (size - icon.height) // 2))
    return canvas


def round_icon(source: Image.Image, size: int) -> Image.Image:
    canvas = square(source, size)
    mask = Image.new("L", (size, size), 0)
    ImageDraw.Draw(mask).ellipse((0, 0, size - 1, size - 1), fill=255)
    result = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    result.paste(canvas, (0, 0), mask)
    return result


def main() -> int:
    if not SOURCE.is_file():
        raise SystemExit(f"icon source missing: {SOURCE}")
    source = Image.open(SOURCE)
    print(f"source: {SOURCE} {source.size}")
    for tree in RES_TREES:
        for density, scale in DENSITIES.items():
            folder = tree / f"mipmap-{density}"
            folder.mkdir(parents=True, exist_ok=True)
            icon_size = int(48 * scale)
            fg_size = int(108 * scale)
            square(source, icon_size).save(folder / "ic_launcher.png")
            round_icon(source, icon_size).save(folder / "ic_launcher_round.png")
            adaptive_foreground(source, fg_size).save(folder / "ic_launcher_foreground.png")
        (tree / "values").mkdir(parents=True, exist_ok=True)
        (tree / "values/ic_launcher_background.xml").write_text(
            '<?xml version="1.0" encoding="utf-8"?>\n'
            "<resources>\n"
            '    <color name="ic_launcher_background">#070B1D</color>\n'
            "</resources>\n",
            encoding="utf-8",
        )
        v26 = tree / "mipmap-anydpi-v26"
        v26.mkdir(parents=True, exist_ok=True)
        for name in ("ic_launcher.xml", "ic_launcher_round.xml"):
            (v26 / name).write_text(
                '<?xml version="1.0" encoding="utf-8"?>\n'
                '<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">\n'
                '    <background android:drawable="@color/ic_launcher_background" />\n'
                '    <foreground android:drawable="@mipmap/ic_launcher_foreground" />\n'
                "</adaptive-icon>\n",
                encoding="utf-8",
            )
        print(f"generated: {tree}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
