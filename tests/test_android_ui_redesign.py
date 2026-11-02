#!/usr/bin/env python3
"""Regression and design checks for the Horizon Android 0.7.6 UI redesign.

These tests do not need the Android SDK/NDK or a Rust toolchain: they verify
the source-level contract that the redesign must keep (every backend action,
the engine untouched, real data only) and the responsive layout geometry.
Run with: python3 -m unittest discover -s tests -p 'test_android*.py'
"""
from __future__ import annotations

import json
import re
import struct
import subprocess
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys_path = str(ROOT)
import sys  # noqa: E402
if sys_path not in sys.path:
    sys.path.insert(0, sys_path)

from tools import android_ui_layout as layout  # noqa: E402

SPEC = json.loads(
    (ROOT / "crates/horizon-client/src/android/ui_spec.json").read_text(encoding="utf-8")
)
PAGES = ROOT / "crates/horizon-client/src/android/launcher_pages.rs"
DESIGN = ROOT / "crates/horizon-client/src/android/design.rs"
LAUNCHER = ROOT / "crates/horizon-client/src/android/launcher.rs"
UI_THEME = ROOT / "crates/horizon-client/src/ui_theme.rs"
HERO = ROOT / "crates/horizon-client/src/android/assets/hero.png"


def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True)


def baseline_sha() -> str:
    return git("log", "--format=%H", "--grep=Import upstream Horizon Android v0.7.6", "-1").strip()


class IdentityTests(unittest.TestCase):
    def test_palette_matches_the_requested_identity(self):
        expected = {
            "bg": "#070B1C",
            "card": "#111A35",
            "primary": "#8B45FF",
            "secondary": "#315BFF",
            "cyan": "#20DDF5",
            "text": "#FFFFFF",
        }
        for name, value in expected.items():
            self.assertEqual(SPEC["palette"][name].upper(), value, name)

    def test_spec_has_the_five_nav_destinations(self):
        self.assertEqual(SPEC["nav_items"], ["Home", "Mods", "Packs", "Installed", "Settings"])

    def test_design_uses_the_shared_spec(self):
        source = DESIGN.read_text(encoding="utf-8")
        self.assertIn('include_str!("ui_spec.json")', source)
        # Every metric() key used by Rust must exist in JSON.
        for section, key in re.findall(r'metric\(\s*"([a-z_]+)"\s*,\s*"([a-z_]+)"\s*\)', source):
            self.assertIn(section, SPEC, f"missing section {section}")
            self.assertIn(key, SPEC[section], f"missing {section}.{key}")
        # One-argument helpers map to fixed sections.
        for section, helper in [("spacing", "space"), ("radius", "radius"), ("motion", "motion")]:
            for key in re.findall(rf'{helper}\(\s*"([a-z_]+)"\s*\)', source):
                self.assertIn(key, SPEC[section], f"missing {section}.{key}")

    def test_hero_asset_is_a_small_embedded_png(self):
        self.assertTrue(HERO.is_file())
        data = HERO.read_bytes()
        self.assertEqual(data[:8], b"\x89PNG\r\n\x1a\n")
        width, height = struct.unpack(">II", data[16:24])
        self.assertEqual((width, height), (512, 256))
        self.assertLess(len(data), 600_000, "hero must stay light for the atlas")
        self.assertIn('include_bytes!("assets/hero.png")', LAUNCHER.read_text(encoding="utf-8"))

    def test_ui_theme_helpers_are_additive(self):
        source = UI_THEME.read_text(encoding="utf-8")
        for needle in [
            "pub fn gradient(", "pub fn image(", "pub fn clip(", "pub fn vertices(",
            "pub fn install_hero(", "pub const HERO_SPRITE", "HERO_RESERVED",
        ]:
            self.assertIn(needle, source)
        # Desktop never reserves the hero region unless Android installs it.
        self.assertIn("static HERO_RESERVED: AtomicBool = AtomicBool::new(false)", source)


class ResponsiveTests(unittest.TestCase):
    def test_all_target_viewports_have_no_overflow(self):
        for w, h, name in layout.TEST_SCREENS:
            with self.subTest(screen=name):
                self.assertEqual(layout.check_layout(w, h), [])

    def test_safe_area_insets_are_respected(self):
        for top, bottom in [(24.0, 16.0), (48.0, 24.0), (0.0, 32.0)]:
            for w, h in [(360.0, 800.0), (393.0, 873.0), (432.0, 960.0)]:
                with self.subTest(screen=(w, h, top, bottom)):
                    self.assertEqual(layout.check_layout(w, h, top, bottom), [])

    def test_navigation_never_disappears_on_small_screens(self):
        for w, h in [(320.0, 568.0), (360.0, 640.0), (360.0, 800.0)]:
            l = layout.Layout.new(w, h)
            self.assertLessEqual(l.nav_y + l.nav_h, h)
            self.assertGreaterEqual(l.page_bottom, l.content_top)
            self.assertEqual(len([l.nav_item(i) for i in range(5)]), 5)

    def test_rust_and_python_layout_share_the_same_numbers(self):
        source = DESIGN.read_text(encoding="utf-8")
        # Both implementations must read the same spec keys.
        for key in ["hero", "play", "version", "feature", "performance", "nav", "header", "subheader"]:
            self.assertIn(key, SPEC)
            self.assertIn(key, source)
        self.assertIn("max_content_width", SPEC)
        self.assertIn("max_content_width", source)


class RegressionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.base = baseline_sha()
        cls.original = git("show", f"{cls.base}:crates/horizon-client/src/android/launcher_pages.rs")
        cls.current = PAGES.read_text(encoding="utf-8")

    def test_every_original_action_variant_survives(self):
        def variants(text):
            block = text.split("pub(super) enum Action {", 1)[1].split("\n}", 1)[0]
            return {m.group(1) for m in re.finditer(r"^\s*([A-Z][A-Za-z0-9_]*)\b", block, re.M)}
        missing = variants(self.original) - variants(self.current)
        self.assertEqual(missing, set(), f"removed actions: {sorted(missing)}")

    def test_every_original_activate_branch_survives(self):
        def arms(text):
            body = text.split("pub(super) fn activate(", 1)[1]
            return {m.group(1) for m in re.finditer(r"Some\(Action::([A-Z][A-Za-z0-9_]*)\)", body)}
        missing = arms(self.original) - arms(self.current)
        self.assertEqual(missing, set(), f"removed backend wiring: {sorted(missing)}")

    def test_engine_and_packaging_are_untouched(self):
        changed = git("diff", "--name-only", self.base, "HEAD").splitlines()
        protected = (
            "crates/horizon-client/src/game", "crates/horizon-client-render/",
            "crates/horizon-core/", "crates/horizon-world/", "crates/horizon-protocol/",
            "crates/horizon-mods/", "crates/horizon-shaders/", "launcher/src/",
            "packaging/android/AndroidManifest.xml", "packaging/android/java/",
            "tools/android_build.py", "tools/android_release.py", "tools/android_prepare.py",
            "Cargo.toml", "crates/horizon-android/Cargo.toml",
        )
        violations = [p for p in changed if p.startswith(protected)]
        self.assertEqual(violations, [], f"engine/build files changed: {violations}")

    def test_no_fake_performance_or_content_numbers(self):
        source = (PAGES.read_text(encoding="utf-8") + DESIGN.read_text(encoding="utf-8")).lower()
        for fake in ["4.2 gb", "8 gb", "12 installed", "5 installed", "60 fps", "120 fps"]:
            self.assertNotIn(fake, source, f"hardcoded demo value: {fake}")
        # Real readings only: /proc plus the engine's own renderer config.
        pages = PAGES.read_text(encoding="utf-8")
        self.assertIn("design::memory()", pages)
        self.assertIn("self.catalog.config.renderer.label()", pages)
        self.assertIn("self.catalog.local.iter()", pages)

    def test_nav_and_feature_cards_are_wired_to_real_pages(self):
        source = PAGES.read_text(encoding="utf-8")
        for action in [
            "Action::Page(Page::Mods)", "Action::Page(Page::Packs)",
            "Action::Page(Page::Shaders)", "Action::Page(Page::Installed)",
            "Action::Page(Page::Settings)", "Action::Play", "Action::Account",
        ]:
            self.assertIn(action, source, action)
        self.assertIn("let destinations = [Page::Play, Page::Mods, Page::Packs, Page::Installed, Page::Settings]", source)

    def test_no_new_dependencies_were_added(self):
        changed = git("diff", "--name-only", self.base, "HEAD").splitlines()
        cargo = [p for p in changed if p.endswith("Cargo.toml") or p == "Cargo.lock"]
        self.assertEqual(cargo, [], f"dependency manifests changed: {cargo}")


if __name__ == "__main__":
    unittest.main()
