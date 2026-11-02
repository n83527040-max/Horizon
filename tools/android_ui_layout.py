#!/usr/bin/env python3
"""Exact Python mirror of the native Android launcher layout (design.rs).

`ui_spec.json` is the single source of truth. This mirror is not shipped in the
APK: it verifies the responsive geometry in CI and drives the layout QA preview.
If a constant changes in Rust, change it here too; `test_android_ui_redesign.py`
checks that both implementations read the same spec keys.
"""
from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC_PATH = ROOT / "crates/horizon-client/src/android/ui_spec.json"
SPEC = json.loads(SPEC_PATH.read_text(encoding="utf-8"))


def metric(section: str, key: str) -> float:
    return float(SPEC[section][key])


def space(name: str) -> float:
    return float(SPEC["spacing"][name])


@dataclass(frozen=True)
class Rect:
    x: float
    y: float
    w: float
    h: float

    def as_tuple(self):
        return (self.x, self.y, self.w, self.h)


@dataclass(frozen=True)
class Layout:
    w: float
    h: float
    pad: float
    gap: float
    content_x: float
    content_w: float
    safe_top: float
    safe_bottom: float
    header_y: float
    header_h: float
    subheader_h: float
    page_top: float
    content_top: float
    page_bottom: float
    nav_y: float
    nav_h: float
    hero_h: float

    @classmethod
    def new(cls, w: float, h: float, top_inset: float = 0.0, bottom_inset: float = 0.0):
        pad = max(12.0, space("pad"))
        gap = max(8.0, space("gap"))
        max_w = float(SPEC["max_content_width"])
        content_w = max(240.0, min(w - 2.0 * pad, max_w))
        content_x = (w - content_w) * 0.5
        safe_top = max(0.0, top_inset) + space("safe_top")
        safe_bottom = max(0.0, bottom_inset) + space("safe_bottom")
        header_h = metric("header", "height")
        header_y = safe_top
        subheader_h = metric("subheader", "height")
        nav_h = metric("nav", "height")
        nav_y = max(h - nav_h - safe_bottom, header_y + header_h + 80.0)
        page_top = header_y + header_h + 6.0
        content_top = page_top + subheader_h
        page_bottom = nav_y - 6.0
        hero_h = min(
            min(max(metric("hero", "height"), metric("hero", "min")), metric("hero", "max")),
            max(0.26 * h, metric("hero", "min")),
        )
        return cls(
            w, h, pad, gap, content_x, content_w, safe_top, safe_bottom,
            header_y, header_h, subheader_h, page_top, content_top, page_bottom,
            nav_y, nav_h, hero_h,
        )

    @property
    def home_height(self) -> float:
        gap = self.gap
        return (
            self.hero_h + gap
            + metric("play", "height") + gap
            + metric("version", "height") + space("section")
            + 22.0 + gap
            + metric("feature", "height") * 2.0 + gap + gap
            + metric("performance", "height") + gap
            + 44.0 + 16.0
        )

    @property
    def home_max_scroll(self) -> float:
        return max(0.0, self.home_height - (self.page_bottom - self.page_top))

    def home(self, scroll: float = 0.0) -> dict[str, Rect]:
        x, w, gap = self.content_x, self.content_w, self.gap
        y = self.page_top - scroll
        hero = Rect(x, y, w, self.hero_h)
        y += self.hero_h + gap
        play = Rect(x, y, w, metric("play", "height"))
        y += metric("play", "height") + gap
        pair_w = (w - gap) * 0.5
        version = Rect(x, y, pair_w, metric("version", "height"))
        profile = Rect(x + pair_w + gap, y, pair_w, metric("version", "height"))
        y += metric("version", "height") + space("section")
        manage = Rect(x, y, w, 22.0)
        y += 22.0 + gap
        fh = metric("feature", "height")
        features = [
            Rect(x, y, (w - gap) * 0.5, fh),
            Rect(x + (w - gap) * 0.5 + gap, y, (w - gap) * 0.5, fh),
            Rect(x, y + fh + gap, (w - gap) * 0.5, fh),
            Rect(x + (w - gap) * 0.5 + gap, y + fh + gap, (w - gap) * 0.5, fh),
        ]
        y += fh * 2.0 + gap + gap
        performance = Rect(x, y, w, metric("performance", "height"))
        y += metric("performance", "height") + gap
        toast = Rect(x, y, w, 44.0)
        return {
            "hero": hero, "play": play, "version": version, "profile": profile,
            "manage": manage, "features": features, "performance": performance,
            "footer": toast,
        }

    def nav_item(self, index: int) -> Rect:
        width = self.w / 5.0
        return Rect(index * width, self.nav_y, width, self.nav_h - self.safe_bottom)

    def half(self, y: float, h: float, right: bool) -> Rect:
        w = (self.content_w - self.gap) * 0.5
        x = self.content_x + w + self.gap if right else self.content_x
        return Rect(x, y, w, h)

    def third(self, y: float, h: float, index: int) -> Rect:
        w = (self.content_w - 2.0 * self.gap) / 3.0
        return Rect(self.content_x + index * (w + self.gap), y, w, h)


TEST_SCREENS = [
    (360.0, 800.0, "compact phone"),
    (393.0, 873.0, "regular phone"),
    (432.0, 960.0, "large phone"),
    (480.0, 768.0, "tablet portrait / small tablet"),
    (800.0, 1280.0, "tablet"),
    (960.0, 540.0, "landscape / foldable"),
]


def check_layout(w: float, h: float, top_inset: float = 0.0, bottom_inset: float = 0.0):
    """Return every invariant violation for one viewport."""
    l = Layout.new(w, h, top_inset, bottom_inset)
    problems: list[str] = []
    home = l.home(0.0)
    rects = [
        home["hero"], home["play"], home["version"], home["profile"],
        home["manage"], home["performance"], home["footer"], *home["features"],
    ]
    for name, r in zip(
        ["hero", "play", "version", "profile", "manage", "performance", "footer",
         "feature0", "feature1", "feature2", "feature3"], rects
    ):
        if r.x < -0.01 or r.x + r.w > w + 0.01:
            problems.append(f"{name} overflows horizontally: {r.as_tuple()}")
        if r.w <= 0 or r.h <= 0:
            problems.append(f"{name} has no size")
    # No card overlaps another card in the 2x2 grid.
    for i in range(4):
        for j in range(i + 1, 4):
            a, b = home["features"][i], home["features"][j]
            separate = (
                a.x + a.w <= b.x + 0.01 or b.x + b.w <= a.x + 0.01
                or a.y + a.h <= b.y + 0.01 or b.y + b.h <= a.y + 0.01
            )
            if not separate:
                problems.append(f"feature cards {i} and {j} overlap")
    # Version and profile are side by side, not stacked or overlapping.
    a, b = home["version"], home["profile"]
    if not (a.y == b.y and a.x + a.w <= b.x + 0.01):
        problems.append("version/profile pair is not a two-column row")
    # Bottom navigation is fully visible and above the safe area.
    if l.nav_y + l.nav_h > h + 0.01:
        problems.append("bottom navigation is off-screen")
    if l.page_bottom > l.nav_y + 0.01:
        problems.append("page content extends under the navigation bar")
    if l.content_top >= l.page_bottom:
        problems.append("no room for page content")
    items = [l.nav_item(i) for i in range(5)]
    for i, item in enumerate(items):
        if item.x < -0.01 or item.x + item.w > w + 0.01:
            problems.append(f"nav item {i} overflows horizontally")
        if item.y + item.h > h + 0.01:
            problems.append(f"nav item {i} is cut off")
    for i in range(4):
        if items[i].x + items[i].w > items[i + 1].x + 0.01:
            problems.append(f"nav items {i} and {i + 1} overlap")
    if l.home_max_scroll < 0:
        problems.append("negative home scroll maximum")
    # Scrolling to the very bottom must bring the footer into view.
    bottom_home = l.home(l.home_max_scroll)
    if bottom_home["footer"].y + bottom_home["footer"].h > l.page_bottom + 1.0:
        problems.append("home footer cannot be scrolled into view")
    return problems


def main():
    failed = False
    for w, h, name in TEST_SCREENS:
        problems = check_layout(w, h)
        print(f"{name:36s} {int(w):4d}x{int(h):4d}  ", end="")
        if problems:
            failed = True
            print("FAIL")
            for p in problems:
                print("   -", p)
        else:
            print("OK")
    # Cutout / gesture-bar insets must not break the layout.
    for top, bottom in [(24.0, 16.0), (48.0, 24.0)]:
        problems = check_layout(360.0, 800.0, top, bottom)
        print(f"safe areas top={top} bottom={bottom}: {'FAIL' if problems else 'OK'}")
        for p in problems:
            print("   -", p)
        failed |= bool(problems)
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
