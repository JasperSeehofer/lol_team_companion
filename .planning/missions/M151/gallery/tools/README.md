# M151 gallery tools

Static stills built only from the app's tokens. Run everything from the repo root; `G=.planning/missions/M151/gallery`.

**Tokens.** `node $G/tools/extract-tokens.mjs` regenerates `$G/tokens.css` from `input.css` (every `@theme` and `:root` property, the `@font-face` rules pointing at `../../../../public/fonts/`, the type helpers `text-caption`..`text-display`, `eyebrow`, `figures`, `note`, `font-display|sans|mono`). Never edit `tokens.css` by hand. Pages link `tokens.css` relatively and use `var(--color-*)`, `var(--text-*)`, `calc(var(--spacing) * N)`.

**Serve** (fonts need same-origin, so no `file://`): `python3 -m http.server 8151 --bind 127.0.0.1` from the repo root. Port 8151 is capture.mjs's default `--base`; pick another free port with `--base http://127.0.0.1:PORT` (or `GALLERY_BASE`).

**Capture** (playwright from `e2e/node_modules`; run `cd e2e && npm ci` once; Chromium from `~/.cache/ms-playwright/chromium-1243`, override with `GALLERY_CHROMIUM`):

    node $G/tools/capture.mjs $G/smoke.html $G/out/smoke-390.png --width 390 --full
    node $G/tools/capture.mjs $G/smoke.html $G/out/smoke-1280.png --width 1280 --full
    node $G/tools/capture.mjs $G/page.html $G/out/page-og.png --og            # 1200x630
    node $G/tools/capture.mjs $G/page.html $G/out/x.png --width 390 --reduced-motion
    node $G/tools/capture.mjs $G/smoke.html $G/out/smoke-motion.png --width 390 \
      --selector '#track' --frames 6 --duration 220 \
      --trigger "document.getElementById('mover').classList.add('go')"

The output line lists the font faces that loaded; check it, and look at the PNG, before trusting a still. `--selector` shoots one element (scrolled into view, 16px pad). Frames mode runs the trigger, pauses every `document.getAnimations()`, seeks them to N evenly spaced times from 0 to the duration, writes `-f1..fN.png` and `-strip.png` (labelled with ms). An animation that has not started by two frames after the trigger is not caught: start transitions from the trigger, do not delay them.

**Lint.** `$G/tools/lint-gallery.sh [files]` sources the rules from `scripts/lint_design.sh` (palette, hex, px-text, rounded-full, blur, gradient, drop-shadow, pulse, ornament, emoji, data-theme, external-url, faded-text, faint-outline, legacy-token) and adds CSS rules: font size under 12px, raw colour functions outside `tokens.css`, `filter: blur`, `outline: none`, and round `border-radius`. It scans every html/css/svg under the gallery except `out/` and `tools/`. Escape hatch: `lint:allow(<rule>)` on the same line, with a reason (for example a round avatar).

**Rubric.** It runs every `run_by: harness` row at 390x844 and 1280x800 and has no row filter, so filter its output:

    node ~/Repositories/garden/wiki/assets/ux-rubric/rubric-check.mjs --app-dir e2e \
      http://127.0.0.1:8151/$G/smoke.html | grep -E '^\| (id|V1|V2|I7|A2|A3) '

V1, V2, I7, A2 and A3 are machine-checked. V5, V7, V8 and A4 are not harness rows: judge them from the captures (V5: capture.mjs always waits for `document.fonts.ready`, so even `--settle 0` is not a true first paint; first-frame stills need a separate pass). I1, I2, I6 and I8 always FIX on a still because they look for app features, so ignore them. V3 flags short label paragraphs as well as long prose, so read its detail. Exit code 1 means any FIX in any row.
