"""Portraits de dialogue en 40x40 pour les personnages récurrents."""
import os

from pixlib import *

P = 40


def bust(bg, skin, hair, style, shirt, extras=None):
    e = extras or {}
    img = new(P, P)
    rect(img, 0, 0, P, P, bg)
    for i in range(P):       # légères rayures de vignettage
        px(img, i, 0, shade(bg, 0.8)); px(img, i, P - 1, shade(bg, 0.8))
    # épaules
    for y in range(29, P):
        w = 13 + (y - 29)
        for x in range(20 - w, 20 + w):
            px(img, x, y, shirt if x > 14 else shade(shirt, 0.85))
    if e.get("collar"):
        for (x, y) in [(17, 29), (18, 30), (22, 29), (21, 30)]:
            px(img, x, y, e["collar"])
    if e.get("tie"):
        rect(img, 19, 30, 2, 10, e["tie"])
    if e.get("vest"):
        for y in range(31, P):
            for x in range(8, 14):
                px(img, x + (y - 31) // 3, y, e["vest"])
            for x in range(27, 33):
                px(img, x - (y - 31) // 3, y, e["vest"])
    if e.get("scarf"):
        rect(img, 12, 28, 16, 3, e["scarf"])
        rect(img, 22, 31, 3, 6, e["scarf"])
    if e.get("sash"):
        for i in range(14):
            px(img, 10 + i, 30 + i // 2, BLUE); px(img, 10 + i, 31 + i // 2, OFFWHITE); px(img, 10 + i, 32 + i // 2, RED)
    if e.get("chain"):
        for i in range(8):
            px(img, 16 + i, 30 + (1 if i in (0, 7) else 2), YELLOW)
    if e.get("logo"):
        rect(img, 24, 34, 3, 3, e["logo"])
    # cou + tête
    rect(img, 17, 25, 6, 5, shade(skin, 0.88))
    for y in range(8, 27):
        for x in range(12, 28):
            if (x - 20) ** 2 / 64 + (y - 17) ** 2 / 100 <= 1:
                px(img, x, y, skin if x > 14 else shade(skin, 0.9))
    # cheveux
    if style != "bald":
        for y in range(6, 14):
            for x in range(11, 29):
                if (x - 20) ** 2 / 85 + (y - 12) ** 2 / 40 <= 1 and y < 12 + (1 if x < 16 else 0):
                    px(img, x, y, hair)
        if style == "long":
            for y in range(10, 30):
                for x in (10, 11, 12, 28, 29):
                    px(img, x, y, hair)
        if style == "bob":
            for y in range(10, 24):
                for x in (11, 12, 27, 28):
                    px(img, x, y, hair)
        if style == "slick":
            for x in range(13, 27):
                px(img, x, 9, shade(hair, 1.3))
        if style == "curly":
            for (x, y) in [(10, 9), (29, 10), (12, 6), (27, 7), (20, 5), (16, 5), (24, 5)]:
                rect(img, x - 1, y - 1, 3, 3, hair)
    else:
        for x in (12, 13, 27, 28):
            for y in range(13, 18):
                px(img, x, y, hair)
    # visage
    px(img, 16, 17, INK); px(img, 23, 17, INK)
    px(img, 16, 16, OFFWHITE) if e.get("wide") else None
    rect(img, 15, 15, 3, 1, shade(hair if style != "bald" else skin, 0.8))
    rect(img, 22, 15, 3, 1, shade(hair if style != "bald" else skin, 0.8))
    px(img, 20, 19, shade(skin, 0.75)); px(img, 20, 20, shade(skin, 0.75))
    mouth = e.get("mouth", "neutral")
    if mouth == "smile":
        rect(img, 18, 23, 4, 1, (150, 70, 70, 255)); px(img, 17, 22, (150, 70, 70, 255)); px(img, 22, 22, (150, 70, 70, 255))
    elif mouth == "grin":
        rect(img, 17, 22, 6, 2, OFFWHITE); rect(img, 17, 24, 6, 1, (150, 70, 70, 255))
    else:
        rect(img, 18, 23, 4, 1, (150, 70, 70, 255))
    if e.get("mustache"):
        rect(img, 16, 21, 8, 2, e["mustache"])
    if e.get("beard"):
        for y in range(21, 27):
            for x in range(14, 26):
                if (x - 20) ** 2 / 40 + (y - 21) ** 2 / 30 <= 1:
                    px(img, x, y, e["beard"])
        rect(img, 18, 23, 4, 1, (150, 70, 70, 255))
    if e.get("glasses"):
        g = e["glasses"]
        for (cx) in (16, 23):
            rect(img, cx - 2, 16, 5, 1, g); rect(img, cx - 2, 19, 5, 1, g)
            px(img, cx - 2, 17, g); px(img, cx - 2, 18, g); px(img, cx + 2, 17, g); px(img, cx + 2, 18, g)
        px(img, 19, 17, g); px(img, 20, 17, g)
    if e.get("sunglasses"):
        rect(img, 13, 16, 14, 3, INK)
        px(img, 15, 16, (90, 90, 120, 255)); px(img, 23, 16, (90, 90, 120, 255))
    if e.get("sunglasses_up"):
        rect(img, 13, 8, 14, 2, INK)
    if e.get("headphones"):
        rect(img, 11, 6, 18, 2, INK); rect(img, 9, 14, 3, 7, GREEN); rect(img, 28, 14, 3, 7, GREEN)
    if e.get("papers"):
        rect(img, 28, 27, 9, 12, OFFWHITE)
        for y in range(29, 38, 2):
            rect(img, 29, y, 7, 1, LIGHTGRAY)
        rect(img, 30, 25, 9, 12, OFFWHITE)
        for y in range(27, 36, 2):
            rect(img, 31, y, 7, 1, LIGHTGRAY)
        px(img, 37, 26, RED)
    outline_alpha(img)
    return img


def narrator():
    img = new(P, P)
    rect(img, 0, 0, P, P, GRAPHITE)
    rect(img, 6, 8, 28, 22, INK)
    rect(img, 7, 9, 26, 20, (20, 34, 24, 255))
    tiny_text(img, 9, 12, ">_", GREEN)
    rect(img, 9, 20, 18, 1, GREEN_DIM)
    rect(img, 9, 23, 12, 1, GREEN_DIM)
    rect(img, 16, 30, 8, 3, METAL_D)
    rect(img, 12, 33, 16, 2, METAL_M)
    return img


# --- Mentors d'A la French ---------------------------------------------------
# Portraits stylisés : casque de podcast, fond noir/vert de l'émission, et un
# accessoire par domaine (badge API, puce GPU, cône de chantier).
ALF_BG = (16, 20, 18, 255)


def podcast_frame(img):
    """Liseré vert de l'émission et petit micro dans le coin."""
    for i in range(P):
        px(img, i, 0, GREEN_DIM); px(img, i, P - 1, GREEN_DIM)
        px(img, 0, i, GREEN_DIM); px(img, P - 1, i, GREEN_DIM)
    rect(img, 34, 3, 3, 4, METAL_L)
    rect(img, 35, 7, 1, 3, METAL_M)
    rect(img, 33, 10, 5, 1, METAL_M)


def mentor(skin, hair, style, shirt, prop, extras=None):
    e = {"headphones": True, "mouth": "smile"}
    e.update(extras or {})
    img = bust(ALF_BG, skin, hair, style, shirt, e)
    if prop == "api":
        # cordon de badge + badge « API »
        for i in range(6):
            px(img, 15 + i // 2, 29 + i, GREEN); px(img, 25 - i // 2, 29 + i, GREEN)
        rect(img, 14, 33, 13, 7, OFFWHITE)
        tiny_text(img, 15, 34, "API", DEEP_GREEN)
    elif prop == "gpu":
        # puce GPU épinglée sur le sweat
        rect(img, 24, 32, 7, 6, GREEN_DIM)
        rect(img, 25, 33, 5, 4, INK)
        px(img, 27, 34, GREEN); px(img, 28, 35, GREEN)
        for k in range(3):
            px(img, 25 + 2 * k, 31, METAL_L); px(img, 25 + 2 * k, 38, METAL_L)
    elif prop == "cone":
        # petit cône de chantier orange à bandes blanches
        for y in range(6):
            w = 1 + y // 2
            rect(img, 27 - w, 31 + y, 2 * w + 1, 1, ORANGE)
        rect(img, 25, 33, 5, 1, OFFWHITE)
        rect(img, 24, 35, 7, 1, OFFWHITE)
        rect(img, 23, 37, 9, 1, shade(ORANGE, 0.8))
    podcast_frame(img)
    return img


HOSTS_REF = os.path.join(os.path.dirname(os.path.abspath(__file__)), "refs", "alafrench_hosts.png")
# centre (x, y) et côté du recadrage de chaque animateur dans l'image de référence
HOSTS_CROP = {"steeve": (44, 126, 70), "jb": (148, 116, 70), "mehdi": (256, 122, 70)}


def host(name):
    """Portrait d'un animateur d'À la French, repris de l'illustration de l'émission."""
    from PIL import ImageEnhance
    cx, cy, s = HOSTS_CROP[name]
    src = Image.open(HOSTS_REF).convert("RGB")
    c = src.crop((cx - s // 2, cy - s // 2, cx + s // 2, cy + s // 2)).resize((P, P), Image.LANCZOS)
    c = ImageEnhance.Contrast(c).enhance(1.12)
    img = c.quantize(32, method=Image.MEDIANCUT).convert("RGBA")
    for i in range(P):
        px(img, i, 0, GREEN_DIM); px(img, i, P - 1, GREEN_DIM)
        px(img, 0, i, GREEN_DIM); px(img, P - 1, i, GREEN_DIM)
    return img


def alf_logo():
    """Portrait générique de l'émission : le titre et trois micros."""
    img = new(P, P)
    rect(img, 0, 0, P, P, ALF_BG)
    podcast_frame(img)
    tiny_text(img, 4, 6, "A LA", OFFWHITE)
    tiny_text(img, 4, 13, "FRENCH", GREEN)
    for k in range(3):
        x = 7 + k * 10
        rect(img, x, 24, 4, 6, METAL_L)
        rect(img, x + 1, 25, 2, 4, METAL_M)
        rect(img, x + 1, 30, 2, 3, METAL_D)
        rect(img, x - 1, 33, 6, 1, METAL_D)
    return img



def portraits():
    S, H = SKINS, HAIRS
    return {
        "founder": bust(DEEP_GREEN, S[1], H[0], "short", GRAPHITE_2, {"logo": GREEN, "mouth": "smile"}),
        "founder_f": bust(DEEP_GREEN, S[1], H[4], "long", GRAPHITE_2, {"logo": GREEN, "mouth": "smile"}),
        "advisor": bust((92, 110, 160, 255), S[0], H[2], "bob", (120, 90, 140, 255), {"glasses": INK, "papers": True}),
        "bpi": bust((240, 210, 90, 255), S[2], H[0], "long", NAVY, {"glasses": RED, "scarf": YELLOW, "mouth": "smile"}),
        "jm": bust((90, 140, 200, 255), S[0], H[5], "slick", (230, 230, 240, 255), {"sunglasses_up": True, "mustache": H[5], "mouth": "grin", "collar": NAVY}),
        "patrick": bust((120, 60, 160, 255), S[1], H[3], "slick", INK, {"sunglasses": True, "chain": True, "mouth": "grin"}),
        "bernard": bust((70, 80, 90, 255), S[0], H[5], "bald", (60, 64, 76, 255), {"glasses": INK, "tie": RED, "collar": OFFWHITE}),
        "sophie": bust((200, 70, 80, 255), S[3], H[3], "bob", INK, {"collar": OFFWHITE, "mouth": "smile"}),
        "narrator": narrator(),
        "researcher": bust((40, 90, 80, 255), S[2], H[1], "curly", OFFWHITE, {"glasses": INK}),
        "cfo": bust((80, 90, 100, 255), S[1], H[0], "short", (120, 124, 132, 255), {"glasses": INK, "tie": INK, "collar": OFFWHITE}),
        "official": bust((50, 70, 130, 255), S[0], H[1], "slick", NAVY, {"sash": True, "collar": OFFWHITE, "mouth": "smile"}),
        "vc": bust((60, 120, 90, 255), S[1], H[1], "short", (110, 150, 220, 255), {"vest": (40, 50, 70, 255), "mouth": "grin"}),
        "generic": bust((90, 96, 90, 255), S[2], H[6], "short", ORANGE, {}),
        "mehdi": host("mehdi"),
        "steeve": host("steeve"),
        "jb": host("jb"),
        "alf": alf_logo(),
        "kyle": bust((96, 150, 20, 255), S[0], H[2], "short", (36, 40, 36, 255), {"sunglasses_up": True, "mouth": "grin", "logo": (118, 185, 0, 255)}),
    }


if __name__ == "__main__":
    ps = portraits()
    sheet = new(P * len(ps), P)
    for i, (k, v) in enumerate(ps.items()):
        sheet.alpha_composite(v, (i * P, 0))
    sheet.resize((sheet.width * 3, sheet.height * 3), Image.NEAREST).save("/tmp/claude-0/art/portraits.png")
    print(list(ps))
