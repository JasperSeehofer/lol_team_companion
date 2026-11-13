#!/usr/bin/env python3
"""Link-preview cards (M152 item 5, recipe card F): three static 1200x630 pages.

    python3 -I scripts/og/build.py <out dir>     # writes landing.html, share.html, default.html
    node scripts/og/capture.mjs                  # builds, captures, writes public/og/*.png

The drawing is the app's own Rift sketch: the path data is read from the
generated module src/components/ui/rift_geometry.rs (made by
scripts/rift-geometry/build.py from authored coordinates), so the cards show
exactly what the app draws. No image is read, nothing comes from a picture of
the game's map, no Riot name or mark is used. Colours are read from the
tokens in input.css, fonts are the self-hosted ones in public/fonts (served
by capture.mjs at /fonts/).

Layout (card F): `page` ground, 64 px margin; left 576 px: "Team Companion"
(Familjen Grotesk 600, 72 px, primary) and one sentence (Public Sans 40 px,
secondary); right: the sketch as a 600 px square from x 720, bleeding 120 px
off the right edge, 2 px ink strokes, no ticks. landing: chalk "YOUR TEAM"
(32 px) with its leader at the blue base. share: cropped to the river-mid
bridge, no chalk, nothing about any team or link (one file for every share
URL). default: the full sheet, no chalk.
"""
import html
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
RS = (ROOT / "src" / "components" / "ui" / "rift_geometry.rs").read_text()
CSS = (ROOT / "input.css").read_text()

S = dict(re.findall(r'pub const (\w+): &str = "([^"]*)";', RS))
P = {k: tuple(float(v) for v in re.findall(r"-?[\d.]+", body))
     for k, body in re.findall(r"pub const (\w+): \(f32, f32\) = \(([^)]*)\);", RS)}
L = {k: [tuple(map(float, p)) for p in re.findall(r"\((-?[\d.]+), (-?[\d.]+)\)", body)]
     for k, body in re.findall(r"pub const (LEADER_\w+): \(\(f32, f32\), \(f32, f32\)\) = (.*);", RS)}


def token(name):
    m = re.search(rf"--color-{name}:\s*([^;]+);", CSS)
    assert m, name
    return m.group(1).strip()


T = {k: token(k) for k in ("page", "primary", "secondary", "muted", "accent")}

# Card F sentences. The landing's is the landing page's own sentence
# (closed_beta.rs); the default is the product line; the share card says
# only what kind of page it is. Each is set in two lines, broken by hand.
SENTENCE = {
    "landing": ("Scouting, drafts, game plans", "and reviews in one loop."),
    "default": ("The coaching workspace", "for one team."),
    "share": ("A read-only page", "shared by a team."),
}

SHEET_X, SHEET_Y, SHEET = 720, 15, 600


def f(v):
    s = f"{v:.3f}".rstrip("0").rstrip(".")
    return "0" if s in ("", "-0") else s


def card(kind):
    full = kind != "share"
    # The share card is cropped to the river-mid bridge (50, 50), 60 units
    # wide so both pit diamonds stay whole inside the visible 480 px.
    vb = (0.0, 0.0, 100.0, 100.0) if full else (20.0, 20.0, 60.0, 60.0)
    vx, vy, vw, vh = vb

    def px(x, y):
        return SHEET_X + (x - vx) / vw * SHEET, SHEET_Y + (y - vy) / vh * SHEET

    paths = ["SKETCH_LANES", "SKETCH_BANKS", "SKETCH_BASE_BLUE", "SKETCH_BASE_RED", "SKETCH_MARKS"]
    if full:
        paths.insert(0, "SKETCH_FRAME")
    ink = "".join(f'<path d="{S[p]}"/>' for p in paths)
    # Bush dots on the full sheet only (the share crop, like the share strip,
    # has none: R1-b), so no cluster is cut at the crop's edge.
    dots = f'<path class="dots" d="{S["SKETCH_BUSH_DOTS"]}"/>' if full else ""
    half = 1.45
    bx, by = px(P["BARON_AT"][0] + half + 0.8, P["BARON_AT"][1])
    dx, dy = px(P["DRAGON_AT"][0] - half - 0.8, P["DRAGON_AT"][1])
    words = (f'<span class="word" style="left:{f(bx)}px;top:{f(by)}px;transform:translateY(-50%)">Baron</span>'
             f'<span class="word" style="right:{f(1200 - dx)}px;top:{f(dy)}px;transform:translateY(-50%)">Dragon</span>')
    chalk = label = ""
    if kind == "landing":
        (x0, y0), (x1, y1) = L["LEADER_BLUE"]
        chalk = (f'<g class="chalk"><path d="M{f(x0)},{f(y0)}L{f(x1)},{f(y1)}"/>'
                 f'<path class="dot" d="M{f(x0)},{f(y0)}h0"/></g>')
        lx, ly = px(x1 + 1.0, y1)
        label = f'<span class="label" style="left:{f(lx)}px;top:{f(ly)}px;transform:translateY(-50%)">Your team</span>'
    return f"""<!doctype html>
<html lang="en"><head><meta charset="utf-8"><title>og {kind}</title>
<style>
@font-face {{ font-family: "Familjen Grotesk"; font-weight: 400 700; src: url("/fonts/familjen-grotesk/familjen-grotesk-latin-wght-normal.woff2") format("woff2"); }}
@font-face {{ font-family: "Public Sans"; font-weight: 100 900; src: url("/fonts/public-sans/public-sans-latin-wght-normal.woff2") format("woff2"); }}
@font-face {{ font-family: "JetBrains Mono"; font-weight: 100 800; src: url("/fonts/jetbrains-mono/jetbrains-mono-latin-wght-normal.woff2") format("woff2"); }}
html, body {{ margin: 0; }}
body {{ width: 1200px; height: 630px; overflow: hidden; position: relative; background: {T['page']}; }}
.text {{ position: absolute; left: 64px; top: 64px; width: 576px; height: 502px;
         display: flex; flex-direction: column; justify-content: center; gap: 28px; }}
.name {{ margin: 0; font: 600 72px/1.05 "Familjen Grotesk", sans-serif; letter-spacing: -0.025em; color: {T['primary']}; }}
.line {{ margin: 0; font: 400 40px/1.25 "Public Sans", sans-serif; color: {T['secondary']}; }}
svg {{ position: absolute; left: {SHEET_X}px; top: {SHEET_Y}px; width: {SHEET}px; height: {SHEET}px; }}
.ink {{ fill: none; stroke: {T['primary']}; stroke-opacity: .7; stroke-width: 2px;
        stroke-linecap: round; stroke-linejoin: round; }}
.ink path, .chalk path {{ vector-effect: non-scaling-stroke; }}
.ink .dots {{ stroke-width: 3px; }}
.chalk {{ fill: none; stroke: {T['accent']}; stroke-width: 2px; stroke-linecap: round; }}
.chalk .dot {{ stroke-width: 5px; }}
.word, .label {{ position: absolute; white-space: nowrap; font-family: "JetBrains Mono", monospace;
                 font-weight: 500; text-transform: uppercase; letter-spacing: .08em; line-height: 1; }}
.word {{ font-size: 18px; color: {T['muted']}; }}
.label {{ font-size: 32px; color: {T['accent']}; }}
</style></head>
<body data-og="{kind}">
<div class="text"><p class="name">Team Companion</p><p class="line">{"<br>".join(html.escape(t) for t in SENTENCE[kind])}</p></div>
<svg viewBox="{f(vx)} {f(vy)} {f(vw)} {f(vh)}" aria-hidden="true"><g class="ink">{ink}{dots}</g>{chalk}</svg>
{words}{label}
</body></html>
"""


def main():
    out = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "/tmp/og-pages")
    out.mkdir(parents=True, exist_ok=True)
    for kind in ("landing", "share", "default"):
        page = card(kind)
        assert "riot" not in page.lower() and "league" not in page.lower()
        (out / f"{kind}.html").write_text(page)
        print(out / f"{kind}.html")


if __name__ == "__main__":
    main()
