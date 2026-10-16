#!/usr/bin/env python3
"""Build the nine marks (M151) from one hand-written table.

Writes public/marks/<name>.svg (16 px master, 1.5 px stroke) and
public/marks/<name>-24.svg (the same drawing at 24 px, stroke held at 2 px,
so 2/1.5 = 1.33333 in the 16-unit viewBox), plus the contact sheet
.planning/missions/M151/gallery/marks.html. The Rust gallery table
(src/pages/design_marks.rs, MARKS) carries the same path data; a unit test
there compares it with the files.

Geometry follows references.md "Mark grid spec" (16 grid, live area 1 to 15,
keylines circle d12, square 11, diamond in the 12 circle). Nothing is traced.
Stroke only, with one documented exception: Baron's solid core (FILLED).

Run from the repo root: python3 .planning/missions/M151/gallery/tools/build-marks.py
"""
import html
import os

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), *[".."] * 5))

DIAMOND = "M8 2L14 8L8 14L2 8Z"
# The single documented exception to "stroke only": Baron's inner diamond is
# filled with currentColor (and stroked), a solid core. An open inner diamond
# with 1.5 px gaps does not fit at 16 px (atelier critique, 2026-10-16).
BARON_CORE = "M8 6.25L9.75 8L8 9.75L6.25 8Z"
FILLED = {BARON_CORE}

# (file name, label, kind, paths). Every coordinate is on a whole or half pixel.
MARKS = [
    ("top", "Top", "Role", [
        "M2 13.5L8 3L14 13.5Z",                               # triangulation point
        "M7.25 10a.75.75 0 1 0 1.5 0a.75.75 0 1 0-1.5 0",     # station dot
    ]),
    ("jungle", "Jungle", "Role", [
        "M2 5a2 2 0 1 0 4 0a2 2 0 1 0-4 0", "M4 7v1",         # isolated trees
        "M10 4.5a2 2 0 1 0 4 0a2 2 0 1 0-4 0", "M12 6.5v1",
        "M6 11a2 2 0 1 0 4 0a2 2 0 1 0-4 0", "M8 13v1",
    ]),
    ("mid", "Mid", "Role", [
        "M4 8a4 4 0 1 0 8 0a4 4 0 1 0-8 0",                   # survey station
        "M8 2v2M8 12v2M2 8h2M12 8h2",                         # axis ticks
    ]),
    ("bot", "Bot", "Role", [
        "M3.25 12a.75.75 0 1 0 1.5 0a.75.75 0 1 0-1.5 0",     # origin
        "M4 3a9 9 0 0 1 9 9",                                 # reach arc
        "M7.06 4.61L7.83 2.76",                               # graduation ticks at
        "M9.66 6.34L11.07 4.93",                              # 22.5, 45, 67.5 degrees
        "M11.39 8.94L13.24 8.17",                             # (a sextant scale)
    ]),
    ("support", "Support", "Role", [
        "M7.25 8.5a.75.75 0 1 0 1.5 0a.75.75 0 1 0-1.5 0",    # held point, inside the cup
        "M3 3V8.5a5 5 0 0 0 10 0V3",                          # open cup (deep U, keyline width)
    ]),
    ("dragon", "Dragon", "Objective", [
        DIAMOND,
        "M5 8q1.5-2 3 0t3 0",                                 # one wave, floating inside
    ]),
    ("baron", "Baron", "Objective", [
        DIAMOND,
        BARON_CORE,                                           # solid core (the one fill)
    ]),
    ("herald", "Herald", "Objective", [
        DIAMOND,
        "M5 8h0M8 8h0M11 8h0",                                # three 1.5 px points (round caps)
    ]),
    ("tower", "Tower", "Objective", [
        "M4 13.5V6.5a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1v7",        # lookout, open at the base
        "M2 13.5h12",                                         # base line
        "M7.25 9.5a.75.75 0 1 0 1.5 0a.75.75 0 1 0-1.5 0",    # centre dot
    ]),
]

def path_el(d):
    fill = ' fill="currentColor"' if d in FILLED else ""
    return f'<path{fill} d="{d}"/>'


STROKE_16 = "1.5"
STROKE_24 = "1.33333"  # 2 px at 24 px


def svg(name, label, paths, size, stroke):
    body = "".join(path_el(d) for d in paths)
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" '
        f'viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="{stroke}" '
        f'stroke-linecap="round" stroke-linejoin="round">'
        f"<title>{html.escape(label)}</title>{body}</svg>\n"
    )


def write_files():
    out = os.path.join(REPO, "public", "marks")
    os.makedirs(out, exist_ok=True)
    for name, label, _, paths in MARKS:
        with open(os.path.join(out, f"{name}.svg"), "w") as f:
            f.write(svg(name, label, paths, 16, STROKE_16))
        with open(os.path.join(out, f"{name}-24.svg"), "w") as f:
            f.write(svg(name, label, paths, 24, STROKE_24))


def inline(paths, px, stroke):
    body = "".join(path_el(d) for d in paths)
    return (
        f'<svg width="{px}" height="{px}" viewBox="0 0 16 16" fill="none" stroke="currentColor" '
        f'stroke-width="{stroke}" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">{body}</svg>'
    )


def big(paths):
    grid = "".join(f'<path d="M{i} 0V16M0 {i}H16"/>' for i in range(17))
    keys = (
        '<circle cx="8" cy="8" r="6"/>'
        '<rect x="2.5" y="2.5" width="11" height="11"/>'
        f'<path d="{DIAMOND}"/>'
        '<path d="M2 13.25L8 2.75L14 13.25Z"/>'
    )
    body = "".join(path_el(d) for d in paths)
    return (
        '<svg width="64" height="64" viewBox="0 0 16 16" fill="none" stroke-linecap="round" '
        'stroke-linejoin="round" aria-hidden="true" class="big">'
        f'<g class="grid" stroke-width="0.0625">{grid}</g>'
        '<rect class="live" x="1" y="1" width="14" height="14" stroke-width="0.0625" stroke-dasharray="0.25 0.25"/>'
        f'<g class="keys" stroke-width="0.0625" stroke-dasharray="0.5 0.25">{keys}</g>'
        f'<g stroke="currentColor" stroke-width="{STROKE_16}">{body}</g></svg>'
    )


def panel(ground, tone, tone_label):
    rows = []
    for name, label, kind, paths in MARKS:
        rows.append(
            f'<div class="cell" data-mark="{name}">'
            f'<div class="pair s16">{inline(paths, 16, STROKE_16)}<span class="text-label">{label}</span></div>'
            f'<div class="pair s24">{inline(paths, 24, STROKE_24)}<span class="text-reading">{label}</span></div>'
            f'{big(paths)}'
            f'<span class="text-caption kind">{kind}</span>'
            "</div>"
        )
    return (
        f'<section class="panel {ground} {tone}"><p class="eyebrow">On {ground.replace("g-", "")}, {tone_label}</p>'
        f'<div class="cells">{"".join(rows)}</div></section>'
    )


def write_sheet():
    panels = "".join(
        panel(g, t, tl)
        for g in ("g-page", "g-ink-2")
        for t, tl in (("t-ink", "ink (primary 70%)"), ("t-secondary", "secondary"), ("t-chalk", "chalk (test only: marks never ship in chalk)"))
    )
    inrow = "".join(
        f'<span class="inrow">{inline(p, 16, STROKE_16)}<span class="text-label">{l}</span></span>'
        for _, l, _, p in MARKS
    )
    doc = f"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Mark contact sheet</title>
<link rel="stylesheet" href="tokens.css">
<style>
  /* generated by tools/build-marks.py; edit the table there */
  main {{ max-width: 80rem; margin: 0 auto; padding: calc(var(--spacing) * 8) calc(var(--spacing) * 4); }}
  h1 {{ font-weight: 600; margin: calc(var(--spacing) * 2) 0; }}
  .lede {{ color: var(--color-secondary); max-width: 65ch; margin: 0 0 calc(var(--spacing) * 6); }}
  .strip {{ display: flex; flex-wrap: wrap; gap: calc(var(--spacing) * 2) calc(var(--spacing) * 5); padding: calc(var(--spacing) * 4); background: var(--color-ink-1); border-top: 1px solid var(--color-rule); border-bottom: 1px solid var(--color-rule); color: var(--color-secondary); }}
  .inrow {{ display: inline-flex; align-items: center; gap: calc(var(--spacing) * 1.5); }}
  .inrow .text-label {{ color: var(--color-primary); }}
  .panel {{ padding: calc(var(--spacing) * 4); margin-top: calc(var(--spacing) * 6); border-top: 1px solid var(--color-rule); }}
  .g-page {{ background: var(--color-page); }}
  .g-ink-2 {{ background: var(--color-ink-2); }}
  .cells {{ display: grid; grid-template-columns: repeat(auto-fill, minmax(7rem, 1fr)); gap: calc(var(--spacing) * 4); margin-top: calc(var(--spacing) * 3); }}
  .cell {{ display: flex; flex-direction: column; gap: calc(var(--spacing) * 2.5); }}
  .pair {{ display: flex; align-items: center; gap: calc(var(--spacing) * 2); color: var(--color-primary); }}
  .pair span {{ color: var(--color-primary); }}
  .kind {{ color: var(--color-muted); }}
  .t-ink svg {{ color: var(--color-primary); }}
  .t-ink svg path {{ stroke-opacity: 0.7; fill-opacity: 0.7; }}
  .t-secondary svg {{ color: var(--color-secondary); }}
  .t-chalk svg {{ color: var(--color-accent); }}
  .big .grid {{ stroke: var(--color-rule); }}
  .big .live, .big .keys {{ stroke: var(--color-outline); }}
  .big .grid path, .big .keys path {{ stroke-opacity: 1; }}
</style>
</head>
<body>
<main>
  <p class="eyebrow">M151 mark set, contact sheet</p>
  <h1 class="font-display text-headline">Nine marks</h1>
  <p class="text-reading lede">Five roles and four objectives on the 16 px master grid: 1.5 px stroke, round joins and caps, stroke only, currentColor. 24 px holds the stroke at 2 px. At 64 the grid, the live area (1 to 15, dotted) and the keylines (circle d12, square 11, diamond, triangle) are shown. Every mark stands beside its word.</p>
  <div class="strip">{inrow}</div>
  {panels}
</main>
</body>
</html>
"""
    path = os.path.join(REPO, ".planning", "missions", "M151", "gallery", "marks.html")
    with open(path, "w") as f:
        f.write(doc)


if __name__ == "__main__":
    write_files()
    write_sheet()
    print("wrote public/marks/*.svg and gallery/marks.html")
