"""Animated employee sprites: 12 roles x 8 looks x 16 frames (16x26 each).

Frame layout per row:
  0-1 idle   2-5 walk   6-7 typing (seated)   8-9 coffee
  10-11 talk 12-13 exhausted   14-15 panic
"""
from pixlib import *

FW, FH = 16, 26
FRAMES = 16
ROLES = ["Founder", "Backend", "Frontend", "AiResearcher", "DevOps", "Sales", "Growth",
         "ProductManager", "Hr", "Cfo", "Intern", "Consultant"]
LOOKS = 8

JEANS = (58, 74, 112, 255)
DARK_PANTS = (44, 48, 58, 255)
KHAKI = (176, 150, 104, 255)
SHOE = (34, 30, 30, 255)

OUTFIT = {
    # role: (shirt, shirt_shade, pants, extras)
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


def look_colors(look):
    skin = SKINS[look % len(SKINS)]
    hair = HAIRS[(look * 3 + 1) % len(HAIRS)]
    style = look % 4  # 0 short, 1 long, 2 bun, 3 curly
    return skin, hair, style


def draw_head(img, x0, y0, skin, hair, style, extras, eyes="open", mouth=None):
    # face 7x8 at (x0..x0+6, y0..y0+7)
    for y in range(y0 + 1, y0 + 8):
        for x in range(x0, x0 + 7):
            px(img, x, y, skin)
    # shading on the far side
    for y in range(y0 + 2, y0 + 8):
        px(img, x0, y, shade(skin, 0.86))
    # hair
    for x in range(x0 - (1 if style in (1, 3) else 0), x0 + 7 + (1 if style == 3 else 0)):
        px(img, x, y0, hair)
        px(img, x, y0 + 1, hair)
    px(img, x0, y0 + 2, hair)
    px(img, x0 + 1, y0 + 2, hair)
    px(img, x0 - (0 if style != 3 else 1), y0 + 3, hair)
    if style == 1:      # long hair down the side
        for y in range(y0 + 2, y0 + 10):
            px(img, x0 - 1, y, hair)
            px(img, x0, y, hair)
    elif style == 2:    # bun
        rect(img, x0 + 1, y0 - 2, 3, 2, hair)
    elif style == 3:    # curly volume
        for x in range(x0 - 1, x0 + 8):
            px(img, x, y0 - 1, hair)
        px(img, x0 + 7, y0 + 2, hair)
        px(img, x0 - 1, y0 + 4, hair)
    # ear
    px(img, x0 + 1, y0 + 4, shade(skin, 0.8))
    # eyes (facing right: features shifted right)
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
    # mouth
    if mouth == "open":
        px(img, x0 + 4, y0 + 6, INK)
        px(img, x0 + 4, y0 + 7, (160, 60, 60, 255))
    elif mouth == "smile":
        px(img, x0 + 3, y0 + 6, shade(skin, 0.6)); px(img, x0 + 5, y0 + 6, shade(skin, 0.6)); px(img, x0 + 4, y0 + 7, shade(skin, 0.6))
    else:
        px(img, x0 + 4, y0 + 6, shade(skin, 0.65))
    # headwear
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
        rect(img, x0 - 3, y0 + 1, 3, 1, shade(c, 0.8))  # backwards visor
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
    # name: (body_dy, head_dy, back_hand, front_hand, legs, eyes, mouth)
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
    by, hy, back_hand, front_hand, legs, eyes, mouth = POSES[pose]
    draw_legs(img, legs, pants, by if legs != "sit" else 1)
    # back arm (behind torso)
    draw_arm(img, (4, 12 + by), (back_hand[0], back_hand[1] + (by if pose.startswith("tired") else 0)), shirt_d, skin)
    # torso
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
    # head
    draw_head(img, 5, 3 + hy, skin, hair, style, extras, eyes, mouth)
    # front arm
    draw_arm(img, (11, 12 + by), (front_hand[0], front_hand[1] + (by if pose.startswith("tired") else 0)), shirt, skin)
    # props
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
    # preview: first look of each role, all frames, x4
    prev = new(FW * FRAMES, FH * len(ROLES))
    for r in range(len(ROLES)):
        prev.alpha_composite(s.crop((0, r * LOOKS * FH, FW * FRAMES, (r * LOOKS + 1) * FH)), (0, r * FH))
    bg = Image.new("RGBA", prev.size, (60, 70, 64, 255))
    bg.alpha_composite(prev)
    bg.resize((bg.width * 4, bg.height * 4), Image.NEAREST).save("/tmp/claude-0/art/chars_preview.png")
