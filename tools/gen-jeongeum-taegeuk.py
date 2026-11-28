"""正音: the 태극 legend's 괘 bars (GWAE in src/ui/components/JeongeumLegend.tsx).

Draws the four 괘 as interlocking bars in a 235-unit square (the layout of the reference the user gave,
2026-11-28), cuts them with the clear ring round the 태극, grinds away what the ring leaves thin (a
morphological opening near the ring) and drops any piece or fragment smaller than 0.3 of a bar's width
squared. Prints the SVG path to paste as GWAE. Needs shapely (pip install shapely).

    python3 tools/gen-jeongeum-taegeuk.py            # the values in use
    python3 tools/gen-jeongeum-taegeuk.py 71 81 1.3  # T CLEAR grind-radius
"""
import sys
from shapely.geometry import Point, box
from shapely.ops import unary_union

S, W, G = 235.0, 23.5, 11.0  # square, bar width, gap between bars
DROP = 0.3


def bars():
    D = 3 * W + 2 * G
    Lb = S - G - D
    h = S / 2

    def split(a, b):
        m = (a + b) / 2
        return [(a, m - G / 2), (m + G / 2, b)]

    out = []
    for i in range(3):  # 건, along the top
        y0 = i * (W + G)
        out.append((0, y0, Lb, y0 + W))
    for i, broken in enumerate([1, 0, 1]):  # 감, down the right
        x0 = Lb + G + i * (W + G)
        for a, b in split(0, Lb) if broken else [(0, Lb)]:
            out.append((x0, a, x0 + W, b))
    for i in range(3):  # 곤, along the bottom
        y0 = S - D + i * (W + G)
        for a, b in split(D + G, S):
            out.append((a, y0, b, y0 + W))
    for i, broken in enumerate([0, 1, 0]):  # 리, up the left
        x0 = i * (W + G)
        for a, b in split(D + G, S) if broken else [(D + G, S)]:
            out.append((x0, a, x0 + W, b))
    return [(x0 - h, y0 - h, x1 - h, y1 - h) for x0, y0, x1, y1 in out]


def ground(clear, r):
    disc = Point(0, 0).buffer(clear, resolution=128)
    near = Point(0, 0).buffer(clear + 2.5 * r, resolution=128)
    out = []
    for x0, y0, x1, y1 in bars():
        b = box(x0, y0, x1, y1)
        if not b.intersects(disc):
            out.append(b)
            continue
        vis = b.difference(disc)
        if vis.is_empty:
            continue
        if r > 0:
            opened = vis.buffer(-r, resolution=32).buffer(r, resolution=32)
            vis = vis.difference(vis.difference(opened).intersection(near)).buffer(0)
        parts = [q for q in getattr(vis, "geoms", [vis]) if q.area >= DROP * W * W]
        if parts:
            out.append(unary_union(parts))
    return out


def path(geoms):
    f = lambda v: ("%.2f" % v).rstrip("0").rstrip(".")
    s = []
    for g in geoms:
        for poly in getattr(g, "geoms", [g]):
            for ring in [poly.exterior, *poly.interiors]:
                s.append("M" + "L".join(f"{f(x)} {f(y)}" for x, y in list(ring.coords)[:-1]) + "z")
    return "".join(s)


if __name__ == "__main__":
    T, CLEAR, R = (float(a) for a in sys.argv[1:4]) if len(sys.argv) > 3 else (71, 81, 1.3)
    print(path(ground(CLEAR, R)))
