"""Salles isométriques en pixel art pour la fin « Les Communs » : la remise de
médaille à l'Élysée, puis l'agence France Travail du lendemain matin.

Même grammaire que le bureau du jeu : cases de 32x16, murs de 60 px, mobilier en
blocs isométriques et personnages de 16x26 tirés de gen_characters. Le héros n'est
pas dessiné ici : le client l'ajoute par-dessus (apparence choisie, animation de marche).
Les positions utiles au client (chemin du héros, mains, écran des tickets) sont
calculées avec ORIGIN et iso_pt, et recopiées dans crates/game-client/src/ui/finale.rs.
"""
import math
import random

from pixlib import *
import gen_characters as C
import gen_furniture as F
import gen_tiles as T

W_IMG, H_IMG = 320, 180
ROOM_W, ROOM_D = 7, 6          # cases le long du mur droit (gx) et du mur gauche (gy)
ORIGIN = (152, 64)              # coin du fond, au sol
WALL = T.WALL_H

BG = (16, 20, 18, 255)


def P(gx, gy, gz=0):
    return iso_pt(ORIGIN, gx, gy, gz)


def wall_quad(img, side, t0, t1, z0, z1, color, outline=None):
    """Quadrilatère plaqué sur le mur gauche (gx = 0, t = gy) ou droit (gy = 0, t = gx)."""
    if side == "L":
        pts = [P(0, t0, z0), P(0, t1, z0), P(0, t1, z1), P(0, t0, z1)]
    else:
        pts = [P(t0, 0, z0), P(t1, 0, z0), P(t1, 0, z1), P(t0, 0, z1)]
    poly(img, pts, color, outline)


def wall_line(img, side, t0, t1, z, color):
    a = P(0, t0, z) if side == "L" else P(t0, 0, z)
    b = P(0, t1, z) if side == "L" else P(t1, 0, z)
    line(img, a, b, color)


def wall_text(img, side, t, z, text, color):
    """Texte en minuscules pixels, cisaillé le long du mur."""
    flat = new(len(text) * 4 + 2, 7)
    tiny_text(flat, 1, 1, text, color)
    x0, y0 = P(0, t, z) if side == "L" else P(t, 0, z)
    for y in range(flat.height):
        for x in range(flat.width):
            c = flat.getpixel((x, y))
            if c[3]:
                # le long du mur, 2 px horizontaux pour 1 px vertical
                dy = -(x // 2) if side == "L" else (x // 2)
                dx = x
                px(img, x0 + dx, y0 + dy + y, c)


def base_room(floor_fn, wall_col, cap_col):
    img = Image.new("RGBA", (W_IMG, H_IMG), BG)
    # murs du fond
    wall_quad(img, "L", 0, ROOM_D, 0, WALL, wall_col)
    wall_quad(img, "R", 0, ROOM_W, 0, WALL, shade(wall_col, 0.9))
    # chant supérieur, comme les murs du bureau
    poly(img, [P(0, 0, WALL), P(0, ROOM_D, WALL), P(-0.15, ROOM_D, WALL), P(-0.15, -0.15, WALL)], cap_col)
    poly(img, [P(0, 0, WALL), P(ROOM_W, 0, WALL), P(ROOM_W, -0.15, WALL), P(-0.15, -0.15, WALL)], shade(cap_col, 0.92))
    line(img, P(0, 0, 0), P(0, 0, WALL), shade(wall_col, 0.7))
    # sol
    for i in range(ROOM_W):
        for j in range(ROOM_D):
            tile = new(T.TW, T.TH)
            for y, xs, xe in T.diamond_rows():
                for x in range(xs, xe + 1):
                    dx = x + 0.5 - HALF_W
                    dy = y + 0.5
                    u = (dx / HALF_W + dy / HALF_H) / 2
                    v = (dy / HALF_H - dx / HALF_W) / 2
                    px(tile, x, y, floor_fn(i + u, j + v, x, y))
            x, y = P(i, j)
            img.alpha_composite(tile, (int(x - HALF_W), int(y)))
    # bords du sol
    line(img, P(0, ROOM_D), P(ROOM_W, ROOM_D), shade(BG, 1.6))
    line(img, P(ROOM_W, 0), P(ROOM_W, ROOM_D), shade(BG, 1.6))
    return img


def put(img, sprite, anchor, at):
    """Pose un sprite dont le point `anchor` (pixels du sprite) tombe sur `at` (pixels de la scène)."""
    img.alpha_composite(sprite, (int(round(at[0] - anchor[0])), int(round(at[1] - anchor[1]))))


def person(role, look, pose="idle0", flip=False, deco=None):
    s = C.draw_frame(role, look, pose)
    if deco:
        deco(s)
    if flip:
        s = s.transpose(Image.FLIP_LEFT_RIGHT)
    return s


def stand(img, gx, gy, sprite):
    put(img, sprite, (C.FW // 2, C.FH - 1), P(gx, gy))


def box(img, x0, y0, w, h, z0, height, color, **kw):
    iso_box(img, ORIGIN, x0, y0, w, h, z0, height, color, **kw)


# ----------------------------------------------------------------------------
# L'Élysée : salon doré, tapis rouge, pupitre, drapeaux, garde républicaine
# ----------------------------------------------------------------------------
GOLD = (214, 172, 72, 255)
GOLD_D = (164, 122, 44, 255)
CREAM = (240, 232, 212, 255)
VELVET = (168, 32, 44, 255)
FR_BLUE = (36, 64, 156, 255)
FR_RED = (214, 48, 52, 255)


def sash(s):
    """Écharpe tricolore en travers du torse (personnage de 16x26)."""
    for i in range(7):
        x, y = 5 + i, 11 + i
        px(s, x, y, FR_BLUE); px(s, x + 1, y, OFFWHITE); px(s, x + 2, y, FR_RED)


def guard_helmet(s):
    """Casque doré à crinière rouge de la garde républicaine."""
    rect(s, 5, 2, 6, 3, GOLD)
    rect(s, 5, 5, 6, 1, GOLD_D)
    for i in range(5):
        px(s, 6 + i, 1, (40, 30, 26, 255))
    px(s, 4, 3, (40, 30, 26, 255)); px(s, 4, 4, (40, 30, 26, 255)); px(s, 3, 5, (40, 30, 26, 255))
    px(s, 8, 0, FR_RED); px(s, 9, 0, FR_RED)
    rect(s, 4, 19, 8, 1, OFFWHITE)   # ceinturon blanc


def camera(s):
    rect(s, 10, 8, 5, 4, INK); px(s, 12, 9, METAL_L); rect(s, 11, 7, 2, 1, INK)


def elysee_floor(u, v, x, y):
    # parquet « Versailles » : grands carrés à croisillons
    fu, fv = u % 1, v % 1
    base = mix(WOOD_M, WOOD_L, 0.55)
    if abs(fu - fv) < 0.05 or abs(fu + fv - 1) < 0.05:
        return shade(base, 0.78)
    if fu < 0.05 or fv < 0.05:
        return WOOD_D
    return shade(base, 0.95 + 0.06 * math.sin(u * 30 + v * 7))


def elysee():
    img = base_room(elysee_floor, CREAM, (250, 244, 228, 255))
    # boiseries : soubassement, cimaise, corniche dorée, pilastres
    for side, n in (("L", ROOM_D), ("R", ROOM_W)):
        wall_quad(img, side, 0, n, 0, 5, OFFWHITE, None)
        wall_line(img, side, 0, n, 5, GOLD_D)
        wall_line(img, side, 0, n, 22, GOLD)
        wall_quad(img, side, 0, n, 54, 59, GOLD, None)
        wall_line(img, side, 0, n, 53, GOLD_D)
    # mur droit : grande tapisserie dans un cadre doré, entre deux pilastres
    for t in (0.6, 6.4):
        wall_quad(img, "R", t, t + 0.18, 5, 53, (250, 244, 228, 255), GOLD)
    wall_quad(img, "R", 1.6, 5.4, 22, 50, GOLD, GOLD_D)
    wall_quad(img, "R", 1.75, 5.25, 24, 48, (122, 160, 196, 255))       # ciel
    wall_quad(img, "R", 1.75, 5.25, 24, 33, (92, 140, 84, 255))         # prairie
    wall_quad(img, "R", 2.4, 3.3, 30, 37, (74, 112, 70, 255))           # bosquet
    wall_quad(img, "R", 3.8, 4.6, 33, 41, (230, 226, 214, 255))         # château au loin
    wall_quad(img, "R", 3.95, 4.45, 41, 44, (60, 80, 120, 255))
    for k in range(6):                                                     # panneaux bas
        wall_quad(img, "R", 0.9 + k, 1.6 + k, 8, 19, CREAM, GOLD_D)
    # mur gauche : trois hautes fenêtres à rideaux rouges
    for g in (0.55, 2.45, 4.35):
        wall_quad(img, "L", g, g + 1.1, 9, 50, (186, 214, 236, 255), OFFWHITE)
        wall_line(img, "L", g, g + 1.1, 30, OFFWHITE)
        a, b = P(0, g + 0.55, 9), P(0, g + 0.55, 50)
        line(img, a, b, OFFWHITE)
        wall_quad(img, "L", g - 0.22, g + 0.08, 6, 52, VELVET, shade(VELVET, 0.7))
        wall_quad(img, "L", g + 1.02, g + 1.32, 6, 52, VELVET, shade(VELVET, 0.7))
        wall_quad(img, "L", g - 0.25, g + 1.35, 50, 54, GOLD, GOLD_D)
    wall_text(img, "R", 2.2, 57, "REPUBLIQUE FRANCAISE", (120, 84, 30, 255))

    # tapis rouge bordé d'or, de l'entrée au pupitre
    poly(img, [P(1.2, 2.55), P(ROOM_W, 2.55), P(ROOM_W, 3.45), P(1.2, 3.45)], GOLD)
    poly(img, [P(1.25, 2.65), P(ROOM_W, 2.65), P(ROOM_W, 3.35), P(1.25, 3.35)], VELVET)

    items = []   # (profondeur, fonction de dessin)

    def at(depth, fn):
        items.append((depth, fn))

    # pupitre présidentiel
    def lectern():
        box(img, 0.75, 2.75, 0.42, 0.5, 0, 15, FR_BLUE, top=(60, 90, 180, 255))
        c = P(1.17, 3.0, 9)
        for yy in range(-2, 3):
            for xx in range(-2, 3):
                if xx * xx + yy * yy <= 5:
                    px(img, c[0] + xx, c[1] + yy, GOLD)
        px(img, c[0], c[1], FR_BLUE)
        m = P(0.95, 3.0, 20)
        line(img, P(0.95, 3.0, 15), m, METAL_D)
        px(img, m[0], m[1], INK)
    at(3.4, lectern)

    # drapeaux français et européen
    def flags():
        for (gx, gy, eu) in ((0.35, 1.5, False), (0.35, 4.3, True)):
            box(img, gx - 0.12, gy - 0.12, 0.24, 0.24, 0, 3, GOLD_D)
            top = P(gx, gy, 58)
            line(img, P(gx, gy, 3), top, GOLD)
            px(img, top[0], top[1] - 1, GOLD)
            fx, fy = top[0] + 1, top[1] + 2
            if eu:
                rect(img, fx, fy, 10, 14, (30, 60, 160, 255))
                for k in range(8):
                    a = k / 8 * math.tau
                    px(img, fx + 5 + round(math.cos(a) * 3), fy + 6 + round(math.sin(a) * 3), (250, 214, 92, 255))
            else:
                for k, col in enumerate((FR_BLUE, OFFWHITE, FR_RED)):
                    for yy in range(16):
                        sway = 1 if yy > 10 else 0
                        rect(img, fx + k * 3 + sway, fy + yy, 3, 1, col)
    at(1.6, flags)

    # le Président, écharpe tricolore, à côté du pupitre
    at(4.9, lambda: stand(img, 1.9, 3.0, person("Sales", 2, "talk0", deco=sash)))
    # deux gardes républicains de part et d'autre du tapis
    at(8.4, lambda: stand(img, 6.4, 2.0, person("Sales", 4, "idle0", flip=True, deco=guard_helmet)))
    at(11.9, lambda: stand(img, 7.0, 4.9, person("Sales", 0, "idle0", flip=True, deco=guard_helmet)))
    # photographes derrière un cordon de velours
    def rope(gy, g0, g1):
        def draw():
            n = int((g1 - g0) / 0.9) + 1
            for k in range(n):
                g = g0 + k * (g1 - g0) / (n - 1)
                p0, p1 = P(g, gy), P(g, gy, 9)
                line(img, p0, p1, GOLD)
                px(img, p1[0], p1[1] - 1, GOLD)
                px(img, p0[0] - 1, p0[1], GOLD_D); px(img, p0[0] + 1, p0[1], GOLD_D)
                if k:
                    a, b = P(g - (g1 - g0) / (n - 1), gy, 8), P(g, gy, 8)
                    for t in range(11):
                        q = t / 10
                        px(img, round(a[0] + (b[0] - a[0]) * q), round(a[1] + (b[1] - a[1]) * q + 3 * math.sin(q * math.pi)), VELVET)
        return draw
    at(4.3, rope(1.75, 2.6, 5.4))
    for k, (gx, look) in enumerate(((3.0, 3), (4.0, 6), (5.0, 1))):
        at(gx + 1.2, lambda gx=gx, look=look, k=k: stand(img, gx, 1.2, person(["Growth", "Intern", "ProductManager"][k], look, "idle1", deco=camera)))
    # invités en face, derrière un second cordon
    at(8.6, rope(4.25, 2.4, 5.6))
    for k, (gx, gy, role, look) in enumerate(((2.6, 5.0, "Hr", 5), (3.5, 5.3, "Cfo", 2), (4.4, 4.9, "AiResearcher", 1),
                                              (5.3, 5.3, "Consultant", 6), (2.2, 5.6, "Frontend", 3))):
        at(gx + gy, lambda gx=gx, gy=gy, role=role, look=look: stand(img, gx, gy, person(role, look, "idle0", flip=True)))
    # plantes en pot dans les coins
    pl, pa = F.plant()
    at(0.6, lambda: put(img, pl, pa, P(0.15, 0.15)))
    at(6.9, lambda: put(img, pl, pa, P(0.2, 5.5)))

    for _, fn in sorted(items, key=lambda it: it[0]):
        fn()

    # lustre de cristal suspendu au centre
    c = P(3.6, 3.0, 82)
    line(img, (c[0], 0), (c[0], c[1] - 6), GOLD_D)
    for r, col in ((9, GOLD_D), (7, GOLD)):
        for a in range(0, 360, 4):
            x = c[0] + round(math.cos(math.radians(a)) * r)
            y = c[1] + round(math.sin(math.radians(a)) * r * 0.35)
            px(img, x, y, col)
    for k in range(7):
        x = c[0] - 9 + k * 3
        px(img, x, c[1] - 2, (255, 236, 160, 255)); px(img, x, c[1] - 3, (255, 250, 220, 255))
        px(img, x, c[1] + 3, (200, 230, 255, 255))
    for k in range(3):
        px(img, c[0] - 2 + k * 2, c[1] + 6, (200, 230, 255, 255))
    return img


# ----------------------------------------------------------------------------
# France Travail : lino, chaises orange, guichets, borne à tickets
# ----------------------------------------------------------------------------
FT_BLUE = (30, 64, 150, 255)
FT_WALL = (222, 228, 234, 255)
CHAIR = (236, 120, 40, 255)


def lino(u, v, x, y):
    a = int(u) + int(v)
    c = (176, 184, 192, 255) if a % 2 == 0 else (162, 170, 180, 255)
    if hash2(x * 3 + int(u * 7), y * 5 + int(v * 3)) > 0.97:
        c = shade(c, 0.9)
    fu, fv = u % 1, v % 1
    if fu < 0.03 or fv < 0.03:
        c = shade(c, 0.86)
    return c


def hash2(a, b):
    return T.hash2(a, b)


def france_travail():
    img = base_room(lino, FT_WALL, (240, 244, 248, 255))
    for side, n in (("L", ROOM_D), ("R", ROOM_W)):
        wall_quad(img, side, 0, n, 0, 16, (196, 204, 212, 255), None)     # soubassement lessivable
        wall_quad(img, side, 0, n, 16, 19, FT_BLUE, None)                 # bandeau bleu
        wall_line(img, side, 0, n, 1, (150, 158, 166, 255))
    # mur gauche : fenêtre à stores, deux affiches
    wall_quad(img, "L", 0.5, 2.1, 24, 50, (186, 210, 230, 255), OFFWHITE)
    for z in range(26, 50, 3):
        wall_line(img, "L", 0.55, 2.05, z, (226, 230, 232, 255))
    wall_quad(img, "L", 2.45, 3.55, 26, 46, OFFWHITE, (120, 128, 136, 255))
    wall_quad(img, "L", 2.45, 3.55, 40, 46, (246, 162, 60, 255))
    wall_text(img, "L", 3.3, 35, "CV", FT_BLUE)
    wall_quad(img, "L", 3.7, 6.0, 22, 51, (250, 214, 92, 255), (120, 128, 136, 255))
    for k, (word, col) in enumerate((("ATELIER", INK), ("CREER", INK), ("STARTUP", INK), ("MARDI", FT_BLUE))):
        wall_text(img, "L", 5.9, 48 - k * 6, word, col)
    # mur droit : enseigne, horloge, écran d'appel des tickets (le numéro est animé par le client)
    wall_quad(img, "R", 0.8, 5.0, 38, 52, FT_BLUE, shade(FT_BLUE, 0.7))
    wall_text(img, "R", 0.75 + 0.3, 48, "FRANCE TRAVAIL", OFFWHITE)
    wall_quad(img, "R", 5.3, 6.8, 30, 48, (20, 24, 22, 255), METAL_D)
    wall_text(img, "R", 5.4, 45, "TICKET", (250, 140, 60, 255))
    c = P(0.45, 0, 44)
    for a in range(0, 360, 20):
        px(img, c[0] + round(math.cos(math.radians(a)) * 4), c[1] + round(math.sin(math.radians(a)) * 4), INK)
    line(img, c, (c[0], c[1] - 3), INK); line(img, c, (c[0] + 2, c[1]), INK)

    items = []

    def at(depth, fn):
        items.append((depth, fn))

    # deux guichets : conseillère (et son écran), conseiller
    def counter(x0, staff_role, staff_look):
        def draw_staff():
            stand(img, x0 + 1.0, 0.45, person(staff_role, staff_look, "idle0"))
        def draw_desk():
            box(img, x0, 0.9, 2.0, 0.7, 0, 15, (206, 196, 176, 255), top=(232, 224, 206, 255))
            box(img, x0 + 0.2, 1.05, 0.45, 0.12, 15, 11, GRAPHITE)
            s = P(x0 + 0.42, 1.17, 21)
            px(img, s[0], s[1], (110, 170, 230, 255)); px(img, s[0] + 1, s[1] - 1, (110, 170, 230, 255))
            p = P(x0 + 1.4, 1.2, 15)
            rect(img, p[0] - 2, p[1] - 1, 5, 2, OFFWHITE)
            # chaise visiteur, côté public
            box(img, x0 + 0.85, 1.95, 0.4, 0.4, 0, 7, CHAIR)
            box(img, x0 + 0.85, 2.3, 0.4, 0.06, 7, 9, CHAIR)
        at(x0 + 1.45, draw_staff)
        at(x0 + 2.5, draw_desk)
    counter(1.2, "Hr", 5)
    counter(4.3, "Cfo", 4)
    # rangée de chaises orange le long du mur gauche, avec des demandeurs d'emploi
    seated = {2.3: ("Growth", 2), 3.1: None, 3.9: ("Intern", 7), 4.7: ("Backend", 6), 5.5: None}
    for gy, who in seated.items():
        def chair(gy=gy, who=who):
            box(img, 0.35, gy - 0.2, 0.4, 0.4, 0, 7, CHAIR)
            box(img, 0.2, gy - 0.2, 0.1, 0.4, 7, 9, CHAIR)
            if who:
                s = person(who[0], who[1], "type0")
                put(img, s, (C.FW // 2, C.FH - 1), P(0.55, gy, 4))
        at(0.55 + gy, chair)
    # borne à tickets près de l'entrée
    def kiosk():
        box(img, 6.1, 4.5, 0.5, 0.45, 0, 20, OFFWHITE, top=(250, 250, 250, 255))
        box(img, 6.1, 4.5, 0.5, 0.45, 20, 3, FT_BLUE)
        sc = P(6.35, 4.95, 15)
        rect(img, sc[0] - 2, sc[1] - 3, 5, 4, (30, 40, 60, 255))
        px(img, sc[0], sc[1] - 2, (120, 200, 255, 255))
        t = P(6.35, 4.95, 8)
        rect(img, t[0] - 1, t[1], 3, 3, OFFWHITE)
    at(11.2, kiosk)
    # fontaine à eau et plante
    def cooler():
        box(img, 6.5, 0.15, 0.35, 0.35, 0, 18, OFFWHITE)
        c2 = P(6.67, 0.32, 18)
        rect(img, c2[0] - 3, c2[1] - 9, 7, 9, (150, 200, 240, 255))
    at(7.0, cooler)
    pl, pa = F.plant()
    at(0.5, lambda: put(img, pl, pa, P(0.2, 0.25)))

    for _, fn in sorted(items, key=lambda it: it[0]):
        fn()
    return img


def positions():
    """Positions (pixels de la scène) utilisées par le client, à recopier dans finale.rs."""
    out = {
        "elysee_start": P(7.4, 3.0), "elysee_end": P(2.75, 3.0), "elysee_president_hand": P(1.9, 3.0, 13),
        "elysee_cameras": [P(3.0, 1.2, 17), P(4.0, 1.2, 17), P(5.0, 1.2, 17)],
        "ft_start": P(7.6, 4.2), "ft_mid": P(5.0, 2.95), "ft_end": P(2.25, 2.35),
        "ft_counselor_head": P(2.2, 0.45, 24), "ft_screen": P(6.05, 0, 36),
    }
    return out


if __name__ == "__main__":
    import os
    os.makedirs("/tmp/claude-0/art", exist_ok=True)
    a, b = elysee(), france_travail()
    sheet = Image.new("RGBA", (W_IMG * 2 + 8, H_IMG), BG)
    sheet.alpha_composite(a, (0, 0)); sheet.alpha_composite(b, (W_IMG + 8, 0))
    sheet.resize((sheet.width * 3, sheet.height * 3), Image.NEAREST).save("/tmp/claude-0/art/rooms.png")
    print(positions())
