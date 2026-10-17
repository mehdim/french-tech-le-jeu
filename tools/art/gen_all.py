"""Génère tous les éléments visuels et les regroupe en atlas.

Usage : python3 tools/art/gen_all.py   (à lancer depuis la racine du dépôt)
Sorties (toutes versionnées, pour que le jeu se compile sans Python) :
  assets/tilesets/office.png        sols, murs, paillasson
  assets/sprites/furniture.png      mobilier (+ sprites d'effets)
  assets/sprites/decor.png          décorations murales, déformées pour chaque mur
  assets/characters/characters.png  images de 16x26, 16 par ligne
  assets/ui/portraits.png           portraits de dialogue en 40x40
  assets/ui/icons.png               icônes du HUD en 12x12
  assets/backgrounds/*.png          écran titre + fins
  assets/fonts/VeridixPixel.ttf     police pixel originale
  assets/sprites/atlas.json         rectangles et ancres de tout ce qui précède
"""
import json
import os
import subprocess
import sys

sys.path.insert(0, os.path.dirname(__file__))
from pixlib import *                       # noqa: E402
import gen_tiles as T                      # noqa: E402
import gen_furniture as F                  # noqa: E402
import gen_characters as C                 # noqa: E402
import gen_portraits as P                  # noqa: E402
import gen_scenes as S                     # noqa: E402
import gen_font as FONT                    # noqa: E402

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
A = os.path.join(ROOT, "assets")


def pack(items, width=512, pad=1):
    """Range {nom: (img, ancre)} en étagères -> (planche, {nom: {rect, ancre}})."""
    order = sorted(items.items(), key=lambda kv: -kv[1][0].height)
    x = y = shelf_h = 0
    placed = {}
    for name, (img, anchor) in order:
        if x + img.width + pad > width:
            x = 0
            y += shelf_h + pad
            shelf_h = 0
        placed[name] = (x, y, img, anchor)
        x += img.width + pad
        shelf_h = max(shelf_h, img.height)
    height = y + shelf_h + pad
    h2 = 1
    while h2 < height:
        h2 *= 2
    sheet = new(width, h2)
    meta = {}
    for name, (x, y, img, anchor) in placed.items():
        sheet.alpha_composite(img, (x, y))
        meta[name] = {"rect": [x, y, img.width, img.height], "anchor": list(anchor)}
    return sheet, meta


def shear_right(flat):
    fw, fh = flat.size
    out = new(fw, fh + fw // 2 + 1)
    for x in range(fw):
        dy = x // 2
        for y in range(fh):
            c = flat.getpixel((x, y))
            if c[3]:
                px(out, x, y + dy, c)
    return out


def fx_sprites():
    out = {}
    # contour de sélection en losange (une case)
    sel = new(TW, TH)
    for y, xs, xe in T.diamond_rows():
        px(sel, xs, y, GREEN); px(sel, xe, y, GREEN)
        if y in (0, TH - 1):
            for x in range(xs, xe + 1):
                px(sel, x, y, GREEN)
    out["fx_select"] = (sel, (HALF_W, 0))
    for name, col in (("fx_place_ok", (124, 255, 82, 110)), ("fx_place_bad", (228, 91, 91, 120))):
        t = new(TW, TH)
        for y, xs, xe in T.diamond_rows():
            for x in range(xs, xe + 1):
                px(t, x, y, col)
        out[name] = (t, (HALF_W, 0))
    steam = new(5, 7)
    for (x, y) in [(1, 6), (2, 5), (2, 4), (1, 3), (1, 2), (2, 1), (3, 0)]:
        px(steam, x, y, (230, 230, 230, 200))
    out["fx_steam"] = (steam, (2, 6))
    euro = new(7, 9)
    tiny_text(euro, 2, 2, "€", GREEN)
    outline_alpha(euro, DEEP_GREEN)
    out["fx_euro"] = (euro, (3, 4))
    smoke = new(9, 9)
    for yy in range(9):
        for xx in range(9):
            if (xx - 4) ** 2 + (yy - 4) ** 2 <= 16:
                px(smoke, xx, yy, (80, 80, 80, 200) if (xx + yy) % 3 else (110, 110, 110, 200))
    out["fx_smoke"] = (smoke, (4, 4))
    spark = new(5, 5)
    for (x, y) in [(2, 0), (2, 1), (0, 2), (1, 2), (2, 2), (3, 2), (4, 2), (2, 3), (2, 4)]:
        px(spark, x, y, YELLOW)
    out["fx_spark"] = (spark, (2, 2))
    for name, txt, col in (("bubble_alert", "!", RED), ("bubble_idea", "?", YELLOW), ("bubble_money", "€", GREEN), ("bubble_heart", "+", PINK)):
        b = new(11, 11)
        rect(b, 0, 0, 11, 8, OFFWHITE)
        px(b, 3, 8, OFFWHITE); px(b, 4, 8, OFFWHITE); px(b, 3, 9, OFFWHITE)
        tiny_text(b, 4, 1, txt, col)
        outline_alpha(b)
        out[name] = (b, (3, 10))
    shadow = new(14, 5)
    for yy in range(5):
        for xx in range(14):
            if ((xx - 6.5) / 7) ** 2 + ((yy - 2) / 2.5) ** 2 <= 1:
                px(shadow, xx, yy, (0, 0, 0, 70))
    out["fx_shadow"] = (shadow, (7, 2))
    return out


def main():
    manifest = {"tile": [TW, TH], "wall_height": T.WALL_H}

    # --- tileset du bureau -------------------------------------------------
    tiles = {}
    for name, pat in T.FLOORS.items():
        for v in range(2):
            tiles[f"floor_{name}_{v}"] = (T.floor_tile(pat, v), (HALF_W, 0))
    for tier in T.WALL_STYLES:
        for side in ("L", "R"):
            for kind in ("plain", "window"):
                img = T.wall_segment(tier, side, kind)
                # ancre = point au sol à l'extrémité du segment côté coin du fond
                anchor = (16, img.height - 9) if side == "L" else (0, img.height - 9)
                tiles[f"wall_{tier}_{side}_{kind}"] = (img, anchor)
    sheet, meta = pack(tiles, 512)
    sheet.save(os.path.join(A, "tilesets", "office.png"))
    manifest["office"] = meta

    # --- mobilier + effets --------------------------------------------------
    furn = {name: fn() for name, fn in F.FURNITURE.items()}
    furn.update(fx_sprites())
    sheet, meta = pack(furn, 512)
    sheet.save(os.path.join(A, "sprites", "furniture.png"))
    manifest["furniture"] = meta

    # --- décorations murales (pré-déformées pour chaque mur) ----------------
    decor = {}
    for name, img in T.decor_images().items():
        decor[f"{name}_L"] = (T.shear_left(img, 17), (img.width, 0))
        decor[f"{name}_R"] = (shear_right(img), (0, 0))
    sheet, meta = pack(decor, 256)
    sheet.save(os.path.join(A, "sprites", "decor.png"))
    manifest["decor"] = meta

    # --- personnages --------------------------------------------------------
    chars = C.atlas()
    chars.save(os.path.join(A, "characters", "characters.png"))
    manifest["characters"] = {
        "frame": [C.FW, C.FH], "frames_per_row": C.FRAMES, "looks": C.LOOKS, "roles": C.ROLES,
        "anims": {"idle": [0, 2], "walk": [2, 4], "work": [6, 2], "coffee": [8, 2], "talk": [10, 2],
                  "tired": [12, 2], "panic": [14, 2]},
        "anchor": [C.FW // 2, C.FH - 1],
    }

    # --- portraits et icônes ------------------------------------------------
    ps = P.portraits()
    sheet = new(P.P * len(ps), P.P)
    pm = {}
    for i, (k, v) in enumerate(ps.items()):
        sheet.alpha_composite(v, (i * P.P, 0))
        pm[k] = {"rect": [i * P.P, 0, P.P, P.P]}
    sheet.save(os.path.join(A, "ui", "portraits.png"))
    manifest["portraits"] = pm
    ic = S.icons()
    sheet = new(12 * len(ic), 12)
    im = {}
    for i, (k, v) in enumerate(ic.items()):
        sheet.alpha_composite(v, (i * 12, 0))
        im[k] = {"rect": [i * 12, 0, 12, 12]}
    sheet.save(os.path.join(A, "ui", "icons.png"))
    manifest["icons"] = im

    # --- fonds --------------------------------------------------------------
    S.title_background().save(os.path.join(A, "backgrounds", "title.png"))
    for name, fn in (("ending_unicorn", S.ending_unicorn), ("ending_ipo", S.ending_ipo), ("ending_profitable", S.ending_profitable),
                     ("ending_acquisition", S.ending_acquisition), ("ending_commons", S.ending_commons)):
        fn().save(os.path.join(A, "backgrounds", f"{name}.png"))

    # --- polices ------------------------------------------------------------
    FONT.build(os.path.join(A, "fonts", "VeridixPixel.ttf"))
    for src, dst in (("DejaVuSans.ttf", "DejaVuSans-Subset.ttf"), ("DejaVuSans-Bold.ttf", "DejaVuSans-Bold-Subset.ttf")):
        path = f"/usr/share/fonts/truetype/dejavu/{src}"
        if os.path.exists(path):
            subprocess.run(["pyftsubset", path, "--unicodes=U+0020-007E,U+00A0-017F,U+2010-2027,U+2030-203A,U+20AC,U+2190-2195,U+2212,U+221E,U+2248,U+2260,U+2264,U+2265,U+25A0-25FF,U+2600-26FF,U+2713,U+2714,U+2717",
                            f"--output-file={os.path.join(A, 'fonts', dst)}", "--layout-features=*"], check=True)

    with open(os.path.join(A, "sprites", "atlas.json"), "w") as f:
        json.dump(manifest, f, indent=1, sort_keys=True)
    print("éléments générés")


if __name__ == "__main__":
    main()
