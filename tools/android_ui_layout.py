#!/usr/bin/env python3
"""Exact Python mirror of the native Android launcher layout (design.rs).

`ui_spec.json` is the single source of truth. This mirror is not shipped in the
APK: it verifies the portrait, edge-to-edge responsive geometry in CI and drives
the layout QA preview/renderer. If a constant changes in Rust, change it here
too; `test_android_ui_redesign.py` checks that both implementations read the
same spec keys.
"""
from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC_PATH = ROOT / "crates/horizon-client/src/android/ui_spec.json"
SPEC = json.loads(SPEC_PATH.read_text(encoding="utf-8"))

ALL_PAGES = [
    "home", "mods", "packs", "shaders", "installed", "settings",
    "accounts", "chooser", "file_dialog",
]


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
    left_inset: float
    right_inset: float
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
        return cls.with_insets(w, h, 0.0, top_inset, 0.0, bottom_inset)

    @classmethod
    def with_insets(cls, w: float, h: float, left: float = 0.0, top: float = 0.0,
                    right: float = 0.0, bottom: float = 0.0):
        pad = max(12.0, space("pad"))
        gap = max(8.0, space("gap"))
        # Full-width portrait layout. `max_content_width` is 0 in the shipped
        # spec: no centred desktop column, no large empty side margins.
        max_w = float(SPEC["max_content_width"])
        wide_start = float(SPEC["side_pad_wide_start"])
        wide_extra = float(SPEC["side_pad_wide_extra"])
        extra = min(max(0.0, w - wide_start) * 0.05, wide_extra)
        side = pad + extra
        left_inset = max(0.0, left)
        right_inset = max(0.0, right)
        content_w = max(240.0, w - 2.0 * side - left_inset - right_inset)
        content_x = side + left_inset
        if max_w > 0.0 and content_w > max_w:
            content_w = max_w
            content_x = (w - content_w) * 0.5
        safe_top = max(0.0, top) + space("safe_top")
        safe_bottom = max(0.0, bottom) + space("safe_bottom")
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
            w, h, pad, gap, content_x, content_w, left_inset, right_inset,
            safe_top, safe_bottom, header_y, header_h, subheader_h, page_top,
            content_top, page_bottom, nav_y, nav_h, hero_h,
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
        usable = self.w - self.left_inset - self.right_inset
        width = usable / 5.0
        return Rect(self.left_inset + index * width, self.nav_y, width,
                    self.nav_h - self.safe_bottom)

    def half(self, y: float, h: float, right: bool) -> Rect:
        w = (self.content_w - self.gap) * 0.5
        x = self.content_x + w + self.gap if right else self.content_x
        return Rect(x, y, w, h)

    def third(self, y: float, h: float, index: int) -> Rect:
        w = (self.content_w - 2.0 * self.gap) / 3.0
        return Rect(self.content_x + index * (w + self.gap), y, w, h)

    def sheet_rect(self) -> tuple[float, float, float, float]:
        """Full-width portrait account/modal sheet, stopping above the nav bar."""
        return (self.content_x, self.safe_top, self.content_w,
                max(240.0, self.nav_y - 12.0 - self.safe_top))

    def modal_rect(self, height: float) -> tuple[float, float, float, float]:
        """Full-width portrait dialog, centred but never under the nav bar."""
        max_y = max(self.safe_top, self.nav_y - 12.0 - height)
        y = min(max((self.h - height) * 0.5, self.safe_top), max_y)
        return (self.content_x, y, self.content_w, height)

    def page_rects(self, page: str) -> list[tuple[str, Rect]]:
        """Representative rects from every redesigned page, in the same units."""
        x, w, gap = self.content_x, self.content_w, self.gap
        out: list[tuple[str, Rect]] = []
        if page == "home":
            home = self.home(0.0)
            for name, rect in home.items():
                if name == "features":
                    out.extend((f"feature{i}", r) for i, r in enumerate(rect))
                else:
                    out.append((name, rect))
            return out
        out.append(("subheader", Rect(x, self.page_top, w, self.subheader_h)))
        if page in ("mods", "packs", "shaders"):
            y = self.content_top
            chip_h = metric("catalog", "chip_height")
            for i in range(3):
                out.append((f"segment{i}", self.third(y, chip_h, i)))
            y += chip_h + gap
            search_h = metric("catalog", "search_height")
            search_w = max(120.0, w - 96.0 - gap)
            out.append(("search_field", Rect(x, y, search_w, search_h)))
            out.append(("search_button", Rect(x + search_w + gap, y, 96.0, search_h)))
            y += search_h + gap
            half = (w - gap) * 0.5
            out.append(("source0", Rect(x, y, half, chip_h)))
            out.append(("source1", Rect(x + half + gap, y, half, chip_h)))
            y += chip_h + gap
            row_h = metric("catalog", "row_height")
            for i in range(3):
                ry = y + i * row_h
                out.append((f"row{i}", Rect(x, ry, w, row_h - 8.0)))
                out.append((f"tile{i}", Rect(x + 12.0, ry + 16.0, 48.0, 48.0)))
                out.append((f"install{i}", Rect(x + w - 100.0, ry + 20.0, 88.0, 42.0)))
        elif page == "installed":
            y = self.content_top
            sh = metric("installed", "section_height")
            rh = metric("installed", "row_height")
            for section in range(3):
                out.append((f"section{section}", Rect(x, y, w, sh)))
                out.append((f"import{section}", Rect(x + w - 104.0, y + 6.0, 96.0, sh - 12.0)))
                y += sh + gap
                out.append((f"row{section}", Rect(x, y, w, rh - 8.0)))
                out.append((f"status{section}", Rect(x + w - 108.0, y + 14.0, 92.0, 22.0)))
                out.append((f"enable{section}", Rect(x + 12.0, y + 46.0, 96.0, 30.0)))
                out.append((f"remove{section}", Rect(x + w - 104.0, y + 46.0, 92.0, 30.0)))
                y += rh
        elif page == "settings":
            y = self.content_top
            sh = metric("settings", "section_height")
            rh = metric("settings", "row_height")
            for i in range(6):
                if i % 3 == 0:
                    out.append((f"section{i}", Rect(x, y, w, sh)))
                    y += sh + gap
                out.append((f"row{i}", Rect(x, y, w, rh - 8.0)))
                out.append((f"switch{i}", Rect(x + w - 108.0, y + 16.0, 94.0, 34.0)))
                y += rh + gap
        elif page == "accounts":
            out.append(("sheet", Rect(*self.sheet_rect())))
        elif page == "chooser":
            top = self.content_top
            for i in range(5):
                out.append((f"option{i}", Rect(x, top + i * 84.0, w, 76.0)))
            out.append(("cancel", Rect(x, self.nav_y - 56.0, min(160.0, w), 44.0)))
        elif page == "file_dialog":
            out.append(("modal", Rect(*self.modal_rect(144.0))))
        return out


TEST_SCREENS = [
    (360.0, 800.0, "compact phone portrait"),
    (393.0, 873.0, "regular phone portrait"),
    (432.0, 960.0, "large phone portrait"),
    (480.0, 768.0, "tablet portrait / small tablet"),
    (800.0, 1280.0, "tablet portrait"),
    (960.0, 540.0, "landscape fallback (still full width)"),
]


def check_layout(w: float, h: float, top_inset: float = 0.0, bottom_inset: float = 0.0):
    """Return every invariant violation for one viewport, on every page."""
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
    # Full-width portrait: never a small centred desktop column.
    left_gap = l.content_x
    right_gap = w - (l.content_x + l.content_w)
    if left_gap > 40.0 or right_gap > 40.0:
        problems.append(f"large empty side margins: left={left_gap:.1f} right={right_gap:.1f}")
    if l.content_w < 0.85 * w:
        problems.append(f"content column too narrow: {l.content_w:.1f} of {w:.1f}")
    if abs(l.content_x + l.content_w - (w - l.content_x)) > 80.0:
        problems.append("content column is not balanced within the screen")
    # Bottom navigation is fully visible, full width and above the safe area.
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
    usable = w - l.left_inset - l.right_inset
    if abs(sum(item.w for item in items) - usable) > 0.02:
        problems.append("bottom navigation does not span the full usable width")
    if l.home_max_scroll < 0:
        problems.append("negative home scroll maximum")
    bottom_home = l.home(l.home_max_scroll)
    if bottom_home["footer"].y + bottom_home["footer"].h > l.page_bottom + 1.0:
        problems.append("home footer cannot be scrolled into view")
    # Every redesigned page, including the dialogs, stays inside the viewport
    # and uses the same full-width column.
    for page in ALL_PAGES:
        for name, r in l.page_rects(page):
            if r.x < -0.01 or r.x + r.w > w + 0.01:
                problems.append(f"{page}/{name} overflows horizontally: {r.as_tuple()}")
            if r.w <= 0 or r.h <= 0:
                problems.append(f"{page}/{name} has no size")
    sheet = Rect(*l.sheet_rect())
    if sheet.y + sheet.h > l.nav_y + 0.01:
        problems.append("account dialog covers the bottom navigation")
    return problems


def main():
    failed = False
    for w, h, name in TEST_SCREENS:
        problems = check_layout(w, h)
        print(f"{name:44s} {int(w):4d}x{int(h):4d}  ", end="")
        if problems:
            failed = True
            print("FAIL")
            for p in problems:
                print("   -", p)
        else:
            print("OK")
    for top, bottom in [(24.0, 16.0), (48.0, 24.0)]:
        problems = check_layout(360.0, 800.0, top, bottom)
        print(f"safe areas top={top} bottom={bottom}: {'FAIL' if problems else 'OK'}")
        for p in problems:
            print("   -", p)
        failed |= bool(problems)
    for left, right in [(16.0, 16.0), (48.0, 0.0)]:
        problems: list[str] = []
        l = Layout.with_insets(393.0, 873.0, left, 0.0, right, 0.0)
        for page in ALL_PAGES:
            for name, r in l.page_rects(page):
                if r.x < -0.01 or r.x + r.w > 393.0 + 0.01:
                    problems.append(f"{page}/{name} overflows with side cutouts")
        print(f"side cutouts left={left} right={right}: {'FAIL' if problems else 'OK'}")
        for p in problems:
            print("   -", p)
        failed |= bool(problems)
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
