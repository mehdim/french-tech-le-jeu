"""Cases de sol isométriques, pans de mur, fenêtres et décorations murales."""
import math
from pixlib import *

WALL_H = 60   # hauteur du mur en pixels
CAP = 3       # épaisseur du haut du mur


def hash2(a, b, s=0):
    h = (int(a) * 374761393 + int(b) * 668265263 + int(s) * 2147483647) & 0xFFFFFFFF
    h = (h ^ (h >> 13)) * 1274126177 & 0xFFFFFFFF
    return (h ^ (h >> 16)) / 0xFFFFFFFF


def diamond_rows():
    """Produit (y, x_début, x_fin) pour un losange 32x16 qui se raccorde parfaitement."""
    for y in range(TH):
        k = y if y < TH // 2 else TH - 1 - y
        xs = HALF_W - 2 - 2 * k
        xe = HALF_W + 1 + 2 * k
        yield y, xs, xe


def floor_tile(pattern, variant):
    img = new(TW, TH)
    for y, xs, xe in diamond_rows():
        for x in range(xs, xe + 1):
            dx = x + 0.5 - HALF_W
            dy = y + 0.5
            u = (dx / HALF_W + dy / HALF_H) / 2
            v = (dy / HALF_H - dx / HALF_W) / 2
            c = pattern(u, v, variant, x, y)
            # léger bord de grille
            edge = min(u, v, 1 - u, 1 - v)
            if edge < 0.035:
                c = shade(c, 0.9)
            px(img, x, y, c)
    return img


def pat_parquet(u, v, var, x, y):
    k = int(v * 4)
    f = (v * 4) % 1
    seg = int(u * 2 + (k % 2) * 0.5)
    base = [WOOD_L, mix(WOOD_L, WOOD_M, 0.5), WOOD_M][int(hash2(k + var * 7, seg) * 3)]
    if f < 0.14:
        return WOOD_D
    if ((u * 2 + (k % 2) * 0.5) % 1) < 0.05:
        return shade(base, 0.82)
    grain = 0.96 + 0.08 * math.sin(u * 40 + k * 3)
    return shade(base, grain)


def pat_concrete(u, v, var, x, y):
    n = hash2(x + var * 31, y)
    c = mix((150, 154, 150, 255), (128, 132, 128, 255), n * 0.6)
    if n > 0.94:
        c = shade(c, 0.85)
    return c


def pat_tiles(u, v, var, x, y):
    a = int(u * 2) + int(v * 2)
    c = OFFWHITE if a % 2 == 0 else (214, 220, 210, 255)
    fu, fv = (u * 2) % 1, (v * 2) % 1
    if fu < 0.06 or fv < 0.06:
        c = LIGHTGRAY
    return c


def pat_wood(u, v, var, x, y):
    k = int(u * 3)
    f = (u * 3) % 1
    base = [(150, 98, 58, 255), (138, 88, 52, 255), (160, 108, 66, 255)][int(hash2(k, var) * 3)]
    if f < 0.08:
        return WOOD_DD
    # soupçon de point de Hongrie : joints en diagonale
    if ((v * 3 + f * 0.6) % 1) < 0.05:
        return shade(base, 0.8)
    return shade(base, 0.96 + 0.06 * math.sin(v * 30 + k))


def pat_marble(u, v, var, x, y):
    t = math.sin((u * 3 + v * 5 + var) * 3.2 + math.sin(v * 9) * 1.4)
    c = mix((236, 236, 230, 255), (204, 208, 204, 255), (t + 1) / 2 * 0.5)
    if abs(t) < 0.04:
        c = (176, 180, 178, 255)
    return c


def pat_carpet(u, v, var, x, y):
    n = hash2(x + var * 13, y + 3)
    return mix((52, 70, 60, 255), (60, 82, 70, 255), n)


FLOORS = {
    "Parquet": pat_parquet,
    "Concrete": pat_concrete,
    "Tiles": pat_tiles,
    "Wood": pat_wood,
    "Marble": pat_marble,
    "Carpet": pat_carpet,
}


# --- murs --------------------------------------------------------------------
WALL_STYLES = {
    # niveau : (couleur de base, texture)
    0: ((236, 226, 206, 255), "stripes"),     # papier peint de l'appartement
    1: (BRICK_L, "brick"),                     # briques du coworking
    2: ((232, 236, 228, 255), "paint_green"),  # incubateur
    3: ((238, 230, 210, 255), "haussmann"),    # bureau parisien
    4: ((150, 156, 152, 255), "concrete"),     # hall de la Station
    5: ((54, 64, 58, 255), "hq"),              # siège en graphite
}


def wall_segment(tier, side, kind="plain"):
    """Un pan de mur le long d'un bord de case. side = 'L' (mur gauche, x=0) ou 'R'.
    Image : 17 x (WALL_H + 9 + CAP) ; ancre = point au sol de l'extrémité côté coin du fond."""
    base, tex = WALL_STYLES[tier]
    W, H = 17, WALL_H + 9 + CAP
    img = new(W, H)
    bottom0 = H - 9  # y du sol à l'extrémité de l'ancre (x=16)
    for x in range(W):
        c = 16 - x
        bot = bottom0 + c / 2
        top = bot - WALL_H
        for y in range(int(top), int(bot) + 1):
            ly = y - top       # 0 en haut
            col = wall_texel(tier, base, tex, c, ly, x, y)
            px(img, x, y, col)
        # chaperon (épaisseur du mur)
        for y in range(int(top) - CAP, int(top)):
            px(img, x, y, shade(base, 0.62) if tex != "hq" else GRAPHITE)
        px(img, x, int(top) - CAP, INK)
    if kind == "window":
        draw_window(img, tier, bottom0)
    elif kind == "door":
        draw_door(img, tier, bottom0)
    # contour de la base du pan
    for x in range(W):
        c = 16 - x
        px(img, x, int(bottom0 + c / 2), shade(base, 0.5))
    if side == "R":
        img = img.transpose(Image.FLIP_LEFT_RIGHT)
        img = darken(img, 0.86)
    return img


def darken(img, f):
    out = img.copy()
    p = out.load()
    for y in range(out.height):
        for x in range(out.width):
            r, g, b, a = p[x, y]
            if a:
                p[x, y] = shade((r, g, b, a), f)
    return out


def wall_texel(tier, base, tex, c, ly, x, y):
    # plinthe
    if ly > WALL_H - 3:
        return shade(base, 0.7) if tex != "hq" else DEEP_GREEN
    if tex == "stripes":
        return shade(base, 0.95) if (c // 3) % 2 == 0 else base
    if tex == "brick":
        row = int(ly // 4)
        off = 4 if row % 2 else 0
        if ly % 4 == 0 or (c + off) % 8 == 0:
            return (196, 180, 160, 255)
        v = hash2(row, (c + off) // 8, 5)
        return mix(BRICK_L, BRICK_D, v * 0.7)
    if tex == "paint_green":
        if 22 <= ly <= 25:
            return DEEP_GREEN
        return base
    if tex == "haussmann":
        # panneaux moulurés
        if ly in (8, 9) or ly in (WALL_H - 14, WALL_H - 13):
            return shade(base, 0.86)
        if (c % 16 in (2, 14)) and 12 < ly < WALL_H - 16:
            return shade(base, 0.88)
        return base
    if tex == "concrete":
        if ly % 20 == 0:
            return shade(base, 0.8)
        return mix(base, shade(base, 0.9), hash2(x, y, 9))
    if tex == "hq":
        if 30 <= ly <= 31:
            return GREEN_DIM
        return mix(base, shade(base, 1.1), hash2(c, ly // 6, 3) * 0.4)
    return base


def window_view(w, h, tier):
    """Les toits de Paris vus par une fenêtre."""
    v = new(w, h)
    for y in range(h):
        sky = mix((150, 186, 220, 255), (232, 214, 190, 255), y / max(1, h - 1))
        for x in range(w):
            px(v, x, y, sky)
    # toits en zinc + cheminées
    for i in range(0, w, 7):
        rh = h - 6 - int(hash2(i, tier) * 5)
        for x in range(i, min(w, i + 7)):
            for y in range(rh, h):
                px(v, x, y, ZINC if (x + y) % 5 else ZINC_D)
        cx = i + 2
        for y in range(rh - 3, rh):
            px(v, cx, y, TERRACOTTA)
            px(v, cx + 1, y, TERRACOTTA)
    if tier >= 3:
        # silhouette d'une tour au loin
        tx = w - 6
        for y in range(2, h - 6):
            spread = (y - 2) // 4
            for x in range(tx - spread, tx + spread + 1):
                if (x + y) % 2 == 0 or spread < 1:
                    px(v, x, y, (90, 96, 110, 255))
    return v


def shear_left(flat, W):
    """Déforme une image plate pour qu'elle épouse un mur gauche (y augmente vers la gauche)."""
    fw, fh = flat.size
    out = new(fw, fh + fw // 2 + 1)
    for x in range(fw):
        dy = (fw - 1 - x) // 2
        for y in range(fh):
            c = flat.getpixel((x, y))
            if c[3]:
                px(out, x, y + dy, c)
    return out


def draw_window(img, tier, bottom0):
    wtall = 30 if tier != 3 else 38
    ww = 10
    view = window_view(ww, wtall, tier)
    frame = new(ww + 4, wtall + 4)
    fc = OFFWHITE if tier in (0, 3) else (METAL_D if tier in (4, 5) else GRAPHITE_2)
    rect(frame, 0, 0, ww + 4, wtall + 4, fc)
    frame.paste(view, (2, 2))
    # meneaux
    for y in range(2, wtall + 2):
        px(frame, (ww + 4) // 2, y, fc)
    for x in range(2, ww + 2):
        px(frame, x, 2 + wtall // 2, fc)
    if tier == 3:
        for x in range(0, ww + 4, 2):
            px(frame, x, wtall + 1, INK)
            px(frame, x, wtall - 2, INK)
    sh = shear_left(frame, 17)
    top = bottom0 + 3 - WALL_H + 10
    img.alpha_composite(sh, (2, top - 2))


def draw_door(img, tier, bottom0):
    door = new(10, 34)
    rect(door, 0, 0, 10, 34, WOOD_D)
    rect(door, 1, 1, 8, 32, WOOD_M)
    px(door, 7, 18, YELLOW)
    sh = shear_left(door, 17)
    img.alpha_composite(sh, (3, bottom0 - 34 + 4))


# --- décorations murales (images plates, déformées au moment du placement) ---
def poster(w, h, bg, draw_fn):
    img = new(w, h)
    rect(img, 0, 0, w, h, INK)
    rect(img, 1, 1, w - 2, h - 2, bg)
    draw_fn(img)
    return img


def decor_images():
    out = {}

    def disrupt(img):
        tiny_text(img, 2, 3, "DIS", GREEN)
        tiny_text(img, 2, 9, "RUPT", GREEN)
        rect(img, 2, 16, 13, 1, GREEN_DIM)
    out["poster_disrupt"] = poster(17, 20, GRAPHITE, disrupt)

    def works(img):
        tiny_text(img, 2, 2, "IT", INK)
        tiny_text(img, 2, 8, "WORKS", INK)
        tiny_text(img, 2, 14, "ON MY", INK)
        tiny_text(img, 2, 20, "MACHINE", RED)
    out["poster_works"] = poster(31, 27, OFFWHITE, works)

    def hype(img):
        rect(img, 1, 1, img.width - 2, img.height - 2, OFFWHITE)
        pts = [(2, 15), (5, 14), (7, 10), (9, 3), (11, 9), (13, 15), (15, 16), (18, 13), (21, 11), (24, 11)]
        for a, b in zip(pts, pts[1:]):
            line(img, a, b, DEEP_GREEN)
        px(img, 9, 3, RED)
        tiny_text(img, 2, 18, "HYPE", MIDGRAY)
    out["hype_cycle"] = poster(27, 24, WOOD_M, hype)

    def flag_aws(img):
        rect(img, 1, 1, 5, 14, BLUE)
        rect(img, 6, 1, 5, 14, OFFWHITE)
        rect(img, 11, 1, 5, 14, RED)
        # facture géante
        rect(img, 18, 0, 14, img.height, OFFWHITE)
        tiny_text(img, 19, 2, "AWS", INK)
        for yy in range(9, img.height - 6, 3):
            rect(img, 19, yy, 10, 1, LIGHTGRAY)
        tiny_text(img, 19, img.height - 6, "999K", RED)
    img = new(33, 34)
    flag_aws(img)
    out["flag_aws"] = img

    def bpi(img):
        rect(img, 1, 1, img.width - 2, img.height - 2, OFFWHITE)
        tiny_text(img, 3, 3, "BOURSE", DEEP_GREEN)
        tiny_text(img, 3, 9, "FRENCH", INK)
        tiny_text(img, 3, 15, "TECH", INK)
        rect(img, 18, 15, 5, 5, YELLOW)
        px(img, 20, 17, RED)
    out["bpi_certificate"] = poster(27, 23, YELLOW, bpi)

    def neon(img):
        rect(img, 0, 0, img.width, img.height, CLEAR)
        tiny_text(img, 1, 1, "VERIDIX", GREEN)
    img = new(30, 7)
    neon(img)
    out["neon_sign"] = img

    def whiteboard(img):
        rect(img, 1, 1, img.width - 2, img.height - 2, OFFWHITE)
        for (x, y, w, h, c) in [(3, 3, 6, 4, BLUE), (13, 3, 6, 4, DEEP_GREEN), (8, 11, 6, 4, RED)]:
            rect(img, x, y, w, 1, c); rect(img, x, y + h, w, 1, c)
            rect(img, x, y, 1, h, c); rect(img, x + w, y, 1, h + 1, c)
        line(img, (9, 5), (13, 5), INK)
        line(img, (11, 7), (11, 11), INK)
        tiny_text(img, 15, 12, "K8S", MIDGRAY)
    out["whiteboard_wall"] = poster(28, 19, METAL_L, whiteboard)

    def eu(img):
        rect(img, 1, 1, img.width - 2, img.height - 2, (38, 64, 150, 255))
        cx, cy = img.width // 2, img.height // 2
        for i in range(12):
            a = i / 12 * 2 * math.pi
            px(img, cx + round(5 * math.cos(a)), cy + round(4 * math.sin(a)), YELLOW)
    out["eu_flag"] = poster(17, 13, (38, 64, 150, 255), eu)

    def shelf(img):
        rect(img, 0, 0, img.width, img.height, WOOD_D)
        for row in range(3):
            y = 2 + row * 7
            rect(img, 1, y + 5, img.width - 2, 1, WOOD_DD)
            x = 2
            while x < img.width - 3:
                bw = 1 + int(hash2(x, row) * 2)
                col = [RED, BLUE, GREEN_DIM, YELLOW, OFFWHITE, PURPLE][int(hash2(x, row, 3) * 6)]
                rect(img, x, y, bw, 5, col)
                x += bw + 1
    img = new(18, 23)
    shelf(img)
    out["bookshelf"] = img
    # --- Clins d'œil au podcast A la French ---------------------------------
    def onair(img):
        rect(img, 1, 1, img.width - 2, img.height - 2, (150, 30, 34, 255))
        tiny_text(img, 4, 3, "ON AIR", OFFWHITE)
        rect(img, 2, 10, img.width - 4, 1, RED)
    out["onair_sign"] = poster(30, 13, INK, onair)

    def alf_cover(img):
        tiny_text(img, 3, 3, "A LA", OFFWHITE)
        tiny_text(img, 3, 9, "FRENCH", GREEN)
        for k in range(3):
            x = 5 + k * 7
            rect(img, x, 17, 3, 4, METAL_L)
            rect(img, x + 1, 21, 1, 2, METAL_M)
            rect(img, x - 1, 23, 5, 1, METAL_D)
    out["poster_alf"] = poster(28, 27, INK, alf_cover)

    def triple(img):
        # « Les systèmes critiques sont triplés » : trois serveurs identiques.
        for k in range(3):
            x = 3 + k * 7
            rect(img, x, 3, 5, 10, GRAPHITE_2)
            for yy in (5, 8, 11):
                px(img, x + 1, yy, GREEN); rect(img, x + 2, yy, 2, 1, METAL_M)
        tiny_text(img, 6, 16, "X3", GREEN)
    out["poster_triple"] = poster(25, 23, INK, triple)

    def engineers(img):
        tiny_text(img, 2, 3, "TRAITEZ", INK)
        tiny_text(img, 2, 9, "AVEC.", DEEP_GREEN)
        rect(img, 2, 15, 27, 1, BLUE); rect(img, 2, 16, 27, 1, OFFWHITE); rect(img, 2, 17, 27, 1, RED)
    out["poster_engineers"] = poster(31, 21, OFFWHITE, engineers)

    def prod(img):
        tiny_text(img, 2, 3, "ILS ONT", INK)
        tiny_text(img, 2, 9, "EFFACE", INK)
        tiny_text(img, 2, 15, "LA PROD", RED)
    out["poster_prod"] = poster(31, 22, OFFWHITE, prod)

    def accent(img):
        tiny_text(img, 1, 3, "LE MONDE", OFFWHITE)
        tiny_text(img, 1, 9, "AVEC", MIDGRAY)
        tiny_text(img, 1, 15, "L'ACCENT", GREEN)
    out["poster_accent"] = poster(33, 22, GRAPHITE, accent)

    def space(img):
        for (x, y) in [(3, 3), (20, 4), (8, 15), (22, 16), (14, 2), (4, 11)]:
            px(img, x, y, OFFWHITE)
        rect(img, 10, 7, 5, 5, METAL_L)
        rect(img, 11, 8, 3, 3, GREEN_DIM)
        rect(img, 3, 9, 6, 1, BLUE); rect(img, 3, 8, 6, 1, NAVY)
        rect(img, 16, 9, 6, 1, BLUE); rect(img, 16, 8, 6, 1, NAVY)
        tiny_text(img, 4, 14, "DC", GREEN)
    out["poster_space"] = poster(25, 20, NAVY, space)

    # Décors liés au mentor principal.
    def api(img):
        tiny_text(img, 3, 3, "API", GREEN)
        tiny_text(img, 3, 9, "DAYS", OFFWHITE)
        tiny_text(img, 3, 15, "PARIS", MIDGRAY)
    out["poster_api"] = poster(24, 22, INK, api)

    def gpu(img):
        rect(img, 4, 3, 13, 9, GREEN_DIM)
        rect(img, 6, 5, 9, 5, INK)
        for k in range(5):
            px(img, 5 + 2 * k, 2, METAL_L); px(img, 5 + 2 * k, 12, METAL_L)
        tiny_text(img, 3, 15, "-50%", GREEN)
    out["poster_gpu"] = poster(21, 22, GRAPHITE, gpu)

    def cone_shelf(img):
        rect(img, 0, img.height - 3, img.width, 2, WOOD_D)
        rect(img, 1, img.height - 1, 2, 1, WOOD_DD); rect(img, img.width - 3, img.height - 1, 2, 1, WOOD_DD)
        base = img.height - 4
        for y in range(10):
            w = 1 + y // 2
            rect(img, 8 - w, base - 10 + y, 2 * w + 1, 1, ORANGE if y % 4 not in (2,) else OFFWHITE)
        rect(img, 2, base, 13, 1, shade(ORANGE, 0.75))
    img = new(17, 18)
    cone_shelf(img)
    out["cone_shelf"] = img
    return out
