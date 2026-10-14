"""Shared helpers for the procedural pixel-art generator.

Everything is drawn at 1x with a fixed palette, no anti-aliasing. The game
renders with nearest-neighbour sampling and an integer camera zoom.
"""
from PIL import Image, ImageDraw

# --- palette (À la French: black / green / off-white, plus support colours) --
INK = (16, 20, 18, 255)            # #101412 outlines
GRAPHITE = (32, 40, 34, 255)       # #202822
GRAPHITE_2 = (44, 54, 47, 255)
GREEN = (124, 255, 82, 255)        # #7CFF52
GREEN_DIM = (88, 196, 62, 255)
DEEP_GREEN = (38, 125, 75, 255)    # #267D4B
OFFWHITE = (244, 241, 232, 255)    # #F4F1E8
LIGHTGRAY = (190, 197, 187, 255)   # #BEC5BB
MIDGRAY = (140, 148, 140, 255)
DARKGRAY = (88, 96, 90, 255)
ORANGE = (246, 162, 60, 255)       # #F6A23C
RED = (228, 91, 91, 255)           # #E45B5B
YELLOW = (250, 214, 92, 255)
BLUE = (86, 132, 196, 255)
NAVY = (44, 62, 104, 255)
SKY = (120, 158, 206, 255)
SKY_DUSK = (232, 160, 120, 255)
PURPLE = (126, 92, 168, 255)
PINK = (236, 140, 170, 255)
WOOD_L = (206, 152, 98, 255)
WOOD_M = (168, 112, 66, 255)
WOOD_D = (116, 74, 44, 255)
WOOD_DD = (82, 52, 32, 255)
TERRACOTTA = (190, 100, 70, 255)
LEAF_L = (110, 196, 92, 255)
LEAF_M = (62, 150, 72, 255)
LEAF_D = (36, 102, 58, 255)
METAL_L = (176, 184, 180, 255)
METAL_M = (122, 130, 126, 255)
METAL_D = (74, 82, 78, 255)
BRICK_L = (170, 88, 66, 255)
BRICK_D = (128, 62, 48, 255)
ZINC = (130, 142, 150, 255)
ZINC_D = (96, 106, 114, 255)
CLEAR = (0, 0, 0, 0)

SKINS = [(246, 210, 176, 255), (228, 178, 134, 255), (192, 134, 94, 255), (146, 96, 64, 255), (102, 66, 44, 255)]
HAIRS = [(42, 32, 26, 255), (98, 64, 36, 255), (214, 170, 96, 255), (30, 30, 34, 255), (156, 66, 42, 255),
         (226, 224, 214, 255), (60, 40, 30, 255), (120, 84, 50, 255)]

TW, TH = 32, 16   # isometric tile size
HALF_W, HALF_H = TW // 2, TH // 2


def shade(c, f):
    """Multiply rgb by f (f<1 darker, f>1 lighter)."""
    r, g, b, a = c
    return (max(0, min(255, int(r * f))), max(0, min(255, int(g * f))), max(0, min(255, int(b * f))), a)


def mix(a, b, t):
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(3)) + (255,)


def new(w, h):
    return Image.new("RGBA", (w, h), CLEAR)


def iso_pt(origin, gx, gy, gz=0):
    """Project grid coords (gx, gy) and height gz (pixels) to image pixel coords."""
    ox, oy = origin
    return (ox + (gx - gy) * HALF_W, oy + (gx + gy) * HALF_H - gz)


def poly(img, pts, fill, outline=None):
    d = ImageDraw.Draw(img)
    pts = [(round(x), round(y)) for x, y in pts]
    d.polygon(pts, fill=fill, outline=outline)


def line(img, a, b, color):
    d = ImageDraw.Draw(img)
    d.line([(round(a[0]), round(a[1])), (round(b[0]), round(b[1]))], fill=color)


def px(img, x, y, c):
    if 0 <= x < img.width and 0 <= y < img.height:
        img.putpixel((int(x), int(y)), c)


def rect(img, x, y, w, h, c):
    for i in range(int(w)):
        for j in range(int(h)):
            px(img, x + i, y + j, c)


def iso_box(img, o, x0, y0, w, h, z0, height, color, top=None, left=None, right=None, outline=INK):
    """Draw an isometric box. Faces: top, front-left (y = y0+h), front-right (x = x0+w)."""
    top = top or shade(color, 1.18)
    left = left or color
    right = right or shade(color, 0.78)
    z1 = z0 + height
    lf = [iso_pt(o, x0, y0 + h, z0), iso_pt(o, x0 + w, y0 + h, z0), iso_pt(o, x0 + w, y0 + h, z1), iso_pt(o, x0, y0 + h, z1)]
    rf = [iso_pt(o, x0 + w, y0, z0), iso_pt(o, x0 + w, y0 + h, z0), iso_pt(o, x0 + w, y0 + h, z1), iso_pt(o, x0 + w, y0, z1)]
    tf = [iso_pt(o, x0, y0, z1), iso_pt(o, x0 + w, y0, z1), iso_pt(o, x0 + w, y0 + h, z1), iso_pt(o, x0, y0 + h, z1)]
    if height > 0:
        poly(img, lf, left, outline)
        poly(img, rf, right, outline)
    poly(img, tf, top, outline)
    return tf


def outline_alpha(img, color=INK):
    """Add a 1px outline around opaque pixels (for characters / props)."""
    w, h = img.size
    src = img.copy()
    pxs = src.load()
    out = img.load()
    for y in range(h):
        for x in range(w):
            if pxs[x, y][3] == 0:
                for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                    nx, ny = x + dx, y + dy
                    if 0 <= nx < w and 0 <= ny < h and pxs[nx, ny][3] > 0 and pxs[nx, ny] != color:
                        out[x, y] = color
                        break
    return img


# --- tiny 3x5 font for signs and posters ------------------------------------
TINY = {
    "A": ["010", "101", "111", "101", "101"], "B": ["110", "101", "110", "101", "110"],
    "C": ["011", "100", "100", "100", "011"], "D": ["110", "101", "101", "101", "110"],
    "E": ["111", "100", "110", "100", "111"], "F": ["111", "100", "110", "100", "100"],
    "G": ["011", "100", "101", "101", "011"], "H": ["101", "101", "111", "101", "101"],
    "I": ["111", "010", "010", "010", "111"], "J": ["001", "001", "001", "101", "010"],
    "K": ["101", "101", "110", "101", "101"], "L": ["100", "100", "100", "100", "111"],
    "M": ["101", "111", "111", "101", "101"], "N": ["110", "101", "101", "101", "101"],
    "O": ["010", "101", "101", "101", "010"], "P": ["110", "101", "110", "100", "100"],
    "Q": ["010", "101", "101", "110", "011"], "R": ["110", "101", "110", "101", "101"],
    "S": ["011", "100", "010", "001", "110"], "T": ["111", "010", "010", "010", "010"],
    "U": ["101", "101", "101", "101", "111"], "V": ["101", "101", "101", "101", "010"],
    "W": ["101", "101", "111", "111", "101"], "X": ["101", "101", "010", "101", "101"],
    "Y": ["101", "101", "010", "010", "010"], "Z": ["111", "001", "010", "100", "111"],
    "0": ["111", "101", "101", "101", "111"], "1": ["010", "110", "010", "010", "111"],
    "2": ["110", "001", "010", "100", "111"], "3": ["110", "001", "010", "001", "110"],
    "4": ["101", "101", "111", "001", "001"], "5": ["111", "100", "110", "001", "110"],
    "6": ["011", "100", "111", "101", "111"], "7": ["111", "001", "010", "010", "010"],
    "8": ["111", "101", "111", "101", "111"], "9": ["111", "101", "111", "001", "110"],
    "€": ["011", "100", "111", "100", "011"], "$": ["011", "110", "010", "011", "110"],
    "!": ["010", "010", "010", "000", "010"], "?": ["110", "001", "010", "000", "010"],
    ".": ["000", "000", "000", "000", "010"], ",": ["000", "000", "000", "010", "100"],
    "-": ["000", "000", "111", "000", "000"], ":": ["000", "010", "000", "010", "000"],
    "'": ["010", "010", "000", "000", "000"], "%": ["101", "001", "010", "100", "101"],
    "/": ["001", "001", "010", "100", "100"], "+": ["000", "010", "111", "010", "000"],
    " ": ["000", "000", "000", "000", "000"], "#": ["101", "111", "101", "111", "101"],
    ">": ["100", "010", "001", "010", "100"], "<": ["001", "010", "100", "010", "001"],
    "_": ["000", "000", "000", "000", "111"], "=": ["000", "111", "000", "111", "000"],
}


def tiny_text(img, x, y, text, color, spacing=1):
    cx = x
    for ch in text.upper():
        g = TINY.get(ch, TINY["?"])
        for j, row in enumerate(g):
            for i, b in enumerate(row):
                if b == "1":
                    px(img, cx + i, y + j, color)
        cx += 3 + spacing
    return cx


def tiny_width(text, spacing=1):
    return len(text) * (3 + spacing) - spacing


def crop_to_content(img, pad=1):
    bbox = img.getbbox()
    if not bbox:
        return img, (0, 0)
    l, t, r, b = bbox
    l, t = max(0, l - pad), max(0, t - pad)
    r, b = min(img.width, r + pad), min(img.height, b + pad)
    return img.crop((l, t, r, b)), (l, t)
