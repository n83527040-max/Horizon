#!/usr/bin/env python3
"""Drive a real CRAFTIX Android game session on an emulator and assert it.

This exercises the *game* process (HorizonActivity), not the Flutter launcher:
first-run resource install, the replacement HUD, real touch input through the
craftix router, the More editor (create/save a custom key button on device),
engine key/mouse commands through the control socket, input-mode arbitration,
menus opened while a touch is held, focus loss, and persistence after a
restart. Every phase records a result in report.json; screenshots and logcat
are written beside it for human review.

The APK must be a non-public build (HORIZON_ANDROID_PUBLIC=0) so the control
socket and android-launch.json are compiled in. Requires adb root (google_apis
system image) to read the app's control.json and to start the non-exported
HorizonActivity directly.
"""
from __future__ import annotations

import argparse
import json
import math
import re
import socket
import subprocess
import sys
import time
from pathlib import Path

PKG = "com.craftix.launcher"
ACTIVITY = "dev.horizon.android.HorizonActivity"
LAUNCH_INTENT = ["am", "start", "-n", f"{PKG}/{ACTIVITY}", "--ez", "horizon.launch_game", "true"]
DATA = f"/data/data/{PKG}"
# Android's internalDataPath (what android_main calls `home`) is the
# Context.getFilesDir() directory, not the package root: control.json,
# craftix-controls-v1.json, saves/ and .local/ all live under files/.
HOME = f"{DATA}/files"
CONTROL_JSON = f"{HOME}/control.json"
LAYOUT_FILE = f"{HOME}/craftix-controls-v1.json"
READY_DIR = f"{HOME}/.local/horizon-client"

# Layout::default() (layout.rs), center/radius in surface fractions.
DEFAULT_BUTTONS = {
    "joystick": (0.11, 0.70, 0.15),
    "sprint": (0.275, 0.73, 0.05),
    "attack": (0.82, 0.50, 0.075),
    "use": (0.82, 0.70, 0.085),
    "jump": (0.94, 0.50, 0.065),
    "sneak": (0.94, 0.70, 0.065),
    "inventory": (0.74, 0.94, 0.04),
    "more": (0.26, 0.94, 0.04),
    "chat": (0.40, 0.05, 0.035),
    "pause": (0.46, 0.05, 0.035),
    "settings": (0.52, 0.05, 0.035),
    "players": (0.58, 0.05, 0.035),
    "perspective": (0.64, 0.05, 0.035),
}
# EditorUi::add_action places new custom buttons here.
NEW_BUTTON = (0.70, 0.30, 0.05)


def hud_scale(width: int, height: int) -> int:
    """HudBatch::new: largest scale keeping >= 320x240 GUI pixels."""
    scale = 1
    while width // (scale + 1) >= 320 and height // (scale + 1) >= 240:
        scale += 1
    return scale


def resolve_center(name: str, cx: float, cy: float, radius: float,
                   width: int, height: int) -> tuple[float, float]:
    """Layout::resolved(): hotbar avoidance + edge clamping (GUI pixels)."""
    scale = hud_scale(width, height)
    logical_w = math.ceil(width / scale)
    logical_h = math.ceil(height / scale)
    aspect = logical_w / logical_h
    r = min(radius, 0.5, aspect * 0.5)
    rx = r / aspect
    cx = min(max(cx, rx), 1.0 - rx)
    cy = min(max(cy, r), 1.0 - r)
    x = cx * logical_w
    y = cy * logical_h
    hotbar = [logical_w / 2.0 - 90.0, logical_h - 22.0, logical_w / 2.0 + 90.0, logical_h]
    closest_x = min(max(x, hotbar[0]), hotbar[2])
    closest_y = min(max(y, hotbar[1]), hotbar[3])
    if math.hypot(x - closest_x, y - closest_y) < r * logical_h:
        cy = min(max((hotbar[1] - r * logical_h - 3.0) / logical_h, r), 1.0 - r)
    return cx, cy


def button_xy(width: int, height: int, name: str) -> tuple[int, int]:
    cx, cy, r = DEFAULT_BUTTONS[name]
    cx, cy = resolve_center(name, cx, cy, r, width, height)
    return round(cx * width), round(cy * height)


class Adb:
    def __init__(self, binary: str = "adb"):
        self.adb = binary

    def run(self, *args: str, timeout: int = 60, check: bool = True) -> str:
        cmd = [self.adb, *args]
        proc = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
        if check and proc.returncode != 0:
            raise RuntimeError(f"adb {' '.join(args)} failed: {proc.stdout}\n{proc.stderr}")
        return proc.stdout + proc.stderr

    def shell(self, command: str, timeout: int = 60, check: bool = True) -> str:
        return self.run("shell", command, timeout=timeout, check=check)

    def root(self) -> None:
        out = self.run("root", check=False)
        if "already running as root" not in out and "adbd is already running as root" not in out:
            time.sleep(2)
        for _ in range(20):
            if self.shell("id -u", check=False).strip().startswith("0"):
                return
            time.sleep(1)
        raise RuntimeError("adb root did not become available")

    def wait_boot(self, timeout: int = 300) -> None:
        deadline = time.time() + timeout
        while time.time() < deadline:
            state = self.run("get-state", check=False)
            if "device" in state and self.shell(
                    "getprop sys.boot_completed", check=False).strip() == "1":
                return
            time.sleep(3)
        raise RuntimeError("emulator did not finish booting")

    def pidof(self) -> str:
        return self.shell(f"pidof {PKG} || true", check=False).strip()

    def force_stop(self) -> None:
        self.shell(f"am force-stop {PKG}", check=False)
        time.sleep(2)

    def start_engine(self) -> None:
        # Non-exported activity: adb root runs am as uid 0, which may start it.
        out = self.shell(" ".join(LAUNCH_INTENT), check=False)
        if "Error" in out or "Exception" in out:
            raise RuntimeError(f"could not start engine activity: {out}")

    def logcat(self, out_dir: Path, label: str) -> None:
        # Bounded: an engine-pegged emulator can stall a full dump.
        text = self.run("logcat", "-d", "-v", "brief", "-t", "8000",
                        timeout=45, check=False)
        (out_dir / f"logcat-{label}.txt").write_text(text, encoding="utf-8", errors="replace")


class Control:
    """horizon-ctl text protocol: auth token, then one JSON reply per line."""

    def __init__(self, host: str, port: int, token: str, timeout: float = 60.0):
        self.sock = socket.create_connection((host, port), timeout=10)
        self.sock.settimeout(timeout)
        self.file = self.sock.makefile("rwb", buffering=0)
        self.command(f"auth {token}", expect_ok=False)

    def command(self, line: str, expect_ok: bool = True) -> dict:
        self.file.write((line + "\n").encode())
        reply = self.file.readline()
        if not reply:
            raise RuntimeError("control socket closed")
        data = json.loads(reply.decode())
        if expect_ok and data.get("ok") is False:
            raise RuntimeError(f"control command {line!r} failed: {data}")
        return data

    def try_command(self, line: str) -> dict:
        try:
            return self.command(line, expect_ok=False)
        except Exception as error:  # noqa: BLE001 - recorded in the report
            return {"ok": False, "error": str(error)}

    def close(self) -> None:
        try:
            self.sock.close()
        except OSError:
            pass


class Session:
    def __init__(self, adb: Adb, out: Path):
        self.adb = adb
        self.out = out
        self.results: list[dict] = []
        self.control: Control | None = None
        self.width = 0
        self.height = 0

    def check(self, name: str, ok: bool, detail: str = "") -> bool:
        self.results.append({"name": name, "ok": bool(ok), "detail": detail})
        # Keep report.json on disk after every check so a crash still leaves
        # evidence in the uploaded artifact.
        try:
            self.write_report("interrupted")
        except OSError:
            pass
        print(f"{'PASS' if ok else 'FAIL'} {name}: {detail}", flush=True)
        if not ok:
            print(f"::error::game-test check failed: {name}: {detail}"[:1400], flush=True)
        return bool(ok)

    def write_report(self, status: str, extra: dict | None = None) -> dict:
        failed = [r for r in self.results if not r["ok"]]
        report = {
            "status": status,
            "passed": len(self.results) - len(failed),
            "failed": len(failed),
            "results": self.results,
        }
        if extra:
            report.update(extra)
        (self.out / "report.json").write_text(
            json.dumps(report, indent=2) + "\n", encoding="utf-8")
        return report

    def fatal(self, name: str, detail: str) -> None:
        # check() already emits the ::error:: annotation.
        self.check(name, False, detail)
        raise SystemExit(f"fatal: {name}: {detail}")

    def phase(self, name: str) -> None:
        print(f"== {name} ==", flush=True)
        print(f"::notice::game-test phase: {name}"[:300], flush=True)

    def install_state(self) -> str:
        """Compact, annotation-friendly view of the installer's on-device state."""
        out = self.adb.shell(
            f"ls {READY_DIR} {READY_DIR}/*/ {DATA}/files/.local "
            f"{DATA}/files/.local/*/ 2>/dev/null | tr '\\n' ' '; "
            f"echo; ls -R {DATA}/cache 2>/dev/null | head -24 | tr '\\n' ' '; "
            f"echo; du -sk {DATA}/cache {DATA}/files/.local 2>/dev/null | tr '\\n' ' '",
            check=False)
        return " ".join(out.split())[:400]

    def find_error_file(self) -> str:
        """Any launcher-written error file, whichever directory is home."""
        out = self.adb.shell(
            f"find {DATA} -maxdepth 4 -name android-error.txt 2>/dev/null | head -3",
            check=False).strip()
        first = out.splitlines()[0].strip() if out.strip() else ""
        if not first:
            return ""
        body = self.adb.shell(f"head -c 700 {first} 2>/dev/null", check=False)
        return f"{first}: {' '.join(body.split())[:700]}"

    def failure_snapshot(self) -> str:
        """One annotation-sized digest: startup log, directory tree, process.

        Artifact downloads are unreachable from the agent sandbox, so the
        evidence has to travel inside a GitHub annotation message.
        """
        out = self.adb.shell(
            f"echo 'HOMES:'; find {DATA} -maxdepth 3 \( -name android-error.txt "
            f"-o -name current-session.json -o -name diagnostic.log \) 2>/dev/null "
            f"| head -8 | tr '\\n' ' '; "
            f"echo; echo 'FILES:'; ls {DATA}/files 2>/dev/null | head -24 | tr '\\n' ' '; "
            f"echo; echo 'ERR:'; for f in $(find {DATA} -maxdepth 4 -name android-error.txt "
            f"2>/dev/null | head -2); do head -c 500 $f; echo; done; "
            f"echo 'DIAG:'; tail -c 900 {HOME}/performance/diagnostic.log 2>/dev/null; "
            f"echo 'PCACHE:'; ls -R {DATA}/cache 2>/dev/null | head -30 | tr '\\n' ' '",
            timeout=60, check=False)
        return " ".join(out.split())[:3600]

    def status(self) -> dict:
        if not self.control:
            raise RuntimeError("control not connected")
        return self.control.command("status")

    def wait_status(self, predicate, timeout: float, what: str) -> dict:
        deadline = time.time() + timeout
        last: dict = {}
        while time.time() < deadline:
            try:
                last = self.status()
                if predicate(last):
                    return last
            except Exception:  # noqa: BLE001
                pass
            time.sleep(2)
        raise RuntimeError(
            f"timeout waiting for {what}; screen={last.get('screen')} "
            f"keys={last.get('keys')} editing={last.get('craftix_editing_text')}")

    def display_size(self) -> tuple[int, int]:
        try:
            status = self.wait_status(
                lambda s: isinstance(s.get("display"), dict), 30, "display size")
            size = status["display"]
            width, height = int(size["width"]), int(size["height"])
            if width and height:
                return (width, height) if width >= height else (height, width)
        except RuntimeError:
            pass
        out = self.adb.shell("wm size", check=False)
        found = re.search(r"(\d+)x(\d+)", out)
        if not found:
            raise RuntimeError(f"cannot determine display size: {out}")
        width, height = int(found.group(1)), int(found.group(2))
        return (width, height) if width >= height else (height, width)

    def screenshot(self, name: str) -> None:
        path = f"{DATA}/craftix-test/{name}.png"
        self.adb.shell(f"mkdir -p {DATA}/craftix-test", check=False)
        if self.control:
            self.control.try_command(f"screenshot {path}")
        for _ in range(30):
            exists = self.adb.shell(f"[ -f {path} ] && echo yes || echo no",
                                    check=False).strip()
            if exists == "yes":
                break
            time.sleep(1)
        self.adb.run("pull", path, str(self.out / f"{name}.png"), check=False)

    def tap(self, x: int, y: int) -> None:
        self.adb.shell(f"input tap {x} {y}")

    def hold(self, x: int, y: int, ms: int) -> subprocess.Popen:
        """Start a touch hold in the background; caller finishes with wait()."""
        return subprocess.Popen(
            ["adb", "shell", "input", "swipe", str(x), str(y), str(x), str(y), str(ms)],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    @staticmethod
    def finish(proc: subprocess.Popen) -> None:
        proc.wait(timeout=30)

    def wait_for(self, predicate, timeout: float, what: str, interval: float = 1.0):
        deadline = time.time() + timeout
        while time.time() < deadline:
            value = predicate()
            if value:
                return value
            time.sleep(interval)
        raise RuntimeError(f"timeout waiting for {what}")

    def connect_control(self, timeout: float = 420.0) -> Control:
        """Poll control.json (written when the game starts), then connect."""
        self.wait_for(
            lambda: self.adb.shell(f"[ -s {CONTROL_JSON} ] && echo yes || echo no",
                                   check=False).strip() == "yes",
            timeout, "control.json")
        raw = self.adb.shell(f"cat {CONTROL_JSON}")
        info = json.loads(raw[raw.find("{"):])
        port = int(info["port"])
        token = info["token"]
        self.adb.run("forward", f"tcp:{port}", f"tcp:{port}", check=False)
        deadline = time.time() + 60
        last: Exception | None = None
        while time.time() < deadline:
            try:
                return Control("127.0.0.1", port, token)
            except Exception as error:  # noqa: BLE001
                last = error
                time.sleep(2)
        raise RuntimeError(f"cannot connect to control socket: {last}")

    def engine_ready_files(self) -> bool:
        out = self.adb.shell(
            f"ls {READY_DIR}/*/horizon-resources.json "
            f"{READY_DIR}/*/android-downloads-v2.json "
            f"{READY_DIR}/*/audio-complete.json 2>/dev/null | wc -l",
            check=False)
        return out.strip().splitlines()[-1].strip() == "3"


def craftix_layout_file() -> dict:
    """A valid Layouts document: the defaults plus a custom K button."""

    def button(bid, action, cx, cy, radius, behavior="hold", label=""):
        return {
            "id": bid, "action": action,
            "circle": {"center": [cx, cy], "radius": radius},
            "opacity": 0.85, "behavior": behavior, "icon": "text",
            "enabled": True, "visibility": "always", "label": label,
        }

    buttons = [
        button("joystick", None, 0.11, 0.70, 0.15),
        button("sprint", "sprint", 0.275, 0.73, 0.05),
        button("attack", "attack", 0.82, 0.50, 0.075),
        button("use", "use", 0.82, 0.70, 0.085),
        button("jump", "jump", 0.94, 0.50, 0.065),
        button("sneak", "sneak", 0.94, 0.70, 0.065),
        button("inventory", "inventory", 0.74, 0.94, 0.04, behavior="single"),
        button("more", "more", 0.26, 0.94, 0.04, behavior="single"),
        button("chat", "chat", 0.40, 0.05, 0.035, behavior="single"),
        button("pause", "pause", 0.46, 0.05, 0.035, behavior="single"),
        button("settings", "settings", 0.52, 0.05, 0.035, behavior="single"),
        button("players", "players", 0.58, 0.05, 0.035),
        button("perspective", "perspective", 0.64, 0.05, 0.035, behavior="single"),
        # The pre-seeded custom key button (outside the inventory container).
        button({"custom": 0}, {"key": "key.keyboard.k"}, 0.88, 0.86, 0.05, label="K"),
    ]
    return {
        "schema": 1, "active": 0,
        "items": [{
            "name": "Standard", "buttons": buttons,
            "joystick_dead_zone": 0.12, "joystick_sensitivity": 1.0,
            "camera_sensitivity": 1.0,
        }],
    }


def push_layout_file(adb: Adb) -> None:
    payload = json.dumps(craftix_layout_file())
    adb.shell(f"mkdir -p {DATA}")
    # Write as root, then hand the file to the app's uid.
    adb.shell(f"cat > {LAYOUT_FILE} <<'EOF'\n{payload}\nEOF")
    uid = adb.shell(f"stat -c %u:%g {DATA}").strip().splitlines()[-1].strip()
    adb.shell(f"chown {uid} {LAYOUT_FILE} && chmod 660 {LAYOUT_FILE}")


def run(args: argparse.Namespace) -> int:
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    adb = Adb(args.adb)
    session = Session(adb, out)
    logcat_label = "start"

    adb.wait_boot()
    adb.root()
    adb.run("install", "-r", args.apk, timeout=600)
    session.check("install", True, f"installed {Path(args.apk).name}")

    # Phase 1: first run installs engine resources (auto_play launch).
    session.phase("first run: engine resource install")
    print(f"::notice::game-test install waiting: {session.install_state()}", flush=True)
    adb.start_engine()
    time.sleep(10)
    startup = adb.shell(
        "logcat -d -v brief -t 600 2>/dev/null | grep -iE 'Horizon|panic|FATAL' "
        "| head -18 | head -c 1700", check=False)
    print(f"::notice::game-test startup-log: {' '.join(startup.split())}", flush=True)
    # The worker creates <cache>/minecraft-downloads as soon as action() runs.
    try:
        session.wait_for(
            lambda: adb.shell(
                f"[ -d {DATA}/cache/minecraft-downloads ] && echo yes || echo no",
                check=False).strip() == "yes",
            90, "install worker download cache", 3)
    except RuntimeError:
        adb.logcat(out, "worker-never-started")
        session.fatal(
            "install_worker_never_started",
            "action() did not spawn the install worker; "
            f"state: {session.install_state()}; snapshot: {session.failure_snapshot()}")
    install_deadline = time.time() + 3600
    last_progress = time.time()
    last_state = ""
    next_notice = time.time() + 300
    install_ready = False
    while time.time() < install_deadline:
        if session.engine_ready_files():
            install_ready = True
            break
        error_file = session.find_error_file()
        if error_file:
            adb.logcat(out, "install-error")
            session.fatal("first_run_install_error", error_file)
        state = session.install_state()
        if state != last_state:
            last_state, last_progress = state, time.time()
        elif time.time() - last_progress > 900:
            adb.logcat(out, "install-stalled")
            session.fatal(
                "first_run_install_stalled",
                f"no on-device progress for 15 minutes; state: {state or '(nothing)'}; "
                f"snapshot: {session.failure_snapshot()}")
        if time.time() >= next_notice:
            print(f"::notice::game-test install waiting: {state}", flush=True)
            next_notice = time.time() + 300
        time.sleep(10)
    if not install_ready:
        adb.logcat(out, "install-timeout")
        session.fatal(
            "first_run_install",
            f"resources not ready within 60 minutes; state: {last_state or '(nothing)'}; "
            f"snapshot: {session.failure_snapshot()}")
    session.check("first_run_install", True, "horizon-resources + downloads + audio present")
    adb.force_stop()

    # Phase 2: relaunch into the game with a pre-seeded controls file.
    session.phase("game launch")
    push_layout_file(adb)
    adb.start_engine()
    try:
        session.control = session.connect_control(420)
    except RuntimeError as error:
        # Renderer fallback: an emulator without Vulkan switches to OpenGL once.
        adb.logcat(out, "launch-failed")
        print(f"control socket missing ({error}); retrying with renderer=open-gl", flush=True)
        adb.force_stop()
        adb.shell(f"echo '{{\"renderer\":\"open-gl\"}}' > {HOME}/android-launcher.json")
        adb.start_engine()
        try:
            session.control = session.connect_control(420)
        except RuntimeError as error2:
            session.fatal(
                "game_launch",
                f"{error2}; state: {session.install_state()}; "
                f"snapshot: {session.failure_snapshot()}")
    status = session.wait_status(
        lambda s: s.get("screen") in ("title", "playing"), 300, "title screen")
    session.check("game_reached_title", True, f"screen={status.get('screen')}")
    title_log = adb.shell("logcat -d", check=False)
    (out / "logcat-title.txt").write_text(title_log, encoding="utf-8", errors="replace")
    session.check("replacement_controls_active_log",
                  "CRAFTIX replacement controls: active" in title_log,
                  "engine logged the replacement path at startup")

    # Phase 3: open a world through the control socket.
    session.phase("open world")
    reply = session.control.try_command("open CraftixTest 0 creative")
    if not reply.get("ok") and "already open" not in str(reply.get("error", "")):
        session.fatal("open_world", str(reply))
    status = session.wait_status(
        lambda s: s.get("screen") == "playing" and isinstance(s.get("player"), dict),
        420, "world open (screen=playing with a player)")
    session.width, session.height = session.display_size()
    session.check("world_playing", True,
                  f"player at {status['player']['position']} window={session.width}x{session.height}")
    time.sleep(4)
    session.screenshot("01-playing-hud")

    keys = set(session.status().get("keys", []))
    phys = set(session.status().get("physical_keys", []))
    session.check("no_stuck_keys_on_entry", not keys and not phys,
                  f"keys={sorted(keys)} physical={sorted(phys)}")

    w, h = session.width, session.height

    # Phase 4: real touch on HUD buttons (craftix router -> engine keys).
    session.phase("HUD touch tests")
    jx, jy = button_xy(w, h, "jump")
    proc = session.hold(jx, jy, 1500)
    saw_space = session.wait_for(
        lambda: "Space" in session.status().get("keys", []), 4, "Space key while holding Jump")
    session.check("hud_jump_hold_engine_key", bool(saw_space),
                  f"tap {jx},{jy} -> keys contains Space")
    session.screenshot("02-jump-held")
    session.finish(proc)
    time.sleep(1)
    keys = set(session.status().get("keys", []))
    session.check("hud_jump_release", "Space" not in keys, f"keys after release: {sorted(keys)}")

    sx, sy = button_xy(w, h, "sprint")
    proc = session.hold(sx, sy, 1200)
    saw_sprint = session.wait_for(
        lambda: "ControlLeft" in session.status().get("keys", []), 4,
        "sprint key while holding Sprint")
    session.check("hud_sprint_hold", bool(saw_sprint), "Sprint -> ControlLeft")
    session.finish(proc)
    time.sleep(1)

    nx, ny = button_xy(w, h, "sneak")
    proc = session.hold(nx, ny, 1200)
    saw_sneak = session.wait_for(
        lambda: "ShiftLeft" in session.status().get("keys", []), 4,
        "sneak key while holding Sneak")
    session.check("hud_sneak_hold", bool(saw_sneak), "Sneak -> ShiftLeft")
    session.finish(proc)
    time.sleep(1)

    # Attack: aim straight down and prove the touch swing lands on a block
    # (creative punches break instantly; mobs are too flaky for an assertion).
    session.control.command("hurtlog")  # drain
    session.control.command("summon zombie")
    zombie_up = session.wait_for(
        lambda: (session.status().get("mobs") or 0) > 0, 15, "zombie in status.mobs")
    session.check("zombie_spawned_for_attacks", bool(zombie_up),
                  f"status.mobs={session.status().get('mobs')}")
    session.control.command("look 0 89")  # nearly straight down
    time.sleep(1)
    st = session.status()
    before = st.get("target") or {}
    aim_ok = session.check(
        "attack_aim_has_target", bool(before),
        f"yaw={st.get('yaw')} pitch={st.get('pitch')} "
        f"pos={(st.get('player') or {}).get('position')} "
        f"on_ground={(st.get('player') or {}).get('on_ground')} "
        f"pending={st.get('pending_chunks')} target={json.dumps(before)[:120]}")
    broke = False
    if aim_ok:
        ax, ay = button_xy(w, h, "attack")
        proc = session.hold(ax, ay, 2500)
        session.finish(proc)
        try:
            broke = session.wait_for(
                lambda: (session.status().get("target") or {}) != before, 6,
                "block under the crosshair changed after the attack hold")
        except RuntimeError:
            broke = False
    session.check("hud_attack_breaks_block", bool(broke),
                  f"before {json.dumps(before)[:160]} -> after "
                  f"{json.dumps(session.status().get('target'))[:160]}")
    hurt = session.control.command("hurtlog")
    attacks = [p for p in hurt.get("players", []) if p.get("attacks")]
    session.check("hud_attack_registers", bool(attacks) or bool(broke),
                  f"mobs={session.status().get('mobs')} attacks={json.dumps(attacks)[:200]}")

    # Joystick: real touch drag -> player velocity. Stay grounded: a mid-air
    # teleport leaves only trivial air-strafe, which reads as a dead stick.
    grounded = session.wait_for(
        lambda: (session.status().get("player") or {}).get("on_ground"), 30,
        "player on the ground before the joystick drag")
    session.check("joystick_player_grounded", bool(grounded),
                  "on_ground before the drag")
    jox, joy = button_xy(w, h, "joystick")
    joy_y = joy
    pos0 = ((session.status().get("player") or {}).get("position") or [0, 0, 0])
    max_speed = 0.0

    stick_seen = {"craftix": 0.0, "pad": 0.0}

    def drag(dx: int, dy: int) -> None:
        """One swipe in screen space; sample velocity and stick signals."""
        nonlocal max_speed
        proc = subprocess.Popen(
            ["adb", "shell", "input", "swipe", str(jox), str(joy_y),
             str(jox + dx), str(joy_y + dy), "2200"],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        deadline = time.time() + 4
        while time.time() < deadline:
            status = session.status()
            player = status.get("player") or {}
            velocity = player.get("velocity") or [0, 0, 0]
            max_speed = max(max_speed, math.hypot(velocity[0], velocity[2]))
            cm = status.get("craftix_movement") or [0, 0]
            pl = status.get("pad_left") or [0, 0]
            stick_seen["craftix"] = max(stick_seen["craftix"],
                                        math.hypot(cm[0], cm[1]))
            stick_seen["pad"] = max(stick_seen["pad"], math.hypot(pl[0], pl[1]))
            time.sleep(0.4)
        session.finish(proc)

    # Push in several screen directions: one heading can be pinned against
    # terrain, which reads as a dead stick.
    step = int(w * 0.24)
    for dx, dy in ((step, 0), (-step, 0), (0, -step)):
        drag(dx, dy)
    pos1 = ((session.status().get("player") or {}).get("position") or [0, 0, 0])
    drift = math.hypot(float(pos1[0]) - float(pos0[0]), float(pos1[2]) - float(pos0[2]))
    session.check("hud_joystick_moves_player",
                  max_speed > 0.3 or drift > 1.0,
                  f"max horizontal speed {max_speed:.2f} m/s; player drift "
                  f"{drift:.2f} m over three swipes (pos {pos0} -> {pos1}); "
                  f"max |craftix_movement|={stick_seen['craftix']:.2f} "
                  f"max |pad_left|={stick_seen['pad']:.2f}")
    moved_log = adb.shell("logcat -d | grep -c 'CRAFTIX touch moved pointer'", check=False)
    session.check("touch_moved_events_arrive", int(moved_log or 0) > 0,
                  f"CRAFTIX touch Moved log lines: {moved_log}")
    # A drag over empty space is the camera (Role::Look): it proves Moved
    # events reach the router even when the stick itself misbehaves.
    yaw0 = float(session.status().get("yaw") or 0)
    session.control.command("look 0 0")
    time.sleep(1)
    yaw0 = float(session.status().get("yaw") or 0)
    lx, ly = round(0.45 * w), round(0.35 * h)
    subprocess.run(
        ["adb", "shell", "input", "swipe", str(lx), str(ly),
         str(lx + int(w * 0.25)), str(ly), "900"],
        check=False, timeout=15)
    time.sleep(1)
    yaw1 = float(session.status().get("yaw") or 0)
    session.check("look_drag_rotates_camera", abs(yaw1 - yaw0) > 2.0,
                  f"yaw {yaw0} -> {yaw1} during an empty-area drag")
    time.sleep(2)

    # Phase 5: More opens the editor; create + save a K button on device.
    session.phase("editor create/save")
    mx, my = button_xy(w, h, "more")
    session.tap(mx, my)
    opened = session.wait_for(
        lambda: "CRAFTIX controls editor opened" in adb.shell("logcat -d", check=False),
        10, "editor opened log")
    session.check("more_opens_editor", bool(opened), f"tap {mx},{my}")
    time.sleep(2)
    session.screenshot("03-editor-open")

    # Buttons page: "+ Add key M" at [0.2, 0.33]; new button lands at [0.7, 0.30].
    session.tap(round(0.2 * w), round(0.33 * h))
    time.sleep(1)
    # Key field at [0.5, 0.55] -> IME with the prefilled key token.
    session.tap(round(0.5 * w), round(0.55 * h))
    time.sleep(3)
    session.screenshot("04-editor-ime")
    def finish_field() -> bool:
        """End the IME session: BACK hides the keyboard first, then finishes
        the text field while keeping the editor open. Status reports both."""
        for _ in range(4):
            state = session.status()
            if not state.get("craftix_editing_text"):
                return bool(state.get("craftix_editor"))
            adb.shell("input keyevent 4")
            time.sleep(2)
        state = session.status()
        return (not state.get("craftix_editing_text")
                and bool(state.get("craftix_editor")))

    # Replace the prefilled value with "k" using injected editor text/keys.
    adb.shell("input keyevent 123")  # MOVE_END
    adb.shell("input keyevent " + " ".join(["67"] * 40))  # DEL x40
    adb.shell("input text k")
    time.sleep(1)
    session.check("editor_key_field_finished", finish_field(),
                  "key field accepted 'k' and the editor stayed open")
    session.screenshot("05-editor-key-set")

    # Name field at [0.5, 0.45] -> label "K".
    session.tap(round(0.5 * w), round(0.45 * h))
    time.sleep(3)
    adb.shell("input keyevent 123")
    adb.shell("input keyevent " + " ".join(["67"] * 40))
    adb.shell("input text K")
    time.sleep(1)
    session.check("editor_name_field_finished", finish_field(),
                  "name field accepted 'K' and the editor stayed open")

    # Save at [0.8, 0.95] (bottom bar, y > 0.90, x >= 0.66).
    time.sleep(1)
    session.tap(round(0.8 * w), round(0.95 * h))
    saved = session.wait_for(
        lambda: "CRAFTIX controls layout saved" in adb.shell("logcat -d", check=False),
        15, "layout saved log")
    session.check("editor_saves_layout", bool(saved), "Save button + log")
    session.screenshot("06-after-save")

    # The saved file must contain the created K button.
    raw = adb.shell(f"cat {LAYOUT_FILE}", check=False)
    try:
        saved_layout = json.loads(raw[raw.find("{"):raw.rfind("}") + 1])
        custom = [b for b in saved_layout["items"][0]["buttons"]
                  if isinstance(b.get("id"), dict)]
        created = [b for b in custom
                   if abs(b["circle"]["center"][0] - 0.70) < 0.01
                   and abs(b["circle"]["center"][1] - 0.30) < 0.01
                   and (b.get("action") or {}).get("key") == "key.keyboard.k"]
        session.check("saved_file_has_k_button", bool(created),
                      f"custom buttons: {json.dumps(custom)[:400]}")
    except Exception as error:  # noqa: BLE001
        session.check("saved_file_has_k_button", False, f"parse error: {error}; {raw[:200]}")

    # Hold the new K button: touch -> router -> bridge -> physical key state.
    kx, ky = round(0.70 * w), round(0.30 * h)
    proc = session.hold(kx, ky, 1500)
    saw_k = session.wait_for(
        lambda: "KeyK" in session.status().get("physical_keys", []), 5,
        "KeyK while holding the new button")
    session.check("hud_custom_k_reaches_engine", bool(saw_k),
                  f"tap {kx},{ky} -> physical_keys contains KeyK")
    session.screenshot("07-k-held")
    session.finish(proc)
    time.sleep(1)
    phys = set(session.status().get("physical_keys", []))
    session.check("hud_custom_k_release", "KeyK" not in phys,
                  f"physical_keys after release: {sorted(phys)}")

    # Editor close path (BrowserBack while no text field is active).
    session.tap(mx, my)
    session.wait_for(
        lambda: adb.shell("logcat -d", check=False).count("CRAFTIX controls editor opened") >= 2,
        10, "second editor open")
    adb.shell("input keyevent 4")
    closed = session.wait_for(
        lambda: "CRAFTIX controls editor closed" in adb.shell("logcat -d", check=False),
        10, "editor closed log")
    session.check("editor_closes_on_back", bool(closed), "BACK closes the editor")
    time.sleep(2)

    # Phase 6: inventory context (custom button visible outside the container).
    session.phase("inventory context")
    w, h = session.display_size()
    session.control.command("key e tap")
    inv = session.wait_status(lambda s: s.get("screen") in ("inventory", "creative"), 15,
                              "inventory screen")
    session.check("inventory_opens", inv.get("screen") in ("inventory", "creative"),
                  f"screen={inv.get('screen')}")
    time.sleep(2)
    session.screenshot("08-inventory")
    ikx, iky = round(0.88 * w), round(0.86 * h)
    proc = session.hold(ikx, iky, 1500)
    saw_k = session.wait_for(
        lambda: "KeyK" in session.status().get("physical_keys", []), 5,
        "KeyK from the inventory overlay")
    session.check("inventory_custom_button_works", bool(saw_k),
                  f"pre-seeded K button at {ikx},{iky} outside the container")
    session.finish(proc)
    time.sleep(1)
    session.control.command("key e tap")
    try:
        session.wait_status(lambda s: s.get("screen") == "playing", 25,
                            "back to playing after inventory close")
    except RuntimeError:
        screen_now = session.status().get("screen")
        if screen_now in ("inventory", "creative"):
            session.control.command("key e tap")
            try:
                session.wait_status(lambda s: s.get("screen") == "playing", 25,
                                    "back to playing after inventory retry")
            except RuntimeError:
                adb.logcat(out, "inventory-stuck")
                session.fatal(
                    "inventory_closes_on_e",
                    f"inventory still open after two 'key e' taps; "
                    f"state: {session.failure_snapshot()}")
        elif screen_now == "paused":
            session.control.command("resume")
            session.wait_status(lambda s: s.get("screen") == "playing", 25,
                                "back to playing after resume")
        else:
            adb.logcat(out, "inventory-stuck")
            session.fatal(
                "inventory_closes_on_e",
                f"screen={screen_now} after closing inventory; "
                f"state: {session.failure_snapshot()}")
    session.check("inventory_closes_on_e", True, "'key e' returned to playing")

    # Phase 7: menu opened while a touch is held; no stuck keys afterwards.
    session.phase("menu while holding")
    jx, jy = button_xy(w, h, "jump")
    proc = session.hold(jx, jy, 4000)
    session.wait_for(lambda: "Space" in session.status().get("keys", []), 4, "Space held")
    time.sleep(0.5)
    session.control.command("key esc tap")
    paused = session.wait_status(lambda s: s.get("screen") == "paused", 10, "pause menu")
    session.check("menu_opens_while_holding", paused.get("screen") == "paused", "esc while held")
    session.screenshot("09-paused-while-holding")
    session.finish(proc)
    time.sleep(1)
    session.control.command("resume")
    session.wait_status(lambda s: s.get("screen") == "playing", 10, "resume")
    time.sleep(1)
    keys = set(session.status().get("keys", []))
    phys = set(session.status().get("physical_keys", []))
    session.check("no_stuck_keys_after_menu", not keys and not phys,
                  f"keys={sorted(keys)} physical={sorted(phys)}")

    # Phase 8: physical mouse probe + engine-level mouse while touch holds.
    session.phase("touch + mouse")
    mouse_probe = adb.shell("input mouse motionevent DOWN 400 400", check=False)
    adb.shell("input mouse motionevent UP 400 400", check=False)
    mode_after_probe = session.status().get("input_mode")
    session.check("mouse_injection_probe", True,
                  f"adb input mouse output: {mouse_probe.strip()[:160]!r}; "
                  f"input_mode now {mode_after_probe}")
    # Engine-level mouse while a HUD button is held at the same time.
    jx, jy = button_xy(w, h, "jump")
    session.control.command("hurtlog")
    session.control.command("summon zombie")
    session.wait_for(lambda: (session.status().get("mobs") or 0) > 0, 15,
                     "zombie for simultaneous click")
    session.control.command("look 0 89")  # nearly straight down
    time.sleep(1)
    click_before = session.status().get("target") or {}
    proc = session.hold(jx, jy, 2500)
    session.wait_for(lambda: "Space" in session.status().get("keys", []), 4, "Space held")
    session.control.command("mouse left click")
    clicked = False
    try:
        clicked = session.wait_for(
            lambda: (session.status().get("target") or {}) != click_before, 6,
            "block changed after engine mouse click")
    except RuntimeError:
        clicked = False
    session.finish(proc)
    time.sleep(1)
    status = session.status()
    keys = set(status.get("keys", []))
    hurt = session.control.command("hurtlog")
    attacks = [p for p in hurt.get("players", []) if p.get("attacks")
               or p.get("hurts")]
    session.check("simultaneous_touch_and_mouse",
                  "Space" not in keys and bool(clicked),
                  f"HUD jump held + engine mouse click; keys after release {sorted(keys)}; "
                  f"block_changed={bool(clicked)}; click_before={json.dumps(click_before)[:100]}; "
                  f"mobs={status.get('mobs')}; mouse target record "
                  f"{json.dumps(attacks)[:120]}")
    # A real touchscreen tap must flip the active input back to Touch.
    session.tap(round(0.5 * w), round(0.45 * h))  # empty area: camera drag start/end
    time.sleep(1)
    status = session.status()
    session.check("touch_keeps_auto_preference",
                  status.get("input_preference") in ("Auto", "Touch", None),
                  f"input_mode={status.get('input_mode')} "
                  f"preference={status.get('input_preference')}")
    session.screenshot("10-after-mouse")

    # Phase 9: chat via engine key + injected text (Android key pipeline).
    session.phase("chat input")
    session.control.command("key t tap")
    session.wait_status(lambda s: s.get("screen") == "chat", 10, "chat screen")
    time.sleep(1)
    adb.shell("input text craftix")
    time.sleep(1)
    adb.shell("input keyevent 66")  # ENTER sends the line
    time.sleep(2)
    log = session.control.try_command("chat-log 10")
    text = json.dumps(log)
    session.check("chat_text_reaches_game", "craftix" in text.lower(),
                  f"chat-log: {text[:300]}")
    # ENTER already closed the chat; BACK here would hit gameplay and open
    # the pause menu, so settle back into playing (resuming if paused).
    try:
        session.wait_status(lambda s: s.get("screen") == "playing", 15,
                            "back to playing after chat")
    except RuntimeError:
        if session.status().get("screen") == "paused":
            session.control.command("resume")
        session.wait_status(lambda s: s.get("screen") == "playing", 15,
                            "back to playing after chat resume")

    # Phase 10: background/foreground focus loss.
    session.phase("focus loss")
    adb.shell("input keyevent KEYCODE_HOME")
    time.sleep(4)
    adb.start_engine()
    time.sleep(4)
    try:
        # The engine process may have restarted (dead socket): reconnect first.
        deadline = time.time() + 180
        status: dict = {}
        while time.time() < deadline:
            try:
                status = session.status()
                break
            except Exception:  # noqa: BLE001
                try:
                    if session.control:
                        session.control.close()
                except Exception:  # noqa: BLE001
                    pass
                try:
                    session.control = session.connect_control(60)
                except Exception:  # noqa: BLE001
                    time.sleep(3)
        if status.get("screen") not in ("playing", "paused", "title"):
            raise RuntimeError(f"no status after backgrounding: {status.get('screen')}")
        keys = set(status.get("keys", []))
        phys = set(status.get("physical_keys", []))
        session.check("resume_after_focus_loss", True,
                      f"screen={status.get('screen')} keys={sorted(keys)} phys={sorted(phys)}")
        if status.get("screen") == "title":
            session.control.try_command("open CraftixTest")
            session.wait_status(
                lambda s: s.get("screen") == "playing" and isinstance(s.get("player"), dict),
                420, "world after backgrounding")
        elif status.get("screen") == "paused":
            session.control.command("resume")
            session.wait_status(lambda s: s.get("screen") == "playing", 10, "resume after home")
        jx, jy = button_xy(w, h, "jump")
        proc = session.hold(jx, jy, 1500)
        saw = session.wait_for(
            lambda: "Space" in session.status().get("keys", []), 5, "Space after resume")
        session.check("hud_works_after_focus_loss", bool(saw), "Jump after backgrounding")
        session.finish(proc)
    except RuntimeError as error:
        session.check("resume_after_focus_loss", False, str(error))

    # Phase 11: full restart keeps the saved layout and the K button.
    session.phase("restart persistence")
    adb.force_stop()
    # A fresh process: control.json must be recreated.
    adb.shell(f"rm -f {CONTROL_JSON}", check=False)
    adb.start_engine()
    try:
        session.control.close()
    except Exception:  # noqa: BLE001
        pass
    session.control = session.connect_control(420)
    session.wait_status(lambda s: s.get("screen") in ("title", "playing"), 300,
                        "title after restart")
    session.control.try_command("open CraftixTest")
    session.wait_status(lambda s: s.get("screen") == "playing" and isinstance(s.get("player"), dict),
                        420, "world after restart")
    time.sleep(4)
    session.screenshot("11-after-restart")
    raw = adb.shell(f"cat {LAYOUT_FILE}", check=False)
    persisted = "key.keyboard.k" in raw
    session.check("layout_persisted_after_restart", persisted, f"file bytes {len(raw)}")
    w, h = session.display_size()
    kx, ky = round(0.70 * w), round(0.30 * h)
    proc = session.hold(kx, ky, 1500)
    saw_k = session.wait_for(
        lambda: "KeyK" in session.status().get("physical_keys", []), 5,
        "KeyK after restart")
    session.check("k_button_works_after_restart", bool(saw_k), "persisted layout live")
    session.finish(proc)

    # Logcat assertions for the replacement path being active.
    logcat = adb.shell("logcat -d", check=False)
    (out / "logcat-final.txt").write_text(logcat, encoding="utf-8", errors="replace")
    session.check("editor_opened_log_count",
                  logcat.count("CRAFTIX controls editor opened") >= 2,
                  f"count={logcat.count('CRAFTIX controls editor opened')}")
    session.check("editor_closed_log",
                  "CRAFTIX controls editor closed" in logcat, "close path logged")
    session.check("layout_saved_log",
                  "CRAFTIX controls layout saved" in logcat, "save path logged")

    # A final HUD screenshot for the human report.
    session.screenshot("12-final-hud")

    failed = [r for r in session.results if not r["ok"]]
    report = session.write_report(
        "completed",
        {"apk": str(args.apk), "window": [session.width, session.height]},
    )
    print(json.dumps({"passed": report["passed"], "failed": report["failed"]}, indent=2))
    return 1 if failed else 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apk", required=True, type=Path)
    parser.add_argument("--out", default="craftix-game-test", type=Path)
    parser.add_argument("--adb", default="adb")
    args = parser.parse_args()
    try:
        return run(args)
    except SystemExit:
        raise
    except Exception as error:  # noqa: BLE001
        print(f"FATAL {error}", flush=True)
        print(f"::error::game-test: {error}"[:1400], flush=True)
        try:
            out = Path(args.out)
            out.mkdir(parents=True, exist_ok=True)
            log = subprocess.run([args.adb, "logcat", "-d", "-t", "8000"],
                                 capture_output=True, text=True, timeout=45)
            (out / "logcat-on-failure.txt").write_text(
                log.stdout[-200000:], encoding="utf-8", errors="replace")
        except Exception:  # noqa: BLE001 - diagnostics are best effort
            pass
        return 2


if __name__ == "__main__":
    sys.exit(main())
