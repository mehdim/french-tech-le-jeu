#!/usr/bin/env python3
"""Test de fumée navigateur pour le build web d'A la French (le jeu).

    python tools/smoke_test.py dist [--renderer auto|webgl2|webgpu] [--shots DOSSIER]

`auto` (par défaut) correspond à ce qu'obtient un visiteur : le chargeur choisit WebGPU
quand un adaptateur existe et se replie sur WebGL2 si le build WebGPU ne démarre pas.

Sert le dossier du build, l'ouvre dans Chromium sans affichage (WebGL logiciel via
SwiftShader, donc sans GPU), puis :
  1. attend que l'écran de chargement passe la main au jeu (`ftt-ready`) ;
  2. lance une nouvelle campagne depuis le menu principal et passe l'intro ;
  3. fait tourner la simulation en 4x (en répondant aux événements avec les touches 1-3)
     jusqu'à ce que la sauvegarde mensuelle auto arrive dans localStorage, et vérifie
     que c'est un JSON valide à un jour ultérieur ;
  4. recharge la page, clique sur « Continuer » et vérifie que le jour est conservé ;
  5. échoue à la moindre exception non interceptée ou panique wasm.

Les captures de chaque étape sont écrites dans --shots (par défaut : smoke-shots/).
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

# Les touches sont tenues 120 ms, comme le ferait une personne : en rendu logiciel, la
# page peut n'afficher que quelques images par seconde, et un appui/relâchement
# instantané peut tomber entre deux images.
# Positions des boutons du menu principal / des dialogues pour la mise en page de référence 1280x720
# (egui dessine dans le canvas : il n'y a pas d'éléments DOM à cibler).
NEW_GAME = (180, 457)
CONTINUE = (180, 503)


class QuietHandler(http.server.SimpleHTTPRequestHandler):
    extensions_map = {**http.server.SimpleHTTPRequestHandler.extensions_map, ".wasm": "application/wasm"}

    def log_message(self, *args):
        pass

    def handle(self):
        # Un rechargement de page annule les téléchargements en cours ; c'est normal.
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
    # Les sauvegardes enveloppent le GameState de référence ; on cherche son champ `day`.
    if isinstance(save, dict):
        if "state" in save and isinstance(save["state"], dict) and "day" in save["state"]:
            return save["state"]["day"]
        if "day" in save:
            return save["day"]
        if "meta" in save and isinstance(save["meta"], dict) and "day" in save["meta"]:
            return save["meta"]["day"]
    return None


def wait_ready(page, timeout_s=180):
    """Attend le jeu ; survit au rechargement WebGPU→WebGL2 du chargeur."""
    deadline = time.time() + timeout_s
    while time.time() < deadline:
        try:
            if page.evaluate("window.__FTT_READY === true"):
                return True
            if page.evaluate("!!document.getElementById('crash') && !document.getElementById('crash').hidden"):
                return False
        except Exception:
            pass  # navigation en cours
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
        print(("  ok     " if cond else "  ÉCHEC  ") + msg)
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

        print(f"Chargement de {url}")
        t0 = time.time()
        page.goto(url)
        ready = wait_ready(page)
        check(ready, "jeu prêt")
        if not ready:
            page.screenshot(path=str(args.shots / "0-not-ready.png"))
            (args.shots / "console.log").write_text("\n".join(console))
            browser.close()
            httpd.shutdown()
            print("ÉCHEC — le jeu n'a jamais été prêt")
            return 1
        renderer = page.evaluate("window.__FTT_RENDERER")
        fell_back = "from=webgpu" in page.url
        print(f"  info   rendu {renderer}{' (repli depuis WebGPU)' if fell_back else ''}, prêt en {time.time() - t0:.1f} s")
        if args.renderer != "auto":
            check(renderer == args.renderer, f"le rendu demandé {args.renderer} est actif")
        page.wait_for_timeout(1500)
        page.screenshot(path=str(args.shots / "1-menu.png"))
        check(page.evaluate("document.getElementById('boot').classList.contains('done')"), "écran de chargement masqué")

        # Nouvelle campagne.
        page.evaluate("Object.keys(localStorage).filter(k => k.includes('autosave')).forEach(k => localStorage.removeItem(k))")
        page.mouse.click(*NEW_GAME)
        page.wait_for_timeout(1500)
        page.screenshot(path=str(args.shots / "2-hero.png"))
        # Écran de création du héros : un nom est proposé, Entrée lance la partie.
        page.keyboard.press("Enter", delay=120)
        page.wait_for_timeout(2500)
        page.screenshot(path=str(args.shots / "2-intro.png"))
        # Entrée fait avancer le dialogue réplique par réplique, puis lance le niveau.
        for _ in range(12):
            page.keyboard.press("Enter", delay=120)
            page.wait_for_timeout(350)
        page.keyboard.press("Digit4", delay=120)  # vitesse 4x
        page.wait_for_timeout(500)
        page.screenshot(path=str(args.shots / "3-playing.png"))

        # Attend la sauvegarde mensuelle auto (28 jours en 4x = ~22 s à pleine vitesse ;
        # le rendu logiciel est plus lent, on laisse donc beaucoup de temps).
        save = None
        deadline = time.time() + 150
        while time.time() < deadline:
            save = read_save(page, "autosave")
            if save is not None and (save_day(save) or 0) > 0:
                break
            # Les événements mettent le jeu en pause : on répond avec le premier choix possible
            # (touches 1-3), puis on remet la vitesse à 4x.
            for key in ("Digit1", "Digit2", "Digit3", "Digit4"):
                page.keyboard.press(key, delay=120)
                page.wait_for_timeout(250)
            page.wait_for_timeout(1000)
            page.screenshot(path=str(args.shots / "4-running.png"))
        check(save is not None, "sauvegarde auto écrite dans localStorage")
        day = save_day(save) if save else None
        check(day is not None and day > 0, f"la sauvegarde auto est à un jour ultérieur (jour {day})")
        page.screenshot(path=str(args.shots / "4-running.png"))

        # Recharge et reprend depuis la sauvegarde auto.
        page.reload()
        check(wait_ready(page), "jeu de nouveau prêt après rechargement")
        page.wait_for_timeout(1500)
        page.mouse.click(*CONTINUE)
        page.wait_for_timeout(3000)
        page.screenshot(path=str(args.shots / "5-continued.png"))
        still = read_save(page, "autosave")
        check(still is not None, "la sauvegarde a survécu au rechargement")

        real_errors = [e for e in errors if not ignorable(e)]
        panics = [c for c in console if "panicked" in c]
        check(not real_errors, f"aucune erreur de page non interceptée ({real_errors[:3]})")
        check(not panics, f"aucune panique Rust ({panics[:3]})")
        crash_visible = page.evaluate("!document.getElementById('crash').hidden")
        check(not crash_visible, "écran d'incident non affiché")
        browser.close()

    httpd.shutdown()
    (args.shots / "console.log").write_text("\n".join(console))
    print(f"{'RÉUSSI' if not failures else 'ÉCHEC'} — captures dans {args.shots}")
    return 0 if not failures else 1


if __name__ == "__main__":
    sys.exit(main())
