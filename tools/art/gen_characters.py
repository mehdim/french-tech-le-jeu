"""Sprites animés des salariés : 12 rôles x 8 apparences x 16 images (16x26 chacune).

Disposition des images par ligne :
  0-1 attente   2-5 marche   6-7 frappe (assis)   8-9 café
  10-11 discussion   12-13 épuisement   14-15 panique
"""
from pixlib import *

FW, FH = 16, 26
FRAMES = 16
ROLES = ["Founder", "Backend", "Frontend", "AiResearcher", "DevOps", "Sales", "Growth",
         "ProductManager", "Hr", "Cfo", "Intern", "Consultant", "HeroM", "HeroF"]

# Le héros ou l'héroïne choisi·e à la création : 7 types, chacun avec son teint et ses cheveux.
# (nom, teint, cheveux, coiffure homme, coiffure femme, barbe pour l'homme)
# coiffures : 0 courts, 1 longs, 2 chignon, 3 bouclés
HEROES = [
    ("blanc", (244, 208, 178, 255), (112, 74, 42, 255), 0, 1, False),
    ("arabe", (212, 164, 120, 255), (34, 26, 22, 255), 0, 1, True),
    ("indien", (170, 114, 76, 255), (22, 20, 24, 255), 0, 2, False),
    ("metis", (192, 140, 100, 255), (58, 38, 26, 255), 3, 3, False),
    ("africain", (104, 68, 46, 255), (20, 18, 18, 255), 3, 3, False),
    ("asiatique", (238, 204, 162, 255), (24, 22, 26, 255), 0, 1, False),
    ("amerindien", (178, 120, 82, 255), (16, 16, 20, 255), 1, 1, False),
]
LOOKS = 8

JEANS = (58, 74, 112, 255)
DARK_PANTS = (44, 48, 58, 255)
KHAKI = (176, 150, 104, 255)
SHOE = (34, 30, 30, 255)

OUTFIT = {
    # rôle : (chemise, ombre de la chemise, pantalon, accessoires)
    "Founder": (GRAPHITE_2, GRAPHITE, JEANS, {"logo": GREEN, "hood": True}),
    "Backend": (DEEP_GREEN, shade(DEEP_GREEN, 0.8), JEANS, {"headphones": True}),
    "Frontend": ((236, 120, 150, 255), (200, 92, 122, 255), KHAKI, {"beanie": (232, 196, 70, 255)}),
    "AiResearcher": (OFFWHITE, LIGHTGRAY, DARK_PANTS, {"glasses": True, "lanyard": True}),
    "DevOps": (INK, GRAPHITE, JEANS, {"crab": ORANGE, "beanie": GRAPHITE_2}),
    "Sales": (NAVY, shade(NAVY, 0.8), DARK_PANTS, {"tie": RED, "collar": OFFWHITE}),
    "Growth": (ORANGE, shade(ORANGE, 0.82), JEANS, {"cap": GREEN}),
    "ProductManager": ((150, 186, 226, 255), (118, 152, 196, 255), KHAKI, {"notebook": True}),
    "Hr": (PURPLE, shade(PURPLE, 0.8), DARK_PANTS, {"scarf": YELLOW}),
    "Cfo": ((120, 124, 132, 255), (96, 100, 108, 255), (96, 100, 108, 255), {"tie": INK, "collar": OFFWHITE, "glasses": True}),
    "Intern": (OFFWHITE, OFFWHITE, JEANS, {"stripes": NAVY}),
    "Consultant": ((40, 44, 56, 255), (30, 32, 42, 255), (40, 44, 56, 255), {"tie": BLUE, "collar": OFFWHITE, "strap": WOOD_D}),
}
OUTFIT["HeroM"] = OUTFIT["Founder"]
OUTFIT["HeroF"] = OUTFIT["Founder"]


def look_colors(look):
    skin = SKINS[look % len(SKINS)]
    hair = HAIRS[(look * 3 + 1) % len(HAIRS)]
    style = look % 4  # 0 courts, 1 longs, 2 chignon, 3 bouclés
    return skin, hair, style


def draw_head(img, x0, y0, skin, hair, style, extras, eyes="open", mouth=None):
    # visage de 7x8 en (x0..x0+6, y0..y0+7)
    for y in range(y0 + 1, y0 + 8):
        for x in range(x0, x0 + 7):
            px(img, x, y, skin)
    # ombre du côté éloigné
    for y in range(y0 + 2, y0 + 8):
        px(img, x0, y, shade(skin, 0.86))
    # cheveux
    for x in range(x0 - (1 if style in (1, 3) else 0), x0 + 7 + (1 if style == 3 else 0)):
        px(img, x, y0, hair)
        px(img, x, y0 + 1, hair)
    px(img, x0, y0 + 2, hair)
    px(img, x0 + 1, y0 + 2, hair)
    px(img, x0 - (0 if style != 3 else 1), y0 + 3, hair)
    if style == 1:      # cheveux longs sur le côté
        for y in range(y0 + 2, y0 + 10):
            px(img, x0 - 1, y, hair)
            px(img, x0, y, hair)
    elif style == 2:    # chignon
        rect(img, x0 + 1, y0 - 2, 3, 2, hair)
    elif style == 3:    # volume des boucles
        for x in range(x0 - 1, x0 + 8):
            px(img, x, y0 - 1, hair)
        px(img, x0 + 7, y0 + 2, hair)
        px(img, x0 - 1, y0 + 4, hair)
    # oreille
    px(img, x0 + 1, y0 + 4, shade(skin, 0.8))
    # yeux (tourné vers la droite : traits décalés à droite)
    ey = y0 + 4
    if eyes == "open":
        px(img, x0 + 3, ey, INK)
        px(img, x0 + 5, ey, INK)
    elif eyes == "closed":
        px(img, x0 + 3, ey + 1, shade(skin, 0.6))
        px(img, x0 + 5, ey + 1, shade(skin, 0.6))
    elif eyes == "wide":
        px(img, x0 + 3, ey, INK); px(img, x0 + 3, ey - 1, OFFWHITE)
        px(img, x0 + 5, ey, INK); px(img, x0 + 5, ey - 1, OFFWHITE)
    if extras.get("glasses"):
        frame = (150, 156, 160, 255)
        for x in range(x0 + 2, x0 + 7):
            px(img, x, ey - 1, frame)
        for x in (x0 + 2, x0 + 4, x0 + 6):
            px(img, x, ey, frame)
    # bouche
    if mouth == "open":
        px(img, x0 + 4, y0 + 6, INK)
        px(img, x0 + 4, y0 + 7, (160, 60, 60, 255))
    elif mouth == "smile":
        px(img, x0 + 3, y0 + 6, shade(skin, 0.6)); px(img, x0 + 5, y0 + 6, shade(skin, 0.6)); px(img, x0 + 4, y0 + 7, shade(skin, 0.6))
    else:
        px(img, x0 + 4, y0 + 6, shade(skin, 0.65))
    # couvre-chef
    if extras.get("headphones"):
        for x in range(x0, x0 + 7):
            px(img, x, y0 - 1, INK)
        rect(img, x0 - 1, y0 + 3, 2, 3, GREEN)
    if extras.get("beanie"):
        c = extras["beanie"]
        for x in range(x0 - 1, x0 + 8):
            px(img, x, y0, c); px(img, x, y0 + 1, shade(c, 0.85))
        rect(img, x0 + 1, y0 - 2, 5, 2, c)
    if extras.get("cap"):
        c = extras["cap"]
        rect(img, x0, y0 - 1, 7, 2, c)
        rect(img, x0 - 3, y0 + 1, 3, 1, shade(c, 0.8))  # casquette à l'envers
    if extras.get("heroine"):
        # cheveux plus longs dans le dos, lèvres et boucle d'oreille
        for y in range(y0 + 2, y0 + 12):
            px(img, x0 - 1, y, hair)
            px(img, x0, y, hair)
        px(img, x0 + 1, y0 + 9, hair)
        px(img, x0 + 4, y0 + 6, (190, 70, 84, 255))
        px(img, x0 + 1, y0 + 5, YELLOW)
    if extras.get("beard"):
        b = extras["beard"]
        for x in range(x0 + 1, x0 + 7):
            px(img, x, y0 + 7, b)
        px(img, x0 + 6, y0 + 6, b); px(img, x0 + 2, y0 + 6, b)
    if extras.get("hood"):
        px(img, x0 - 1, y0 + 6, GRAPHITE)
        px(img, x0 - 1, y0 + 7, GRAPHITE)


def draw_arm(img, shoulder, hand, sleeve, skin):
    (sx, sy), (hx, hy) = shoulder, hand
    n = max(abs(hx - sx), abs(hy - sy), 1)
    for i in range(n + 1):
        t = i / n
        x = round(sx + (hx - sx) * t)
        y = round(sy + (hy - sy) * t)
        c = sleeve if t < 0.75 else skin
        px(img, x, y, c)
        px(img, x + (1 if hx >= sx else -1) * 0, y + 1, c)


POSES = {
    # nom : (body_dy, head_dy, main_arrière, main_avant, jambes, yeux, bouche)
    "idle0": (0, 0, (3, 19), (12, 19), "stand", "open", None),
    "idle1": (0, 1, (3, 19), (12, 19), "stand", "open", None),
    "walk0": (0, 0, (2, 18), (13, 18), "walk_a", "open", None),
    "walk1": (-1, -1, (3, 19), (12, 19), "walk_mid", "open", None),
    "walk2": (0, 0, (4, 18), (11, 18), "walk_b", "open", None),
    "walk3": (-1, -1, (3, 19), (12, 19), "walk_mid", "open", None),
    "type0": (1, 1, (6, 17), (10, 16), "sit", "open", None),
    "type1": (1, 1, (6, 16), (10, 17), "sit", "open", None),
    "coffee0": (0, 0, (3, 19), (12, 16), "stand", "open", None),
    "coffee1": (0, 0, (3, 19), (11, 12), "stand", "closed", None),
    "talk0": (0, 0, (3, 19), (13, 14), "stand", "open", "open"),
    "talk1": (0, 0, (3, 19), (14, 11), "stand", "open", "smile"),
    "tired0": (2, 3, (4, 21), (11, 21), "stand", "closed", None),
    "tired1": (2, 4, (4, 21), (11, 21), "stand", "closed", None),
    "panic0": (-1, -1, (2, 9), (14, 9), "walk_a", "wide", "open"),
    "panic1": (0, 0, (1, 10), (15, 8), "walk_b", "wide", "open"),
}
ORDER = ["idle0", "idle1", "walk0", "walk1", "walk2", "walk3", "type0", "type1", "coffee0", "coffee1",
         "talk0", "talk1", "tired0", "tired1", "panic0", "panic1"]


def draw_legs(img, kind, pants, by):
    hip = 19 + by
    if kind == "sit":
        rect(img, 5, hip, 6, 2, pants)
        rect(img, 5, hip + 2, 2, 2, pants); rect(img, 9, hip + 2, 2, 2, pants)
        return
    offs = {"stand": (0, 0, 0, 0), "walk_a": (-1, 0, 1, -1), "walk_b": (1, -1, -1, 0), "walk_mid": (0, 0, 0, 0)}[kind]
    lx, ly, rx, ry = offs
    for (x, dx, dy) in ((5, lx, ly), (9, rx, ry)):
        for y in range(hip, 24 + dy):
            px(img, x + dx, y, pants)
            px(img, x + dx + 1, y, shade(pants, 0.85))
        px(img, x + dx, 24 + dy, SHOE)
        px(img, x + dx + 1, 24 + dy, SHOE)
        px(img, x + dx + 2, 24 + dy, SHOE)


def draw_frame(role, look, pose):
    img = new(FW, FH)
    shirt, shirt_d, pants, extras = OUTFIT[role]
    skin, hair, style = look_colors(look)
    if role in ("HeroM", "HeroF"):
        _, skin, hair, sm, sf, beard = HEROES[look % len(HEROES)]
        style = sm if role == "HeroM" else sf
        extras = dict(extras, beard=hair if (beard and role == "HeroM") else None, heroine=(role == "HeroF"))
    by, hy, back_hand, front_hand, legs, eyes, mouth = POSES[pose]
    draw_legs(img, legs, pants, by if legs != "sit" else 1)
    # bras arrière (derrière le torse)
    draw_arm(img, (4, 12 + by), (back_hand[0], back_hand[1] + (by if pose.startswith("tired") else 0)), shirt_d, skin)
    # torse
    for y in range(11 + by, 19 + by + (1 if legs == "sit" else 0)):
        for x in range(4, 12):
            c = shirt if x > 5 else shirt_d
            if extras.get("stripes") and (y - by) % 2 == 0:
                c = extras["stripes"]
            px(img, x, y, c)
    if extras.get("collar"):
        px(img, 7, 11 + by, extras["collar"]); px(img, 9, 11 + by, extras["collar"])
        px(img, 8, 11 + by, extras["collar"])
    if extras.get("tie"):
        for y in range(12 + by, 17 + by):
            px(img, 8, y, extras["tie"])
    if extras.get("logo"):
        px(img, 9, 14 + by, extras["logo"]); px(img, 10, 14 + by, extras["logo"]); px(img, 9, 15 + by, extras["logo"])
    if extras.get("crab"):
        px(img, 8, 14 + by, extras["crab"]); px(img, 9, 14 + by, extras["crab"]); px(img, 10, 14 + by, extras["crab"])
        px(img, 7, 13 + by, extras["crab"]); px(img, 11, 13 + by, extras["crab"])
    if extras.get("lanyard"):
        px(img, 7, 12 + by, GREEN); px(img, 8, 13 + by, GREEN); px(img, 8, 14 + by, OFFWHITE)
    if extras.get("scarf"):
        rect(img, 5, 11 + by, 7, 1, extras["scarf"]); px(img, 10, 12 + by, extras["scarf"]); px(img, 10, 13 + by, extras["scarf"])
    if extras.get("strap"):
        for i in range(6):
            px(img, 10 - i, 11 + by + i, extras["strap"])
    # tête
    draw_head(img, 5, 3 + hy, skin, hair, style, extras, eyes, mouth)
    # bras avant
    draw_arm(img, (11, 12 + by), (front_hand[0], front_hand[1] + (by if pose.startswith("tired") else 0)), shirt, skin)
    # accessoires
    if pose.startswith("coffee"):
        hx, hyy = front_hand
        rect(img, hx, hyy - 2, 2, 2, OFFWHITE)
        px(img, hx + 1, hyy - 3 - (1 if pose == "coffee1" else 0), LIGHTGRAY)
    if extras.get("notebook") and pose in ("idle0", "idle1", "walk0", "walk2", "talk0"):
        rect(img, front_hand[0], front_hand[1] - 2, 2, 3, OFFWHITE)
    if pose.startswith("tired"):
        zx, zy = (12, 2) if pose == "tired0" else (13, 0)
        for (dx, dy) in ((0, 0), (1, 0), (1, 1), (0, 2), (1, 2)):
            px(img, zx + dx, zy + dy, OFFWHITE)
    if pose.startswith("panic"):
        px(img, 12, 4 + hy, (120, 190, 255, 255))
        px(img, 12, 5 + hy, (120, 190, 255, 255))
    outline_alpha(img)
    return img


def atlas():
    rows = len(ROLES) * LOOKS
    sheet = new(FW * FRAMES, FH * rows)
    for r, role in enumerate(ROLES):
        for look in range(LOOKS):
            row = r * LOOKS + look
            for f, pose in enumerate(ORDER):
                sheet.alpha_composite(draw_frame(role, look, pose), (f * FW, row * FH))
    return sheet


if __name__ == "__main__":
    s = atlas()
    s.save("/tmp/claude-0/art/chars.png")
    # aperçu : première apparence de chaque rôle, toutes les images, x4
    prev = new(FW * FRAMES, FH * len(ROLES))
    for r in range(len(ROLES)):
        prev.alpha_composite(s.crop((0, r * LOOKS * FH, FW * FRAMES, (r * LOOKS + 1) * FH)), (0, r * FH))
    bg = Image.new("RGBA", prev.size, (60, 70, 64, 255))
    bg.alpha_composite(prev)
    bg.resize((bg.width * 4, bg.height * 4), Image.NEAREST).save("/tmp/claude-0/art/chars_preview.png")
