# Phase 30: Art Production - Pattern Map

**Mapped:** 2026-06-15
**Files analyzed:** 8 new/modified artifacts
**Analogs found:** 8 / 8

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/pages/rooms/solo_journey.rs` (overlay section) | component | event-driven (CSS data-attr) | itself (current overlay section, lines 63–66) | self-modify |
| `src/pages/rooms/war_room.rs` (overlay section) | component | event-driven (CSS data-attr) | `solo_journey.rs` lines 63–66; own lines 108–110 | exact |
| `src/pages/rooms/library.rs` (overlay section) | component | event-driven (CSS data-attr) | `solo_journey.rs` lines 63–66; own lines 268–270 | exact |
| `src/pages/rooms/throne.rs` (overlay section) | component | event-driven (CSS data-attr) | `solo_journey.rs` lines 63–66; own lines 231–233 | exact |
| `input.css` (per-tier CSS block) | config/style | event-driven (CSS data-attr) | `input.css` lines 425–474 (existing opacity-ramp block) | exact |
| `scripts/gen-room-art.sh`, `remove-bg.sh`, `encode-webp.sh`, `verify-budget.sh` | utility (offline scripts) | batch / file-I/O | `scripts/flow_test.sh` (bash structure, curl patterns); `AI-IMAGES.md` (ImageMagick pipeline) | role-match |
| `.github/workflows/ci.yml` (new `asset_budget` job) | config (CI) | batch | existing `style_guardrails` job (lines 31–95) | exact |
| `e2e/tests/room-solo-journey.spec.ts` (new per-tier overlay test) | test | event-driven | existing tests in `room-solo-journey.spec.ts` lines 103–119; `fixtures.ts` `seedTier1Team` | exact |
| `.planning/assets/AI-IMAGES.md` (provenance entries) | documentation | — | `AI-IMAGES.md` lines 65–97 (existing room-plane entry format) | exact |

---

## Pattern Assignments

### Room files: per-tier `GrowthOverlay` markup (D-10)

**Applies to:** `src/pages/rooms/solo_journey.rs`, `war_room.rs`, `library.rs`, `throne.rs`

**Analog (current pattern to REPLACE):** `src/pages/rooms/solo_journey.rs` lines 63–66

Current single-overlay-per-axis pattern:
```rust
// ── Growth overlays (fade in by tier via CSS data-*-tier) ───────
<GrowthOverlay axis="banner" src=format!("{dir}/overlay-banner.webp") z=4 />
<GrowthOverlay axis="lib"    src=format!("{dir}/overlay-lib.webp")    z=5 />
<GrowthOverlay axis="armory" src=format!("{dir}/overlay-armory.webp") z=6 />
```

**Same pattern in war_room.rs lines 108–110:**
```rust
<GrowthOverlay axis="tactics" src=format!("{dir}/overlay-tactics.webp") z=4 />
<GrowthOverlay axis="intel"   src=format!("{dir}/overlay-intel.webp")   z=5 />
<GrowthOverlay axis="command" src=format!("{dir}/overlay-command.webp") z=6 />
```

**Same pattern in library.rs lines 268–270:**
```rust
<GrowthOverlay axis="records"  src=format!("{dir}/overlay-records.webp")  z=4 />
<GrowthOverlay axis="tomes"    src=format!("{dir}/overlay-tomes.webp")    z=5 />
<GrowthOverlay axis="analysis" src=format!("{dir}/overlay-analysis.webp") z=6 />
```

**Same pattern in throne.rs lines 231–233:**
```rust
<GrowthOverlay axis="roster"      src=format!("{dir}/overlay-roster.webp")      z=4 />
<GrowthOverlay axis="progression" src=format!("{dir}/overlay-progression.webp") z=5 />
<GrowthOverlay axis="crown"       src=format!("{dir}/overlay-crown.webp")       z=6 />
```

**`GrowthOverlay` component renders** (`src/components/room/parallax.rs` lines 383–405):
```rust
#[component]
pub fn GrowthOverlay(
    #[prop(into)]
    axis: String,
    #[prop(into)] src: String,
    z: i32,
) -> impl IntoView {
    view! {
        <img
            src=src
            alt=""
            aria-hidden="true"
            loading="eager"
            class=format!("room-overlay room-overlay--{axis} w-full h-full object-cover")
            style=format!("z-index:{z};")
        />
    }
}
```

**New per-tier pattern (D-09/D-10 — three calls per axis, axis key encodes tier):**
The `axis` prop value drives the CSS class `room-overlay--{axis}`. Use `axis="lib--tier-1"` etc. to produce distinct CSS classes per tier. The `GrowthOverlay` component is NOT changed; only the call sites change.

```rust
// NEW: Solo Journey (banner/lib/armory) — 3 stacked tiers per axis
<GrowthOverlay axis="banner--tier-1" src=format!("{dir}/overlay-banner-1.webp") z=4 />
<GrowthOverlay axis="banner--tier-2" src=format!("{dir}/overlay-banner-2.webp") z=4 />
<GrowthOverlay axis="banner--tier-3" src=format!("{dir}/overlay-banner-3.webp") z=4 />
<GrowthOverlay axis="lib--tier-1"    src=format!("{dir}/overlay-lib-1.webp")    z=5 />
<GrowthOverlay axis="lib--tier-2"    src=format!("{dir}/overlay-lib-2.webp")    z=5 />
<GrowthOverlay axis="lib--tier-3"    src=format!("{dir}/overlay-lib-3.webp")    z=5 />
<GrowthOverlay axis="armory--tier-1" src=format!("{dir}/overlay-armory-1.webp") z=6 />
<GrowthOverlay axis="armory--tier-2" src=format!("{dir}/overlay-armory-2.webp") z=6 />
<GrowthOverlay axis="armory--tier-3" src=format!("{dir}/overlay-armory-3.webp") z=6 />
```

**ROOM-03 compliance check:** All 9 `<img>` elements are statically present in the DOM — no `if`, `match`, or reactive branch on tier values. The tier selection is pure CSS only. The `dir` binding is constructed at component entry from `region` (read once at room top level, lines 46 and 18 in `solo_journey.rs`). No reactive closure wraps the `format!` calls. This is identical in structure to the current single-overlay approach — just more calls.

**Reminder — `region` read pattern (CLAUDE.md #58 / ROOM-03):** `src/pages/rooms/solo_journey.rs` lines 15–18 and 46–47:
```rust
#[component]
pub fn SoloJourneyRoom() -> impl IntoView {
    // Region read ONCE at room entry (never inside primitives)
    let region = use_context::<InitialTheme>().unwrap_or_default().0;
    // ...
    let dir = format!("/img/rooms/solo-journey/{region}");
    let surfaces_region = region.clone();
```
The new overlay `src` values follow the same `format!("{dir}/overlay-{axis}-{tier}.webp")` static string interpolation. No reactive closure involved.

---

### `input.css` — per-tier CSS block (D-10)

**Analog (current opacity-ramp block to REPLACE/EXTEND):** `input.css` lines 425–474

Base rule (keep, anchors defaults):
```css
/* Growth overlays: always in the DOM, opacity driven purely by the
   `data-*-tier` attribute on the room root (no reactive structural
   branching → hydration stays symmetric). Tier 0 = hidden. */
.room-overlay {
  position: fixed;
  inset: 0;
  pointer-events: none;
  opacity: 0;
  transition: opacity 400ms ease;
  will-change: opacity;
}
```
(lines 425–435)

**Current per-axis opacity-ramp rules to REPLACE** (lines 436–474, one axis shown as sample):
```css
[data-lib-tier="1"] .room-overlay--lib { opacity: 0.34; }
[data-lib-tier="2"] .room-overlay--lib { opacity: 0.67; }
[data-lib-tier="3"] .room-overlay--lib { opacity: 1; }
```
(Repeated for armory, banner, tactics, intel, command, records, tomes, analysis, roster, progression, crown.)

**New per-tier discrete-toggle rules** (one axis shown as template — repeat for all 12 axes):
```css
/* Per-tier discrete overlays — exactly one tier image visible at a time.
   Base .room-overlay { opacity: 0 } covers tier-0 (all hidden). */
/* lib axis */
[data-lib-tier="1"] .room-overlay--lib--tier-1 { opacity: 1; }
[data-lib-tier="2"] .room-overlay--lib--tier-2 { opacity: 1; }
[data-lib-tier="3"] .room-overlay--lib--tier-3 { opacity: 1; }
/* (tier-2/3 at tier-1, tier-1/3 at tier-2, tier-1/2 at tier-3
   are all opacity:0 by default from .room-overlay — no explicit zero rules needed) */
```

Axes × rooms:
- Solo Journey: `banner`, `lib`, `armory`
- War Room: `tactics`, `intel`, `command`
- Library: `records`, `tomes`, `analysis`
- Throne: `roster`, `progression`, `crown`

Total: 12 axes × 3 rules = 36 new rule lines.

**Reduced-motion block (lines 483–494) stays unchanged** — it references `.room-overlay` base class (transition:none), which still applies to all per-tier overlays.

---

### Offline shell scripts (`scripts/gen-room-art.sh`, `remove-bg.sh`, `encode-webp.sh`, `verify-budget.sh`)

**Analog:** `scripts/flow_test.sh` (bash conventions, curl patterns, `set -euo pipefail`, `trap` cleanup, env-var guards, pass/fail tracking). Also `AI-IMAGES.md` lines 80–96 (ImageMagick reproducibility commands).

**Shell script conventions** from `scripts/flow_test.sh` lines 1–17:
```bash
#!/usr/bin/env bash
# Description of what this script does.
# Requires: <dependencies>
# Usage: ./scripts/script-name.sh [args]
set -euo pipefail

BASE_URL="${1:-http://127.0.0.1:3020}"   # env-var pattern with fallback
COOKIE_JAR="$(mktemp /tmp/lol_flow_cookies_XXXXXX)"
trap 'rm -f "$COOKIE_JAR" /tmp/lol_flow_resp' EXIT  # always clean up temp files
```

**Existing ImageMagick invocation** from `AI-IMAGES.md` lines 83–96 (WebP placeholder pipeline):
```bash
magick \
  \( -size 1920x1080 gradient:'#1a1208'-'#070912' \) \
  \( -size 1920x1080 radial-gradient:'#d4af5a'-'#070912' -modulate 100,60,100 -blur 0x35 \) \
  -compose Screen -composite \
  \( -size 1920x1080 plasma:fractal -blur 0x6 -modulate 90,40,100 \) \
  -compose SoftLight -composite \
  -modulate 95,80,100 -blur 0x1.2 -quality 78 \
  public/img/beta-landing-demacia.jpg
```

**BFL EU FLUX polling pattern** (from RESEARCH.md Key Finding 1 — the production pattern for `gen-room-art.sh`):
```bash
# 1. Submit generation
RESPONSE=$(curl -s -X POST https://api.eu.bfl.ai/v1/flux-2-pro \
  -H "x-key: $BFL_KEY" \
  -H "Content-Type: application/json" \
  -d '{"prompt": "...", "width": 1920, "height": 1080, "steps": 28, "seed": 7421337}')
POLLING_URL=$(echo $RESPONSE | jq -r '.polling_url')

# 2. Poll until Ready
until [ "$(curl -s -H "x-key: $BFL_KEY" "$POLLING_URL" | jq -r '.status')" = "Ready" ]; do
  sleep 2
done

# 3. Download result (PNG)
curl -s -H "x-key: $BFL_KEY" "$POLLING_URL" | jq -r '.result.sample' | xargs curl -L -o out.png
```

**BiRefNet call pattern** (from RESEARCH.md Key Finding 2 — for `remove-bg.sh`):
```bash
RESULT=$(curl -s -X POST https://fal.run/fal-ai/birefnet/v2 \
  -H "Authorization: Key $FAL_KEY" \
  -H "Content-Type: application/json" \
  -d '{"image_url": "https://<public-url>", "model": "heavy"}')
echo $RESULT | jq -r '.image.url' | xargs curl -L -o plane-transparent.png
```

**Budget check pattern** (for `verify-budget.sh`, from RESEARCH.md Key Finding 4):
```bash
# Per-file cap check
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

---

### `.github/workflows/ci.yml` — new `asset_budget` job

**Analog:** existing `style_guardrails` job in `ci.yml` lines 31–95.

**Job skeleton pattern** (lines 31–44):
```yaml
  style_guardrails:
    name: Style Guardrails (G-01 / G-12 / raw-hex)
    runs-on: ubuntu-latest
    # Explanatory comment about the job's scope and intent.
    steps:
      - uses: actions/checkout@v4

      - name: G-01 — No Google Fonts CDN
        run: |
          # Inline bash block; multi-line; uses set-e (implied) and exit 1 on failure
          if grep -rE "..." ...; then
            echo "::error::Violation description"
            exit 1
          fi
```

**Key conventions from ci.yml:**
- Top-level `on: [push, pull_request]` — new job inherits this automatically (no per-job trigger needed)
- `runs-on: ubuntu-latest` — all jobs use this runner
- `- uses: actions/checkout@v4` — first step in every job
- `echo "::error::message"` — GitHub Actions annotation syntax for errors
- `FAIL=0 / FAIL=1 / exit $FAIL` pattern (from RESEARCH.md) — used when the step must collect multiple failures before exiting
- No `needs:` dependency — `asset_budget` runs parallel to `check` and `style_guardrails`

**New job template** (from RESEARCH.md Key Finding 4):
```yaml
  asset_budget:
    name: Room art asset budget (per-file + per-room/region aggregate)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Room art asset budget (per-file + per-room/region aggregate)
        run: |
          FAIL=0
          if find public/img/rooms -name "*.webp" -size +120k | grep -q .; then
            echo "::error::Room art file exceeds 120 KB cap"
            find public/img/rooms -name "*.webp" -size +120k
            FAIL=1
          fi
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

---

### `e2e/tests/room-solo-journey.spec.ts` — new per-tier overlay test

**Analog A (attribute assertion pattern):** existing `growth-state tier attributes are present and in range` test, `room-solo-journey.spec.ts` lines 103–119:
```typescript
test("growth-state tier attributes are present and in range", async ({
  authedPage,
}) => {
  await authedPage.goto("/journey");
  await authedPage.waitForLoadState("networkidle");
  await authedPage.waitForTimeout(800); // growth resource resolve

  const root = authedPage.locator(".room-atmosphere");
  for (const axis of ["lib", "armory", "banner"]) {
    const val = await root.getAttribute(`data-${axis}-tier`);
    expect(val, `data-${axis}-tier present`).not.toBeNull();
    const n = parseInt(val ?? "", 10);
    expect(Number.isNaN(n)).toBe(false);
    expect(n).toBeGreaterThanOrEqual(0);
    expect(n).toBeLessThanOrEqual(3);
  }
});
```

**Analog B (computed-style assertion):** `room-solo-journey.spec.ts` lines 147–164 (budget test showing `authedPage.evaluate()` pattern):
```typescript
test("room art stays within the 400 KB asset budget", async ({
  authedPage,
}) => {
  let roomBytes = 0;
  authedPage.on("response", async (resp) => {
    if (ROOM_BG_RE.test(resp.url())) {
      const len = resp.headers()["content-length"];
      if (len) roomBytes += parseInt(len, 10);
    }
  });
  await authedPage.goto("/journey");
  await authedPage.waitForLoadState("networkidle");
  await authedPage.waitForTimeout(500);
  expect(roomBytes).toBeLessThan(400 * 1024);
});
```

**Analog C (tier-1 seed fixture):** `e2e/tests/fixtures.ts` `seedTier1Team` function (lines 194–234). This seeds a draft to push `tactics_tier` to ≥ 1 in the War Room. The new test for Solo Journey overlay needs an analogous seeder that pushes a Solo axis to tier ≥ 1.

**Import pattern** (`room-solo-journey.spec.ts` line 22):
```typescript
import { test, expect } from "./fixtures";
```

**New test template** (per-tier overlay visibility check):
```typescript
test("per-tier overlay: tier-1 shows tier-1 art only", async ({
  authedPage,
}) => {
  // Seed growth so at least one Solo axis reaches tier ≥ 1.
  // (New fixture: seedSoloTier1Growth — analogous to seedTier1Team)
  await authedPage.goto("/journey");
  await authedPage.waitForLoadState("networkidle");
  await authedPage.waitForTimeout(800); // growth resource resolve

  const root = authedPage.locator(".room-atmosphere");
  const libTier = parseInt(
    (await root.getAttribute("data-lib-tier")) ?? "0",
    10
  );

  if (libTier >= 1) {
    // CSS computed opacity — tier-1 image must be visible at tier 1
    const tier1Opacity = await authedPage
      .locator(".room-overlay--lib--tier-1")
      .evaluate((el) => getComputedStyle(el).opacity);
    expect(parseFloat(tier1Opacity)).toBeGreaterThan(0);

    // tier-2 and tier-3 must remain hidden when tier === 1
    if (libTier === 1) {
      const tier2Opacity = await authedPage
        .locator(".room-overlay--lib--tier-2")
        .evaluate((el) => getComputedStyle(el).opacity);
      expect(parseFloat(tier2Opacity)).toBe(0);

      const tier3Opacity = await authedPage
        .locator(".room-overlay--lib--tier-3")
        .evaluate((el) => getComputedStyle(el).opacity);
      expect(parseFloat(tier3Opacity)).toBe(0);
    }
  }
});
```

---

### `.planning/assets/AI-IMAGES.md` — provenance entry format

**Analog:** existing room-plane entry, `AI-IMAGES.md` lines 65–97 (demacia beta-landing entry):

```markdown
### `public/img/rooms/<room>/<region>/plane-sky.webp`

- **Generated:** 2026-MM-DD
- **Surface:** `/<route>` atmosphere — sky depth plane, Aurelian Reach region
- **Compute path:** BFL EU FLUX.2 [pro] → BiRefNet v2 (sky: skip BiRefNet) → ImageMagick → WebP
- **Runtime:** BFL EU endpoint `https://api.eu.bfl.ai/v1/flux-2-pro`
- **Prompt:** "..."  ← Path A vocabulary only; no Riot proper nouns
- **Seed:** 7421337  ← integer; record exact integer used, not alias
- **Steps:** 28
- **Aspect / resolution:** 16:9 / 1920×1080
- **Final size:** XX KB WebP q80
- **Reproducibility command:**
  ```bash
  # scripts/gen-room-art.sh <room> <region> plane-sky <seed>
  ```
- **BiRefNet applied:** No (sky plane is opaque)
```

For per-tier overlays the entry adds `**Tier:**` and `**BiRefNet applied:** Yes` (overlays are transparent).

---

## Shared Patterns

### ROOM-03: Region read once, passed as prop

**Source:** `src/pages/rooms/solo_journey.rs` lines 15–47 and `CLAUDE.md` gotcha #58
**Apply to:** All four room overlay-markup changes (D-10)

```rust
// ALWAYS correct — read once at room component top, build dir statically
let region = use_context::<InitialTheme>().unwrap_or_default().0;
let dir = format!("/img/rooms/solo-journey/{region}");
// Then use dir in format! calls inside view! — no closure wraps dir
<GrowthOverlay axis="lib--tier-1" src=format!("{dir}/overlay-lib-1.webp") z=5 />
```

```rust
// NEVER do this — reading context inside a reactive closure panics at html/mod.rs:217
let region = Signal::derive(|| use_context::<InitialTheme>().unwrap_or_default().0);
```

### CSS data-attr-driven overlay toggling (not reactive JS)

**Source:** `input.css` lines 425–435 (`.room-overlay` base rule)
**Apply to:** All 12 axes in the new per-tier CSS block

The `.room-overlay { opacity: 0; }` base rule handles tier-0 (all images hidden) automatically. The per-tier rules only need to add the `opacity: 1` case for each tier. No explicit `[data-{axis}-tier="0"]` rule is needed and none should be added (Pitfall 6 in RESEARCH.md).

### `loading="eager"` on atmosphere images

**Source:** `src/components/room/parallax.rs` lines 367–368 (Plane) and 398–399 (GrowthOverlay)
**Apply to:** All placeholder WebP files generated for new per-tier overlay paths

All room art images use `loading="eager"`. This is already enforced in `GrowthOverlay` — the new three-per-axis calls inherit it automatically.

### `set -euo pipefail` + `trap` cleanup

**Source:** `scripts/flow_test.sh` lines 12–17
**Apply to:** All four new offline scripts (`gen-room-art.sh`, `remove-bg.sh`, `encode-webp.sh`, `verify-budget.sh`)

### `FAIL=0 / exit $FAIL` accumulator pattern

**Source:** ci.yml `style_guardrails` job (inline bash) and RESEARCH.md Key Finding 4
**Apply to:** `scripts/verify-budget.sh` and the new `asset_budget` CI job step — allows collecting all budget failures before reporting rather than stopping at the first.

---

## No Analog Found

All artifacts have close analogs in the codebase. No file requires falling back to RESEARCH.md patterns alone.

| File | Role | Data Flow | Note |
|---|---|---|---|
| (none) | — | — | All 8 artifacts have direct analogs |

---

## Metadata

**Analog search scope:** `src/pages/rooms/`, `src/components/room/`, `input.css`, `scripts/`, `.github/workflows/ci.yml`, `e2e/tests/`, `.planning/assets/AI-IMAGES.md`
**Files read:** 13 source files
**Pattern extraction date:** 2026-06-15
