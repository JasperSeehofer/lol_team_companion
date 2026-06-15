# Phase 30: Art Production — Research

**Researched:** 2026-06-15
**Domain:** AI art generation pipeline (FLUX.2 [pro] + BiRefNet), per-tier overlay code change (Leptos/parallax), CI budget enforcement
**Confidence:** HIGH (all pipeline specifics verified from STACK.md which was sourced against official docs; all code details verified from codebase)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Path A (original world). No Riot proper nouns or "in the style of [Riot product]" in any committed prompt/asset.
- **D-02:** Grounded in first-party read of Riot API Terms, Legal page, and Developer General Policies. Copyright-derivative exposure removed; "closely resembles Riot's games" clause avoided.
- **D-03:** Invent fictional faction names — Aurelian Reach (warm limestone/gilded dawn) and Sundered Hollow (fractured neon-obsidian). Names pending user sign-off at planning.
- **D-04:** Internal code IDs stay `demacia`/`pandemonium` (dir names, region prop, CSS tokens, data-accent, e2e selectors). Only FLUX prompts and user-facing labels change.
- **D-05:** `/consult legal` is a mandatory gate before the first real FLUX run. Path A decision MUST be logged in `PROJECT.md` Key Decisions first.
- **D-06:** Painterly, Arcane-adjacent fidelity. Generic technique vocabulary only — no "in the style of [proper noun]". Hand-painted, textured, moody.
- **D-07:** Per-room moods locked — War Room: tactical war-table chamber, tense light, maps/banners. Library: scholarly archive, towering shelves, reading light, dust motes. Throne: regal ceremonial hall, throne dais, grand vertical space, heraldry.
- **D-08:** Restyled same composition — same compositional skeleton across regions (sky/mid/near zones align), but materials/arch detail/light genuinely differ. Aurelian Reach = intact warm limestone; Sundered Hollow = fractured neon-lit obsidian. Not a flat color-swap.
- **D-09:** Discrete per-tier overlay art. Produce `overlay-{axis}-{1,2,3}.webp` per growth axis.
- **D-10:** Code change: update each room's overlay markup from one `<img>` faded by opacity to 3 stacked tier images toggled by `data-{axis}-tier`. Applies to all 4 rooms. ROOM-03 hydration rule is a hard constraint.
- **D-11:** Budget rework. 3 planes + 9 per-tier overlays stay ≤ 400 KB/room/region. CI aggregate check per-room/region, not just per-file.
- **D-12:** Generation = FLUX.2 [pro] via BFL EU endpoint (`https://api.eu.bfl.ai/v1/flux-2-pro`, `BFL_KEY`). Layer separation = fal.ai BiRefNet v2 (`fal-ai/birefnet/v2`, `FAL_KEY`) as documented EU-policy exception. Post-process = ImageMagick → WebP. BiRefNet validated on ONE test plane before full run.
- **D-13:** Solo Journey in scope. Regenerate its art AND migrate its overlays (banner/lib/armory) to per-tier model.

### Claude's Discretion

- Exact FLUX prompt wording per room × region × plane (surface in `AI-IMAGES.md`; faction names for user sign-off).
- Final per-file budget caps and CI aggregate-check mechanism.
- Seeds, inference steps, compute-path recording per asset.
- Exact stacking/toggling markup for per-tier overlays (subject to ROOM-03).

### Deferred Ideas (OUT OF SCOPE)

None — all scope expansions were deliberately pulled in.

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ART-01 | Original world & art identity — no trademarked place-names, no "in the style of [Riot product]" in committed assets; `AI-IMAGES.md` prompts revised; `/consult legal` gates the first real generation run | D-01 through D-05 lock the legal approach; SC1 validation = PROJECT.md Key Decisions entry + revised AI-IMAGES.md prompts |
| ART-02 | Layer-separated environment art produced per room (first region) via BFL EU FLUX endpoint + BiRefNet layer separation, CI-enforced ~400 KB/room/region budget | D-12 specifies the exact pipeline; STACK.md has verified request shapes; D-11 specifies CI enforcement |
| ART-03 | Second region art produced for all rooms once first region is locked | Same pipeline as ART-02; triggered after first-region QA passes |

</phase_requirements>

---

## Summary

Phase 30 is a hybrid art-production + code-change phase. The roadmap premise that "art swaps in at existing paths with no code changes" is superseded by decisions D-09 through D-13: the per-tier overlay model requires a Leptos/CSS code change in all four rooms before the new art can be consumed correctly.

The art pipeline is fully specified in STACK.md: FLUX.2 [pro] via BFL EU endpoint (async polling), BiRefNet v2 for background removal, ImageMagick for WebP encoding. The existing STACK.md call patterns are production-ready — the gap is (a) rewriting prompts for Path A / original-world, (b) running the pipeline, and (c) recording seeds/sizes in AI-IMAGES.md.

The code change (D-10) is a surgical mod to `GrowthOverlay` usage in 4 room files plus a new CSS block in `input.css`. The existing single-`<img>`-per-axis pattern (one `GrowthOverlay` per axis, opacity-ramp driven by CSS `data-{axis}-tier`) must be replaced by 3 stacked `<img>` elements per axis — tier-1, tier-2, tier-3 — where exactly one is visible at any tier value. This must be implemented without any reactive structural branching (ROOM-03 hard constraint).

The CI budget check currently enforces per-file size with `find public/img/rooms -name "*.webp" -size +100k` in CI. This is not in the current `.github/workflows/ci.yml` (only style guardrails are in CI); it exists only as a Playwright e2e network-bytes check. A new CI job step must be added.

**Primary recommendation:** Sequence the work as: (1) legal gate + PROJECT.md entry, (2) Wave 0 code change (per-tier overlay markup + CSS + placeholder placeholder WebPs for new paths), (3) FLUX generation first region, (4) BiRefNet + WebP encode + budget verify, (5) second region, (6) AI-IMAGES.md update + CI aggregate check addition.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| FLUX image generation | Offline script (human-in-loop) | — | Generation is offline; images are committed as static assets; server never calls generation APIs |
| BiRefNet background removal | Offline script (fal.ai API) | — | Same: post-process step outside the Rust binary |
| WebP encoding + budget check | Offline script (ImageMagick) | CI (aggregate gate) | Encoding is offline; CI enforces the budget contract |
| Per-tier overlay visibility | Browser / Client (CSS data-attr) | — | `data-{axis}-tier` on root + CSS selector; no JS/WASM needed |
| Growth tier derivation | API / Backend (server fn) | — | `get_growth_state` is a `#[server]` fn; tier math lives in `src/models/growth.rs` (shared) |
| Region selection for asset paths | Frontend Server (SSR) | Browser hydrate | Region read once at room top from `InitialTheme` context; path is a static `String` interpolation |
| Legal gate sequencing | Human decision | — | `/consult legal` + PROJECT.md entry is a human task before any generation |

---

## Standard Stack

### Core (additions only — base Rust/Leptos/Axum/SurrealDB/Tailwind unchanged)

| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| BFL FLUX.2 [pro] | EU endpoint `api.eu.bfl.ai` | Generate environment art | German company, explicit EU endpoint, GDPR-compliant, ~$0.045/image. Model author — prompts transfer exactly. `[CITED: docs.bfl.ml/api_integration]` |
| fal.ai BiRefNet v2 | `fal-ai/birefnet/v2` | Background removal for mid/near/overlay planes | Best-in-class painterly edge handling; returns PNG with alpha; no local Python dep. `[CITED: fal.ai/models/fal-ai/birefnet]` |
| ImageMagick | 6.9+ (already installed) | PNG → WebP encoding, compression | Already installed for placeholder pipeline; supports lossless alpha WebP. `[ASSUMED: system-installed; verified available from placeholder scripts]` |

### New Environment Variables

| Variable | Purpose | Where |
|----------|---------|-------|
| `BFL_KEY` | BFL API key for FLUX.2 [pro] | `.env` (never committed); add to `.env.example` |
| `FAL_KEY` | fal.ai key for BiRefNet | Already in `.env.example` |

Both are offline-script-only. Never loaded by the server binary.

### Package Legitimacy Audit

> No new npm/pip/cargo packages are installed in this phase. The pipeline uses existing CLI tools (curl, ImageMagick) and external HTTP APIs. No package legitimacy check is required.

**Packages removed due to slopcheck [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none

---

## Architecture Patterns

### System Architecture Diagram

```
[Human / offline]
    │
    ▼
/consult legal + PROJECT.md entry ────────────────── SC1 gate
    │
    ▼
scripts/gen-room-art.sh
  POST https://api.eu.bfl.ai/v1/flux-2-pro
    prompt, width=1920, height=1080, steps=28, seed=N
  ← {id, polling_url}
  POLL polling_url until status=="Ready"
  ← {result.sample} (PNG URL)
  curl download → /tmp/<room>/<region>/plane-*.png
    │
    ▼ (for non-sky planes + overlays only)
scripts/remove-bg.sh
  POST https://fal.run/fal-ai/birefnet/v2
    image_url=<public URL or base64>, model="heavy"
  ← {image.url} (transparent PNG)
  curl download → /tmp/<room>/<region>/plane-*-transparent.png
    │
    ▼
scripts/encode-webp.sh
  magick in.png -quality 80 [-define webp:alpha-quality=100] out.webp
  → public/img/rooms/<room>/<region>/{plane-sky,plane-mid,plane-near}.webp
  → public/img/rooms/<room>/<region>/overlay-{axis}-{1,2,3}.webp
    │
    ▼
scripts/verify-budget.sh
  per-room/region aggregate sum ≤ 400 KB
  per-file checks per budget caps
    │
    ▼
Update AI-IMAGES.md (seed, compute-path, final size, remove TODO)
    │
    ▼
[Browser / runtime]
AtmosphereRoot (data-{axis}-tier="N" on root div)
    │
    CSS: [data-{axis}-tier="1"] .room-overlay--{axis}--tier-1 { opacity: 1; }
         [data-{axis}-tier="1"] .room-overlay--{axis}--tier-2 { opacity: 0; }
         [data-{axis}-tier="2"] .room-overlay--{axis}--tier-1 { opacity: 0; }
         [data-{axis}-tier="2"] .room-overlay--{axis}--tier-2 { opacity: 1; }
         ... (tier-3 analogous)
    │
    ▼
<img class="room-overlay room-overlay--{axis}--tier-1" src=".../overlay-{axis}-1.webp" />
<img class="room-overlay room-overlay--{axis}--tier-2" src=".../overlay-{axis}-2.webp" />
<img class="room-overlay room-overlay--{axis}--tier-3" src=".../overlay-{axis}-3.webp" />
```

### Recommended Project Structure

```
public/img/rooms/<room>/<region>/
├── plane-sky.webp              # opaque, sky depth band
├── plane-mid.webp              # transparent, mid depth band
├── plane-near.webp             # transparent, near depth band
├── overlay-{axis}-1.webp       # tier-1 (Sprout) art for this axis
├── overlay-{axis}-2.webp       # tier-2 (Established) art for this axis
└── overlay-{axis}-3.webp       # tier-3 (Flourishing) art for this axis

scripts/
├── gen-room-art.sh             # BFL EU POST + poll + download
├── remove-bg.sh                # fal.ai BiRefNet call + download
├── encode-webp.sh              # ImageMagick PNG→WebP
└── verify-budget.sh            # per-room/region aggregate + per-file checks
```

Rooms × regions: `solo-journey`, `war-room`, `library`, `throne` × `demacia`, `pandemonium`.
Each axis per room: Solo Journey (banner/lib/armory), War Room (tactics/intel/command), Library (records/tomes/analysis), Throne (roster/progression/crown).

---

## Key Research Finding 1: FLUX.2 [pro] BFL EU Pipeline

**Source:** `[CITED: .planning/research/STACK.md — verified against docs.bfl.ml]`

### Request Shape

```bash
# 1. Submit generation
RESPONSE=$(curl -s -X POST https://api.eu.bfl.ai/v1/flux-2-pro \
  -H "x-key: $BFL_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "...",
    "width": 1920, "height": 1080,
    "steps": 28, "seed": 7421337
  }')
POLLING_URL=$(echo $RESPONSE | jq -r '.polling_url')

# 2. Poll until Ready
until [ "$(curl -s -H "x-key: $BFL_KEY" "$POLLING_URL" | jq -r '.status')" = "Ready" ]; do
  sleep 2
done

# 3. Download result (PNG)
curl -s -H "x-key: $BFL_KEY" "$POLLING_URL" | jq -r '.result.sample' | xargs curl -L -o out.png
```

Key points for reproducibility:
- Use the `polling_url` returned from the submit call (not a hardcoded endpoint).
- Header is `x-key`, not `Authorization`.
- Record the integer seed used; include in AI-IMAGES.md as `Seed: <integer>`.
- `steps: 28` is standard; lower to 24 if output exceeds budget after WebP encoding.
- `width: 1920, height: 1080` for 16:9 full-bleed (1920×1080 matches existing placeholder dims).

### Prompt Architecture for Path A / Original-World

Each prompt must:
1. Use Aurelian Reach vocabulary (warm limestone, gilded heraldry, dawn light) or Sundered Hollow vocabulary (fractured neon-obsidian, magenta/cyan arc-light, storm-haze) — NO "Demacia", "Pandemonium", "League of Legends".
2. Specify exactly one depth band per plane: `FAR background only` / `MIDGROUND only on transparent background` / `FOREGROUND only on transparent background`.
3. Use painterly technique vocabulary: `oil painting with visible brushwork`, `hand-painted`, `textured`, `moody atmospheric` — never `in the style of [proper noun]`.
4. For overlays: specify the growth element in isolation (`isolated growth element on transparent background`).

Existing AI-IMAGES.md prompts reference "Demacia", "Pandemonium", and "League of Legends splash art style" — all must be rewritten for Path A before any generation run. `[VERIFIED from codebase: AI-IMAGES.md prompts contain these phrases]`

---

## Key Research Finding 2: BiRefNet v2 Layer Separation

**Source:** `[CITED: .planning/research/STACK.md — verified against fal.ai model page]`

### Request Shape

```bash
# Upload the source PNG to a public URL first (or use fal storage upload)
RESULT=$(curl -s -X POST https://fal.run/fal-ai/birefnet/v2 \
  -H "Authorization: Key $FAL_KEY" \
  -H "Content-Type: application/json" \
  -d '{"image_url": "https://<public-url-to-generated-plane.png>", "model": "heavy"}')
echo $RESULT | jq -r '.image.url' | xargs curl -L -o plane-transparent.png
```

- Use `model: "heavy"` for painterly architectural elements (best edge quality).
- Returns a PNG with alpha channel (transparent background).
- Input: JPEG/PNG/WebP/AVIF up to 2048×2048.
- Apply only to mid/near planes and overlays — sky plane is opaque, no background removal needed.

### EU Policy Exception Requirement (D-12)

fal.ai is US-domiciled. Using it for BiRefNet is a documented exception because parallax plane images contain no personal data — GDPR exposure is zero for decorative art. Exception must be logged in AI-IMAGES.md and per the values-charter exception process. Exact log entry to record: service, justification (no personal data), date.

### Halo Fringe Validation Gate (SC3)

Before committing any BiRefNet output:
1. Open resulting WebP in browser against `background: #070912` (dark base).
2. Play parallax at full amplitude (mouse to corner).
3. Inspect mid/near plane edges at 1× zoom — no visible white/grey halo ring.
4. If halo visible: apply 1px erode mask shrink in ImageMagick before export, OR regenerate source with explicit `isolated on pure black background` prompt, then use luminance-key extraction.

Validate on ONE test plane before the full ~96-asset run. `[CITED: PITFALLS.md Pitfall 9]`

---

## Key Research Finding 3: Per-Tier Overlay Code Change (D-10)

**Source:** `[VERIFIED from codebase: parallax.rs, solo_journey.rs, war_room.rs, library.rs, throne.rs, input.css]`

### Current State (what exists today)

All four rooms use the same single-overlay-per-axis pattern:

```rust
// Current (all 4 rooms) — ONE image per axis, opacity-ramp via CSS
<GrowthOverlay axis="banner" src=format!("{dir}/overlay-banner.webp") z=4 />
<GrowthOverlay axis="lib"    src=format!("{dir}/overlay-lib.webp")    z=5 />
<GrowthOverlay axis="armory" src=format!("{dir}/overlay-armory.webp") z=6 />
```

`GrowthOverlay` in `parallax.rs` renders:
```rust
<img
    src=src
    alt=""
    aria-hidden="true"
    loading="eager"
    class=format!("room-overlay room-overlay--{axis} w-full h-full object-cover")
    style=format!("z-index:{z};")
/>
```

CSS in `input.css` controls opacity by `data-{axis}-tier` on the root:
```css
.room-overlay { opacity: 0; transition: opacity 400ms ease; }
[data-lib-tier="1"] .room-overlay--lib { opacity: 0.34; }
[data-lib-tier="2"] .room-overlay--lib { opacity: 0.67; }
[data-lib-tier="3"] .room-overlay--lib { opacity: 1;    }
```

### Required Change (D-09/D-10)

Replace one-overlay-per-axis with three-overlays-per-axis, where exactly one is visible at each tier value.

**Option A: Extend `GrowthOverlay` component** to accept `tier_count: u8` and render all tiers internally. Then update CSS class naming.

**Option B (recommended — minimal diff):** In each room's view block, replace the single `<GrowthOverlay>` call with three calls using a new tier-specific naming convention. The `GrowthOverlay` component itself does not change; the class naming differentiates the tiers.

Recommended approach (Option B — keeps `parallax.rs` untouched):

```rust
// NEW: 3 stacked tier images per axis — tier 0 = all hidden (CSS default)
// axis="lib--tier-1" means class "room-overlay room-overlay--lib--tier-1"
<GrowthOverlay axis="lib--tier-1" src=format!("{dir}/overlay-lib-1.webp") z=5 />
<GrowthOverlay axis="lib--tier-2" src=format!("{dir}/overlay-lib-2.webp") z=5 />
<GrowthOverlay axis="lib--tier-3" src=format!("{dir}/overlay-lib-3.webp") z=5 />
```

New CSS block in `input.css` (one pattern per axis, replaces old opacity-ramp):
```css
/* Per-tier discrete overlays: exactly one tier image visible at a time */
/* lib axis */
[data-lib-tier="0"] .room-overlay--lib--tier-1 { opacity: 0; }
[data-lib-tier="1"] .room-overlay--lib--tier-1 { opacity: 1; }
[data-lib-tier="2"] .room-overlay--lib--tier-1 { opacity: 0; }
[data-lib-tier="3"] .room-overlay--lib--tier-1 { opacity: 0; }
[data-lib-tier="1"] .room-overlay--lib--tier-2 { opacity: 0; }
[data-lib-tier="2"] .room-overlay--lib--tier-2 { opacity: 1; }
[data-lib-tier="3"] .room-overlay--lib--tier-2 { opacity: 0; }
[data-lib-tier="2"] .room-overlay--lib--tier-3 { opacity: 0; }
[data-lib-tier="3"] .room-overlay--lib--tier-3 { opacity: 1; }
/* (all overlays default opacity: 0 from .room-overlay base rule) */
```

Repeat for all 12 axes (4 rooms × 3 axes each).

**ROOM-03 compliance:** All tier `<img>` elements are always present in the DOM at SSR time. No reactive structural branching. The tier selection is pure CSS `data-*-tier` on the root — identical SSR and WASM structure. This is the same pattern as the current system. `[VERIFIED: parallax.rs GrowthOverlay doc comment]`

**No change needed to `GrowthOverlay` component or `AtmosphereRoot`** — the `data-{axis}-tier` attribute system already exists for all 12 axes.

### Files to Change

| File | Change |
|------|--------|
| `src/pages/rooms/solo_journey.rs` | Replace 3 `GrowthOverlay` calls with 9 (3 per axis: banner, lib, armory) |
| `src/pages/rooms/war_room.rs` | Replace 3 `GrowthOverlay` calls with 9 (3 per axis: tactics, intel, command) |
| `src/pages/rooms/library.rs` | Replace 3 `GrowthOverlay` calls with 9 (3 per axis: records, tomes, analysis) |
| `src/pages/rooms/throne.rs` | Replace 3 `GrowthOverlay` calls with 9 (3 per axis: roster, progression, crown) |
| `input.css` | Add per-tier CSS block (12 axes × 3 tiers each = 36 new rule blocks). Remove or keep old opacity-ramp rules (keep if backward-compatible with placeholder assets; remove once FLUX assets are committed). |

`[VERIFIED from codebase: exact current overlay markup in all 4 room files]`

### Solo Journey overlay names

Solo Journey axes: `banner`, `lib`, `armory`. New file paths:
- `overlay-banner-1.webp`, `overlay-banner-2.webp`, `overlay-banner-3.webp`
- `overlay-lib-1.webp`, `overlay-lib-2.webp`, `overlay-lib-3.webp`
- `overlay-armory-1.webp`, `overlay-armory-2.webp`, `overlay-armory-3.webp`

(Old `overlay-banner.webp`, `overlay-lib.webp`, `overlay-armory.webp` become dead files — remove or keep as fallback during transition.)

---

## Key Research Finding 4: Budget Rework + CI Aggregate Check (D-11)

**Source:** `[VERIFIED from codebase: .github/workflows/ci.yml, e2e/tests/room-solo-journey.spec.ts]`

### Current CI State

The `.github/workflows/ci.yml` contains only:
- `check` job: `cargo check` (SSR + WASM), `cargo test`, `cargo clippy`, `cargo fmt --check`
- `style_guardrails` job: G-01 (no Google Fonts), G-12 (no outline:none), raw-hex check

**There is no asset budget CI step today.** The only budget check is the Playwright e2e test in `room-solo-journey.spec.ts` ("room art stays within the 400 KB asset budget") which measures network bytes at runtime. This is not a CI gate on committed files.

The CONTEXT.md and PITFALLS.md reference `find public/img/rooms -name "*.webp" -size +100k` as the intended CI check, but it does not exist in `ci.yml` yet.

### Budget Arithmetic

Current (placeholder): 6 files/room/region × ~4-20 KB each ≈ 52 KB per room per region (well under 400 KB).

Post-D-09 structure: 12 files/room/region (3 planes + 9 overlays). Budget ≤ 400 KB/room/region.

Recommended per-file caps (planner's discretion per CONTEXT.md, but providing recommended values):

| File type | Count/room/region | Recommended cap | Rationale |
|-----------|-------------------|-----------------|-----------|
| `plane-sky.webp` | 1 | 120 KB | Largest, opaque, most detail; FLUX at q75-80 |
| `plane-mid.webp` | 1 | 100 KB | Transparent, slightly less detail needed |
| `plane-near.webp` | 1 | 80 KB | Transparent foreground element; smallest depth |
| `overlay-{axis}-{tier}.webp` | 9 | 11 KB each | Isolated growth element, small canvas coverage |

**Total: 120 + 100 + 80 + (9 × 11) = 399 KB** — just under 400 KB.

If FLUX planes run large, compress to q65-70 or resize to 1600×900 (CSS `object-cover` handles scale). `[CITED: PITFALLS.md Pitfall 14]`

### New CI Job Step (to add to ci.yml)

```yaml
- name: Room art asset budget (per-file + per-room/region aggregate)
  run: |
    FAIL=0
    # Per-file cap: individual file hard limits
    if find public/img/rooms -name "*.webp" -size +120k | grep -q .; then
      echo "::error::Room art file exceeds 120 KB cap"
      find public/img/rooms -name "*.webp" -size +120k
      FAIL=1
    fi
    # Per-room/region aggregate ≤ 400 KB
    for room in public/img/rooms/*/; do
      for region in "${room}"*/; do
        total=$(find "$region" -name "*.webp" -exec du -b {} \; | awk '{sum+=$1} END{print sum+0}')
        if [ "$total" -gt 409600 ]; then
          echo "::error::$region exceeds 400 KB budget (${total} bytes)"
          FAIL=1
        fi
      done
    done
    exit $FAIL
```

This adds a second job `asset_budget` parallel to `check` and `style_guardrails`. It checks:
1. No single file > 120 KB (the hardest per-file limit, sky plane cap).
2. Per-room/region aggregate ≤ 400 KB (400 × 1024 = 409600 bytes).

The Playwright e2e network-bytes check remains as a runtime validation (measures what the browser actually fetches; the CI check measures what is committed).

---

## Key Research Finding 5: Legal Gate Sequencing (SC1)

**Source:** `[VERIFIED from codebase: STATE.md, CONTEXT.md D-05, REQUIREMENTS.md ART-01]`

SC1 requires two events to happen BEFORE any real FLUX generation task runs:
1. `/consult legal` run (the formal review session).
2. `PROJECT.md` Key Decisions entry: "AI art IP: Path A (original world), decided [date], basis: [Riot API Terms / Legal page / General Policies read + discussion]."

In the plan, this must be structured as a Wave 0 or Wave 1 gating task with a `checkpoint:human-verify` annotation. No generation tasks can be in the same wave or any earlier wave than this gate.

The `AI-IMAGES.md` prompt revision (rewrite all prompts to remove Riot proper nouns and "splash art style" phrasing) must also happen in this pre-generation wave, so the generation scripts have Path A prompts from the start.

Riot canonical references (for the `/consult legal` run):
- https://developer.riotgames.com/terms
- https://www.riotgames.com/en/legal
- https://developer.riotgames.com/policies/general

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Background removal from generated planes | Custom depth-map cutting or local rembg | fal.ai BiRefNet v2 | Continuous depth fields composite poorly with painterly art; BiRefNet handles painterly edges; no Python dep |
| WebP encoding with alpha | Custom codec | ImageMagick (`-define webp:alpha-quality=100`) | Already installed; handles lossless alpha + lossy RGB split |
| Per-tier CSS toggle | JS reactive signal or Leptos reactive branch | CSS `data-{axis}-tier` attribute + CSS selectors | Reactive branching causes hydration panic (ROOM-03 lesson); pure CSS is already the established pattern |
| Budget enforcement | Runtime-only check | CI step + e2e network check | Runtime check catches browser fetches; CI check catches committed oversized files before deployment |
| Generation API from Rust | `reqwest` in server code | Shell scripts (offline, human-in-loop) | Assets are committed to git; generation is a one-time offline operation, not a server concern |

---

## Common Pitfalls

### Pitfall 1: Reactive Structural Branching in Tier Overlays

**What goes wrong:** `if tier >= 1 { view! { <img .../> } }` inside a room view causes SSR/WASM DOM structure mismatch → tachys panic at `html/mod.rs:217`.
**Why:** The Phase 18.2 incident — the tier value is different at SSR time vs. WASM hydration time.
**How to avoid:** All 9 per-tier `<img>` elements for all axes must be unconditionally present in the DOM. CSS `data-{axis}-tier` selectors toggle visibility. No `if/match` on tier values inside `view!`.
**Source:** `[VERIFIED: ROOM-03 in leptos-patterns.md and CLAUDE.md gotcha #58]`

### Pitfall 2: Region Read Inside Reactive Closure

**What goes wrong:** Reading `use_context::<InitialTheme>()` inside a `move || {}`, `Signal::derive()`, or child component causes hydration panic.
**How to avoid:** Region read exactly once at room top level (already correct in all 4 rooms). The D-10 code change must not move region reading into any closure. The `dir = format!("{dir}/overlay-{axis}-{tier}.webp")` constructions are all static at component construction time — no reactive closure needed.
**Source:** `[VERIFIED: solo_journey.rs:18 and leptos-patterns.md rule 58]`

### Pitfall 3: FLUX Asset Budget Overrun

**What goes wrong:** FLUX generates plane-sky.webp at 200-400 KB; 3 planes alone blow the 400 KB/room/region budget.
**How to avoid:** After generation, immediately run `verify-budget.sh`. Recompress to q65-70 or resize to 1600×900 if over cap. Run budget check before committing any WebP.
**Source:** `[CITED: PITFALLS.md Pitfall 14]`

### Pitfall 4: BiRefNet Halo Fringe on Painterly Edges

**What goes wrong:** Background removal from non-green-screen source produces 1-3 px white/grey halo ring around pillar/architecture edges, visible during parallax motion.
**How to avoid:** (1) Generate planes with `isolated on pure black background` prompt so luminance-key extraction works; (2) apply 1px erode mask shrink before WebP export; (3) validate visually against `#070912` base with parallax at max amplitude BEFORE committing.
**Source:** `[CITED: PITFALLS.md Pitfall 9]`

### Pitfall 5: Old overlay-{axis}.webp Files Left as Dead Assets

**What goes wrong:** After migrating to `overlay-{axis}-{1,2,3}.webp`, the old `overlay-{axis}.webp` files remain at their paths. They are loaded by the old CSS rules (if those remain) and counted against the budget.
**How to avoid:** In the same commit that changes the room Rust files and CSS, remove the old single-overlay placeholder files. Update `AI-IMAGES.md` to reflect the new naming convention.

### Pitfall 6: Missing Tier-0 State in CSS

**What goes wrong:** If no explicit `[data-{axis}-tier="0"]` rule is written, tier-0 relies on `.room-overlay` default `opacity: 0`. If the new per-tier CSS rules accidentally override to `opacity: 1` when no data-attr matches, tier-0 rooms show overlay art.
**How to avoid:** The new CSS must not add any rule that fires when `data-{axis}-tier` is absent or "0". The `.room-overlay { opacity: 0; }` base rule covers this. Only add explicit overrides for tier-1, tier-2, tier-3.

### Pitfall 7: fal.ai Image URL Expiration for BiRefNet Input

**What goes wrong:** BiRefNet requires a `image_url` parameter — a publicly accessible URL. Locally generated PNG files don't have public URLs. Using a temporary file host whose URLs expire before the pipeline completes.
**How to avoid:** Use fal.ai's own storage API to upload the generated PNG before calling BiRefNet, or host temporarily via `ngrok` / upload to a public S3 bucket during the offline pipeline run. The fal.ai SDK provides `fal.storage.upload()`. Alternatively, use base64 encoding if the API supports it.

---

## Runtime State Inventory

> This phase does not rename or migrate any existing code identifiers. Region IDs (`demacia`/`pandemonium`), CSS class names, data-attr names, and server function names are all unchanged (D-04). No runtime state migration is required.

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None — growth counts in SurrealDB are keyed by user/team, not by region or art path | None |
| Live service config | None — no external service config references room asset paths | None |
| OS-registered state | None | None |
| Secrets/env vars | `BFL_KEY` (new), `FAL_KEY` (exists in `.env.example`) | Add `BFL_KEY` placeholder to `.env.example` |
| Build artifacts | Old `overlay-{axis}.webp` files (6 per room × 4 rooms × 2 regions = 48 files) become dead assets after D-10 | Delete old overlay files when migrating to per-tier naming |

---

## State of the Art

| Old Approach | Current Approach | Impact |
|--------------|------------------|--------|
| Single overlay per axis, opacity ramp (0→0.34→0.67→1.0) | Discrete per-tier art (overlay-{axis}-{1,2,3}.webp), one shown at a time | "Shelves visibly filling" rather than "gradually brightening"; richer visual growth |
| FLUX.1 [pro] via fal.ai runner | FLUX.2 [pro] via BFL EU endpoint directly | Better quality at same price; EU data residency; BFL is the model author |
| Placeholders as ImageMagick gradients | Real FLUX-generated painterly art | Establishes original visual identity; eliminates placeholder gradient appearance |
| Per-file budget check only | Per-file + per-room/region aggregate CI gate | Guarantees total budget is not broken even if no single file exceeds individual cap |

---

## Validation Architecture

Maps each Success Criterion to a concrete, executable validation method.

### SC1: Legal Gate + Path A Decision Logged

**What:** `/consult legal` run completed; `PROJECT.md` Key Decisions has an "AI art IP: Path A" entry; `AI-IMAGES.md` prompts revised to remove all Riot proper nouns and "splash art style".

**Validation method:** Manual human gate — a `checkpoint:human-verify` task that checks:
1. `PROJECT.md` grep for "Path A" in Key Decisions section.
2. `AI-IMAGES.md` grep (negative) for "Demacia", "Pandemonium", "League of Legends splash art style".

**Automated check:**
```bash
grep -q "Path A" .planning/PROJECT.md || exit 1
! grep -iE "demacia|pandemonium|splash art style" .planning/assets/AI-IMAGES.md || exit 1
```

**Sequencing:** Must pass before any generation script is run. Wave 0 gate.

---

### SC2: Visual Distinctness (Aurelian Reach vs. Sundered Hollow read as distinct, not color-swap)

**What:** Each room's two region variants are visually distinguishable — materials, light, architecture differ beyond a tint shift. Parallax alignment preserved (sky/mid/near zones align across regions so surface panels land correctly).

**Validation method:** Manual human review — screenshots of each room in both regions side-by-side, comparing:
- Sky plane: warm limestone/dawn gold (Reach) vs. fractured obsidian/cyan storm (Hollow)
- Mid plane: intact stone pillars (Reach) vs. cracked dark glass (Hollow)
- Tier overlays: gilded heraldry filling (Reach) vs. arc-light elements filling (Hollow)

**Automated proxy:** e2e hydration-no-panic tests run in both `demacia` and `pandemonium` themes (already required by A11Y-02; confirm they still pass after art swap).

**Acceptance criteria:** A reviewer who has not seen the prompts can correctly label which screenshot is which region purely from visual content.

---

### SC3: Pipeline Integrity + BiRefNet Halo Validation

**What:** (a) BiRefNet tested on ONE test plane before full run; (b) all 96 assets generated, halo-free, within budget; (c) each asset recorded in AI-IMAGES.md with seed + compute-path + final size.

**Validation: Part A — halo test (before full run)**
1. Generate ONE plane (e.g. War Room Aurelian Reach mid plane).
2. Run BiRefNet on it.
3. Open result in browser against `background: #070912`.
4. Move mouse to corner (full parallax amplitude).
5. Inspect edge at 1× zoom — confirm no halo ring.

**Validation: Part B — budget gate (after each room/region)**
```bash
# Per-file check
find public/img/rooms -name "*.webp" -size +120k  # must return empty
# Per-room/region aggregate
for room in public/img/rooms/*/; do
  for region in "${room}"*/; do
    total=$(find "$region" -name "*.webp" -exec du -b {} \; | awk '{sum+=$1} END{print sum+0}')
    echo "$region: $total bytes"
    [ "$total" -le 409600 ] || echo "OVER BUDGET: $region"
  done
done
```

**Validation: Part C — AI-IMAGES.md completeness**
```bash
# Count entries in AI-IMAGES.md vs. count of committed WebP files
webp_count=$(find public/img/rooms -name "*.webp" | wc -l)
# Each entry has a "Seed:" line — count those
entry_count=$(grep -c "^- \*\*Seed:\*\*" .planning/assets/AI-IMAGES.md 2>/dev/null || echo 0)
[ "$entry_count" -ge "$webp_count" ] || echo "AI-IMAGES.md entries incomplete"
```

---

### SC4: Per-Tier Overlay Code Change — Correct Behavior

**What:** All 4 rooms render 3 stacked tier images per axis; the correct tier image is visible (others hidden) at each tier value; no hydration panic in either region.

**Validation method (e2e):** Extend the existing room e2e tests (one per room) with a tier-overlay assertion. Use the existing `seedTier1Team` / `seedTier1Growth` fixtures from `e2e/tests/fixtures.ts`.

```typescript
// Pattern: seed tier ≥ 1, navigate to room, assert correct overlay visible
test("per-tier overlay: tier-1 shows tier-1 art, hides tier-2/3", async ({ authedPage }) => {
  await seedTier1Growth(authedPage); // seeds growth to tier ≥ 1
  await authedPage.goto("/journey");
  await authedPage.waitForLoadState("networkidle");
  await authedPage.waitForTimeout(800);

  const root = authedPage.locator(".room-atmosphere");
  const libTier = parseInt(await root.getAttribute("data-lib-tier") ?? "0", 10);

  if (libTier >= 1) {
    // Tier-1 image for lib axis should be visible (opacity ≠ 0 via CSS)
    const tier1Overlay = authedPage.locator(`.room-overlay--lib--tier-1`);
    // CSS-driven opacity; check computed style
    const opacity = await tier1Overlay.evaluate(
      el => getComputedStyle(el).opacity
    );
    expect(parseFloat(opacity)).toBeGreaterThan(0);
    // Tier-2 and tier-3 should be hidden at tier-1
    if (libTier === 1) {
      const tier2Opacity = await authedPage.locator(`.room-overlay--lib--tier-2`)
        .evaluate(el => getComputedStyle(el).opacity);
      expect(parseFloat(tier2Opacity)).toBe(0);
    }
  }
});
```

**Hydration no-panic** (already existing via `hydration-no-panic.spec.ts`): must remain green after D-10 code change in both regions.

**The Phase 27 lesson (from MEMORY.md):** "room growth-tier SCs silently fail without input.css tier→opacity selectors + a seed-and-assert-opacity e2e test." This phase must create `overlay-{axis}--tier-{N}` CSS selectors AND a seed-and-assert e2e test for at least one room (recommend Solo Journey as it has the most complete existing test suite).

---

## Validation Architecture: Test Framework

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Playwright (TypeScript) — `e2e/tests/` |
| Config file | `e2e/playwright.config.ts` |
| Quick run command | `cd e2e && npx playwright test room-solo-journey.spec.ts` |
| Full suite command | `cd e2e && npx playwright test` |
| Unit tests | `cargo test --features ssr --lib` (growth.rs tier logic already tested) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File |
|--------|----------|-----------|-------------------|------|
| ART-01 | No Riot proper nouns in AI-IMAGES.md / PROJECT.md Path A entry | Manual gate + shell grep | `grep -q "Path A" .planning/PROJECT.md` | Wave 0 gate |
| ART-01 | AI-IMAGES.md prompts revised (no "League of Legends", "Demacia", "Pandemonium", "splash art") | Shell grep (negative) | `! grep -iE "demacia\|pandemonium\|splash art style" .planning/assets/AI-IMAGES.md` | Wave 0 gate |
| ART-02 | Per-file budget: no file > 120 KB | CI (new job step) | `find public/img/rooms -name "*.webp" -size +120k` returns empty | ci.yml new step |
| ART-02 | Per-room/region aggregate ≤ 400 KB | CI (new job step) | aggregate sum via du | ci.yml new step |
| ART-02 | Layer separation: no halo fringe on mid/near planes | Manual visual QA | Open in browser against dark bg, parallax at max | Wave 2 gate (before full run) |
| ART-02 | All 96 assets recorded in AI-IMAGES.md | Shell count | `grep -c "Seed:" AI-IMAGES.md` ≥ asset count | Post-generation task |
| ART-02 | D-10 per-tier overlay: correct tier visible | e2e (new test) | `cd e2e && npx playwright test room-solo-journey.spec.ts -g "per-tier overlay"` | New test file |
| ART-02 | D-10 per-tier overlay: no hydration panic | e2e (existing) | `cd e2e && npx playwright test hydration-no-panic.spec.ts` | Existing test |
| ART-03 | Second region art generated and within budget | Same as ART-02 checks | Same CI job step | After ART-02 complete |

### Sampling Rate

- **Per task commit:** `cargo check --features ssr` (code changes) + `find public/img/rooms -name "*.webp" -size +120k` (art commits)
- **Per wave merge:** `cd e2e && npx playwright test room-solo-journey.spec.ts hydration-no-panic.spec.ts`
- **Phase gate:** Full CI green (check + style_guardrails + asset_budget) + Playwright full suite green

### Wave 0 Gaps (tasks before implementation can start)

- [ ] Legal gate task: `/consult legal` run + `PROJECT.md` Key Decisions entry (human-in-loop, `checkpoint:human-verify`)
- [ ] `AI-IMAGES.md` prompt rewrite (Path A vocabulary — Aurelian Reach / Sundered Hollow; faction names for user sign-off)
- [ ] Per-tier overlay CSS block in `input.css` (new selectors for all 12 axes × 3 tiers)
- [ ] Placeholder WebP files for 9 new overlay paths per room per region (108 new placeholder files, replacing 36 old overlay files) — so the code change compiles and e2e smoke passes before real art
- [ ] New `asset_budget` CI job step in `ci.yml`
- [ ] E2e per-tier overlay assertion test (at least for Solo Journey)

*(D-10 code changes in the 4 room files are also Wave 0 — they must land before any art is committed so the correct file paths exist.)*

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| ImageMagick | WebP encoding | ✓ | 6.9+ (system; used for placeholder pipeline) | None needed |
| curl + jq | BFL API polling + BiRefNet calls | ✓ | System tools | None needed |
| BFL_KEY | FLUX generation | Unknown (offline key) | — | Human must provision before generation wave |
| FAL_KEY | BiRefNet | Unknown (offline key) | — | Human must provision before BiRefNet wave |
| Playwright / Chromium | e2e tests | ✓ | (e2e/node_modules installed) | — |
| cargo (nightly) | Code changes | ✓ | Nightly (pinned) | — |

**Missing dependencies with no fallback:**
- `BFL_KEY` — required for FLUX generation; planner must add a `checkpoint:human-verify` gate before any generation task that requires it.
- `FAL_KEY` — required for BiRefNet; same gate.

**Missing dependencies with fallback:**
- None (all pipeline tools available; API keys are the only gap).

---

## Security Domain

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | — |
| V3 Session Management | No | — |
| V4 Access Control | No | — |
| V5 Input Validation | Partial | `region` prop used to construct asset paths must come from server-validated `InitialTheme` context, not URL params (already correct in all 4 rooms) |
| V6 Cryptography | No | — |

| Threat Pattern | STRIDE | Mitigation |
|----------------|--------|-----------|
| Path traversal via `region` in asset src | Tampering | `region` is read from `InitialTheme` context (server-set cookie value) — not from user-supplied URL params. Already enforced in all room components. `[VERIFIED from codebase]` |
| Committed assets without provenance | Repudiation | Each asset must have an AI-IMAGES.md entry with prompt + seed + generation date before commit. CI check enforces entry count ≥ asset count. |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | ImageMagick is available on the machine that runs the offline pipeline | Standard Stack | Pipeline fails; install `imagemagick` package |
| A2 | fal.ai BiRefNet v2 accepts PNG uploads via public URL or fal storage upload | Key Finding 2 | May need to add fal storage upload step to pipeline script |
| A3 | FLUX.2 [pro] returns `result.sample` as the image URL in polling response | Key Finding 1 | Download step fails; check actual BFL API response shape against current docs |
| A4 | The per-tier CSS naming convention `room-overlay--{axis}--tier-{N}` (double dash before tier) will be used | Key Finding 3 | If planner chooses different naming, CSS and Rust must stay in sync |
| A5 | Existing 36 placeholder overlay files (overlay-{axis}.webp, one per axis) will be deleted when the per-tier files are added | Runtime State Inventory | Dead files inflate asset count; old CSS rules may cause confusion |

---

## Open Questions

1. **Faction name sign-off (D-03)**
   - What we know: "Aurelian Reach" and "Sundered Hollow" are drafts.
   - What's unclear: User has not formally approved these names.
   - Recommendation: Add an explicit checkpoint in Wave 0 for user sign-off on faction names before prompts are finalized.

2. **fal.ai image upload mechanism for BiRefNet**
   - What we know: BiRefNet requires a public `image_url`.
   - What's unclear: Whether FLUX-generated PNGs can be uploaded directly to fal.ai storage or require a third-party host.
   - Recommendation: Research fal.ai storage API (`fal.storage.upload`) during Wave 2 before running BiRefNet at scale. If unavailable, use a temporary S3 bucket or `ngrok` tunnel.

3. **Exact per-file caps (D-11)**
   - What we know: 400 KB/room/region total; 3 planes + 9 overlays = 12 files.
   - What's unclear: Whether the recommended caps (120/100/80/11 KB) are achievable with FLUX at q80 or require q65-70.
   - Recommendation: Run ONE test generation (single plane) and measure file size before committing to caps. Adjust in the CI step accordingly.

4. **AI-IMAGES.md structure for 96 assets**
   - What we know: Current format has one named section per asset with prompt/seed/size.
   - What's unclear: With 96 assets, the file becomes very long. Consider a tabular format or separate per-room files.
   - Recommendation: Planner may structure as `AI-IMAGES-<room>.md` files; or add a summary table at top with detailed entries below.

---

## Sources

### Primary (HIGH confidence)

- `.planning/research/STACK.md` — BFL EU endpoint, BiRefNet v2 call patterns, ImageMagick WebP flags, env vars, scripts layout. Sourced from official BFL API docs + fal.ai model page.
- `.planning/phases/30-art-production/30-CONTEXT.md` — All locked decisions (D-01 through D-13).
- `src/components/room/parallax.rs` — `GrowthOverlay` component definition (current markup), `AtmosphereRoot` (data-tier attrs), `Plane` component.
- `src/pages/rooms/solo_journey.rs`, `war_room.rs`, `library.rs`, `throne.rs` — Current overlay markup in all 4 rooms.
- `input.css` (lines 386–494) — Current room CSS: `.room-overlay`, `.room-plane`, `data-{axis}-tier` opacity rules.
- `src/models/growth.rs` — `Tier::level()` (0..=3), tier thresholds, all 4 growth state types.
- `.github/workflows/ci.yml` — Confirmed: no asset budget CI step exists today.
- `e2e/tests/room-solo-journey.spec.ts` — Existing budget check pattern (runtime network bytes), tier attribute assertions.
- `.claude/rules/leptos-patterns.md` — ROOM-03 rule (rule 58).
- `CLAUDE.md` — Gotcha #58 (region-read-once).

### Secondary (MEDIUM confidence)

- `.planning/research/PITFALLS.md` — Pitfalls 1, 9, 14 (hydration, halo artifacts, budget overrun). MEDIUM — sourced from codebase analysis + referenced web sources.
- `.planning/assets/AI-IMAGES.md` — Existing prompt vocabulary (to be replaced), placeholder pipeline pattern, per-region 400 KB budget.

---

## Metadata

**Confidence breakdown:**
- Art pipeline (FLUX call shape, BiRefNet call shape): HIGH — verified from STACK.md which was sourced against official BFL and fal.ai docs.
- Code change (D-10): HIGH — verified exact current markup in all 4 room files from codebase.
- Budget arithmetic: MEDIUM — recommended caps are engineering estimates; actual FLUX output sizes unknown until first generation run.
- CI aggregate check: HIGH — confirmed gap exists (no budget step in ci.yml); proposed bash logic is standard.
- Legal gate: HIGH — SC1 gate requirement confirmed in CONTEXT.md, STATE.md, and REQUIREMENTS.md.

**Research date:** 2026-06-15
**Valid until:** 2026-07-15 (30 days for stable stack; BFL/fal.ai API endpoints may update)
