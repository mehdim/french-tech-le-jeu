"""Sprites de mobilier isométrique. Chaque fonction renvoie (image, ancre), où l'ancre est la
position en pixels du coin arrière de l'emprise (point de grille (0,0) au niveau du sol)."""
import math
from pixlib import *


def canvas(w, h, zmax):
    W = HALF_W * (w + h) + 8
    H = HALF_H * (w + h) + zmax + 10
    o = (HALF_W * h + 4, zmax + 6)
    return new(W, H), o


def finish(img, o):
    cropped, (l, t) = crop_to_content(img, pad=1)
    return cropped, (o[0] - l, o[1] - t)


def legs(img, o, x0, y0, w, h, height, color, inset=0.12):
    for (gx, gy) in [(x0 + inset, y0 + h - inset), (x0 + w - inset, y0 + h - inset), (x0 + w - inset, y0 + inset)]:
        a = iso_pt(o, gx, gy, 0)
        for z in range(int(height)):
            px(img, a[0], a[1] - z, color)
            px(img, a[0] + 1, a[1] - z, shade(color, 0.8))


def table(img, o, x0, y0, w, h, height, top_col, leg_col=METAL_D, thick=2):
    legs(img, o, x0, y0, w, h, height - thick, leg_col)
    return iso_box(img, o, x0, y0, w, h, height - thick, thick, top_col)


def desk():
    img, o = canvas(2, 1, 40)
    table(img, o, 0, 0, 2, 1, 14, WOOD_L)
    # laptop vu de dos (le salarié fait face à la caméra)
    iso_box(img, o, 0.65, 0.35, 0.7, 0.35, 14, 1, METAL_M)
    lid = [iso_pt(o, 0.65, 0.42, 15), iso_pt(o, 1.35, 0.42, 15), iso_pt(o, 1.35, 0.42, 25), iso_pt(o, 0.65, 0.42, 25)]
    poly(img, lid, GRAPHITE, INK)
    c = iso_pt(o, 1.0, 0.42, 20)
    px(img, c[0], c[1], GREEN)
    px(img, c[0] + 1, c[1], GREEN)
    # mug
    m = iso_pt(o, 1.7, 0.7, 14)
    rect(img, m[0] - 1, m[1] - 4, 3, 4, OFFWHITE)
    px(img, m[0] + 2, m[1] - 3, OFFWHITE)
    px(img, m[0], m[1] - 4, WOOD_DD)
    # papiers
    p = iso_pt(o, 0.3, 0.6, 14)
    rect(img, p[0] - 2, p[1] - 1, 4, 2, OFFWHITE)
    return finish(img, o)


def workstation():
    img, o = canvas(2, 1, 44)
    table(img, o, 0, 0, 2, 1, 14, GRAPHITE_2, leg_col=METAL_M)
    # tour PC sous le bureau
    iso_box(img, o, 1.55, 0.2, 0.35, 0.5, 0, 11, GRAPHITE)
    t = iso_pt(o, 1.9, 0.5, 8)
    px(img, t[0] - 1, t[1], GREEN)
    # dos d'écran sur pied
    s = iso_pt(o, 1.0, 0.35, 14)
    rect(img, s[0], s[1] - 6, 1, 6, METAL_D)
    mon = [iso_pt(o, 0.45, 0.35, 19), iso_pt(o, 1.55, 0.35, 19), iso_pt(o, 1.55, 0.35, 32), iso_pt(o, 0.45, 0.35, 32)]
    poly(img, mon, INK, INK)
    mon2 = [iso_pt(o, 0.5, 0.35, 20), iso_pt(o, 1.5, 0.35, 20), iso_pt(o, 1.5, 0.35, 31), iso_pt(o, 0.5, 0.35, 31)]
    poly(img, mon2, GRAPHITE_2, None)
    g = iso_pt(o, 1.0, 0.35, 26)
    px(img, g[0], g[1], GREEN_DIM)
    # clavier mécanique aux touches colorées (côté spectateur)
    for i in range(6):
        k = iso_pt(o, 0.55 + i * 0.15, 0.78, 14)
        px(img, k[0], k[1] - 1, [GREEN, OFFWHITE, ORANGE, OFFWHITE, GREEN, OFFWHITE][i])
    # lampe de bureau
    l = iso_pt(o, 0.2, 0.3, 14)
    rect(img, l[0], l[1] - 7, 1, 7, METAL_D)
    rect(img, l[0] - 1, l[1] - 9, 4, 2, YELLOW)
    return finish(img, o)


def plant():
    img, o = canvas(1, 1, 34)
    iso_box(img, o, 0.28, 0.28, 0.44, 0.44, 0, 8, TERRACOTTA)
    c = iso_pt(o, 0.5, 0.5, 18)
    leaves = [(0, -6, 5), (-5, -2, 4), (5, -2, 4), (-3, 3, 4), (4, 3, 3), (0, 0, 5), (-6, -8, 3), (6, -9, 3)]
    for i, (dx, dy, r) in enumerate(leaves):
        col = [LEAF_M, LEAF_L, LEAF_D][i % 3]
        for yy in range(-r, r + 1):
            for xx in range(-r, r + 1):
                if xx * xx + yy * yy * 1.4 <= r * r:
                    px(img, c[0] + dx + xx, c[1] + dy + yy, col)
        # trous du monstera
        px(img, c[0] + dx, c[1] + dy, LEAF_D)
    outline_alpha(img)
    return finish(img, o)


def coffee_moka():
    img, o = canvas(1, 1, 30)
    table(img, o, 0.15, 0.15, 0.7, 0.7, 13, WOOD_M)
    m = iso_pt(o, 0.5, 0.5, 13)
    rect(img, m[0] - 2, m[1] - 4, 5, 4, METAL_L)
    rect(img, m[0] - 1, m[1] - 8, 3, 4, METAL_M)
    px(img, m[0] - 1, m[1] - 9, INK)
    px(img, m[0] + 3, m[1] - 6, INK)
    px(img, m[0] + 3, m[1] - 5, INK)
    return finish(img, o)


def coffee_machine():
    img, o = canvas(1, 1, 40)
    iso_box(img, o, 0, 0, 1, 1, 0, 15, WOOD_M)
    iso_box(img, o, 0.2, 0.15, 0.6, 0.5, 15, 14, GRAPHITE)
    iso_box(img, o, 0.25, 0.2, 0.5, 0.4, 29, 2, METAL_M)
    f = iso_pt(o, 0.55, 0.65, 24)
    px(img, f[0], f[1], GREEN)          # LED d'uptime
    px(img, f[0] - 2, f[1], ORANGE)
    sp = iso_pt(o, 0.5, 0.65, 18)
    rect(img, sp[0] - 1, sp[1], 2, 1, METAL_L)
    cup = iso_pt(o, 0.5, 0.75, 15)
    rect(img, cup[0] - 1, cup[1] - 3, 3, 3, OFFWHITE)
    for i, (gx, gy) in enumerate([(0.15, 0.85), (0.8, 0.8)]):
        c2 = iso_pt(o, gx, gy, 15)
        rect(img, c2[0], c2[1] - 2, 2, 2, [RED, OFFWHITE][i])
    return finish(img, o)


def sofa():
    img, o = canvas(2, 1, 30)
    iso_box(img, o, 0, 0.1, 2, 0.9, 0, 7, DEEP_GREEN)
    iso_box(img, o, 0, 0, 2, 0.3, 7, 8, shade(DEEP_GREEN, 0.9))
    iso_box(img, o, 0, 0.1, 0.25, 0.9, 7, 4, shade(DEEP_GREEN, 0.95))
    iso_box(img, o, 1.75, 0.1, 0.25, 0.9, 7, 4, shade(DEEP_GREEN, 0.95))
    for gx in (0.6, 1.3):
        c = iso_pt(o, gx, 0.6, 8)
        rect(img, c[0] - 2, c[1] - 2, 4, 2, shade(DEEP_GREEN, 1.25))
    # coussin
    c = iso_pt(o, 1.5, 0.35, 12)
    rect(img, c[0] - 2, c[1] - 3, 4, 4, ORANGE)
    return finish(img, o)


def whiteboard_item():
    img, o = canvas(1, 1, 44)
    for gx in (0.15, 0.85):
        a = iso_pt(o, gx, 0.5, 0)
        rect(img, a[0], a[1] - 30, 1, 30, METAL_D)
    board = [iso_pt(o, 0.05, 0.5, 14), iso_pt(o, 0.95, 0.5, 14), iso_pt(o, 0.95, 0.5, 36), iso_pt(o, 0.05, 0.5, 36)]
    poly(img, board, OFFWHITE, METAL_D)
    # boîtes et flèches
    for (gx, z, col) in [(0.25, 30, BLUE), (0.6, 30, DEEP_GREEN), (0.42, 20, RED)]:
        p = iso_pt(o, gx, 0.5, z)
        rect(img, p[0] - 2, p[1] - 2, 5, 3, col)
    a = iso_pt(o, 0.3, 0.5, 27)
    b = iso_pt(o, 0.45, 0.5, 22)
    line(img, a, b, INK)
    return finish(img, o)


def meeting_table():
    img, o = canvas(2, 2, 30)
    # chaises
    for (gx, gy) in [(0.3, 2.05), (1.2, 2.05), (2.05, 0.4), (2.05, 1.3)]:
        iso_box(img, o, gx, gy, 0.45, 0.4, 0, 6, ORANGE)
    table(img, o, 0.2, 0.2, 1.6, 1.6, 11, OFFWHITE, leg_col=METAL_D)
    for (gx, gy) in [(0.6, 0.7), (1.3, 1.2)]:
        p = iso_pt(o, gx, gy, 11)
        rect(img, p[0] - 3, p[1] - 1, 6, 2, GRAPHITE)
        px(img, p[0], p[1] - 1, GREEN)
    p = iso_pt(o, 1.0, 1.0, 11)
    rect(img, p[0] - 1, p[1] - 3, 2, 3, OFFWHITE)
    return finish(img, o)


def server_rack():
    img, o = canvas(1, 1, 48)
    iso_box(img, o, 0.1, 0.1, 0.8, 0.8, 0, 38, GRAPHITE)
    for z in range(4, 36, 4):
        a = iso_pt(o, 0.15, 0.9, z)
        b = iso_pt(o, 0.85, 0.9, z)
        line(img, a, b, INK)
        led = iso_pt(o, 0.25 + (z % 12) * 0.03, 0.9, z + 2)
        px(img, led[0], led[1], GREEN if z % 8 else ORANGE)
        led2 = iso_pt(o, 0.7, 0.9, z + 2)
        px(img, led2[0], led2[1], GREEN_DIM)
    return finish(img, o)


def gpu_server():
    img, o = canvas(1, 1, 40)
    iso_box(img, o, 0.1, 0.1, 0.8, 0.8, 0, 28, GRAPHITE_2)
    for i, z in enumerate((8, 18)):
        c = iso_pt(o, 0.45, 0.9, z)
        for yy in range(-3, 4):
            for xx in range(-3, 4):
                if xx * xx + yy * yy <= 9:
                    px(img, c[0] + xx, c[1] + yy, GREEN if (xx + yy + i) % 3 else DEEP_GREEN)
        px(img, c[0], c[1], INK)
    t = iso_pt(o, 0.9, 0.3, 24)
    tiny_text(img, t[0] - 2, t[1] - 4, "AI", GREEN)
    return finish(img, o)


def k8s_cluster():
    img, o = canvas(2, 1, 50)
    iso_box(img, o, 0, 0, 2, 1, 0, 6, METAL_D)
    for i, (gx, h) in enumerate([(0.1, 24), (0.55, 30), (1.0, 18), (1.45, 26)]):
        iso_box(img, o, gx, 0.2, 0.4, 0.5, 6, h, [BLUE, NAVY, BLUE, NAVY][i])
        for z in range(10, 6 + h, 5):
            p = iso_pt(o, gx + 0.2, 0.7, z)
            px(img, p[0], p[1], [GREEN, ORANGE, GREEN, RED][(i + z) % 4])
    # des câbles partout
    for (a, b) in [((0.3, 0.7, 30), (0.75, 0.7, 36)), ((0.75, 0.7, 20), (1.2, 0.7, 24)), ((1.2, 0.7, 14), (1.65, 0.7, 28)), ((0.3, 0.7, 12), (1.65, 0.7, 10))]:
        pa, pb = iso_pt(o, *a), iso_pt(o, *b)
        mx, my = (pa[0] + pb[0]) / 2, max(pa[1], pb[1]) + 4
        line(img, pa, (mx, my), RED)
        line(img, (mx, my), pb, RED)
    # un engrenage au-dessus, qui tourne sans raison
    g = iso_pt(o, 0.75, 0.45, 42)
    for k in range(8):
        a = k / 8 * 2 * math.pi
        px(img, g[0] + round(4 * math.cos(a)), g[1] + round(3 * math.sin(a)), OFFWHITE)
    px(img, g[0], g[1], OFFWHITE)
    return finish(img, o)


def pingpong():
    img, o = canvas(2, 2, 24)
    legs(img, o, 0.1, 0.1, 1.8, 1.8, 10, METAL_D)
    iso_box(img, o, 0.1, 0.1, 1.8, 1.8, 10, 2, DEEP_GREEN)
    a = iso_pt(o, 0.1, 1.0, 12)
    b = iso_pt(o, 1.9, 1.0, 12)
    line(img, a, b, OFFWHITE)
    # le filet a disparu ; il reste un poteau
    p = iso_pt(o, 0.1, 1.0, 12)
    rect(img, p[0], p[1] - 3, 1, 3, METAL_L)
    # fissure
    c = iso_pt(o, 1.4, 0.5, 12)
    line(img, (c[0] - 3, c[1] - 1), (c[0] + 2, c[1] + 1), INK)
    ball = iso_pt(o, 1.5, 1.6, 0)
    px(img, ball[0], ball[1] - 1, OFFWHITE)
    return finish(img, o)


def beanbag():
    img, o = canvas(1, 1, 20)
    c = iso_pt(o, 0.5, 0.5, 5)
    for yy in range(-6, 6):
        for xx in range(-8, 9):
            if (xx / 8) ** 2 + (yy / 6) ** 2 <= 1:
                col = ORANGE if yy > -2 else shade(ORANGE, 1.15)
                if xx > 3:
                    col = shade(col, 0.85)
                px(img, c[0] + xx, c[1] + yy, col)
    outline_alpha(img)
    return finish(img, o)


def arcade():
    img, o = canvas(1, 1, 50)
    iso_box(img, o, 0.15, 0.15, 0.7, 0.7, 0, 34, PURPLE)
    scr = [iso_pt(o, 0.25, 0.85, 20), iso_pt(o, 0.75, 0.85, 20), iso_pt(o, 0.75, 0.85, 30), iso_pt(o, 0.25, 0.85, 30)]
    poly(img, scr, INK, INK)
    s = iso_pt(o, 0.5, 0.85, 25)
    px(img, s[0] - 1, s[1], GREEN)
    px(img, s[0] + 1, s[1] - 1, GREEN)
    px(img, s[0], s[1] + 1, RED)
    j = iso_pt(o, 0.4, 0.85, 17)
    rect(img, j[0], j[1] - 2, 1, 2, INK)
    px(img, j[0], j[1] - 3, RED)
    m = iso_pt(o, 0.5, 0.85, 33)
    rect(img, m[0] - 3, m[1] - 2, 6, 2, YELLOW)
    return finish(img, o)


def bookshelf_item():
    img, o = canvas(1, 1, 46)
    iso_box(img, o, 0.1, 0.25, 0.8, 0.5, 0, 34, WOOD_D)
    for row, z in enumerate((6, 15, 24)):
        for i in range(6):
            p = iso_pt(o, 0.18 + i * 0.12, 0.75, z)
            col = [RED, BLUE, GREEN_DIM, YELLOW, OFFWHITE, PURPLE][(i + row * 2) % 6]
            rect(img, p[0], p[1] - 6, 1, 6, col)
    return finish(img, o)


def doormat():
    img, o = canvas(1, 1, 4)
    poly(img, [iso_pt(o, 0.15, 0.2), iso_pt(o, 0.85, 0.2), iso_pt(o, 0.85, 0.8), iso_pt(o, 0.15, 0.8)], (150, 110, 70, 255), WOOD_DD)
    return finish(img, o)


FURNITURE = {
    "desk": desk,
    "workstation": workstation,
    "plant": plant,
    "coffee_moka": coffee_moka,
    "coffee_machine": coffee_machine,
    "sofa": sofa,
    "whiteboard": whiteboard_item,
    "meeting_table": meeting_table,
    "server_rack": server_rack,
    "gpu_server": gpu_server,
    "k8s_cluster": k8s_cluster,
    "pingpong": pingpong,
    "beanbag": beanbag,
    "arcade": arcade,
    "bookshelf_item": bookshelf_item,
    "doormat": doormat,
}
