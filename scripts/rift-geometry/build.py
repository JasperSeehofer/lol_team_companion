#!/usr/bin/env python3
"""Build the Rift schematic from the authored coordinate file (M152).

    python3 -I scripts/rift-geometry/build.py           # write both outputs
    python3 -I scripts/rift-geometry/build.py --check   # fail if an output is stale

Input:  scripts/rift-geometry/rift-source.json (hand-authored numbers; see
        docs/rift-schematic.md for how each was measured).
Output: assets/rift/map11-geometry.json (game units, every feature with its
        source) and src/components/ui/rift_geometry.rs (generated Rust:
        SVG path data for the two levels of detail).

Standard library only. This script reads no image: it never imports an
image library and never opens a picture of the map. Everything it draws is
arithmetic on the authored numbers (recipe card A/B, recipes-r1.md).

Drawing frame (recipe cards): 150 game units = 1 drawing unit,
x_d = x / 150, y_d = 100 - y / 150 (game y points north).
"""
import json
import math
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
SRC = ROOT / "scripts" / "rift-geometry" / "rift-source.json"
OUT_JSON = ROOT / "assets" / "rift" / "map11-geometry.json"
OUT_RS = ROOT / "src" / "components" / "ui" / "rift_geometry.rs"

SOURCE_KINDS = ("fixture:", "measured:", "reported", "approximate")

# ---- Recipe constants (recipes-r1.md card A; critique-r1-rift.md fixes 1-6) --
# The sketch level draws no wall hatch (critique-r1-surfaces.md ruling 4): with
# no wall outline at this level the hatch fragments had no visible mass and read
# as arbitrary bars. Hatch is survey-only (rift.rs draws it from SURVEY_WALLS).
BANK = 1.5                # sketch river banks either side of the centreline
BRIDGE = 2.0              # banks break this close to mid
INSET_BRIDGE = 5.0        # Match Day inset: centreline breaks this close to mid
BASE_R = 3.0              # blue base circle radius
BASE_SQ = 6.0             # red base square side
BASE_GAP = 2.5            # lanes stop this far from a base shape
MID_GAP = 3.0             # mid stops this far from a base shape
LANE_SNAP = 1.5           # lanes snap to an axis within this deviation
MARK = 2.9                # Baron/Dragon diamond size (sketch)
LEADER = 7.0              # chalk leader length (card D)
R1A = [((14, 14), (24, 32), (34, 44), (50, 50)), ((50, 50), (66, 56), (76, 68), (86, 86))]


def d(x, y):
    """Game units to drawing units."""
    return (x / 150.0, 100.0 - y / 150.0)


def fmt(v):
    s = f"{v:.2f}".rstrip("0").rstrip(".")
    return "0" if s in ("-0", "") else s


def path_poly(pts, closed=False, f=fmt):
    s = "M" + " ".join(f"{f(x)},{f(y)}" for x, y in pts)
    return s + ("Z" if closed else "")


# ---- Geometry helpers -------------------------------------------------------
def area(ring):
    a = 0.0
    for (x0, y0), (x1, y1) in zip(ring, ring[1:] + ring[:1]):
        a += x0 * y1 - x1 * y0
    return a / 2.0


def centroid(ring):
    a = area(ring)
    cx = cy = 0.0
    for (x0, y0), (x1, y1) in zip(ring, ring[1:] + ring[:1]):
        c = x0 * y1 - x1 * y0
        cx += (x0 + x1) * c
        cy += (y0 + y1) * c
    return (cx / (6 * a), cy / (6 * a))


def inside(pt, ring):
    x, y = pt
    c = False
    for (x0, y0), (x1, y1) in zip(ring, ring[1:] + ring[:1]):
        if (y0 > y) != (y1 > y):
            if x < x0 + (y - y0) * (x1 - x0) / (y1 - y0):
                c = not c
    return c


def seg_dist(p, a, b):
    ax, ay = a
    bx, by = b
    px, py = p
    dx, dy = bx - ax, by - ay
    L = dx * dx + dy * dy
    t = 0.0 if L == 0 else max(0.0, min(1.0, ((px - ax) * dx + (py - ay) * dy) / L))
    return math.hypot(px - (ax + t * dx), py - (ay + t * dy))


def poly_dist(p, line):
    return min(seg_dist(p, a, b) for a, b in zip(line, line[1:]))


def segs_cross(a, b, c, e):
    def orient(p, q, r):
        return (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0])
    d1, d2 = orient(c, e, a), orient(c, e, b)
    d3, d4 = orient(a, b, c), orient(a, b, e)
    return (d1 > 0) != (d2 > 0) and (d3 > 0) != (d4 > 0)


def rings_overlap(r1, r2):
    if any(inside(p, r2) for p in r1) or any(inside(p, r1) for p in r2):
        return True
    for a, b in zip(r1, r1[1:] + r1[:1]):
        for c, e in zip(r2, r2[1:] + r2[:1]):
            if segs_cross(a, b, c, e):
                return True
    return False


def bez(c, t):
    (x0, y0), (x1, y1), (x2, y2), (x3, y3) = c
    u = 1 - t
    x = u**3 * x0 + 3 * u * u * t * x1 + 3 * u * t * t * x2 + t**3 * x3
    y = u**3 * y0 + 3 * u * u * t * y1 + 3 * u * t * t * y2 + t**3 * y3
    dx = 3 * u * u * (x1 - x0) + 6 * u * t * (x2 - x1) + 3 * t * t * (x3 - x2)
    dy = 3 * u * u * (y1 - y0) + 6 * u * t * (y2 - y1) + 3 * t * t * (y3 - y2)
    return x, y, dx, dy


# ---- Load and validate ------------------------------------------------------
def load():
    src = json.loads(SRC.read_text())
    feats = []
    for key in ("walls", "lanes", "pits", "objectives", "camps", "nexuses",
                "inhibitors", "towers", "bushes"):
        feats += [(key, f) for f in src[key]]
    feats += [("boundary", src["boundary"]), ("river", src["river"])]
    ids = set()
    for kind, f in feats:
        fid = f.get("id")
        assert fid and fid not in ids, f"duplicate or missing id: {fid}"
        ids.add(fid)
        s = f.get("source", "")
        assert any(s.startswith(k) for k in SOURCE_KINDS), f"{fid}: bad source {s!r}"
        for k in ("ring", "line"):
            if k in f:
                for p in f[k]:
                    assert len(p) == 2 and all(isinstance(v, int) for v in p), f"{fid}: {p}"
                    assert -10 <= p[0] <= 15010 and -10 <= p[1] <= 15010, f"{fid}: {p} out of range"
    expect = {"towers": 22, "inhibitors": 6, "nexuses": 2, "pits": 2}
    for k, n in expect.items():
        assert len(src[k]) == n, f"{k}: {len(src[k])} != {n}"
    return src


def check(src):
    """Consistency checks between the authored layers. Returns notes."""
    walls = [f["ring"] for f in src["walls"]]
    notes = []
    for key in ("towers", "inhibitors", "nexuses", "camps"):
        for f in src[key]:
            for w, wf in zip(walls, src["walls"]):
                assert not inside(f["at"], w), f"{f['id']} lies inside {wf['id']}"
    for b in src["bushes"]:
        for w, wf in zip(walls, src["walls"]):
            assert not rings_overlap(b["ring"], w), f"{b['id']} overlaps {wf['id']}"
        assert not rings_overlap(b["ring"], src["river"]["ring"]), f"{b['id']} overlaps the river"
    for i, a in enumerate(src["walls"]):
        for b in src["walls"][i + 1:]:
            assert not rings_overlap(a["ring"], b["ring"]), f"{a['id']} overlaps {b['id']}"
    lanes = {f["id"].split("-")[1]: f["line"] for f in src["lanes"]}
    for t in src["towers"]:
        if t["lane"] in lanes:
            dist = poly_dist(t["at"], lanes[t["lane"]])
            notes.append((t["id"], dist))
    worst = max(notes, key=lambda n: n[1])
    assert worst[1] < 900, f"tower far off its lane: {worst}"
    fx = next(o for o in src["objectives"] if o["id"] == "obj-dragon")["at"]
    pit = next(p for p in src["pits"] if p["id"] == "pit-dragon")
    assert math.dist(fx, pit["centre"]) < pit["r"], "fixture dragon outside the measured pit"
    return {"tower_lane_max": round(worst[1]), "tower_lane_worst": worst[0]}


# ---- Sketch (card A) --------------------------------------------------------
def river_centreline(src):
    """Measured centreline: the middle of the two banks, pits left out."""
    ring = [d(*p) for p in src["river"]["ring"]]
    pits = [(d(*p["centre"]), p["r"] / 150.0) for p in src["pits"]]

    def keep(p):
        return all(math.dist(p, c) > r + 0.6 for c, r in pits)

    n = len(ring)
    # The south-west bank runs from vertex 0 to the bottom mouth (the vertex
    # farthest down and right on the sheet); the north-east bank comes back.
    turn = max(range(n), key=lambda i: ring[i][0] + ring[i][1])
    sw = [p for p in ring[: turn + 1] if keep(p)]
    ne = [p for p in ring[turn:] if keep(p)][::-1]

    def cut(line, s):
        """Where the line x + y = s crosses a bank: the cross coordinate x - y."""
        for (x0, y0), (x1, y1) in zip(line, line[1:]):
            s0, s1 = x0 + y0, x1 + y1
            if (s0 - s) * (s1 - s) <= 0 and s0 != s1:
                t = (s - s0) / (s1 - s0)
                return (x0 - y0) + t * ((x1 - y1) - (x0 - y0))
        return None

    out = []
    s = 28.0
    while s <= 172.0:
        a, b = cut(sw, s), cut(ne, s)
        if a is not None and b is not None:
            c = (a + b) / 2
            out.append(((s + c) / 2, (s - c) / 2))
        s += 1.0
    return out


def fit_cubic(pts, p0, p3):
    """Least-squares inner control points for a cubic through p0 and p3."""
    cum = [0.0]
    for a, b in zip(pts, pts[1:]):
        cum.append(cum[-1] + math.dist(a, b))
    ts = [c / cum[-1] for c in cum]
    a11 = a12 = a22 = 0.0
    bx1 = by1 = bx2 = by2 = 0.0
    for (x, y), t in zip(pts, ts):
        u = 1 - t
        b0, b1, b2, b3 = u**3, 3 * u * u * t, 3 * u * t * t, t**3
        rx = x - b0 * p0[0] - b3 * p3[0]
        ry = y - b0 * p0[1] - b3 * p3[1]
        a11 += b1 * b1
        a12 += b1 * b2
        a22 += b2 * b2
        bx1 += b1 * rx
        by1 += b1 * ry
        bx2 += b2 * rx
        by2 += b2 * ry
    det = a11 * a22 - a12 * a12
    p1 = ((a22 * bx1 - a12 * bx2) / det, (a22 * by1 - a12 * by2) / det)
    p2 = ((a11 * bx2 - a12 * bx1) / det, (a11 * by2 - a12 * by1) / det)
    return (p0, p1, p2, p3)


def curve_dev(cubics, line):
    samples = []
    for c in cubics:
        samples += [bez(c, i / 200)[:2] for i in range(201)]
    return max(min(math.dist(p, s) for s in samples) for p in line)


def river_cubics(src):
    line = river_centreline(src)
    # Only the stretch the sketch draws (the cubics' extent) is compared.
    clipped = [p for p in line if 14 <= p[0] <= 86]
    dev = curve_dev(R1A, clipped)
    if dev <= 1.5:
        return R1A, dev, None
    # The join sits where the river meets mid (x + y = 100 on the sheet).
    mid = min(range(len(line)), key=lambda i: abs(line[i][0] + line[i][1] - 100))
    centre = line[mid]
    first = [p for p in line[: mid + 1] if p[0] >= 14]
    second = [p for p in line[mid:] if p[0] <= 86]
    p0, p3 = first[0], second[-1]
    c1 = fit_cubic(first, p0, centre)
    c2 = fit_cubic(second, centre, p3)
    # Smooth join (one tangent through the centre), then a deterministic
    # pattern search on the free handles to bring the worst deviation down.
    vx, vy = c2[1][0] - c1[2][0], c2[1][1] - c1[2][1]
    ang = math.atan2(vy, vx)
    params = [c1[1][0], c1[1][1], c2[2][0], c2[2][1], ang,
              math.dist(c1[2], centre), math.dist(c2[1], centre)]

    def make(q):
        a1x, a1y, b2x, b2y, an, t1, t2 = q
        ux, uy = math.cos(an), math.sin(an)
        return [(p0, (a1x, a1y), (centre[0] - ux * t1, centre[1] - uy * t1), centre),
                (centre, (centre[0] + ux * t2, centre[1] + uy * t2), (b2x, b2y), p3)]

    def score(q):
        cs = make(q)
        samples = []
        for c in cs:
            samples += [bez(c, i / 60)[:2] for i in range(61)]
        return max(min(math.dist(p, s) for s in samples) for p in clipped)

    best = score(params)
    steps = [2.0, 2.0, 2.0, 2.0, 0.2, 2.0, 2.0]
    for _ in range(40):
        improved = False
        for i in range(len(params)):
            for sgn in (1, -1):
                q = params[:]
                q[i] += sgn * steps[i]
                sc = score(q)
                if sc < best - 1e-6:
                    params, best, improved = q, sc, True
        if not improved:
            steps = [st / 2 for st in steps]
    cub = [tuple(tuple(round(v, 1) for v in p) for p in c) for c in make(params)]
    return cub, dev, curve_dev(cub, clipped)


def snap_line(line):
    """Snap near-axis segments (deviation <= LANE_SNAP) to the axis."""
    out = [list(p) for p in line]
    for i in range(len(out) - 1):
        a, b = out[i], out[i + 1]
        if abs(a[0] - b[0]) <= LANE_SNAP and abs(a[0] - b[0]) > 0:
            x = (a[0] + b[0]) / 2
            a[0] = b[0] = x
        if abs(a[1] - b[1]) <= LANE_SNAP and abs(a[1] - b[1]) > 0:
            y = (a[1] + b[1]) / 2
            a[1] = b[1] = y
    return [tuple(p) for p in out]


def build_sketch(src):
    blue = d(*next(n for n in src["nexuses"] if n["side"] == "blue")["at"])
    red = d(*next(n for n in src["nexuses"] if n["side"] == "red")["at"])
    sq = (red[0] - BASE_SQ / 2, red[1] - BASE_SQ / 2)

    def mid_dist(p):
        # Distance to the mid lane: the straight line between the bases.
        (x0, y0), (x1, y1) = blue, red
        return abs((y1 - y0) * p[0] - (x1 - x0) * p[1] + x1 * y0 - y1 * x0) / math.dist(blue, red)

    def in_red_sq(p, pad):
        return sq[0] - pad < p[0] < sq[0] + BASE_SQ + pad and sq[1] - pad < p[1] < sq[1] + BASE_SQ + pad

    # Lanes: measured centrelines, snapped, ends trimmed BASE_GAP short of the bases.
    lanes = {}
    for f in src["lanes"]:
        name = f["id"].split("-")[1]
        pts = snap_line([d(*p) for p in f["line"]])
        if name == "mid":
            u = ((red[0] - blue[0]) / math.dist(blue, red), (red[1] - blue[1]) / math.dist(blue, red))
            half = BASE_SQ / 2 * math.sqrt(2)
            a = (blue[0] + u[0] * (BASE_R + MID_GAP), blue[1] + u[1] * (BASE_R + MID_GAP))
            b = (red[0] - u[0] * (half + MID_GAP), red[1] - u[1] * (half + MID_GAP))
            lanes[name] = [a, b]
            continue
        # Extend the first segment towards the blue base until BASE_GAP from the circle,
        # and the last towards the red square until BASE_GAP from it.
        def extend(p, q, stop):
            # walk from q through p (and beyond) until stop(pt) is true
            ux, uy = p[0] - q[0], p[1] - q[1]
            L = math.hypot(ux, uy)
            ux, uy = ux / L, uy / L
            k = 0.0
            pt = q
            while k < 150:
                cand = (q[0] + ux * k, q[1] + uy * k)
                if stop(cand):
                    return pt
                pt = cand
                k += 0.05
            return pt
        first = extend(pts[0], pts[1], lambda c: math.dist(c, blue) < BASE_R + BASE_GAP)
        last = extend(pts[-1], pts[-2], lambda c: in_red_sq(c, BASE_GAP))
        lanes[name] = [first] + pts[1:-1] + [last]

    cubics, dev_r1a, dev_fit = river_cubics(src)

    def centre_samples():
        out = []
        for ci, c in enumerate(cubics):
            for i in range(49):
                if ci == 1 and i == 0:
                    continue
                out.append(bez(c, i / 48))
        return out

    samples = centre_samples()
    def runs_of(points, cut):
        runs, run = [], []
        for p in points:
            if cut(p):
                if len(run) > 1:
                    runs.append(run)
                run = []
            else:
                run.append(p)
        if len(run) > 1:
            runs.append(run)
        return runs

    banks = []
    for s in (1, -1):
        pts = []
        for x, y, dx, dy in samples:
            L = math.hypot(dx, dy)
            pts.append((x - dy / L * BANK * s, y + dx / L * BANK * s))
        banks += runs_of(pts, lambda p: mid_dist(p) < BRIDGE)
    inset_river = runs_of([(x, y) for x, y, _, _ in samples], lambda p: mid_dist(p) < INSET_BRIDGE)

    # Inset lanes (critique r1 surfaces, fix 2): the outer lanes without their
    # diagonal corner bends. Top and bottom together otherwise close into an
    # octagon that reads as a frame (an X in a box at 88 px); the inset has none.
    inset_lanes = []
    for name in ("top", "mid", "bot"):
        pts = lanes[name]
        if name == "mid":
            inset_lanes.append(pts)
            continue
        run = [pts[0]]
        for a, b in zip(pts, pts[1:]):
            if abs(a[0] - b[0]) > 0.01 and abs(a[1] - b[1]) > 0.01:
                if len(run) > 1:
                    inset_lanes.append(run)
                run = [b]
            else:
                run.append(b)
        if len(run) > 1:
            inset_lanes.append(run)

    # Bases.
    base_blue = (f"M{fmt(blue[0] - BASE_R)},{fmt(blue[1])}a{fmt(BASE_R)},{fmt(BASE_R)} 0 1 0 "
                 f"{fmt(2 * BASE_R)},0a{fmt(BASE_R)},{fmt(BASE_R)} 0 1 0 {fmt(-2 * BASE_R)},0")
    base_red = f"M{fmt(sq[0])},{fmt(sq[1])}h{fmt(BASE_SQ)}v{fmt(BASE_SQ)}h{fmt(-BASE_SQ)}Z"

    # Frame: four strokes stopping 1.5 short of each corner.
    frame = [[(3.5, 2), (96.5, 2)], [(98, 3.5), (98, 96.5)], [(96.5, 98), (3.5, 98)], [(2, 96.5), (2, 3.5)]]
    ticks = [[(2 + 96 * i / 8, 2), (2 + 96 * i / 8, 0.4)] for i in range(1, 8)]
    ticks += [[(2, 2 + 96 * i / 5), (0.4, 2 + 96 * i / 5)] for i in range(1, 5)]

    # Marks: Baron and Dragon diamonds; words sit on the side away from the river.
    baron = d(*next(o for o in src["objectives"] if o["id"] == "obj-baron")["at"])
    dragon = d(*next(o for o in src["objectives"] if o["id"] == "obj-dragon")["at"])
    h = MARK / 2

    def diamond(c):
        return f"M{fmt(c[0])},{fmt(c[1] - h)}l{fmt(h)},{fmt(h)}l{fmt(-h)},{fmt(h)}l{fmt(-h)},{fmt(-h)}Z"

    # Leader anchors: from the edge of the base mark, 7 units outward.
    ub = (0.94, -0.34)
    lb0 = (blue[0] + ub[0] * (BASE_R + 0.5), blue[1] + ub[1] * (BASE_R + 0.5))
    lb1 = (lb0[0] + ub[0] * LEADER, lb0[1] + ub[1] * LEADER)
    ur = (-0.94, 0.34)
    edge = BASE_SQ / 2 / abs(ur[0]) + 0.5
    lr0 = (red[0] + ur[0] * edge, red[1] + ur[1] * edge)
    lr1 = (lr0[0] + ur[0] * LEADER, lr0[1] + ur[1] * LEADER)
    # Bush clusters: the ten largest bushes, three dots in a triangle (0.9 units apart).
    bushes = sorted(src["bushes"], key=lambda b: (-abs(area(b["ring"])), b["id"]))[:10]
    dots = []
    for b in bushes:
        c = centroid([d(*p) for p in b["ring"]])
        s = 0.9
        for ang in (90, 210, 330):
            r = s / math.sqrt(3)
            dots.append((c[0] + r * math.cos(math.radians(ang)), c[1] - r * math.sin(math.radians(ang))))

    # Verify: every sketch vertex within 1.5 units of the measured geometry.
    lane_dev = 0.0
    for f in src["lanes"]:
        name = f["id"].split("-")[1]
        meas = [d(*p) for p in f["line"]]
        if name == "mid":
            # The sketch's mid runs base to base along the measured axis:
            # compare with the axis, not the shorter measured segment.
            (x0, y0), (x1, y1) = meas
            for p in lanes[name]:
                lane_dev = max(lane_dev, abs((y1 - y0) * p[0] - (x1 - x0) * p[1] + x1 * y0 - y1 * x0) / math.dist(meas[0], meas[1]))
            continue
        for p in lanes[name][1:-1]:
            lane_dev = max(lane_dev, poly_dist(p, meas))

    return {
        "frame": frame, "ticks": ticks, "lanes": lanes, "banks": banks, "inset_river": inset_river,
        "base_blue": base_blue, "base_red": base_red, "blue": blue, "red": red,
        "inset_lanes": inset_lanes, "dots": dots, "baron": baron, "dragon": dragon,
        "diamonds": [diamond(baron), diamond(dragon)],
        "leader_blue": (lb0, lb1), "leader_red": (lr0, lr1), "cubics": cubics,
        "stats": {
            "river_dev_r1a": round(dev_r1a, 2),
            "river_dev_fit": None if dev_fit is None else round(dev_fit, 2),
            "lane_dev": round(lane_dev, 2),
            "bush_clusters": len(bushes),
        },
    }


# ---- Outputs ----------------------------------------------------------------
def geometry_json(src, stats):
    def feat(f, kind, **extra):
        out = {"id": f["id"], "kind": kind, "source": f["source"]}
        for k in ("note", "name", "side", "lane", "tier"):
            if k in f:
                out[k] = f[k]
        out.update(extra)
        for k in ("ring", "line", "at", "centre", "r", "elev"):
            if k in f:
                out[k] = f[k]
        return out

    features = []
    features.append(feat(src["boundary"], "boundary"))
    features += [feat(f, "wall") for f in src["walls"]]
    features.append(feat(src["river"], "river"))
    features += [feat(f, "lane") for f in src["lanes"]]
    features += [feat(f, "bush") for f in src["bushes"]]
    features += [feat(f, "pit", objective=f["objective"]) for f in src["pits"]]
    features += [feat(f, "objective", objective=f["kind"]) for f in src["objectives"]]
    features += [feat(f, "camp", camp=f["kind"]) for f in src["camps"]]
    features += [feat(f, "nexus") for f in src["nexuses"]]
    features += [feat(f, "inhibitor") for f in src["inhibitors"]]
    features += [feat(f, "tower") for f in src["towers"]]
    counts = {}
    for f in features:
        counts[f["kind"]] = counts.get(f["kind"], 0) + 1
    doc = {
        "generated_by": "scripts/rift-geometry/build.py from scripts/rift-geometry/rift-source.json; do not edit",
        "units": "game units, 0..15000, y north (Match-V5 positions)",
        "calibration": src["calibration"],
        "sources": src["sources"],
        "elevation": {"default": 0, "note": "the river polygon carries elev -1; nothing else differs; no contours are drawn"},
        "counts": dict(sorted(counts.items())),
        "checks": stats,
        "features": features,
    }
    return json.dumps(doc, indent=1, sort_keys=False) + "\n"


def rust_str(s):
    assert '"' not in s and "\\" not in s
    return f'"{s}"'


def rust_module(src, sk):
    walls = " ".join(path_poly([d(*p) for p in f["ring"]], True) for f in src["walls"])
    bushes = " ".join(path_poly([d(*p) for p in f["ring"]], True) for f in src["bushes"])
    river = path_poly([d(*p) for p in src["river"]["ring"]], True)
    boundary = path_poly([d(*p) for p in src["boundary"]["ring"]], True)
    lanes = " ".join(path_poly(sk["lanes"][k]) for k in ("top", "mid", "bot"))
    banks = " ".join(path_poly(r) for r in sk["banks"])
    inset_river = " ".join(path_poly(r) for r in sk["inset_river"])
    frame = " ".join(path_poly(s) for s in sk["frame"])
    ticks = " ".join(path_poly(s) for s in sk["ticks"])
    inset_lanes = " ".join(path_poly(r) for r in sk["inset_lanes"])
    dots = " ".join(f"M{fmt(x)},{fmt(y)}h0" for x, y in sk["dots"])
    marks = " ".join(sk["diamonds"])
    (lb0, lb1), (lr0, lr1) = sk["leader_blue"], sk["leader_red"]

    def pts(arr):
        return ", ".join(f"({fmt(x)}, {fmt(y)})" for x, y in arr)

    def floats(v):
        s = fmt(v)
        return s if "." in s else s + ".0"

    def pt(p):
        return f"({floats(p[0])}, {floats(p[1])})"

    towers = [d(*t["at"]) for t in src["towers"]]
    inhibs = [d(*t["at"]) for t in src["inhibitors"]]
    buffs = [d(*c["at"]) for c in src["camps"] if c["kind"] == "buff"]
    small = [d(*c["at"]) for c in src["camps"] if c["kind"] == "small"]
    pits = {p["objective"]: d(*p["centre"]) for p in src["pits"]}
    blue, red = sk["blue"], sk["red"]

    def arr(name, doc, items):
        body = ", ".join(pt(p) for p in items)
        return f"/// {doc}\npub const {name}: [(f32, f32); {len(items)}] = [{body}];\n"

    lines = [
        "// @generated by scripts/rift-geometry/build.py from scripts/rift-geometry/rift-source.json.",
        "// Do not edit by hand: change the source file and run",
        "// `python3 -I scripts/rift-geometry/build.py`. Measurement note: docs/rift-schematic.md.",
        "//! Generated Rift geometry for `RiftSketch` (M152). Drawing units: a",
        "//! 100 x 100 sheet, 150 game units per unit, y down. No image was read",
        "//! to make it: every number comes from the authored coordinate file.",
        "",
        f"/// Walls in the source ({len(src['walls'])}), bushes ({len(src['bushes'])}), towers ({len(towers)}).",
        f"pub const WALL_COUNT: usize = {len(src['walls'])};",
        f"pub const BUSH_COUNT: usize = {len(src['bushes'])};",
        f"pub const CAMP_COUNT: usize = {len(buffs) + len(small)};",
        "",
        "// ---- Sketch level (card A) ----",
        f"/// Frame: four strokes, open corners.\npub const SKETCH_FRAME: &str = {rust_str(frame)};",
        f"/// Edge ticks (1280 only).\npub const SKETCH_TICKS: &str = {rust_str(ticks)};",
        f"/// Lanes: measured centrelines, snapped, open at the bases.\npub const SKETCH_LANES: &str = {rust_str(lanes)};",
        f"/// River banks either side of the two-cubic centreline, broken at mid.\npub const SKETCH_BANKS: &str = {rust_str(banks)};",
        f"/// Blue base: circle r3 on the blue nexus.\npub const SKETCH_BASE_BLUE: &str = {rust_str(sk['base_blue'])};",
        f"/// Red base: square 6 on the red nexus.\npub const SKETCH_BASE_RED: &str = {rust_str(sk['base_red'])};",
        f"/// Bush clusters: three dots each, the ten largest bushes.\npub const SKETCH_BUSH_DOTS: &str = {rust_str(dots)};",
        f"/// Baron and Dragon diamonds.\npub const SKETCH_MARKS: &str = {rust_str(marks)};",
        f"/// Baron pit centre (word to its right).\npub const BARON_AT: (f32, f32) = {pt(sk['baron'])};",
        f"/// Dragon position (word to its left).\npub const DRAGON_AT: (f32, f32) = {pt(sk['dragon'])};",
        f"/// Match Day inset: the lanes with the outer lanes' corner bends left out, so\n/// no closed outline reads as a frame (critique r1 surfaces, fix 2).\npub const INSET_LANES: &str = {rust_str(inset_lanes)};",
        f"/// Match Day inset: the river centreline, broken 5 units either side of mid.\npub const INSET_RIVER: &str = {rust_str(inset_river)};",
        f"/// Chalk leader from the blue base mark (start, end).\npub const LEADER_BLUE: ((f32, f32), (f32, f32)) = ({pt(lb0)}, {pt(lb1)});",
        f"/// Chalk leader from the red base mark (start, end).\npub const LEADER_RED: ((f32, f32), (f32, f32)) = ({pt(lr0)}, {pt(lr1)});",
        f"/// Blue nexus (drawing units).\npub const NEXUS_BLUE: (f32, f32) = {pt(blue)};",
        f"/// Red nexus (drawing units).\npub const NEXUS_RED: (f32, f32) = {pt(red)};",
        "",
        "// ---- Survey level (card B) ----",
        f"/// Edge of the playable ground.\npub const SURVEY_BOUNDARY: &str = {rust_str(boundary)};",
        f"/// Wall outlines, exact as measured.\npub const SURVEY_WALLS: &str = {rust_str(walls)};",
        f"/// Bush outlines (approximate; see the measurement note).\npub const SURVEY_BUSHES: &str = {rust_str(bushes)};",
        f"/// River outline with both pits (elevation -1 in the JSON; not drawn).\npub const SURVEY_RIVER: &str = {rust_str(river)};",
        arr("TOWERS", "Towers and nexus towers.", towers),
        arr("INHIBITORS", "Inhibitors.", inhibs),
        arr("BUFF_CAMPS", "Buff camps.", buffs),
        arr("SMALL_CAMPS", "Small camps and the two scuttle crabs.", small),
        f"/// Baron pit centre.\npub const PIT_BARON: (f32, f32) = {pt(pits['baron'])};",
        f"/// Dragon pit centre.\npub const PIT_DRAGON: (f32, f32) = {pt(pits['dragon'])};",
    ]
    return "\n".join(lines).rstrip() + "\n"


def main():
    src = load()
    stats = check(src)
    sk = build_sketch(src)
    stats.update(sk["stats"])
    js = geometry_json(src, stats)
    rs = rust_module(src, sk)
    if "--check" in sys.argv:
        stale = [p for p, s in ((OUT_JSON, js), (OUT_RS, rs)) if not p.exists() or p.read_text() != s]
        if stale:
            print("stale:", ", ".join(str(p.relative_to(ROOT)) for p in stale))
            sys.exit(1)
        print("rift geometry: up to date")
        return
    OUT_JSON.parent.mkdir(parents=True, exist_ok=True)
    OUT_JSON.write_text(js)
    OUT_RS.write_text(rs)
    print(json.dumps(stats, indent=1))
    print(f"{OUT_JSON.relative_to(ROOT)}: {len(js)} bytes; {OUT_RS.relative_to(ROOT)}: {len(rs)} bytes")


if __name__ == "__main__":
    main()
