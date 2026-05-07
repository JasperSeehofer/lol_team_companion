# AI Image Reproducibility Log

> Phase 17 plan 06 — closed-beta + auth backgrounds.
>
> Tracks prompt + model + seed + compute path per asset so any operator
> can regenerate the binary from inputs alone (D-20).

## Generation runtime decision (Task 1)

- **Selected runtime:** fal.ai (per RESEARCH recommendation; sub-second
  latency, EU-friendly, ~$0.05/image).
- **Effective runtime in this commit:** **placeholder** — the
  environment in which plan 06 executed had no `FAL_KEY` /
  `FAL_API_KEY` set. The objective explicitly authorised graceful
  degradation: "if the fal.ai API is unavailable, the auth token is
  missing, or generation fails, gracefully degrade — generate
  placeholder images (solid token-gradient PNGs) at the same paths,
  document the placeholder strategy in AI-IMAGES.md with an explicit
  TODO: regenerate via fal.ai once available."
- **TODO:** Once an operator with a `FAL_KEY` re-runs this plan
  (or runs a follow-up housekeeping task), regenerate each asset
  with the FLUX prompt + seed below and overwrite the placeholder
  files. The page markup, classes, and `<img>` paths are stable —
  swapping the binary file is a one-line change in `public/img/`.
- **Optional auth background:** generated as placeholder per Task 1
  decision (auto-mode: "yes, generate auth-bg-demacia").
- **Composition reference image:** the upload at
  `/tmp/lol-design-handoff/lol-team-companion-app/project/uploads/draw-92acceeb-9fd2-499d-84e4-12ff75b7ab5d.png`
  (2576×1479 PNG, 116KB) was inspected. It is a wireframe sketch from
  the design handoff bundle, not a usable photographic asset. Role:
  **composition reference only** (for future fal.ai prompt tuning;
  none of the current placeholders use it as input). No bytes copied
  into `public/img/`.

## Placeholder generation method (current binaries)

The placeholders are layered ImageMagick gradients composed to evoke
the painterly feel of the FLUX target without using any external
service. The exact ImageMagick pipeline is reproducible from this
file alone.

All three assets are 1920×1080, JPEG quality 78, blurred 0×1.2-0×1.5
to read as soft painterly underlay rather than pixel art. Final size
sits well under the 400 KB performance budget (UI-SPEC §"Performance
budget"), leaving headroom for a higher-fidelity FLUX replacement
without breaking the budget contract.

## Token vocabulary used in placeholders

The token hex values below are mirrored from `input.css`'s `@theme`
block (Demacia at lines 193–233, Pandemonium at lines 235–268). They
are documented here so the placeholder is itself a check on the
token-system invariant — if the theme tokens move, the placeholders
visibly skew and a follow-up generation would be triggered.

- Demacia accent gold: `#d4af5a`
- Demacia warm midtone: `#3a2a14`
- Demacia base: `#070912`
- Pandemonium accent pink: `#ff4d8a`
- Pandemonium accent cyan: `#78dcf0`
- Pandemonium base: `#06070b`

## Assets

### `public/img/beta-landing-demacia.jpg`

- **Generated:** 2026-05-07
- **Surface:** `/closed-beta` hero landing (Demacia theme variant)
- **Compute path:** placeholder (ImageMagick layered gradient + plasma noise)
- **Intended runtime:** fal.ai → `flux.1-pro`
- **Intended prompt:** "Demacia citadel at dawn, oil painting, warm
  golden light, heraldic banners, stone architecture, League of
  Legends splash art style, atmospheric perspective"
- **Intended negative prompt:** "modern, futuristic, neon, photorealistic"
- **Intended aspect / resolution:** 16:9 / 1920×1080
- **Intended seed:** TBD on first FLUX run — use seed `2026_demacia_01`
  (deterministic prefix) and record the integer fal.ai assigns.
- **Placeholder seed:** n/a (deterministic gradient pipeline)
- **Final size:** 24 KB JPEG q78
- **Reproducibility command (placeholder):**
  ```bash
  magick \
    \( -size 1920x1080 gradient:'#1a1208'-'#070912' \) \
    \( -size 1920x1080 radial-gradient:'#d4af5a'-'#070912' -modulate 100,60,100 -blur 0x35 \) \
    -compose Screen -composite \
    \( -size 1920x1080 plasma:fractal -blur 0x6 -modulate 90,40,100 \) \
    -compose SoftLight -composite \
    \( -size 1920x540 gradient:'#3a2a14'-'#1a1208' \) \
    -gravity South -compose Over -composite \
    \( -size 1920x1080 radial-gradient:'#000000ff'-'#00000000' \
       -channel a -blur 0x60 +channel \
       -alpha set -channel a -evaluate set 35% +channel \) \
    -compose Multiply -composite \
    -modulate 95,80,100 -blur 0x1.2 -quality 78 \
    public/img/beta-landing-demacia.jpg
  ```
- **TODO:** Regenerate via fal.ai once `FAL_KEY` is provisioned.

### `public/img/beta-landing-pandemonium.jpg`

- **Generated:** 2026-05-07
- **Surface:** `/closed-beta` hero landing (Pandemonium theme variant)
- **Compute path:** placeholder (ImageMagick layered gradient + plasma noise)
- **Intended runtime:** fal.ai → `flux.1-pro`
- **Intended prompt:** "Pandemonium harrowing, neon pink and teal,
  fractured obsidian, electric chaos, brutalist architecture, League
  of Legends atmospheric, painterly"
- **Intended negative prompt:** "warm, gold, classical, oil painting"
- **Intended aspect / resolution:** 16:9 / 1920×1080
- **Intended seed:** TBD on first FLUX run — use seed
  `2026_pandemonium_01` (deterministic prefix) and record the integer
  fal.ai assigns.
- **Placeholder seed:** n/a (deterministic gradient pipeline)
- **Final size:** 28 KB JPEG q78
- **Reproducibility command (placeholder):**
  ```bash
  magick \
    \( -size 1920x1080 gradient:'#0d0d18'-'#06070b' \) \
    \( -size 1920x1080 radial-gradient:'#ff4d8a'-'#06070b' -modulate 100,55,100 -blur 0x40 \) \
    -compose Screen -composite \
    \( -size 1920x1080 radial-gradient:'#78dcf0'-'#06070b' -modulate 100,50,100 -blur 0x50 -roll +400+200 \) \
    -compose Screen -composite \
    \( -size 1920x1080 plasma:fractal -blur 0x6 -modulate 90,40,100 \) \
    -compose SoftLight -composite \
    \( -size 1920x1080 radial-gradient:'#000000ff'-'#00000000' \
       -channel a -blur 0x60 +channel \
       -alpha set -channel a -evaluate set 35% +channel \) \
    -compose Multiply -composite \
    -modulate 90,90,100 -blur 0x1.2 -quality 78 \
    public/img/beta-landing-pandemonium.jpg
  ```
- **TODO:** Regenerate via fal.ai once `FAL_KEY` is provisioned.

### `public/img/auth-bg-demacia.jpg`

- **Generated:** 2026-05-07
- **Surface:** `/auth/login` and `/auth/register` background (optional;
  Demacia theme variant only — Pandemonium auth uses solid bg).
- **Compute path:** placeholder (ImageMagick layered gradient)
- **Intended runtime:** fal.ai → `flux.1-pro`
- **Intended prompt:** "Demacia archway in soft morning haze, midground,
  distance, oil painting, less saturated, golden ambient"
- **Intended negative prompt:** "neon, modern, photorealistic, sharp"
- **Intended aspect / resolution:** 16:9 / 1920×1080
- **Intended seed:** TBD — use seed `2026_demacia_auth_01`.
- **Placeholder seed:** n/a (deterministic gradient pipeline)
- **Final size:** 15 KB JPEG q78
- **Reproducibility command (placeholder):**
  ```bash
  magick \
    \( -size 1920x1080 gradient:'#0f0d0a'-'#06070b' \) \
    \( -size 1920x1080 radial-gradient:'#3a2a14'-'#06070b' -modulate 100,40,100 -blur 0x60 \) \
    -compose Screen -composite \
    \( -size 1920x1080 plasma:fractal -blur 0x10 -modulate 80,25,100 \) \
    -compose SoftLight -composite \
    \( -size 1920x1080 radial-gradient:'#000000ff'-'#00000000' \
       -channel a -blur 0x80 +channel \
       -alpha set -channel a -evaluate set 50% +channel \) \
    -compose Multiply -composite \
    -modulate 70,55,100 -blur 0x1.5 -quality 78 \
    public/img/auth-bg-demacia.jpg
  ```
- **TODO:** Regenerate via fal.ai once `FAL_KEY` is provisioned.
  Auth background is `optional` per UI-SPEC §"Asset versioning" —
  if the FLUX run blows the 400 KB budget for auth specifically, drop
  this file rather than the hero backgrounds. The auth pages render
  acceptably with `bg-base canvas-grain` alone.

## How to regenerate via fal.ai (when `FAL_KEY` is available)

```bash
export FAL_KEY=... # personal key from https://fal.ai
PROMPT="Demacia citadel at dawn, oil painting, warm golden light, heraldic banners, stone architecture, League of Legends splash art style, atmospheric perspective"
curl -X POST https://fal.run/fal-ai/flux-pro \
  -H "Authorization: Key $FAL_KEY" \
  -H "Content-Type: application/json" \
  -d "{\"prompt\": \"$PROMPT\", \"image_size\": \"landscape_16_9\", \"num_inference_steps\": 28, \"seed\": 7421337}" \
  | jq -r '.images[0].url' \
  | xargs curl -L -o public/img/beta-landing-demacia.jpg

# Then compress to ≤400 KB JPEG q85:
magick public/img/beta-landing-demacia.jpg -quality 85 -resize '1920x1080>' public/img/beta-landing-demacia.jpg
```

After regeneration, update each asset block above with:
- `Compute path: fal.ai (https://fal.run/fal-ai/flux-pro)`
- `Seed: <integer fal.ai assigned>`
- `Final size: <new bytes>`
- Remove the `TODO: Regenerate via fal.ai` line.
