#!/usr/bin/env python3
"""Standalone preview stills of RiftSketch for the atelier (M152 item 2).

Reads the generated Rust module (src/components/ui/rift_geometry.rs), so the
pages draw exactly the path data the app ships, and mirrors the markup of
src/components/ui/rift.rs. Tokens come from the M151 gallery's tokens.css
(generated from input.css). No image is read.

    python3 -I .planning/missions/M152/gallery/tools/build-rift-preview.py
    # then serve the repo root and capture (see capture-rift.mjs)
"""
import math
import pathlib
import re

HERE = pathlib.Path(__file__).resolve().parent
GAL = HERE.parent
ROOT = HERE.parents[4]
RS = (ROOT / "src" / "components" / "ui" / "rift_geometry.rs").read_text()

S = dict(re.findall(r'pub const (\w+): &str = "([^"]*)";', RS))
P = {k: tuple(float(v) for v in re.findall(r"-?[\d.]+", body))
     for k, body in re.findall(r"pub const (\w+): \(f32, f32\) = \(([^)]*)\);", RS)}
A = {k: [tuple(map(float, p)) for p in re.findall(r"\((-?[\d.]+), (-?[\d.]+)\)", body)]
     for k, body in re.findall(r"pub const (\w+): \[\(f32, f32\); \d+\] = \[(.*)\];", RS)}
L = {k: [tuple(map(float, p)) for p in re.findall(r"\((-?[\d.]+), (-?[\d.]+)\)", body)]
     for k, body in re.findall(r"pub const (LEADER_\w+): \(\(f32, f32\), \(f32, f32\)\) = (.*);", RS)}


def f(v):
    s = f"{v:.3f}".rstrip("0").rstrip(".")
    return "0" if s in ("", "-0") else s


def circle(c, r):
    return f"M{f(c[0] - r)},{f(c[1])}a{f(r)},{f(r)} 0 1 0 {f(2 * r)},0a{f(r)},{f(r)} 0 1 0 -{f(2 * r)},0"


def diamond(c, h):
    return f"M{f(c[0])},{f(c[1] - h)}l{f(h)},{f(h)}l-{f(h)},{f(h)}l-{f(h)},-{f(h)}Z"


def square(c, h):
    return f"M{f(c[0] - h)},{f(c[1] - h)}h{f(2 * h)}v{f(2 * h)}h-{f(2 * h)}Z"


CSS = """
<link rel="stylesheet" href="../../M151/gallery/tokens.css">
<style>
  html, body { margin: 0; background: var(--color-page); color: var(--color-primary); }
  body { font-family: var(--font-sans); }
  .rift { position: relative; }
  .rift svg { display: block; width: 100%; height: auto; }
  .ink { fill: none; stroke: var(--color-primary); stroke-opacity: .7; stroke-width: var(--w, 1px);
         stroke-linecap: round; stroke-linejoin: round; }
  .ink path, .chalk path, .ticks path, .erase { vector-effect: non-scaling-stroke; }
  .hatch { shape-rendering: crispEdges; }
  .dots { stroke-width: 2px; }
  .chalk { fill: none; stroke: var(--color-accent); stroke-width: 1px; stroke-linecap: round; }
  .ticks { fill: none; stroke: var(--color-muted); stroke-width: 1px; }
  .eyebrow { font-family: var(--font-mono); font-size: 12px; line-height: 1.3; font-weight: 500;
             letter-spacing: .08em; text-transform: uppercase; color: var(--color-muted);
             position: absolute; white-space: nowrap; pointer-events: none; }
  .label { color: var(--color-accent); }
  .tick { font-family: var(--font-mono); font-size: 12px; color: var(--color-muted); position: absolute; }
  .fillink { fill: var(--color-primary); fill-opacity: .7; }
  .wide { --w: 1.25px; }
  .wide .label { font-size: 13px; }
</style>
"""


def sketch(width, wide, label=True, ticks=False):
    """Full sheet, sketch level (mirrors RiftCrop::Full + RiftLevel::Sketch)."""
    half = 1.45
    bx, by = P["BARON_AT"]
    dx, dy = P["DRAGON_AT"]
    (x0, y0), (x1, y1) = L["LEADER_BLUE"]
    tick_lines = f'<g class="ticks"><path d="{S["SKETCH_TICKS"]}"/></g>' if ticks else ""
    tick_labels = ""
    if ticks:
        for i, c in enumerate("ABCDEFGH"):
            tick_labels += f'<span class="tick" style="left:{f(2 + 96 * (i + .5) / 8)}%;top:0;transform:translate(-50%,-110%)">{c}</span>'
        for i in range(1, 6):
            tick_labels += f'<span class="tick" style="left:0;top:{f(2 + 96 * (i - .5) / 5)}%;transform:translate(-140%,-50%)">{i}</span>'
    chalk = ""
    lab = ""
    if label:
        chalk = f'<g class="chalk"><path d="M{f(x0)},{f(y0)}L{f(x1)},{f(y1)}"/><path d="M{f(x0)},{f(y0)}h0"/></g>'
        lab = f'<span class="eyebrow label" style="left:{f(x1 + 1)}%;top:{f(y1)}%;transform:translateY(-50%)">Your team</span>'
    return f"""<div class="rift {'wide' if wide else ''}" style="width:{width}px" data-rift="sketch">
<svg viewBox="0 0 100 100" aria-hidden="true">{tick_lines}<g class="ink">
<path d="{S['SKETCH_FRAME']}"/><path d="{S['SKETCH_LANES']}"/><path d="{S['SKETCH_BANKS']}"/>
<path d="{S['SKETCH_BASE_BLUE']}"/><path d="{S['SKETCH_BASE_RED']}"/><path d="{S['SKETCH_MARKS']}"/>
<path class="hatch" d="{S['SKETCH_HATCH']}"/><path class="dots" d="{S['SKETCH_BUSH_DOTS']}"/></g>{chalk}</svg>
<span class="eyebrow" style="left:{f(bx + half + .8)}%;top:{f(by)}%;transform:translateY(-50%)">Baron</span>
<span class="eyebrow" style="right:{f(100 - (dx - half - .8))}%;top:{f(dy)}%;transform:translateY(-50%)">Dragon</span>
{tick_labels}{lab}</div>"""


def strip(width, w, h, wide):
    x = max(0, min(100 - w, 50 - w / 2))
    y = max(0, min(100 - h, 50 - h / 2))
    return f"""<div class="rift {'wide' if wide else ''}" style="width:{width}px" data-rift="share">
<svg viewBox="{f(x)} {f(y)} {f(w)} {f(h)}" preserveAspectRatio="xMidYMid slice" aria-hidden="true"><g class="ink">
<path d="{S['SKETCH_LANES']}"/><path d="{S['SKETCH_BANKS']}"/><path d="{S['SKETCH_BASE_BLUE']}"/><path d="{S['SKETCH_BASE_RED']}"/></g></svg></div>"""


def inset(width):
    return f"""<div class="rift" style="width:{width}px" data-rift="inset"><svg viewBox="0 0 100 100" aria-hidden="true"><g class="ink">
<path d="{S['SKETCH_LANES']}"/><path d="{S['INSET_RIVER']}"/><path d="{S['SKETCH_BASE_BLUE']}"/><path d="{S['SKETCH_BASE_RED']}"/></g></svg></div>"""


def survey(stop):
    """Mirrors RiftLevel::Survey (rift.rs) at a 560 px base sheet."""
    k = 1 / (5.6 * stop)
    p = 6 * k
    sx, sy = 7 * k, 7 * math.sqrt(3) * k
    stipple = "".join(circle(c, k) for c in [(0, 0), (sx, 0), (0, sy), (sx, sy), (sx / 2, sy / 2)])
    fill = "".join(circle(t, 2.5 * k) for t in A["TOWERS"])
    fill += "".join(circle(n, 2 * k) for n in (P["NEXUS_BLUE"], P["NEXUS_RED"]))
    stroke = ""
    for c in A["BUFF_CAMPS"]:
        fill += circle(c, k)
        stroke += circle(c, 3.5 * k)
    for c in A["SMALL_CAMPS"]:
        stroke += circle(c, 2 * k)
    for i in A["INHIBITORS"]:
        stroke += diamond(i, 4.5 * k)
    stroke += circle(P["NEXUS_BLUE"], 8 * k) + square(P["NEXUS_RED"], 8 * k)
    stroke += diamond(P["PIT_BARON"], 7 * k) + diamond(P["PIT_DRAGON"], 7 * k)
    return f"""<div class="rift" style="width:{560 * stop}px" data-rift="survey">
<svg viewBox="0 0 100 100" aria-hidden="true"><defs>
<pattern id="h{stop}" patternUnits="userSpaceOnUse" width="{f(p)}" height="{f(p)}" patternTransform="rotate(45)">
<path d="M0,0V{f(p)}" fill="none" stroke="var(--color-primary)" stroke-opacity=".7" stroke-width="{f(k)}"/></pattern>
<pattern id="s{stop}" patternUnits="userSpaceOnUse" width="{f(sx)}" height="{f(sy)}"><path d="{stipple}" class="fillink"/></pattern>
<clipPath id="w{stop}"><path d="{S['SURVEY_WALLS']}"/></clipPath><clipPath id="b{stop}"><path d="{S['SURVEY_BUSHES']}"/></clipPath></defs>
<rect x="0" y="0" width="100" height="100" fill="url(#h{stop})" clip-path="url(#w{stop})"/>
<path d="{S['SURVEY_WALLS']}" class="erase" fill="none" stroke="var(--color-page)" stroke-width="8"/>
<rect x="0" y="0" width="100" height="100" fill="url(#s{stop})" clip-path="url(#b{stop})"/>
<g class="ink"><path d="{S['SURVEY_BOUNDARY']}"/><path d="{S['SURVEY_RIVER']}"/><path d="{S['SURVEY_WALLS']}"/>
<path d="{S['SURVEY_BUSHES']}"/><path d="{stroke}"/></g><path d="{fill}" class="fillink"/></svg></div>"""


def page(title, body, pad):
    return f"""<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>{title}</title>{CSS}</head><body><div style="padding:{pad}">{body}</div></body></html>
"""


OUT = {
    "rift-sketch-1280.html": page("Rift sketch 1280", sketch(560, True, ticks=True), "48px 0 0 64px"),
    "rift-sketch-390.html": page("Rift sketch 390", sketch(358, False), "16px"),
    "rift-survey-1280.html": page(
        "Rift survey 2x",
        f'<div style="width:1200px;height:740px;overflow:hidden;position:relative">'
        f'<div style="position:absolute;left:40px;top:-200px">{survey(2)}</div></div>',
        "16px 40px"),
    "rift-survey-4x-1280.html": page(
        "Rift survey 4x",
        f'<div style="width:1200px;height:740px;overflow:hidden;position:relative">'
        f'<div style="position:absolute;left:-560px;top:-380px">{survey(4)}</div></div>',
        "16px 40px"),
    "rift-share-390.html": page("Rift share strip 390", strip(358, 40, 10.7, False), "16px"),
    "rift-share-1280.html": page("Rift share strip 1280", strip(640, 50, 9.4, True), "16px"),
    "rift-inset-390.html": page("Rift inset 88", f'<div style="background:var(--color-ink-1);display:inline-block;padding:12px">{inset(88)}</div>', "16px"),
}

for name, html in OUT.items():
    (GAL / name).write_text(html)
    print(GAL / name, len(html))
