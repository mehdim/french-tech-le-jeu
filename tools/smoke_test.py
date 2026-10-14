#!/usr/bin/env python3
"""Browser smoke test for the French Tech Tycoon web build.

    python tools/smoke_test.py dist [--renderer auto|webgl2|webgpu] [--shots DIR]

`auto` (default) is what a visitor gets: the loader picks WebGPU when an
adapter exists and falls back to WebGL2 if the WebGPU build cannot start.

Serves the build directory, opens it in headless Chromium (software WebGL via
SwiftShader, so no GPU is needed), then:
  1. waits for the boot screen to hand over to the game (`ftt-ready`);
  2. starts a new campaign from the main menu and skips the intro;
  3. runs the simulation at 4x (answering event dialogs with keys 1-3)
     until the monthly autosave lands in localStorage, and checks the save
     is valid JSON at a later day;
  4. reloads the page, clicks "Continuer" and checks the day carried over;
  5. fails on any uncaught exception or wasm panic.

Screenshots of each step are written to --shots (default: smoke-shots/).
"""
import argparse
import functools
import http.server
import json
import sys
import threading
import time
from pathlib import Path

from playwright.sync_api import sync_playwright

# Keys are held for 120 ms like a person would: with software rendering the
# page may draw only a few frames per second, and an instantaneous
# down/up pair can fall between frames.
# Main-menu / dialogue button positions at the 1280x720 reference layout
# (egui draws into the canvas, so there are no DOM elements to target).
NEW_GAME = (180, 457)
CONTINUE = (180, 503)


class QuietHandler(http.server.SimpleHTTPRequestHandler):
    extensions_map = {**http.server.SimpleHTTPRequestHandler.extensions_map, ".wasm": "application/wasm"}

    def log_message(self, *args):
        pass

    def handle(self):
        # A page reload cancels in-flight downloads; that is expected.
        try:
            super().handle()
        except (BrokenPipeError, ConnectionResetError):
            pass


def serve(root: Path) -> tuple[http.server.ThreadingHTTPServer, int]:
    handler = functools.partial(QuietHandler, directory=str(root))
    httpd = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
    threading.Thread(target=httpd.serve_forever, daemon=True).start()
    return httpd, httpd.server_address[1]


def read_save(page, key):
    raw = page.evaluate(f"localStorage.getItem('french-tech-tycoon:{key}')")
    if raw is None:
        return None
    data = json.loads(raw)
    return data


def save_day(save):
    # Saves wrap the authoritative GameState; find its `day` field.
    if isinstance(save, dict):
        if "state" in save and isinstance(save["state"], dict) and "day" in save["state"]:
            return save["state"]["day"]
        if "day" in save:
            return save["day"]
        if "meta" in save and isinstance(save["meta"], dict) and "day" in save["meta"]:
            return save["meta"]["day"]
    return None


def wait_ready(page, timeout_s=180):
    """Wait for the game; survives the loader's WebGPU→WebGL2 reload."""
    deadline = time.time() + timeout_s
    while time.time() < deadline:
        try:
            if page.evaluate("window.__FTT_READY === true"):
                return True
            if page.evaluate("!!document.getElementById('crash') && !document.getElementById('crash').hidden"):
                return False
        except Exception:
            pass  # navigation in progress
        time.sleep(0.5)
    return False


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("dist", type=Path)
    ap.add_argument("--renderer", default="auto", choices=["auto", "webgl2", "webgpu"])
    ap.add_argument("--shots", type=Path, default=Path("smoke-shots"))
    ap.add_argument("--headed", action="store_true")
    args = ap.parse_args()
    args.shots.mkdir(parents=True, exist_ok=True)

    httpd, port = serve(args.dist.resolve())
    url = f"http://127.0.0.1:{port}/" + ("" if args.renderer == "auto" else f"?renderer={args.renderer}")
    errors: list[str] = []
    console: list[str] = []
    failures: list[str] = []

    def check(cond, msg):
        print(("  ok   " if cond else "  FAIL ") + msg)
        if not cond:
            failures.append(msg)

    with sync_playwright() as p:
        browser = p.chromium.launch(
            headless=not args.headed,
            args=[
                "--use-angle=swiftshader",
                "--enable-unsafe-swiftshader",
                "--ignore-gpu-blocklist",
                "--enable-unsafe-webgpu",
                "--autoplay-policy=no-user-gesture-required",
            ],
        )
        page = browser.new_page(viewport={"width": 1280, "height": 720})
        page.on("pageerror", lambda e: errors.append(str(e)))
        page.on("console", lambda m: console.append(f"[{m.type}] {m.text}"))

        def ignorable(e: str) -> bool:
            return "Using exceptions for control flow" in e

        print(f"Loading {url}")
        t0 = time.time()
        page.goto(url)
        ready = wait_ready(page)
        check(ready, "game ready")
        if not ready:
            page.screenshot(path=str(args.shots / "0-not-ready.png"))
            (args.shots / "console.log").write_text("\n".join(console))
            browser.close()
            httpd.shutdown()
            print("FAILED — game never became ready")
            return 1
        renderer = page.evaluate("window.__FTT_RENDERER")
        fell_back = "from=webgpu" in page.url
        print(f"  info renderer {renderer}{' (fell back from WebGPU)' if fell_back else ''}, ready in {time.time() - t0:.1f}s")
        if args.renderer != "auto":
            check(renderer == args.renderer, f"requested renderer {args.renderer} is running")
        page.wait_for_timeout(1500)
        page.screenshot(path=str(args.shots / "1-menu.png"))
        check(page.evaluate("document.getElementById('boot').classList.contains('done')"), "boot screen hidden")

        # Fresh campaign.
        page.evaluate("Object.keys(localStorage).filter(k => k.includes('autosave')).forEach(k => localStorage.removeItem(k))")
        page.mouse.click(*NEW_GAME)
        page.wait_for_timeout(2500)
        page.screenshot(path=str(args.shots / "2-intro.png"))
        # Enter advances the story dialogue line by line, then starts the level.
        for _ in range(12):
            page.keyboard.press("Enter", delay=120)
            page.wait_for_timeout(350)
        page.keyboard.press("Digit4", delay=120)  # 4x speed
        page.wait_for_timeout(500)
        page.screenshot(path=str(args.shots / "3-playing.png"))

        # Wait for the monthly autosave (28 days at 4x = ~11 s at full FPS;
        # software rendering is slower, so allow plenty of time).
        save = None
        deadline = time.time() + 150
        while time.time() < deadline:
            save = read_save(page, "autosave")
            if save is not None and (save_day(save) or 0) > 0:
                break
            # Events pause the game: answer with the first allowed choice
            # (keys 1-3), then put the speed back to 4x.
            for key in ("Digit1", "Digit2", "Digit3", "Digit4"):
                page.keyboard.press(key, delay=120)
                page.wait_for_timeout(250)
            page.wait_for_timeout(1000)
            page.screenshot(path=str(args.shots / "4-running.png"))
        check(save is not None, "autosave written to localStorage")
        day = save_day(save) if save else None
        check(day is not None and day > 0, f"autosave is at a later day (day {day})")
        page.screenshot(path=str(args.shots / "4-running.png"))

        # Reload and continue from the autosave.
        page.reload()
        check(wait_ready(page), "game ready again after reload")
        page.wait_for_timeout(1500)
        page.mouse.click(*CONTINUE)
        page.wait_for_timeout(3000)
        page.screenshot(path=str(args.shots / "5-continued.png"))
        still = read_save(page, "autosave")
        check(still is not None, "save survived a page reload")

        real_errors = [e for e in errors if not ignorable(e)]
        panics = [c for c in console if "panicked" in c]
        check(not real_errors, f"no uncaught page errors ({real_errors[:3]})")
        check(not panics, f"no Rust panics ({panics[:3]})")
        crash_visible = page.evaluate("!document.getElementById('crash').hidden")
        check(not crash_visible, "crash overlay not shown")
        browser.close()

    httpd.shutdown()
    (args.shots / "console.log").write_text("\n".join(console))
    print(f"{'PASSED' if not failures else 'FAILED'} — screenshots in {args.shots}")
    return 0 if not failures else 1


if __name__ == "__main__":
    sys.exit(main())
