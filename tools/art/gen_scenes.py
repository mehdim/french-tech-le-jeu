"""Fond de l'écran titre, illustrations des fins et icônes du HUD."""
import math, random
from pixlib import *
from gen_tiles import hash2

def sky(img, top, bottom, stars=True, seed=1):
    r = random.Random(seed)
    for y in range(img.height):
        c = mix(top, bottom, y / img.height)
        for x in range(img.width):
            px(img, x, y, c)
    if stars:
        for _ in range(img.width * img.height // 220):
            x, y = r.randrange(img.width), r.randrange(img.height // 2)
            px(img, x, y, (200, 220, 200, 255) if r.random() < 0.7 else GREEN_DIM)


def tower(img, cx, base, height, col):
    for y in range(base - height, base):
        t = (y - (base - height)) / height
        w = int(1 + t ** 2.2 * height * 0.33)
        for x in range(cx - w, cx + w + 1):
            if abs(x - cx) == w or (y + x) % 3 == 0 or w < 2:
                px(img, x, y, col)
        if abs(t - 0.62) < 0.02 or abs(t - 0.3) < 0.015:
            rect(img, cx - w - 1, y, 2 * w + 3, 1, col)
    # arche
    for x in range(cx - height // 5, cx + height // 5):
        dy = int((1 - ((x - cx) / (height / 5)) ** 2) * 6)
        for y in range(base - dy, base):
            px(img, x, y, CLEAR)
    px(img, cx, base - height - 3, RED)


def rooftops(img, base_y, seed, lit=0.25, color=(26, 32, 30, 255), window_cols=((250, 214, 120, 255), GREEN_DIM)):
    r = random.Random(seed)
    x = -4
    while x < img.width:
        w = r.randrange(18, 36)
        h = r.randrange(16, 42)
        top = base_y - h
        for yy in range(top, img.height):
            for xx in range(x, x + w):
                px(img, xx, yy, color)
        # toit mansardé en zinc
        for i in range(5):
            for xx in range(x + i, x + w - i):
                px(img, xx, top - i, shade(color, 1.25))
        for c in range(r.randrange(1, 3)):
            cx = x + r.randrange(2, max(3, w - 4))
            rect(img, cx, top - 9, 3, 5, shade(color, 1.1))
            rect(img, cx, top - 10, 3, 1, TERRACOTTA)
        for wy in range(top + 4, img.height - 4, 7):
            for wx in range(x + 3, x + w - 3, 5):
                if r.random() < lit:
                    col = window_cols[0] if r.random() < 0.8 else window_cols[1]
                    rect(img, wx, wy, 2, 3, col)
        x += w + r.randrange(0, 3)


def title_background():
    img = new(320, 180)
    sky(img, (8, 12, 14, 255), (34, 52, 44, 255), seed=4)
    # lune
    for y in range(-6, 7):
        for x in range(-6, 7):
            if x * x + y * y <= 36 and (x + 2) ** 2 + (y - 1) ** 2 > 22:
                px(img, 262 + x, 30 + y, OFFWHITE)
    tower(img, 230, 150, 98, (54, 70, 64, 255))
    rooftops(img, 150, 7, lit=0.18, color=(30, 38, 36, 255))
    rooftops(img, 172, 11, lit=0.32, color=(18, 22, 21, 255))
    # enseigne au néon sur un toit
    tiny_text(img, 40, 128, "VERIDIX", GREEN)
    for x in range(38, 70):
        px(img, x, 134, GREEN_DIM)
    return img


def unicorn(img, cx, cy):
    body = [(x, y) for y in range(-6, 6) for x in range(-14, 14) if (x / 14) ** 2 + (y / 6) ** 2 <= 1]
    for x, y in body:
        px(img, cx + x, cy + y, OFFWHITE if y < 3 else LIGHTGRAY)
    for i in range(10):                         # cou et tête
        rect(img, cx + 10 + i // 2, cy - 6 - i, 6, 3, OFFWHITE)
    rect(img, cx + 15, cy - 17, 9, 5, OFFWHITE)
    px(img, cx + 21, cy - 16, INK)
    for i in range(7):                          # corne
        px(img, cx + 18 + i // 2, cy - 18 - i, GREEN)
    for i in range(10):                         # crinière
        px(img, cx + 12 + i // 3, cy - 8 - i, [GREEN, ORANGE, RED, BLUE, PURPLE][i % 5])
    for lx, d in ((-9, 4), (-5, -3), (6, 3), (10, -4)):   # pattes au galop
        for i in range(8):
            px(img, cx + lx + (d * i) // 8, cy + 5 + i, LIGHTGRAY)
    for i in range(9):                          # queue arc-en-ciel
        px(img, cx - 14 - i, cy - 3 + i // 3, [RED, ORANGE, YELLOW, GREEN, BLUE][i % 5])
    outline_alpha(img)


def ending_unicorn():
    img = new(160, 90)
    sky(img, (24, 10, 40, 255), (70, 40, 90, 255), seed=2)
    pts = [(0, 80), (30, 74), (55, 70), (80, 56), (105, 44), (130, 22), (150, 8)]
    for a, b in zip(pts, pts[1:]):
        line(img, a, b, GREEN); line(img, (a[0], a[1] + 1), (b[0], b[1] + 1), GREEN_DIM)
    unicorn(img, 78, 46)
    r = random.Random(3)
    for _ in range(60):
        px(img, r.randrange(160), r.randrange(90), [YELLOW, GREEN, PINK, OFFWHITE][r.randrange(4)])
    tiny_text(img, 4, 4, "VALO: 1 000 000 000 EUR", YELLOW)
    tiny_text(img, 4, 82, "ARR X 83", RED)
    return img


def ending_profitable():
    img = new(160, 90)
    sky(img, (240, 170, 110, 255), (250, 214, 160, 255), stars=False)
    rooftops(img, 80, 5, lit=0.0, color=(120, 90, 90, 255))
    # immeuble aux fenêtres éclairées
    rect(img, 50, 30, 60, 60, (226, 214, 190, 255))
    for i in range(4):
        for x in range(48 + i, 112 - i):
            px(img, x, 30 - i, ZINC)
    for wy in range(36, 84, 12):
        for wx in range(56, 104, 12):
            rect(img, wx, wy, 6, 8, (250, 214, 120, 255))
            rect(img, wx + 2, wy + 4, 2, 4, LEAF_M)
    tiny_text(img, 62, 22, "VERIDIX", DEEP_GREEN)
    tiny_text(img, 4, 4, "EBITDA > 0", DEEP_GREEN)
    # une courbe qui monte dans une fenêtre
    pts = [(116, 70), (126, 64), (136, 66), (146, 52), (156, 40)]
    for a, b in zip(pts, pts[1:]):
        line(img, a, b, DEEP_GREEN)
    return img


def ending_acquisition():
    img = new(160, 90)
    sky(img, (110, 150, 200, 255), (200, 220, 236, 255), stars=False)
    # tour de verre
    for y in range(6, 90):
        for x in range(70, 120):
            c = (120, 160, 200, 255) if (x // 5 + y // 6) % 2 else (150, 190, 226, 255)
            px(img, x, y, c)
    for y in range(6, 90, 6):
        rect(img, 70, y, 50, 1, (90, 120, 160, 255))
    rect(img, 74, 20, 42, 9, OFFWHITE)
    tiny_text(img, 76, 22, "ACQUIRED", RED)
    # fondateur portant un carton avec une cafetière italienne
    rect(img, 30, 66, 6, 7, SKINS[1]); rect(img, 30, 64, 6, 2, HAIRS[0])
    rect(img, 29, 73, 8, 9, GRAPHITE_2)
    rect(img, 30, 82, 2, 6, JEANS if False else (58, 74, 112, 255)); rect(img, 34, 82, 2, 6, (58, 74, 112, 255))
    rect(img, 24, 74, 18, 8, WOOD_L)
    rect(img, 31, 70, 4, 4, METAL_L)
    outline_alpha(img)
    tiny_text(img, 4, 4, "PAYE EN GPU", NAVY)
    return img


def ending_ipo():
    img = new(160, 90)
    sky(img, (230, 236, 244, 255), (250, 214, 160, 255), stars=False)
    # façade néoclassique de la Bourse : fronton, colonnes, marches
    for i in range(10):
        for x in range(40 + i * 4, 120 - i * 4):
            px(img, x, 22 - i // 2, (226, 214, 190, 255))
    rect(img, 36, 24, 88, 4, (210, 198, 172, 255))
    for cx in range(42, 122, 12):
        rect(img, cx, 28, 5, 40, (236, 228, 208, 255))
        rect(img, cx + 4, 28, 1, 40, (190, 178, 150, 255))
    rect(img, 36, 68, 88, 3, (200, 188, 162, 255))
    rect(img, 30, 71, 100, 3, (186, 174, 150, 255))
    rect(img, 24, 74, 112, 3, (170, 160, 138, 255))
    # bandeau de cotation lumineux
    rect(img, 0, 79, 160, 11, (18, 22, 21, 255))
    tiny_text(img, 4, 82, "VRDX +38%", GREEN)
    tiny_text(img, 70, 82, "CAC +0,2%", OFFWHITE)
    # flèche verte qui monte au-dessus du fronton
    pts = [(10, 60), (30, 50), (48, 54), (70, 34), (96, 30), (124, 12), (150, 4)]
    for a, b in zip(pts, pts[1:]):
        line(img, a, b, GREEN); line(img, (a[0], a[1] + 1), (b[0], b[1] + 1), GREEN_DIM)
    # la cloche
    for y in range(6):
        w = 2 + y // 2
        rect(img, 76 - w, 32 + y, 2 * w + 1, 1, YELLOW)
    px(img, 76, 39, ORANGE)
    tiny_text(img, 4, 4, "INTRODUCTION EN BOURSE", NAVY)
    return img


def ending_commons():
    img = new(160, 90)
    sky(img, (12, 18, 16, 255), (26, 40, 34, 255), seed=9)
    # un arbre de commits lumineux
    def branch(x, y, a, l, d):
        if d == 0:
            return
        x2, y2 = x + math.cos(a) * l, y - math.sin(a) * l
        line(img, (x, y), (x2, y2), GREEN if d < 3 else GREEN_DIM)
        px(img, round(x2), round(y2), OFFWHITE)
        branch(x2, y2, a + 0.45, l * 0.7, d - 1)
        branch(x2, y2, a - 0.45, l * 0.7, d - 1)
    branch(80, 76, math.pi / 2, 18, 6)
    # de petits contributeurs autour
    r = random.Random(5)
    for i in range(14):
        x = 10 + i * 10 + r.randrange(-2, 3)
        y = 78 + (i % 2) * 4
        rect(img, x, y - 6, 4, 4, SKINS[i % 5]); rect(img, x, y - 7, 4, 1, HAIRS[i % 8])
        rect(img, x - 1, y - 2, 6, 6, [DEEP_GREEN, ORANGE, BLUE, PURPLE, OFFWHITE][i % 5])
    tiny_text(img, 4, 4, "OPEN SOURCE - FOSDEM", GREEN)
    return img


def icon(draw):
    img = new(12, 12)
    draw(img)
    return img


def icons():
    def cash(i):
        rect(i, 1, 3, 10, 6, DEEP_GREEN); rect(i, 2, 4, 8, 4, GREEN_DIM); rect(i, 5, 5, 2, 2, OFFWHITE)
    def mrr(i):
        for k in range(5):
            rect(i, 2 + k * 2, 10 - k * 2, 2, 1 + k * 2, GREEN)
    def runway(i):
        rect(i, 2, 1, 8, 1, ORANGE); rect(i, 2, 10, 8, 1, ORANGE)
        for y in range(2, 10):
            w = abs(5.5 - y)
            rect(i, round(6 - w), y, max(1, round(2 * w)), 1, OFFWHITE)
    def people(i):
        rect(i, 2, 2, 3, 3, OFFWHITE); rect(i, 1, 6, 5, 5, OFFWHITE)
        rect(i, 7, 3, 3, 3, LIGHTGRAY); rect(i, 6, 7, 5, 4, LIGHTGRAY)
    def customers(i):
        rect(i, 1, 4, 10, 7, BLUE); rect(i, 3, 2, 6, 2, BLUE)
        for x in (3, 6, 9):
            rect(i, x - 1, 6, 1, 2, YELLOW)
    def churn(i):
        for k in range(5):
            rect(i, 2 + k * 2, 1 + k, 2, 1 + k, RED)
    def day(i):
        rect(i, 1, 2, 10, 9, OFFWHITE); rect(i, 1, 2, 10, 3, RED); px(i, 3, 1, INK); px(i, 8, 1, INK)
        rect(i, 3, 7, 2, 2, INK)
    def quality(i):
        pts = [(6, 1), (7, 4), (11, 4), (8, 6), (9, 10), (6, 8), (3, 10), (4, 6), (1, 4), (5, 4)]
        poly(i, pts, YELLOW, None)
    def accuracy(i):
        for r, c in ((5, RED), (3, OFFWHITE), (1, RED)):
            for y in range(-r, r + 1):
                for x in range(-r, r + 1):
                    if x * x + y * y <= r * r:
                        px(i, 6 + x, 6 + y, c)
    def debt(i):
        rect(i, 3, 3, 6, 7, RED); rect(i, 4, 1, 4, 2, RED)
        for y in (4, 6, 8):
            px(i, 2, y, INK); px(i, 9, y, INK)
    def morale(i):
        for y in range(0, 10):
            for x in range(0, 12):
                if ((x - 3.5) ** 2 + (y - 3.5) ** 2 <= 7) or ((x - 7.5) ** 2 + (y - 3.5) ** 2 <= 7) or (y >= 4 and abs(x - 5.5) <= 9.5 - y):
                    px(i, x, y + 1, PINK)
    def fatigue(i):
        tiny_text(i, 1, 5, "Z", LIGHTGRAY); tiny_text(i, 6, 1, "Z", OFFWHITE)
    def reputation(i):
        poly(i, [(1, 5), (5, 5), (10, 1), (10, 11), (5, 7), (1, 7)], ORANGE, None)
    def valuation(i):
        poly(i, [(2, 4), (4, 1), (8, 1), (10, 4), (6, 11)], (130, 220, 230, 255), INK)
    def research(i):
        rect(i, 4, 1, 4, 2, OFFWHITE); rect(i, 5, 3, 2, 3, OFFWHITE)
        poly(i, [(2, 10), (5, 5), (7, 5), (10, 10)], GREEN, None)
    def leads(i):
        poly(i, [(1, 1), (11, 1), (7, 6), (7, 11), (5, 11), (5, 6)], BLUE, None)
    return {k: icon(f) for k, f in {
        "cash": cash, "mrr": mrr, "runway": runway, "people": people, "customers": customers, "churn": churn,
        "day": day, "quality": quality, "accuracy": accuracy, "debt": debt, "morale": morale, "fatigue": fatigue,
        "reputation": reputation, "valuation": valuation, "research": research, "leads": leads}.items()}


if __name__ == "__main__":
    title_background().resize((960, 540), Image.NEAREST).save("/tmp/claude-0/art/title.png")
    strip = new(160 * 4 + 12, 90)
    for i, f in enumerate([ending_unicorn, ending_profitable, ending_acquisition, ending_commons]):
        strip.alpha_composite(f(), (i * 164, 0))
    strip.resize((strip.width * 2, strip.height * 2), Image.NEAREST).save("/tmp/claude-0/art/endings.png")
    ic = icons(); s = new(14 * len(ic), 12)
    for i, v in enumerate(ic.values()):
        s.alpha_composite(v, (i * 14, 0))
    bg = Image.new("RGBA", s.size, GRAPHITE); bg.alpha_composite(s)
    bg.resize((bg.width * 5, bg.height * 5), Image.NEAREST).save("/tmp/claude-0/art/icons.png")
