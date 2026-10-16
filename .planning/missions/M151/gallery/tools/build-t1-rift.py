#!/usr/bin/env python3
"""Build surfaces/art/t1-rift.svg: the T1 field-sketch Rift (recipe card T1).

Pure geometry on a 100x100 unit square, never traced. One ink, stroke only;
the chalk leader is the only accent. Irregularity is deterministic (seeded),
small (<= 0.45 units) and in the geometry, never a filter.
Run from the repo root:  python3 -I .planning/missions/M151/gallery/tools/build-t1-rift.py
"""
import math, random, pathlib

OUT = pathlib.Path(__file__).resolve().parent.parent / "surfaces" / "art" / "t1-rift.svg"
rnd = random.Random(151)
f = lambda v: f"{v:.2f}".rstrip("0").rstrip(".")
def j(a=0.3): return rnd.uniform(-a, a)

BLUE = (12.0, 88.0)          # base circle r3 (blue side shape)
RED = (85.0, 9.0, 6.0)       # base square: top-left corner, side 6 (centre 88,12)
LABEL_CLEAR = [(20.5, 80.5, 46.0, 87.6)]          # "YOUR TEAM" clearing (x0,y0,x1,y1)
MARK_CLEAR = [(29.5, 19.6, 42.5, 25.4), (55.5, 74.6, 69.5, 80.4)]  # Baron, Dragon (1280)

def seg_path(pts):
    return "M" + " L".join(f"{f(x)},{f(y)}" for x, y in pts)

def wobbly(p0, p1, bow):
    """A straight feature drawn by hand: one quadratic with a small bow."""
    mx, my = (p0[0] + p1[0]) / 2, (p0[1] + p1[1]) / 2
    dx, dy = p1[0] - p0[0], p1[1] - p0[1]
    L = math.hypot(dx, dy) or 1
    nx, ny = -dy / L, dx / L
    return f"M{f(p0[0])},{f(p0[1])} Q{f(mx + nx * bow)},{f(my + ny * bow)} {f(p1[0])},{f(p1[1])}"

ink = []
# Frame: four strokes stopping 1.5 short of each corner (open corners).
ink += [wobbly((3.5, 2), (96.5, 2.1), 0.15), wobbly((98, 3.5), (97.9, 96.5), -0.2),
        wobbly((96.5, 98), (3.5, 97.9), 0.15), wobbly((2, 96.5), (2.1, 3.5), -0.2)]
# Lanes. Top: up the left edge, along the top; bot mirrored; mid on the diagonal.
ink.append(f"M10.1,84.3 Q9.7,47 10,10.2 Q47,9.7 83.5,10.1")
ink.append(f"M15.7,90 Q53,90.35 89.9,89.8 Q90.3,53 90,16.5")
ink.append(wobbly((15.2, 84.8), (83.5, 16.5), 0.35))
# River: two lines 3 apart along y=x, broken 4 units where mid crosses.
o = 1.5 / math.sqrt(2)              # 1.5 units across = 1.06 per axis
for s in (+1, -1):
    d = -2 * s * o                     # line y = x + d
    cx = 50 + s * o                    # where it meets mid (x + y = 100)
    g = 2 / math.sqrt(2)               # 2 units either side along the line
    for t0, t1 in ((15.0, cx - g), (cx + g, 85.0)):
        ink.append(wobbly((t0, t0 + d), (t1, t1 + d), s * 0.25 + j(0.1)))
# Bases (side shapes, in ink).
ink.append(f"M{f(BLUE[0]-3)},{f(BLUE[1])} a3,3 0 1 0 6,0 a3,3 0 1 0 -6,0")
x, y, w = RED
ink.append(f"M{x},{y} H{x+w} V{y+w} H{x} Z")

# Jungle hatching: one 45-degree family (parallel to mid), spacing 4 units,
# each hatch anchored at the lane edge and ending >= 2 units before a lane,
# the river (lines at +-1.5) or mid; tone fades by lines ending early.
QUADS = {  # triangle, outer-edge test (distance from the lane)
    "left":   [(10, 10), (10, 90), (50, 50)],
    "top":    [(10, 10), (90, 10), (50, 50)],
    "right":  [(90, 10), (90, 90), (50, 50)],
    "bottom": [(10, 90), (90, 90), (50, 50)],
}
OUTER = {"left": lambda p: p[0] - 10, "top": lambda p: p[1] - 10,
         "right": lambda p: 90 - p[0], "bottom": lambda p: 90 - p[1]}

def halfplanes(name):
    """Inset constraints as (a,b,c): a*x+b*y+c >= 0."""
    r = 1 / math.sqrt(2)
    river = 2 + 1.5      # river centre line y=x, lines at 1.5, margin 2
    mid = 2 + 0.0
    hp = []
    if name == "left":   hp = [(1, 0, -12), (-r, r, -river), (-r, -r, 100 * r - mid)]
    if name == "top":    hp = [(0, 1, -12), (r, -r, -river), (-r, -r, 100 * r - mid)]
    if name == "right":  hp = [(-1, 0, 88), (r, -r, -river), (r, r, -100 * r - mid)]
    if name == "bottom": hp = [(0, -1, 88), (-r, r, -river), (r, r, -100 * r - mid)]
    return hp

def clip_line(c, hps):
    """Line x+y=c param x=t, y=c-t; return [t0,t1] inside all half-planes."""
    lo, hi = -1e9, 1e9
    for a, b, k in hps:
        # a*t + b*(c-t) + k >= 0  ->  (a-b) t >= -(b*c + k)
        coef, rhs = a - b, -(b * c + k)
        if abs(coef) < 1e-9:
            if rhs > 0: return None
            continue
        if coef > 0: lo = max(lo, rhs / coef)
        else: hi = min(hi, rhs / coef)
    return (lo, hi) if hi - lo > 0.8 else None

def blocked(px, py):
    if math.hypot(px - BLUE[0], py - BLUE[1]) < 3 + 2.5: return True
    if RED[0] - 2.5 < px < RED[0] + RED[2] + 2.5 and RED[1] - 2.5 < py < RED[1] + RED[2] + 2.5: return True
    for x0, y0, x1, y1 in LABEL_CLEAR + MARK_CLEAR:
        if x0 < px < x1 and y0 < py < y1: return True
    return False

hatch = []
step = 4 * math.sqrt(2)
for name in QUADS:
    hps = halfplanes(name)
    cs = []
    c = 20.0 + (step / 2 if name in ("top", "bottom") else 0)
    while c < 182:
        seg = clip_line(c, hps)
        if seg: cs.append((c, seg))
        c += step
    n = len(cs)
    for i, (c, (t0, t1)) in enumerate(cs):
        p0, p1 = (t0, c - t0), (t1, c - t1)
        if OUTER[name](p1) < OUTER[name](p0): p0, p1 = p1, p0   # p0 = lane end
        L = math.hypot(p1[0] - p0[0], p1[1] - p0[1])
        u = i / max(1, n - 1)
        # Longest hatches mid-quadrant, shorter towards its ends: the jungle
        # reads as a mass with open verges along every road and the river.
        frac = 0.22 + 0.62 * math.sin(math.pi * u) ** 0.8 + j(0.07)
        frac = max(0.15, min(1.0, frac))
        keep = L * frac
        start = (L - keep) * (0.5 + j(0.12))       # centred, a little off
        ux0, uy0 = (p1[0] - p0[0]) / L, (p1[1] - p0[1]) / L
        p0 = (p0[0] + ux0 * start, p0[1] + uy0 * start)
        p1 = (p0[0] + ux0 * keep, p0[1] + uy0 * keep)
        L = keep
        ux, uy = (p1[0] - p0[0]) / L, (p1[1] - p0[1]) / L
        # walk the kept part, splitting around clearings
        pts, run = [], []
        k, dt = 0.0, 0.25
        while k <= keep:
            px, py = p0[0] + ux * k, p0[1] + uy * k
            if blocked(px, py):
                if len(run) > 1: pts.append(run)
                run = []
            else:
                run.append((px, py))
            k += dt
        if len(run) > 1: pts.append(run)
        for r in pts:
            a, b = r[0], r[-1]
            if math.hypot(b[0] - a[0], b[1] - a[1]) < 1.2: continue
            sa, sb = j(0.25), j(0.35)
            hatch.append(f"M{f(a[0]+ux*sa)},{f(a[1]+uy*sa)} L{f(b[0]+ux*sb+j(0.12))},{f(b[1]+uy*sb+j(0.12))}")

# Edge ticks: A..H along the top (8 columns), 1..5 down the left (5 rows), 1280 only.
ticks = []
for i in range(1, 8):
    xx = 2 + 96 * i / 8
    ticks.append(f"M{f(xx)},2 V0.4")
for i in range(1, 5):
    yy = 2 + 96 * i / 5
    ticks.append(f"M2,{f(yy)} H0.4")

# Chalk leader for the surveyed team label: from the blue base edge to the label.
leader = "M15.2,86.6 L22.3,84"

svg = f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" class="rift rift-t1" aria-hidden="true" focusable="false">
<!-- GENERATED by tools/build-t1-rift.py (recipe card T1). Do not edit by hand. -->
<g class="ticks">{"".join(f'<path d="{d}"/>' for d in ticks)}</g>
<g class="ink">{"".join(f'<path d="{d}"/>' for d in ink)}</g>
<g class="ink hatch">{"".join(f'<path d="{d}"/>' for d in hatch)}</g>
<g class="chalk" data-for="team-label"><path d="{leader}"/><path d="M15.2,86.6 h0"/></g>
</svg>
'''
OUT.write_text(svg)
print(f"{OUT} {len(ink)} ink paths, {len(hatch)} hatches, {len(svg)} bytes")
