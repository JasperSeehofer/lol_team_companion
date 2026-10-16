#!/usr/bin/env python3
"""Build surfaces/art/t2-contour.svg + t2.json: the T2 engraved contour field (recipe card T2).

No figurative Rift: a seeded height field on the 100x100 unit square, traced
with marching squares. Two pits on the river diagonal (top-left to
bottom-right) give two small nests of rings that merge across a saddle where
the mid diagonal crosses; the outer rings square off towards the edges, so the
perimeter (top and bot) and the saddle (mid) are implied by the contours alone.

Grades (swisstopo key): ordinary contours `rule` 1 px; every fourth an index
contour (ink at 40 %) carrying a `figures` 12 px label set along the line, the
line broken 4 px either side of the figure; one intermediate (dashed) contour
only between the two innermost rings. The index contour is written in three
variants because the label has a fixed screen size and the break is geometry:
  v-w  hero at >= 1024 px (560 px square) and the share strip (crop at the same 5.6 px/unit)
  v-n  hero below 1024 px (358 px wide field, 3.58 px/unit)
  v-0  unbroken, unlabelled (Match Day inset)
Run from the repo root:  python3 -I .planning/missions/M151/gallery/tools/build-t2-contour.py
"""
import json, math, pathlib, random

ART = pathlib.Path(__file__).resolve().parent.parent / "surfaces" / "art"
SVG_OUT, JSON_OUT = ART / "t2-contour.svg", ART / "t2.json"
f = lambda v: f"{v:.2f}".rstrip("0").rstrip(".")

# ---------------------------------------------------------------- height field
rnd = random.Random(151)
WAVES = [(rnd.uniform(0.04, 0.11), rnd.uniform(0, math.tau), rnd.uniform(0, math.tau), rnd.uniform(0.25, 0.5))
         for _ in range(5)]                        # (frequency, direction, phase, amplitude)
D, A, C = 17.0, 1.5, 1.0                           # pits at +-D along the river axis; ellipse axes
N_SQ, K_SQ, P_SQ = 6.0, 40.0, 3.0                  # outer squaring: superellipse power, lane radius, steepness

def smin(a, b, k):
    h = max(k - abs(a - b), 0.0) / k
    return min(a, b) - h * h * k * 0.25

def smax(a, b, k):
    return -smin(-a, -b, k)

def height(x, y):
    a = ((x - 50) + (y - 50)) / math.sqrt(2)       # along the river (top-left negative)
    c = ((x - 50) - (y - 50)) / math.sqrt(2)       # across it
    e1 = math.hypot((a + D) / A, c / C)
    e2 = math.hypot((a - D) / A, c / C)
    b = smin(e1, e2, 7.0)
    s = (abs(x - 50) ** N_SQ + abs(y - 50) ** N_SQ) ** (1 / N_SQ)
    sq = K_SQ * (s / K_SQ) ** P_SQ
    h = smax(b, sq, 5.0)
    for fr, th, ph, am in WAVES:
        h += am * math.sin(fr * (x * math.cos(th) + y * math.sin(th)) + ph)
    return h

STEP = 0.4
NG = int(100 / STEP) + 1
GRID = [[height(i * STEP, j * STEP) for i in range(NG)] for j in range(NG)]  # GRID[j][i]

# -------------------------------------------------------------- marching squares
def contour(level):
    """Closed/open polylines of GRID == level, joined through shared grid edges."""
    def pt(edge):
        kind, i, j = edge
        if kind == "h":   # between (i,j) and (i+1,j)
            v0, v1 = GRID[j][i], GRID[j][i + 1]
            t = (level - v0) / (v1 - v0)
            return ((i + t) * STEP, j * STEP)
        v0, v1 = GRID[j][i], GRID[j + 1][i]
        t = (level - v0) / (v1 - v0)
        return (i * STEP, (j + t) * STEP)
    adj = {}
    def link(e0, e1):
        adj.setdefault(e0, []).append(e1)
        adj.setdefault(e1, []).append(e0)
    for j in range(NG - 1):
        for i in range(NG - 1):
            tl, tr, br, bl = GRID[j][i], GRID[j][i + 1], GRID[j + 1][i + 1], GRID[j + 1][i]
            idx = (tl > level) << 3 | (tr > level) << 2 | (br > level) << 1 | (bl > level)
            if idx in (0, 15):
                continue
            T, R, B, L = ("h", i, j), ("v", i + 1, j), ("h", i, j + 1), ("v", i, j)
            centre = (tl + tr + br + bl) / 4 > level
            table = {1: [(L, B)], 2: [(B, R)], 3: [(L, R)], 4: [(T, R)], 6: [(T, B)], 7: [(L, T)],
                     8: [(L, T)], 9: [(T, B)], 11: [(T, R)], 12: [(L, R)], 13: [(B, R)], 14: [(L, B)],
                     5: [(L, T), (B, R)] if not centre else [(L, B), (T, R)],
                     10: [(T, R), (L, B)] if not centre else [(L, T), (B, R)]}
            for e0, e1 in table[idx]:
                link(e0, e1)
    seen, lines = set(), []
    for start in adj:
        if start in seen:
            continue
        # walk from an end if open, else anywhere
        cur = start
        path = [cur]
        seen.add(cur)
        prev = None
        while True:
            nxt = [n for n in adj[cur] if n != prev and n not in seen]
            if not nxt:
                closed = start in adj[cur] and len(path) > 2
                break
            prev, cur = cur, nxt[0]
            seen.add(cur)
            path.append(cur)
        lines.append(([pt(e) for e in path], closed))
    return lines

def rdp(pts, eps):
    if len(pts) < 3:
        return pts
    (x0, y0), (x1, y1) = pts[0], pts[-1]
    dx, dy = x1 - x0, y1 - y0
    L = math.hypot(dx, dy) or 1e-9
    dmax, imax = 0.0, 0
    for k in range(1, len(pts) - 1):
        d = abs(dy * (pts[k][0] - x0) - dx * (pts[k][1] - y0)) / L
        if d > dmax:
            dmax, imax = d, k
    if dmax <= eps:
        return [pts[0], pts[-1]]
    return rdp(pts[: imax + 1], eps)[:-1] + rdp(pts[imax:], eps)

def d_path(pts, closed):
    s = "M" + " L".join(f"{f(x)},{f(y)}" for x, y in pts)
    return s + (" Z" if closed else "")

# ------------------------------------------------------------------ the levels
L0, STEP_L = 4.5, 5.5
LEVELS = [L0 + STEP_L * k for k in range(7)]       # rings 1..7; ring 4 is the index contour
INDEX = 3                                          # 0-based: every fourth
INTERMEDIATE = (LEVELS[0] + LEVELS[1]) / 2         # dashed, only between the two innermost
FIG = {k: 300 + 25 * (k + 1) for k in range(len(LEVELS))}   # metres: 325 .. 475, index = 400
SCALE = {"v-w": 5.6, "v-n": 3.58}                  # px per unit where each variant is shown
CHAR_PX, PAD_PX = 0.6 * 12, 4.0                    # JetBrains Mono advance at 12 px; break each side
# label anchors (approximate; snapped to the index line). >= 200 px apart.
LABELS = {"v-w": [(40.0, 17.0), (86.0, 60.0)], "v-n": [(76.0, 34.0)]}

def cyc_pts(pts):
    """Closed polyline -> list with cumulative arclength."""
    ring = pts + [pts[0]]
    acc = [0.0]
    for a, b in zip(ring, ring[1:]):
        acc.append(acc[-1] + math.hypot(b[0] - a[0], b[1] - a[1]))
    return ring, acc

def at(ring, acc, s):
    total = acc[-1]
    s %= total
    for k in range(len(acc) - 1):
        if acc[k] <= s <= acc[k + 1]:
            t = (s - acc[k]) / ((acc[k + 1] - acc[k]) or 1e-9)
            (x0, y0), (x1, y1) = ring[k], ring[k + 1]
            return (x0 + (x1 - x0) * t, y0 + (y1 - y0) * t)
    return ring[-1]

def piece(ring, acc, s0, s1):
    """Points of the closed ring from arclength s0 forward to s1 (wrapping)."""
    total = acc[-1]
    if s1 < s0:
        s1 += total
    out = [at(ring, acc, s0)]
    for lap in (0, total):
        for k, s in enumerate(acc[:-1]):
            if s0 < s + lap < s1:
                out.append(ring[k])
    out.append(at(ring, acc, s1))
    return out

def main():
    ordinary, index_lines, dashed = [], [], []
    for k, lv in enumerate(LEVELS):
        for pts, closed in contour(lv):
            if len(pts) < 8:
                continue
            (index_lines if k == INDEX else ordinary).append((pts, closed))
    for pts, closed in contour(INTERMEDIATE):
        if len(pts) >= 8:
            dashed.append((pts, closed))
    assert all(c for _, c in ordinary + index_lines + dashed), "a contour runs off the field"
    assert len(index_lines) == 1, f"index contour must be one ring, got {len(index_lines)}"

    idx_pts = index_lines[0][0]
    ring, acc = cyc_pts(idx_pts)
    text = str(FIG[INDEX])
    variants, labels_out = {}, []
    for v, anchors in LABELS.items():
        scale = SCALE[v]
        half = (len(text) * CHAR_PX / 2 + PAD_PX) / scale
        gaps = []
        for ax, ay in anchors:
            k = min(range(len(ring) - 1), key=lambda n: math.hypot(ring[n][0] - ax, ring[n][1] - ay))
            s = acc[k]
            p0, p1 = at(ring, acc, s - half), at(ring, acc, s + half)
            cx, cy = at(ring, acc, s)
            ang = math.degrees(math.atan2(p1[1] - p0[1], p1[0] - p0[0]))
            if ang > 90: ang -= 180
            if ang <= -90: ang += 180
            g0 = (s - half) % acc[-1]
            gaps.append((g0, g0 + 2 * half))
            labels_out.append((v, cx, cy, ang, 1 / scale))
        gaps.sort()
        pieces = []
        for n, (g0, g1) in enumerate(gaps):
            nxt = gaps[(n + 1) % len(gaps)][0] + (acc[-1] if n + 1 == len(gaps) else 0)
            pieces.append(rdp(piece(ring, acc, g1, nxt), 0.03))
        variants[v] = pieces

    out = ['<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" class="contour contour-t2" aria-hidden="true" focusable="false">',
           "<!-- GENERATED by tools/build-t2-contour.py (recipe card T2). Do not edit by hand. -->",
           '<rect class="ground" x="0" y="0" width="100" height="100"/>',
           '<g class="ct ord">' + "".join(f'<path d="{d_path(rdp(p, 0.03), True)}"/>' for p, _ in ordinary) + "</g>",
           '<g class="ct mid">' + "".join(f'<path d="{d_path(rdp(p, 0.03), True)}"/>' for p, _ in dashed) + "</g>",
           f'<g class="ct idx v-0"><path d="{d_path(rdp(idx_pts, 0.03), True)}"/></g>']
    for v, pieces in variants.items():
        out.append(f'<g class="ct idx {v}">' + "".join(f'<path d="{d_path(p, False)}"/>' for p in pieces) + "</g>")
    for v, x, y, ang, k in labels_out:
        out.append(f'<text class="idx-lab {v}" transform="translate({f(x)} {f(y)}) rotate({f(ang)}) scale({k:.4f})" '
                   f'text-anchor="middle" dominant-baseline="central">{text}</text>')
    # frames: the 358x240 field below 1024 px (y 16.48..83.52), the full square above
    out.append('<g class="frame"><path class="v-n" d="M0,16.48 H100 V83.52 H0 Z"/><path class="v-w" d="M0,0 H100 V100 H0 Z"/></g>')
    out.append("</svg>")
    SVG_OUT.write_text("\n".join(out) + "\n")

    cfg = {
        "name": "T2 engraved contour",
        "svg": SVG_OUT.name,
        "css": "t2.css",
        "label": {"x": 7.0, "y": 79.0},
        "marks": [
            {"name": "baron", "word": "Baron", "x": 33.0, "y": 33.0},
            {"name": "herald", "word": "Herald", "x": 24.0, "y": 44.0},
            {"name": "dragon", "word": "Dragon", "x": 67.0, "y": 67.0},
            {"name": "mid", "word": "Mid", "x": 50.0, "y": 50.0},
            {"name": "top", "word": "Top", "x": 6.0, "y": 9.0},
            {"name": "jungle", "word": "Jungle", "x": 26.0, "y": 64.0},
            {"name": "tower", "word": "Tower", "x": 70.0, "y": 22.0},
            {"name": "bot", "word": "Bot", "x": 80.0, "y": 93.0},
            {"name": "support", "word": "Support", "x": 70.0, "y": 85.0},
        ],
        "crops": {"share": "18 12 63.93 17.14", "inset": "26 26 48 48"},
        "shareLabel": {"x": 57.0, "y": 25.0},
    }
    JSON_OUT.write_text(json.dumps(cfg, indent=2) + "\n")
    print(f"wrote {SVG_OUT.name} ({SVG_OUT.stat().st_size} B), {JSON_OUT.name}; "
          f"rings {len(ordinary)} ordinary + 1 index + {len(dashed)} intermediate")

if __name__ == "__main__":
    main()
