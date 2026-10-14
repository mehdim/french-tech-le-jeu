"""Compose an office scene from the generated pieces (art iteration aid)."""
import sys
from pixlib import *
import gen_tiles as T
import gen_furniture as F

def scene(tier, w, h, floor, items, chars=()):
    W = (w + h) * HALF_W + 80
    H = (w + h) * HALF_H + T.WALL_H + 60
    img = Image.new("RGBA", (W, H), GRAPHITE)
    o = (h * HALF_W + 40, T.WALL_H + 30)
    for y in range(h):
        for x in range(w):
            t = T.floor_tile(T.FLOORS[floor], (x * 7 + y * 3) % 2)
            p = iso_pt(o, x, y)
            img.alpha_composite(t, (p[0] - HALF_W, p[1]))
    for y in range(h):
        seg = T.wall_segment(tier, "L", "window" if y % 3 == 1 else "plain")
        p = iso_pt(o, 0, y)
        img.alpha_composite(seg, (p[0] - 16, p[1] - (seg.height - 9)))
    for x in range(w):
        seg = T.wall_segment(tier, "R", "window" if x % 3 == 1 else "plain")
        p = iso_pt(o, x + 1, 0)
        img.alpha_composite(seg, (p[0] - 16, p[1] - (seg.height - 9) - 8))
    order = sorted(items, key=lambda it: it[1] + it[2])
    for name, x, y in order:
        spr, (ax, ay) = F.FURNITURE[name]()
        p = iso_pt(o, x, y)
        img.alpha_composite(spr, (p[0] - ax, p[1] - ay))
    return img

if __name__ == "__main__":
    tier = int(sys.argv[1]) if len(sys.argv) > 1 else 0
    floors = ["Parquet", "Concrete", "Tiles", "Wood", "Concrete", "Marble"]
    items = [("desk", 1, 1), ("workstation", 4, 1), ("sofa", 0, 4), ("coffee_machine", 6, 0), ("plant", 6, 3),
             ("server_rack", 0, 0), ("gpu_server", 3, 0), ("whiteboard", 1, 3), ("meeting_table", 3, 3),
             ("k8s_cluster", 5, 5), ("pingpong", 7, 2), ("beanbag", 2, 6), ("arcade", 8, 0), ("bookshelf_item", 0, 6), ("doormat", 4, 7)]
    img = scene(tier, 9, 8, floors[tier], items)
    img = img.resize((img.width * 3, img.height * 3), Image.NEAREST)
    img.save(f"/tmp/claude-0/art/preview_{tier}.png")
    print(img.size)
