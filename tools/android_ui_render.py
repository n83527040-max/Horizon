#!/usr/bin/env python3
"""Render portrait QA images of every redesigned Horizon Android launcher page.

The rectangles come from ``tools/android_ui_layout.py`` - the exact Python mirror
of ``crates/horizon-client/src/android/design.rs`` - and the palette, fonts and
hero artwork come from the shipped assets. These are layout QA renders produced
from the same geometry as the native UI, NOT device screenshots: they prove the
portrait, edge-to-edge layout of all pages at 360x800, 393x873 and 432x960.
No demo performance numbers are invented; the launcher reads those at runtime.
"""
from __future__ import annotations

import argparse
import html
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont, ImageOps

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
from android_ui_layout import ALL_PAGES, SPEC, Layout, Rect, check_layout  # noqa: E402

HERO = ROOT / "crates/horizon-client/src/android/assets/hero.png"
FONT_DIR = ROOT / "crates/horizon-theme/fonts"
FALLBACK = Path("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf")
NAV_ICONS = ["home", "puzzle", "layers", "download", "gear"]
NAV_LABELS = SPEC["nav_items"]
SCREENS = [(360, 800), (393, 873), (432, 960)]

PAGE_TITLES = {
    "home": "Home", "mods": "Mods", "packs": "Packs", "shaders": "Shaders",
    "installed": "Installed", "settings": "Settings", "accounts": "Accounts dialog",
    "chooser": "Orientation chooser", "file_dialog": "File dialog",
}
CATALOG = {
    "mods": ("Mods", "Native .hmod packages • install, enable or remove", "Search or paste an HTTPS .hmod link", "Native .hmod package"),
    "packs": ("Resource Packs", "Verified downloads • enable in Installed", "Search packs", "Resource pack"),
    "shaders": ("Shaders", "Native .hshader packs • compatibility and activation", "Search shaders", "Shader pack"),
}

_FONT_CACHE: dict[tuple[str, int], ImageFont.FreeTypeFont] = {}


def font(size: int, weight: str = "regular"):
    key = (weight, size)
    if key not in _FONT_CACHE:
        name = {"regular": "Geist-Regular.ttf", "medium": "Geist-Medium.ttf",
                "semi": "Geist-SemiBold.ttf", "bold": "Geist-SemiBold.ttf"}[weight]
        path = FONT_DIR / name
        if not path.is_file():
            path = FALLBACK
        try:
            _FONT_CACHE[key] = ImageFont.truetype(str(path), max(6, int(size)))
        except OSError:
            _FONT_CACHE[key] = ImageFont.load_default(size=max(6, int(size)))
    return _FONT_CACHE[key]


def rgb(name: str, a: int = 255):
    h = SPEC["palette"][name].lstrip("#")
    return (int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16), a)


def mix(c1, c2, t: float):
    return tuple(int(c1[i] * (1 - t) + c2[i] * t) for i in range(4))


class R:
    """A rect that supports both attribute access and tuple indexing."""

    def __init__(self, r):
        if isinstance(r, Rect):
            self.x, self.y, self.w, self.h = r.x, r.y, r.w, r.h
        else:
            self.x, self.y, self.w, self.h = r

    def __iter__(self):
        return iter((self.x, self.y, self.w, self.h))

    def __getitem__(self, i):
        return (self.x, self.y, self.w, self.h)[i]


def box(r) -> list[float]:
    if isinstance(r, Rect):
        return [r.x, r.y, r.x + r.w, r.y + r.h]
    x, y, w, h = r
    return [x, y, x + w, y + h]


class Canvas:
    def __init__(self, w: float, h: float):
        self.w, self.h = w, h
        self.img = Image.new("RGBA", (int(w), int(h)), rgb("bg"))
        self.d = ImageDraw.Draw(self.img)

    def rect(self, r, radius=0.0, fill=None, outline=None, width=1):
        self.d.rounded_rectangle(box(r), radius=max(0.0, radius),
                                 fill=fill, outline=outline, width=width)

    def gradient(self, r, left, right, radius=0.0):
        x, y, w, h = (r if not isinstance(r, Rect) else (r.x, r.y, r.w, r.h))
        w, h = max(1, int(w)), max(1, int(h))
        strip = Image.new("RGBA", (w, 1))
        px = strip.load()
        for i in range(w):
            t = i / max(1, w - 1)
            px[i, 0] = tuple(int(left[k] * (1 - t) + right[k] * t) for k in range(4))
        grad = strip.resize((w, h))
        if radius > 0:
            mask = Image.new("L", (w, h), 0)
            ImageDraw.Draw(mask).rounded_rectangle([0, 0, w - 1, h - 1], radius=radius, fill=255)
            self.img.paste(grad, (int(x), int(y)), mask)
        else:
            self.img.paste(grad, (int(x), int(y)))

    def fit(self, s: str, size: float, weight: str, max_w: float) -> str:
        f = font(int(size), weight)
        if self.d.textlength(s, font=f) <= max_w:
            return s
        out = s
        while out and self.d.textlength(out + "…", font=f) > max_w:
            out = out[:-1]
        return out + "…"

    def text(self, xy, s, size=12, color=None, weight="regular", center_w=None):
        f = font(int(size), weight)
        text = s
        if center_w is not None:
            text = self.fit(s, size, weight, center_w)
            tw = self.d.textlength(text, font=f)
            xy = (xy[0] + (center_w - tw) * 0.5, xy[1])
        self.d.text(xy, text, fill=color or rgb("text"), font=f)

    def icon(self, name: str, x: float, y: float, s: float, color):
        d = self.d
        P = lambda a, b: (x + a * s / 24.0, y + b * s / 24.0)  # noqa: E731
        lw = max(1, int(1.8 * s / 24.0))
        def line(points, w=lw):
            d.line([P(*p) for p in points], fill=color, width=w, joint="curve")
        def outline_polygon(points):
            d.line([P(*p) for p in points + [points[0]]], fill=color, width=lw, joint="curve")
        if name == "home":
            outline_polygon([(3, 11), (12, 3), (21, 11), (18, 11), (18, 20), (6, 20), (6, 11)])
            d.rectangle([P(10, 14), P(14, 20)], outline=color, width=lw)
        elif name == "play":
            d.polygon([P(7, 4), P(20, 12), P(7, 20)], fill=color)
        elif name == "puzzle":
            d.rounded_rectangle([P(3, 4), P(20, 20)], radius=3 * s / 24, outline=color, width=lw)
            d.ellipse([P(9, 2), P(15, 8)], outline=color, width=lw)
            d.line([P(12, 8), P(12, 20)], fill=color, width=lw)
        elif name == "layers":
            outline_polygon([(12, 2), (22, 8), (12, 14), (2, 8)])
            outline_polygon([(2, 13), (12, 19), (22, 13)])
            outline_polygon([(2, 18), (12, 24), (22, 18)])
        elif name == "download":
            line([(12, 3), (12, 15)])
            line([(7, 10), (12, 15), (17, 10)])
            line([(4, 19), (20, 19)])
        elif name == "gear":
            d.ellipse([P(8, 8), P(16, 16)], outline=color, width=lw)
            for a in range(0, 360, 45):
                import math
                rad = math.radians(a)
                line([(12 + 7 * math.cos(rad), 12 + 7 * math.sin(rad)),
                      (12 + 10 * math.cos(rad), 12 + 10 * math.sin(rad))], max(1, lw))
        elif name == "sparkle":
            for cx, cy in [(12, 4), (12, 20), (4, 12), (20, 12)]:
                d.ellipse([P(cx - 2, cy - 2), P(cx + 2, cy + 2)], fill=color)
            d.ellipse([P(9, 9), P(15, 15)], fill=color)
        elif name == "gauge":
            d.arc([P(3, 3), P(21, 21)], start=200, end=340, fill=color, width=lw)
            line([(12, 12), (17, 7)])
        elif name == "search":
            d.ellipse([P(3, 3), P(15, 15)], outline=color, width=lw)
            line([(15, 15), (21, 21)])
        elif name == "chevron":
            line([(9, 5), (16, 12), (9, 19)])
        elif name == "check":
            line([(4, 12), (10, 18), (20, 5)])
        elif name == "user":
            d.ellipse([P(8, 3), P(16, 11)], outline=color, width=lw)
            d.arc([P(3, 10), P(21, 25)], start=180, end=360, fill=color, width=lw)
        elif name == "sliders":
            for cy, cx in [(7, 9), (12, 16), (17, 7)]:
                line([(3, cy), (21, cy)])
                d.ellipse([P(cx - 2, cy - 2), P(cx + 2, cy + 2)], fill=color)
        elif name == "shield":
            outline_polygon([(12, 2), (21, 6), (19, 17), (12, 22), (5, 17), (3, 6)])
        else:
            d.rounded_rectangle([P(4, 4), P(20, 20)], radius=3 * s / 24, outline=color, width=lw)


def glows(c: Canvas, l: Layout):
    c.rect((l.w * 0.52, -80, l.w * 0.78, 250), 125,
           mix(rgb("bg"), rgb("primary"), 0.055))
    c.rect((-100, l.h * 0.60, l.w * 0.75, 280), 140,
           mix(rgb("bg"), rgb("secondary"), 0.05))


def header(c: Canvas, l: Layout):
    x, w, y = l.content_x, l.content_w, l.header_y
    for i in range(6):
        ww = 34.0 - i * 5.0
        c.rect((x + (34.0 - ww) * 0.5, y + 7.0 + i * 5.0, ww, 4.0), 2.0,
               rgb("primary") if i < 3 else rgb("secondary"))
    c.rect((x + 8.0, y + 35.0, 18.0, 3.0), 1.5, rgb("cyan"))
    c.text((x + 46.0, y + 2.0), "HORIZON", 22, rgb("text"), "semi")
    c.text((x + 46.0, y + 28.0), "Minecraft Java • Android", 11, rgb("muted"))
    gear = (x + w - 40.0, y + 8.0, 40.0, 40.0)
    c.rect(gear, 14, rgb("card_alt"), rgb("outline"))
    c.icon("gear", gear[0] + 9.0, gear[1] + 9.0, 22, rgb("text"))
    badge = (gear[0] - 86.0, y + 14.0, 78.0, 28.0)
    c.rect(badge, 14, mix(rgb("bg"), rgb("primary"), 0.24), rgb("primary"))
    c.text((badge[0], badge[1] + 7.0), "v0.7.6", 11, rgb("text"), "medium", center_w=badge[2])


def subheader(c: Canvas, l: Layout, title: str, subtitle: str):
    c.text((l.content_x, l.page_top + 1.0), title, 22, rgb("text"), "semi", center_w=l.content_w)
    c.text((l.content_x, l.page_top + 29.0), subtitle, 12, rgb("muted"), center_w=l.content_w)


def nav(c: Canvas, l: Layout, active: int):
    # Edge-to-edge glass strip including the bottom gesture safe area.
    c.rect((0, l.nav_y, l.w, l.h - l.nav_y), 0, rgb("glass"))
    c.rect((0, l.nav_y, l.w, l.nav_h), 26, rgb("glass"), rgb("outline"))
    c.gradient((26, l.nav_y, l.w - 52, 18), mix(rgb("glass"), rgb("primary"), 0.12), rgb("glass"))
    for i in range(5):
        item = l.nav_item(i)
        col = rgb("primary") if i == active else rgb("muted")
        c.icon(NAV_ICONS[i], item.x + (item.w - 22.0) * 0.5, item.y + 12.0, 22.0, col)
        c.text((item.x, item.y + 39.0), NAV_LABELS[i], 10, col, "medium", center_w=item.w)
        if i == active:
            c.rect((item.x + item.w * 0.32, item.y + 3.0, item.w * 0.36, 3.0), 2.0, rgb("primary"))


def card(c: Canvas, r, glass=False):
    c.rect(r, 22, rgb("card"), rgb("outline"))
    if glass:
        c.gradient((r[0] if not isinstance(r, Rect) else r.x + 10,
                    (r[1] if not isinstance(r, Rect) else r.y) + 1,
                    (r[2] if not isinstance(r, Rect) else r.w) - 20, 22),
                   mix(rgb("card"), rgb("primary"), 0.09), rgb("card"))


def gradient_button(c: Canvas, r, label=None, icon=None):
    c.rect(r, 18, rgb("secondary"), mix(rgb("primary"), rgb("outline"), 0.55))
    c.gradient((r[0] + 18, r[1], r[2] - 36, r[3]), rgb("primary"), rgb("secondary"), 18)
    if icon:
        c.icon(icon, r[0] + 18, r[1] + (r[3] - 22) * 0.5, 22, rgb("text"))
    if label:
        c.text((r[0], r[1] + (r[3] - 18) * 0.5), label, 16, rgb("text"), "semi", center_w=r[2])


def chip(c: Canvas, r, active, label):
    c.rect(r, 14, mix(rgb("card_alt"), rgb("primary"), 0.32) if active else rgb("card_alt"),
           rgb("primary") if active else rgb("outline"))
    c.text((r[0], r[1] + (r[3] - 13) * 0.5), label, 12, rgb("text"), "medium", center_w=r[2])


def draw_home(c: Canvas, l: Layout, rects):
    hero = rects["hero"]
    art = Image.open(HERO).convert("RGBA")
    art = ImageOps.fit(art, (max(1, int(hero.w)), max(1, int(hero.h))), method=Image.Resampling.LANCZOS)
    mask = Image.new("L", art.size, 0)
    ImageDraw.Draw(mask).rounded_rectangle([0, 0, art.width - 1, art.height - 1], radius=26, fill=255)
    c.img.paste(art, (int(hero.x), int(hero.y)), mask)
    # Dark bottom scrim so the white wordmark stays legible.
    scrim = Image.new("RGBA", art.size, (0, 0, 0, 0))
    sd = ImageDraw.Draw(scrim)
    for row in range(art.height):
        t = row / max(1, art.height - 1)
        sd.line([(0, row), (art.width, row)], fill=(7, 11, 28, int(190 * t)))
    from PIL import ImageChops
    scrim_mask = ImageChops.multiply(mask, scrim.getchannel("A"))
    c.img.paste(scrim, (int(hero.x), int(hero.y)), scrim_mask)
    c.text((hero.x + 18, hero.y + hero.h - 70), "HORIZON", 26, rgb("text"), "semi")
    c.text((hero.x + 18, hero.y + hero.h - 40), "Minecraft Java Edition", 13, rgb("text"))
    c.text((hero.x + hero.w - 96, hero.y + 14), "26.3", 11, rgb("text"), "medium", center_w=78)
    gradient_button(c, rects["play"], "Play", "play")
    for name, label, value, detail, icon in [
        ("version", "MINECRAFT VERSION", "Java 26.3", "Engine-pinned", "layers"),
        ("profile", "PROFILE", "Player", "Offline / Sign in", "user"),
    ]:
        r = rects[name]
        card(c, r)
        c.icon(icon, r.x + 14, r.y + 12, 24, rgb("cyan") if name == "version" else rgb("primary"))
        c.text((r.x + 46, r.y + 11), label, 11, rgb("muted"))
        c.text((r.x + 14, r.y + 31), value, 17, rgb("text"), "semi", center_w=r.w - 28)
        c.text((r.x + 14, r.y + 55), detail, 11, rgb("muted"), center_w=r.w - 28)
    c.text((rects["manage"].x, rects["manage"].y + 3), "Content on this device", 13, rgb("muted"))
    for i, (label, detail, icon, accent) in enumerate([
        ("Mods", "Native .hmod", "puzzle", "primary"),
        ("Packs", "Resource packs", "layers", "cyan"),
        ("Shaders", "Shader packs", "sparkle", "secondary"),
        ("Installed", "Manage content", "download", "primary"),
    ]):
        r = rects[f"feature{i}"]
        card(c, r)
        c.icon(icon, r.x + 14, r.y + 13, 26, rgb(accent))
        c.text((r.x + 14, r.y + 44), label, 15, rgb("text"), "semi")
        c.text((r.x + 14, r.y + 63), detail, 11, rgb("muted"))
        c.icon("chevron", r.x + r.w - 30, r.y + 56, 20, rgb("muted"))
    perf = rects["performance"]
    card(c, perf, glass=True)
    c.icon("gauge", perf.x + 14, perf.y + 12, 22, rgb("cyan"))
    c.text((perf.x + 44, perf.y + 10), "Performance", 15, rgb("text"), "semi")
    c.text((perf.x + perf.w - 130, perf.y + 12), "live /proc", 11, rgb("muted"), center_w=116)
    rows = ["RAM Usage", "RAM Allocated", "Renderer"]
    for i, label in enumerate(rows):
        y = perf.y + 44 + i * 24
        c.text((perf.x + 14, y), label, 11, rgb("muted"))
        c.rect((perf.x + perf.w * 0.45, y + 3, perf.w * 0.5 - 14, 8), 4, rgb("card_alt"))
    c.text((perf.x + 14, perf.y + perf.h - 18), "Values are read at runtime; no demo numbers", 10, rgb("muted"))
    toast = rects["footer"]
    c.rect(toast, 14, mix(rgb("card"), rgb("primary"), 0.16), rgb("primary"))
    c.icon("check", toast.x + 12, toast.y + 11, 20, rgb("primary"))
    c.text((toast.x + 40, toast.y + 12), "Horizon Android 0.7.6 • Minecraft Java 26.3", 12, rgb("text"))


def draw_catalog(c: Canvas, l: Layout, rects, page: str):
    title, subtitle, placeholder, item = CATALOG[page]
    subheader(c, l, title, subtitle)
    active = {"mods": 0, "packs": 1, "shaders": 2}[page]
    for i, label in enumerate(["Mods", "Packs", "Shaders"]):
        chip(c, rects[f"segment{i}"], i == active, label)
    field = rects["search_field"]
    c.rect(field, 14, rgb("card_alt"), rgb("outline"))
    c.icon("search", field.x + 12, field.y + 12, 22, rgb("muted"))
    c.text((field.x + 42, field.y + 13), placeholder, 13, rgb("muted"), center_w=field.w - 46)
    gradient_button(c, rects["search_button"], "Search")
    for i, label in enumerate(["Horizon registry", "Modrinth browsing"]):
        chip(c, rects[f"source{i}"], i == 0, label)
    for i in range(3):
        r = rects[f"row{i}"]
        card(c, r)
        tile = rects[f"tile{i}"]
        c.rect(tile, 12, mix(rgb("card_alt"), rgb("primary"), 0.22))
        c.text((tile.x, tile.y + 12), item[0].upper(), 20, rgb("text"), "semi", center_w=tile.w)
        c.text((r.x + 72, r.y + 10), f"{item} {i + 1}", 14, rgb("text"), "semi", center_w=r.w - 180)
        c.text((r.x + 72, r.y + 34), "Compatibility / version", 11, rgb("muted"))
        btn = rects[f"install{i}"]
        c.rect(btn, 14, mix(rgb("card_alt"), rgb("primary"), 0.30), rgb("primary"))
        c.text((btn.x, btn.y + 12), "Install", 12, rgb("text"), "medium", center_w=btn.w)


def draw_installed(c: Canvas, l: Layout, rects):
    subheader(c, l, "Installed", "Enable, import or remove content by type")
    for section, (name, icon) in enumerate([("Mods", "puzzle"), ("Resource Packs", "layers"), ("Shaders", "sparkle")]):
        r = rects[f"section{section}"]
        c.rect(r, 14, rgb("card_alt"), rgb("outline"))
        c.icon(icon, r.x + 12, r.y + 11, 22, rgb("primary"))
        c.text((r.x + 42, r.y + 12), name, 14, rgb("text"), "semi")
        imp = rects[f"import{section}"]
        c.rect(imp, 10, mix(rgb("card_alt"), rgb("cyan"), 0.20), rgb("cyan"))
        c.text((imp.x, imp.y + 8), "Import", 11, rgb("text"), "medium", center_w=imp.w)
        row = rects[f"row{section}"]
        card(c, row)
        c.text((row.x + 14, row.y + 10), f"{name} item", 14, rgb("text"), "semi",
               center_w=row.w - 140)
        status = rects[f"status{section}"]
        c.rect(status, 11, mix(rgb("card"), rgb("success"), 0.18))
        c.text((status.x, status.y + 5), "Enabled", 11, rgb("success"), "medium", center_w=status.w)
        for key, label, accent in [("enable", "Disable", "primary"), ("remove", "Remove", "danger")]:
            b = rects[f"{key}{section}"]
            c.rect(b, 14, rgb("card_alt"), rgb("outline"))
            c.text((b.x, b.y + 7), label, 11, rgb(accent), "medium", center_w=b.w)


def draw_settings(c: Canvas, l: Layout, rects):
    subheader(c, l, "Settings", "Graphics, performance, accounts and app")
    entries = [
        ("section", "Graphics"),
        ("row", "Rendering", "Vulkan / compatibility"),
        ("row", "Screen orientation", "Portrait (default)"),
        ("row", "Client resolution", "100%"),
        ("section", "Performance"),
        ("row", "Touch controls", "Joystick / Crosshair"),
        ("row", "Accounts", "Offline / Sign in"),
        ("row", "Version", "Horizon Android 0.7.6"),
    ]
    row_index = 0
    for kind, *rest in entries:
        if kind == "section":
            # Sections are emitted at rows 0 and 3 by the shared layout mirror.
            key = "section0" if row_index == 0 else "section3"
            s = rects[key]
            c.text((s.x, s.y + 4), rest[0], 15, rgb("text"), "semi")
            c.rect((s.x, s.y + s.h - 5, s.w, 1), 0, rgb("outline"))
            continue
        label, value = rest
        r = rects[f"row{row_index}"]
        card(c, r)
        c.text((r.x + 14, r.y + 10), label, 14, rgb("text"), "semi")
        c.text((r.x + 14, r.y + 36), value, 12, rgb("muted"))
        if row_index in (3, 5):
            sw = rects[f"switch{row_index}"]
            c.rect(sw, 17, mix(rgb("card_alt"), rgb("primary"), 0.35), rgb("primary"))
            c.d.ellipse([sw.x + sw.w - 32, sw.y + 3, sw.x + sw.w - 4, sw.y + 31], fill=rgb("text"))
        else:
            c.icon("chevron", r.x + r.w - 32, r.y + 24, 20, rgb("muted"))
        row_index += 1


def draw_accounts(c: Canvas, l: Layout, rects):
    r = rects["sheet"]
    c.rect((0, 0, l.w, l.h), 0, mix(rgb("bg"), rgb("card"), 0.6))
    card(c, r, glass=True)
    c.text((r.x + 20, r.y + 18), "Accounts", 24, rgb("text"), "medium")
    c.text((r.x + 20, r.y + 58), "Offline / Sign in", 18, rgb("muted"))
    c.text((r.x + 20, r.y + 104), "Your password is entered in Microsoft's browser.", 16, rgb("text"))
    c.text((r.x + 20, r.y + 166), "Offline player name", 14, rgb("muted"))
    name = (r.x + 20, r.y + 190, r.w - 40, 38)
    c.rect(name, 14, rgb("card_alt"), rgb("outline"))
    c.text((name[0] + 12, name[1] + 9), "Enter an offline name", 16, rgb("muted"))
    gradient_button(c, (r.x + 20, r.y + r.h - 148, r.w - 40, 42), "Sign in with browser")
    half = (r.w - 52) / 2
    for i, label in enumerate(["Use a device code instead", "Done"]):
        b = (r.x + 20 + i * (half + 12), r.y + r.h - 98, half, 38)
        c.rect(b, 14, rgb("card_alt"), rgb("outline"))
        c.text((b[0], b[1] + 10), label, 12, rgb("text"), "medium", center_w=b[2])
    b = (r.x + 20, r.y + r.h - 52, r.w - 40, 36)
    c.rect(b, 14, rgb("card_alt"), rgb("outline"))
    c.text((b[0], b[1] + 9), "Close", 12, rgb("text"), "medium", center_w=b[2])


def draw_chooser(c: Canvas, l: Layout, rects):
    c.text((l.content_x, l.safe_top + 12), "Screen orientation", 28, rgb("text"), "medium", center_w=l.content_w)
    c.text((l.content_x, l.safe_top + 54), "Applies to the launcher and game. Portrait is the default.", 15,
           rgb("muted"), center_w=l.content_w)
    options = ["Landscape (sensor)", "Portrait (sensor)", "Landscape (reversed)",
               "Portrait (reversed)", "Automatic rotation"]
    for i in range(5):
        r = rects[f"option{i}"]
        d = ImageDraw.Draw(c.img)
        d.ellipse([r.x + 2, r.y + 10, r.x + 24, r.y + 32], outline=rgb("text"), width=2)
        if i == 1:
            d.ellipse([r.x + 8, r.y + 16, r.x + 18, r.y + 26], fill=rgb("text"))
        c.text((r.x + 48, r.y + 8), options[i], 19, rgb("text"), "medium", center_w=r.w - 60)
        c.rect((r.x, r.y + 75, r.w, 1), 0, rgb("outline"))
    b = rects["cancel"]
    c.rect(b, 14, rgb("card_alt"), rgb("outline"))
    c.text((b.x, b.y + 12), "Cancel", 14, rgb("text"), "medium", center_w=b.w)


def draw_file_dialog(c: Canvas, l: Layout, rects):
    c.rect((0, 0, l.w, l.h), 0, mix(rgb("bg"), (0, 0, 0, 255), 0.72))
    r = rects["modal"]
    card(c, r, glass=True)
    c.text((r.x + 12, r.y + 12), "File exists", 20, rgb("text"), "medium")
    c.text((r.x + 12, r.y + 46), "Choose what to do with this file", 16, rgb("muted"),
           center_w=r.w - 24)
    labels = ["Keep both", "Replace", "Cancel"]
    for i, label in enumerate(labels):
        w = (r.w - 24) / 3
        b = (r.x + 12 + i * w, r.y + 92, w - 4, 38)
        c.rect(b, 14, rgb("card_alt"), rgb("outline"))
        c.text((b[0], b[1] + 10), label, 16, rgb("text"), "medium", center_w=b[2])


def render(page: str, w: int, h: int) -> Image.Image:
    l = Layout.new(float(w), float(h))
    c = Canvas(float(w), float(h))
    glows(c, l)
    if page in ("accounts", "chooser", "file_dialog"):
        # Dialogs are overlays: the fixed bar stays visible underneath.
        pass
    else:
        header(c, l)
    rects = {name: R(r) for name, r in l.page_rects(page)}
    if page == "home":
        draw_home(c, l, rects)
    elif page in CATALOG:
        draw_catalog(c, l, rects, page)
    elif page == "installed":
        draw_installed(c, l, rects)
    elif page == "settings":
        draw_settings(c, l, rects)
    elif page == "accounts":
        draw_accounts(c, l, rects)
    elif page == "chooser":
        draw_chooser(c, l, rects)
    elif page == "file_dialog":
        draw_file_dialog(c, l, rects)
    active = {"home": 0, "mods": 1, "packs": 2, "shaders": 2, "installed": 3, "settings": 4}.get(page, 0)
    nav(c, l, active)
    return c.img.convert("RGB")


def overview(images: dict[str, Image.Image], w: int, h: int) -> Image.Image:
    scale = 0.45
    tw, th = int(w * scale), int(h * scale)
    label_h = 30
    cols = 3
    rows = (len(ALL_PAGES) + cols - 1) // cols
    sheet = Image.new("RGB", (cols * tw, rows * (th + label_h)), (7, 11, 28))
    d = ImageDraw.Draw(sheet)
    for i, page in enumerate(ALL_PAGES):
        thumb = images[page].resize((tw, th), Image.Resampling.LANCZOS)
        x, y = (i % cols) * tw, (i // cols) * (th + label_h)
        sheet.paste(thumb, (x, y))
        d.rectangle([x, y + th, x + tw, y + th + label_h], fill=(12, 19, 48))
        d.text((x + 8, y + th + 7), PAGE_TITLES[page], fill=(255, 255, 255), font=font(14, "medium"))
    return sheet


def gallery(out: Path, screens: list[tuple[int, int]]):
    parts = [
        "<!doctype html><html lang='en'><head><meta charset='utf-8'>",
        "<meta name='viewport' content='width=device-width,initial-scale=1'>",
        "<title>Horizon Android 0.7.6 — Portrait UI QA renders</title>",
        "<style>body{margin:0;background:#070B1C;color:#fff;font-family:system-ui,sans-serif;padding:28px}"
        "h1{font-size:22px;margin:0 0 6px}h2{font-size:16px;color:#A9B4D0;margin:28px 0 12px}"
        "p{color:#A9B4D0;max-width:900px;line-height:1.55;font-size:14px}"
        ".row{display:flex;flex-wrap:wrap;gap:18px}.card{background:#111A35;border:1px solid #2A3560;"
        "border-radius:18px;padding:12px;width:210px}img{width:100%;display:block;border-radius:12px}"
        ".cap{font-size:13px;margin-top:8px}.tag{display:inline-block;background:#8B45FF22;border:1px solid #8B45FF;"
        "color:#c9a8ff;border-radius:999px;padding:3px 10px;font-size:12px;margin-bottom:10px}"
        "a{color:#20DDF5}</style></head><body>",
        "<h1>Horizon Android – Redesigned Edition · Portrait UI</h1>",
        "<span class='tag'>portrait locked</span> <span class='tag'>edge-to-edge full width</span>"
        " <span class='tag'>fixed bottom navigation</span>",
        "<p>These are <b>layout QA renders</b> generated from the exact same geometry as the native Rust"
        " launcher (<code>tools/android_ui_layout.py</code> mirrors <code>design.rs</code>), the shipped"
        " <code>ui_spec.json</code> palette, the project's Geist fonts and the bundled hero artwork."
        " They are not device screenshots. No demo performance numbers are drawn: the app reads real"
        " RAM/renderer values at runtime. The rebuilt, signed APK is published in the"
        " <a href='https://github.com/n83527040-max/Horizon/releases/tag/redesign-0.7.6'>GitHub Release</a>.</p>",
        "<p>Every card spans the whole screen minus a small responsive side padding; there is no centred"
        " desktop column and no large empty side margins. The five-destination bar spans 100% of the"
        " usable width and sits above the Android gesture safe area.</p>",
        "<h2>All pages at a glance · 393 × 873</h2>",
        "<img src='portrait-overview.png' style='max-width:100%;width:560px;border-radius:16px'>",
    ]
    for w, h in screens:
        parts.append(f"<h2>{w} × {h} — all {len(ALL_PAGES)} pages</h2><div class='row'>")
        for page in ALL_PAGES:
            src = f"screens/{w}x{h}/{page}.png"
            parts.append(f"<div class='card'><img src='{src}' alt='{html.escape(PAGE_TITLES[page])}'>"
                         f"<div class='cap'>{html.escape(PAGE_TITLES[page])}</div></div>")
        parts.append("</div>")
    parts.append("</body></html>")
    (out / "index.html").write_text("\n".join(parts), encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "tools/ui-preview")
    parser.add_argument("--check", action="store_true", help="Validate geometry only, write nothing")
    args = parser.parse_args()
    out: Path = args.output
    if args.check:
        for w, h in SCREENS:
            problems = check_layout(float(w), float(h))
            if problems:
                for p in problems:
                    print("ERROR:", p)
                return 1
        print(f"Portrait geometry OK for {len(ALL_PAGES)} pages on {len(SCREENS)} screens")
        return 0
    out.mkdir(parents=True, exist_ok=True)
    all_images: dict[tuple[int, int], dict[str, Image.Image]] = {}
    for w, h in SCREENS:
        folder = out / "screens" / f"{w}x{h}"
        folder.mkdir(parents=True, exist_ok=True)
        images: dict[str, Image.Image] = {}
        for page in ALL_PAGES:
            image = render(page, w, h)
            image.save(folder / f"{page}.png", optimize=True)
            images[page] = image
        all_images[(w, h)] = images
    overview(all_images[(393, 873)], 393, 873).save(out / "portrait-overview.png", optimize=True)
    gallery(out, SCREENS)
    print(f"Wrote {len(SCREENS) * len(ALL_PAGES)} portrait QA images + gallery to {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
