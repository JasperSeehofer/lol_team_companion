# Surfaces (M151 identity stills)

Three surfaces, one skeleton: `landing.html`, `share.html`, `matchday.html`, each taking `?t=t1|t2|t3`. Copy and structure come from the app (`closed_beta.rs`, `share.rs` shared plan, `home.rs` Match Day + the field card) with the seed tool's fictional teams. `surfaces.css` mirrors the primitives (nav, PageShell, PageHeader, SectionHeader, Button, PrepSequence, monograms) with `../tokens.css` values only.

**The art layer is the only thing a treatment changes.** `surface.js` reads `?t=`, loads `art/<t>.json` and the SVG it names, and injects it into every `[data-art]` slot:

- `data-art="hero"`: the landing figure (full 0 0 100 100), plus the chalk label (`data-label`), mark spots and A-H / 1-5 tick labels (shown from 1024 px).
- `data-art="share"`: a crop (`crops.share`, a viewBox with the slot's 358x96 aspect) and the opponent label (`shareLabel`, page halo).
- `data-art="inset"`: the 88 px Match Day inset, ink only (the `.chalk` group is hidden).

To add T2/T3: write `art/t2-....svg` on the same 100x100 unit square with groups `.ink` (styled by `surfaces.css`: primary at 70 %, `--ink-w`), optional `.ticks` (muted) and `.chalk` (accent leader), or your own classes styled in an optional `art/t2.css` (named by `css` in the json; tokens only, e.g. T3's terrain fills); then `art/t2.json` with `svg`, `css`, `label`, `marks`, `ticks`, `crops`, `shareLabel` (all in 100x100 units; omit what you do not use). Nothing else changes; capture with `?t=t2`.

Marks are fetched from `public/marks/<name>.svg` at load (`[data-mark]` slots, `currentColor`), so a fix to a mark reaches every still on the next capture. T1's SVG is generated: `python3 -I .planning/missions/M151/gallery/tools/build-t1-rift.py`.

Serve the repo root (`python3 -m http.server 8151 --bind 127.0.0.1`, or another port with `GALLERY_BASE`), then e.g. `node $G/tools/capture.mjs "$G/surfaces/landing.html?t=t1" $G/out/t1-landing-390.png --width 390`. Production note: the gallery fetches the art after load; the app must render it inline on the server so the first paint carries it (V5).
