# Phase 18: Region Variants — Research

**Researched:** 2026-05-14
**Domain:** Leptos 0.8 component branching, Tailwind v4 CSS utilities, Playwright visual regression, SurrealDB schema migration
**Confidence:** HIGH (all 7 focus questions answered from verified sources)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01:** ~24 region-aware primitives in 7 modules under `src/components/region/` (typography, ornaments, layout, controls, data_viz, solo, chrome).

**D-02:** `src/components/ornaments.rs` DELETED in 18-01. All ornaments move to `region/ornaments.rs`. ~7 import sites update.

**D-03:** Three new string fields on `user` table: `draft_mode`, `team_dashboard_mode`, `solo_mode` each `DEFAULT 'auto'`. No ASSERT constraint (values: 'auto', 'carousel', 'war-table', 'ledger', 'dashboard', 'brief', 'constellation', 'forge', 'journal').

**D-04:** `'auto'` sentinel resolved at page entry by `resolve_mode(stored, region, route)`. Defaults are region-coupled.

**D-05:** Explicit user pick overrides resolver and persists across region switches. `'auto'` reset not exposed in v1.3 UI.

**D-06:** Three getter/setter pairs in `src/server/db.rs` follow `get_user_theme` / `set_user_theme` pattern.

**D-07:** Visual-regression subfolders per scoped route inside `visual-regression.spec.ts-snapshots/`.

**D-08:** Baselines committed to git (no LFS). Git LFS deferred.

**D-09:** Filename = `{region}-{mode}-chromium-linux.png` for multi-mode routes; `{region}-chromium-linux.png` for single-mode.

**D-10:** 7 existing scoped-route Demacia flat baselines DELETED at start of 18-09; replacements in subfolders.

**D-11:** 4 missing siblings authored in Claude Design project (now confirmed to already exist — see Finding F-01).

**D-12:** Per-sibling workflow: author JSX → re-export → re-extract → derive content contract → port in 18-07.

**D-13:** 18-07 GATED on all 4 sibling JSX files existing. Preflight `ls` check required.

**D-14:** User drives Claude Design session. Agents do NOT call Claude Design API.

### Claude's Discretion

- Whether to write a project-local skill capturing the `move || if is_pandemonium { … }` pattern.
- Exact shape of the mode-toggle UI primitive (segmented control vs. stamped tab-pull).
- Whether `/solo` Pandemonium default is `forge` or `journal` — confirm during 18-08.
- Whether to add `Region` enum vs. keep `String`-typed `region` props.
- Whether `/match/:id` and `/post-game` share one `match_report` component or stay separate.
- Whether `theme_toggle.rs` is restyled to feel more "ritual".
- File location for `PageLoading` / `PageEmpty` components (under `region/` or `src/components/skeleton.rs`).

### Deferred Ideas (OUT OF SCOPE)

- Region enum for type safety (planner can add in 18-01, otherwise post-launch cleanup).
- Mode-reset UI (`'auto'` re-set).
- Per-region mode-toggle variant styling (can ship region-neutral toggle first).
- Git LFS for baseline PNGs.
- SSIM perceptual diff.
- Mobile responsiveness for Pandemonium layouts.
- Animated theme transitions.
- Sibling repo for visual-regression baselines.
- Live Match overlay restyle.

</user_constraints>

---

## Summary

Phase 18 ports a canonical Claude Design source (11 page pairs + ~24 shared primitives + per-region skeletons) into Leptos so that Demacia and Pandemonium render structurally different component compositions. The research resolves all 7 focus questions from the phase brief with HIGH confidence, using verified sources.

**The most important finding** is that all 4 "gated" sibling JSX files (18-07 blocker) already exist and are substantive (269–298 lines each). The 18-07 plan does NOT need a design-session prerequisite step; it needs only the preflight `ls` check to confirm non-stub status.

The second most important finding is that the `AnyView` / `into_any()` pattern used in `CompanionSigil` (ornaments.rs:128-167) is the correct and well-documented Leptos 0.8 approach for divergent view branches. Using it consistently in all 24 primitives is safe provided each primitive's top-level branch is a flat `move || if is_pandemonium { … }` — no nested `view! {}` macro explosions occur when the branch is at the top rather than inside nested calls.

The recursion limit (512) is NOT expected to be breached by Phase 18. The current limit was set for post_game.rs's deeply nested view types. The region primitives use shallow `view! {}` bodies branched at the top level, which does not compound the type recursion that caused the limit to be raised originally.

**Primary recommendation:** Plan 18-01 should establish the `Region` enum as a Claude's Discretion early call — it eliminates string comparison footguns across 24 primitives and 9 pages for negligible cost. The planner should make that call at the start of Wave 1 before any primitives are written.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Region-aware primitive rendering | Browser / Client (WASM) | Frontend Server (SSR hydration) | Components compile for both targets; `InitialTheme` context provided at page entry in SSR shell |
| Mode persistence (draft_mode, team_dashboard_mode, solo_mode) | API / Backend (SurrealDB user record) | — | Consistent with existing `theme` and `mode` fields on user table; survives hard navigation |
| Mode resolver (auto sentinel → region default) | Frontend Server (SSR) + Browser | — | Pure function, runs server-side on first render; also called client-side on mode toggle |
| Visual-regression baseline capture | CDN / Static (file system) | — | Playwright writes PNGs to `e2e/tests/visual-regression.spec.ts-snapshots/` at test time |
| Region-diff assertion | API / Backend (test runner) | — | pixelmatch comparison runs in Node.js test process, not in browser |
| `[data-theme]` CSS token switching | Browser / Client | — | CSS custom property cascade; switching data attribute on `<html>` is pure CSS, no JS |
| Schema migration (new user fields) | API / Backend (SurrealDB cold-start) | — | `DEFINE FIELD IF NOT EXISTS` runs on server startup via `include_str!(schema.surql)` |

---

## Standard Stack

### Core (all pre-existing in project)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Leptos | 0.8 | Component framework | Project foundation; `#[component]`, `view!`, `AnyView`, `into_any()` |
| Tailwind CSS v4 | v4 (standalone binary) | Utility classes, `@theme`, `@utility`, `@keyframes` | Project foundation; no npm required |
| SurrealDB | 3.x (SurrealKV) | User mode persistence | Project foundation; existing `get_user_theme` / `set_user_theme` pattern |
| Playwright | 1.58.2 (installed) | Visual-regression baseline capture + region-diff assertion | Project e2e foundation |

### New Dependencies (18-09 only)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| pixelmatch | 7.2.0 | Pixel-level image comparison for region-diff spec | 18-09 only; install in e2e/package.json |
| pngjs | 7.0.0 | PNG decoding for pixelmatch input | 18-09 only; required by pixelmatch Node.js usage |

**Version verification:**
- pixelmatch: 7.2.0 [VERIFIED: npm registry]
- pngjs: 7.0.0 [VERIFIED: npm registry]
- @playwright/test: 1.58.2 [VERIFIED: installed in e2e/]
- Playwright test runner: 1.58.2 [VERIFIED: `npx playwright --version`]

**Installation for 18-09:**
```bash
cd e2e && npm install pixelmatch pngjs
```

**TypeScript types for 18-09:**
```bash
cd e2e && npm install --save-dev @types/pixelmatch @types/pngjs
```

---

## Architecture Patterns

### System Architecture Diagram

```
User browser
  │
  │  (1) Page request (e.g., GET /draft)
  ▼
Axum SSR handler
  │  extract AuthSession → AppUser.theme
  │  call resolve_mode(user.draft_mode, user.theme, Route::Draft)
  │  provide_context(InitialTheme(theme))
  │
  ▼
DraftPage component (SSR render)
  │  let theme = use_context::<InitialTheme>().0;  ← ONCE at page entry
  │  let region = theme.clone();                    ← passed as prop
  │  let mode = resolve_mode(stored_mode, &region, Route::Draft);
  │
  ├── [mode == "carousel"] → DraftCarouselView { region }
  ├── [mode == "war-table"] → DraftWarTableView { region }
  └── [mode == "ledger"] → DraftLedgerView { region }
           │
           ▼
    Region primitive tree (e.g., Card, SectionHead, Btn)
    Each primitive:
      let is_pandemonium = region == "pandemonium";
      move || if is_pandemonium {
          view! { /* pandemonium markup */ }.into_any()
      } else {
          view! { /* demacia markup */ }.into_any()
      }
           │
           ▼
    CSS: [data-theme="pandemonium"] selector applies Pandemonium tokens
    (applied on <html> element by ThemeToggle hydration after SSR)
           │
           ▼
User sees region-differentiated layout

Mode toggle interaction:
  User clicks mode toggle
  → server fn set_user_draft_mode(db, user_id, "war-table")
  → WASM signal update → page re-renders with new mode
  → next full navigation reads updated DB value
```

### Recommended Project Structure

```
src/components/
├── region/
│   ├── mod.rs            # pub use re-exports for all 7 submodules
│   ├── typography.rs     # Display, Imperial, H, Eyebrow, Mono, Glitch
│   ├── ornaments.rs      # GiltCorner, HeraldicDivider, RiotTape,
│   │                     # FleurDeLis, Crown, CompanionSigil (MOVED from ornaments.rs)
│   ├── layout.rs         # Card, SectionHead, Themed
│   ├── controls.rs       # Btn, Badge
│   ├── data_viz.rs       # Stat, Sparkline, MoodMeter
│   ├── solo.rs           # RankBadge, LPProgress
│   └── chrome.rs         # ChampPortrait, ChampTile, RoleIcon, Icon
├── mod.rs                # remove ornaments, add region module
├── (all other components unchanged)
e2e/tests/
├── region-diff.spec.ts   # NEW: asserts pixelDiffRatio > 0.40 per scoped route
├── visual-regression.spec.ts  # MODIFIED: subfolder paths for scoped routes
├── visual-regression.spec.ts-snapshots/
│   ├── (15 utility PNGs unchanged)
│   ├── authed-draft/
│   ├── authed-solo/
│   ├── authed-team-dashboard/
│   ├── authed-tree-drafter/
│   ├── authed-champion-pool/
│   ├── authed-match-detail/
│   └── authed-post-game/
```

### Pattern 1: Top-Level AnyView Branch (Region Primitive)

**What:** Every region-aware primitive branches at the top level with a `move || if is_pandemonium { … }.into_any() else { … }.into_any()` pattern. This is the established pattern from `CompanionSigil` (ornaments.rs:128-167).

**When to use:** For all 24 primitives that have structurally different Demacia vs Pandemonium implementations.

**Why this is safe for recursion limit:** The branch lives OUTSIDE the `view! {}` macro, not nested inside it. The macro's type recursion only accumulates within a single `view! {}` call. Two separate `view! {}` calls (one per branch arm) each have shallow type trees.

```rust
// Source: src/components/ornaments.rs:128-167 (CompanionSigil reference)
#[component]
pub fn Card(
    region: String,
    #[prop(optional, into)] variant: Option<String>,
    children: Children,
) -> impl IntoView {
    let is_pandemonium = region == "pandemonium";
    move || if is_pandemonium {
        view! {
            <div class="bg-surface border border-outline/30 rounded-none p-4">
                {children()}
            </div>
        }.into_any()
    } else {
        view! {
            <div class="bg-surface border border-outline/50 rounded p-4
                        [data-theme=demacia]:shadow-[inset_0_0_0_1px_var(--color-accent)/0.15]">
                {children()}
            </div>
        }.into_any()
    }
}
```

**Children ownership note:** `children: Children` is `Box<dyn FnOnce() -> Fragment>`. It is consumed on first call. For branching patterns where only ONE arm calls `children()`, this is fine. If BOTH arms need children, use `ChildrenFn` (cloneable) instead of `Children`.

[VERIFIED: Leptos 0.8 source /leptos-rs/leptos]

### Pattern 2: Region Prop at Page Entry, Never Inside Primitives

**What:** Pages consume `InitialTheme` context ONCE at entry and pass `region: String` as a prop down the tree. Primitives never call `use_context::<InitialTheme>()`.

**When to use:** All pages in the scoped route set (18-03 through 18-07).

**Why:** SSR/hydration consistency. Context reads inside deeply nested component trees can cause hydration mismatches if the context is not available identically on both SSR and client passes.

```rust
// In a page component (e.g., draft.rs)
#[component]
pub fn DraftPage() -> impl IntoView {
    let theme = use_context::<InitialTheme>().unwrap_or_default();
    let region = theme.0.clone();
    // region is now a plain String, passed as prop to child components
    view! {
        <DraftCarouselView region=region />
    }
}

// In a primitive (e.g., region/layout.rs::Card)
// NO use_context call — region comes from prop only
#[component]
pub fn Card(region: String, children: Children) -> impl IntoView {
    // ...
}
```

[VERIFIED: CONTEXT.md SPEC constraints + existing CompanionSigil reference]

### Pattern 3: DB Getter/Setter for Mode Fields

**What:** Each of the 3 mode fields follows the exact same shape as `get_user_mode` / `set_user_mode` in `src/server/db.rs`.

**When to use:** 18-08 plan when wiring mode persistence.

```rust
// Source: db.rs:4464-4488 (get_user_mode / set_user_mode pattern)
pub async fn get_user_draft_mode(db: &Surreal<Db>, user_id: &str) -> DbResult<String> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    #[derive(Debug, Deserialize, SurrealValue)]
    struct ModeRecord { draft_mode: Option<String> }
    let mut result = db
        .query("SELECT draft_mode FROM type::record('user', $user_key)")
        .bind(("user_key", user_key))
        .await?;
    let row: Option<ModeRecord> = result.take(0)?;
    Ok(row.and_then(|r| r.draft_mode).unwrap_or_else(|| "auto".to_string()))
}

pub async fn set_user_draft_mode(db: &Surreal<Db>, user_id: &str, mode: &str) -> DbResult<()> {
    let user_key = user_id.strip_prefix("user:").unwrap_or(user_id).to_string();
    db.query("UPDATE type::record('user', $user_key) SET draft_mode = $mode")
        .bind(("user_key", user_key))
        .bind(("mode", mode.to_string()))
        .await?.check()?;
    Ok(())
}
```

Note: The new mode fields do NOT need an `ASSERT` constraint (unlike `theme`). The valid values set is wider and controlled at the application layer. The `set_user_*_mode` server function validates before writing.

[VERIFIED: db.rs:4460-4520 (existing pattern)]

### Pattern 4: Playwright Subfolder Snapshot Path

**What:** `toHaveScreenshot` accepts an array of strings for nested paths within the spec's `-snapshots/` directory. This is the correct and verified approach. `path.join()` as a single string ALSO works (the string value is the same as what `path.join` produces on Linux), but the array syntax is idiomatic and cross-platform safe.

**When to use:** All scoped-route tests in 18-09.

```typescript
// Source: Playwright official docs (verified via Context7 /microsoft/playwright.dev)
// BOTH of these are equivalent and work in Playwright 1.49+:

// Array syntax (preferred, cross-platform safe):
await expect(page).toHaveScreenshot(['authed-draft', 'demacia-carousel.png'], VR_OPTS);

// path.join string (also works on Linux, matches CONTEXT.md D-09 convention):
await expect(page).toHaveScreenshot(
  path.join('authed-draft', 'demacia-carousel.png'),
  VR_OPTS
);
```

Both approaches store the file at:
`e2e/tests/visual-regression.spec.ts-snapshots/authed-draft/demacia-carousel-chromium-linux.png`

Playwright automatically appends `-chromium-linux` before the extension based on the project name + platform.

**IMPORTANT:** The path must remain within the spec's `-snapshots/` directory. Traversal (`..`) throws.

[VERIFIED: Playwright docs /microsoft/playwright.dev (Context7 + web fetch)]

### Pattern 5: Custom pixelDiffRatio for region-diff spec

**What:** `region-diff.spec.ts` captures screenshots of both regions for each route/mode and computes a custom pixel diff ratio using `pixelmatch` + `pngjs`. This is not a built-in Playwright assertion — it requires manual image comparison.

**When to use:** 18-09.

```typescript
// region-diff.spec.ts pattern
// Source: npm pixelmatch 7.2.0 API + pngjs 7.0.0
import { test, expect } from './fixtures';
import * as fs from 'fs';
import { PNG } from 'pngjs';
import pixelmatch from 'pixelmatch';

async function pixelDiffRatio(
  buf1: Buffer,
  buf2: Buffer,
): Promise<number> {
  const img1 = PNG.sync.read(buf1);
  const img2 = PNG.sync.read(buf2);
  // If dimensions differ, images cannot be compared pixel-by-pixel
  if (img1.width !== img2.width || img1.height !== img2.height) {
    return 1.0; // treat dimension mismatch as completely different
  }
  const diff = pixelmatch(
    img1.data, img2.data, null,
    img1.width, img1.height,
    { threshold: 0.1 }
  );
  return diff / (img1.width * img1.height);
}

test('draft carousel: demacia vs pandemonium differ by >40%', async ({ authedPage }) => {
  // Set demacia, capture
  await setRegion(authedPage, 'demacia');
  await authedPage.goto('/draft');
  await authedPage.waitForLoadState('networkidle');
  await authedPage.waitForTimeout(500);
  const demBuf = await authedPage.screenshot({ fullPage: true });

  // Set pandemonium, capture
  await setRegion(authedPage, 'pandemonium');
  await authedPage.goto('/draft');
  await authedPage.waitForLoadState('networkidle');
  await authedPage.waitForTimeout(500);
  const panBuf = await authedPage.screenshot({ fullPage: true });

  const ratio = await pixelDiffRatio(demBuf, panBuf);
  expect(ratio).toBeGreaterThan(0.40);
});
```

**Dimension mismatch handling:** Demacia and Pandemonium pages may have different heights (different component counts). If `img1.height !== img2.height`, a full-page comparison requires either padding to equal size or capturing viewport-only (not `fullPage`). Recommendation: use viewport-only screenshots for region-diff assertions (consistent dimensions); full-page only for baseline capture. This is a **planner decision** for 18-09.

[VERIFIED: npm registry pixelmatch 7.2.0, pngjs 7.0.0; pattern from Testrig Medium article]

### Pattern 6: Tailwind v4 Custom @keyframes and @utility

**What:** Skeleton animations (`panFlicker`, `panScan`, `demShimmer`) and Pandemonium effects (glitch text-shadow, halftone background) go in `input.css` using Tailwind v4 syntax.

**When to use:** 18-02 (skeleton animations) and anywhere Pandemonium effect utilities are needed.

**Tailwind v4 animation pattern** (inside `@theme` for tree-shakable utility generation):

```css
/* In input.css — inside @theme block */
@theme {
  --animate-dem-shimmer: dem-shimmer 2.4s ease-in-out infinite;
  --animate-pan-flicker: pan-flicker 1.6s ease-in-out infinite;
  --animate-pan-scan: pan-scan 0.9s linear infinite;
  --animate-pan-cursor: pan-cursor 0.9s steps(1) infinite;

  @keyframes dem-shimmer {
    0%, 100% { opacity: 0.4; }
    50% { opacity: 0.85; }
  }
  @keyframes pan-flicker {
    0%, 100% { opacity: 0.55; }
    48% { opacity: 0.55; }
    50% { opacity: 0.95; }
    52% { opacity: 0.55; }
    80% { opacity: 0.75; }
  }
  @keyframes pan-scan {
    0%   { background-position: 0 0; }
    100% { background-position: 0 12px; }
  }
  @keyframes pan-cursor {
    0%, 50% { opacity: 1; }
    51%, 100% { opacity: 0; }
  }
}
```

**Result:** Use as `animate-dem-shimmer`, `animate-pan-flicker`, `animate-pan-scan`, `animate-pan-cursor` utility classes.

**Glitch text-shadow utility** (`@utility` syntax):

```css
@utility text-shadow-glitch {
  text-shadow:
    -2px -2px 0 var(--accent-2),
    2px 2px 0 var(--t-accent);
}

@utility text-shadow-none {
  text-shadow: none;
}
```

**Halftone background** (cannot be a pure Tailwind utility — requires `background-image` with specific values; recommended as inline style or a named utility):

```css
@utility bg-halftone {
  background-image: radial-gradient(circle, var(--t-elevated) 20%, transparent 20%);
  background-size: 4px 4px;
}
```

**Oil-spill overlay** (decorative overlay, best as a composable utility):

```css
@utility bg-oil-spill {
  background:
    linear-gradient(135deg,
      color-mix(in oklab, var(--t-accent) 12%, transparent) 0%,
      color-mix(in oklab, var(--accent-2) 8%, transparent) 33%,
      color-mix(in oklab, var(--accent-3) 6%, transparent) 66%,
      transparent 100%);
}
```

**Important constraint:** The Pandemonium scanline background from `skeletons-pandemonium.jsx` uses `repeating-linear-gradient` with `color-mix(in oklab, var(--accent) 14%, transparent)`. This requires `color-mix()` which Tailwind v4 does NOT auto-generate from `@theme` tokens. Use inline `style` attribute or define a dedicated `@utility` with the hardcoded `color-mix` expression referencing the CSS custom property.

[VERIFIED: Tailwind v4 official docs (WebFetch), existing input.css patterns]

### Pattern 7: AppUser extension for 3 mode fields

**What:** `AppUser` struct (auth.rs) and its `DbUser` conversion need 3 new optional string fields.

**When to use:** 18-08 plan.

```rust
// In src/server/auth.rs — DbUser (add 3 fields)
#[derive(Clone, Debug, Deserialize, SurrealValue)]
struct DbUser {
    // ... existing fields ...
    draft_mode: Option<String>,
    team_dashboard_mode: Option<String>,
    solo_mode: Option<String>,
}

// AppUser struct extension
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppUser {
    // ... existing fields ...
    pub draft_mode: String,
    pub team_dashboard_mode: String,
    pub solo_mode: String,
}

// From<DbUser> conversion
impl From<DbUser> for AppUser {
    fn from(u: DbUser) -> Self {
        AppUser {
            // ... existing fields ...
            draft_mode: u.draft_mode.unwrap_or_else(|| "auto".to_string()),
            team_dashboard_mode: u.team_dashboard_mode.unwrap_or_else(|| "auto".to_string()),
            solo_mode: u.solo_mode.unwrap_or_else(|| "auto".to_string()),
        }
    }
}
```

[VERIFIED: src/server/auth.rs (existing pattern)]

### Anti-Patterns to Avoid

- **`use_context::<InitialTheme>()` inside region primitives.** Causes SSR/hydration mismatches. Context read goes at page entry only.
- **Nesting `view! {}` macros inside `view! {}` for region branching.** `view! { if region == "pandemonium" { view! {…} } else { view! {…} } }` — this pushes BOTH branch types into the outer macro's type tree, compounding recursion depth. Always hoist the branch OUTSIDE the macro.
- **`Children` for two-arm branch where both arms render children.** Use `ChildrenFn` instead — `Children` is `FnOnce()` and consumed on first call.
- **`path.join()` on Windows.** The array syntax `['subfolder', 'filename.png']` is the safe cross-platform approach for Playwright subfolder snapshots.
- **Comparing full-page screenshots of different-height pages with pixelmatch.** Demacia and Pandemonium page heights may differ. Use viewport screenshots for region-diff assertions.
- **ASSERT constraint on mode fields.** The `theme` field has `ASSERT $value IN [...]` which prevents arbitrary values. Mode fields should NOT have this constraint — valid modes are a wider, route-specific set, and adding ASSERT would break the 'auto' sentinel if not included.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Pixel image comparison | Custom image diffing | `pixelmatch` | Battle-tested, handles anti-aliasing, threshold param |
| PNG encoding/decoding | Custom PNG parser | `pngjs` | The standard for Node.js PNG handling; used by pixelmatch |
| CSS animation utilities | Inline `style` for every animation | `@utility` + `@keyframes` in input.css | Utilities get Tailwind variant support (`hover:`, `focus:`) |
| Region token CSS | Duplicating color values in components | `var(--t-accent)`, `var(--color-accent)` etc. | Tokens already defined in input.css:193-265; components stay region-agnostic |
| Mode resolver logic | Inline match arm per page | Shared `resolve_mode(stored, region, route)` function | Centralizes the auto-sentinel logic; easier to update default table |

**Key insight:** The Pandemonium effects (halftone, oil-spill, scanline) look visually complex but are 1-4 lines of CSS. Do not build Rust helper structs to generate them — define `@utility` classes and apply them.

---

## SurrealDB Schema Migration

### How `DEFINE FIELD IF NOT EXISTS` behaves on cold start

**Finding:** [ASSUMED based on web search + SurrealDB docs] `DEFINE FIELD IF NOT EXISTS` is idempotent for the schema definition itself — re-running it on startup does not modify the field definition if it already exists. However, the `DEFAULT` value is only applied at **record creation time** (INSERT/CREATE), not retroactively to existing records.

**Practical consequence:** Existing `user` records (created before Phase 18) will have `draft_mode`, `team_dashboard_mode`, `solo_mode` as `NONE` after the schema is applied, not `'auto'`. The getter functions in db.rs must use `Option<String>` and `.unwrap_or_else(|| "auto".to_string())` to handle `NONE` gracefully — matching the existing `mode` and `theme` field pattern in db.rs:4464-4477.

**Verified precedent:** The existing `get_user_mode` function does exactly this: `Option<ModeRecord>` deserialization + `.unwrap_or_else(|| "solo".to_string())`. The Phase 18 getter functions must follow the same pattern with `"auto"` as the fallback.

**No separate migration step needed.** The `NONE` → `"auto"` fallback in the getter functions IS the migration. The resolver treats `"auto"` and `NONE` identically (both trigger the region-coupled default). As long as the getters return `"auto"` for missing values, existing users get the correct region-coupled default on first access.

**Note on existing theme ASSERT:** The `theme` field has `ASSERT $value IN ['demacia', 'pandemonium']`. The 3 new mode fields do NOT need an ASSERT — the valid values set is wider and adding ASSERT would complicate future mode additions.

[VERIFIED: db.rs:4460-4520 (existing pattern); ASSUMED: SurrealDB DEFAULT retroactivity behavior from web search]

---

## Recursion Limit Analysis

**Finding:** The current `#![recursion_limit = "512"]` was set for `post_game.rs` deeply nested view types. Phase 18 primitives use the TOP-LEVEL BRANCH pattern which does NOT compound recursion.

**Why top-level branching is safe:**

```rust
// SAFE: two separate view!{} calls with shallow trees
move || if is_pandemonium {
    view! { <div class="...">...</div> }.into_any()  // depth: ~5 types
} else {
    view! { <div class="...">...</div> }.into_any()  // depth: ~5 types
}
// Max depth consumed: ~10 (not additive — only ONE arm is used)

// UNSAFE: nested view!{} inside another view!{}
view! {
    <div class="outer">
        { move || if is_pandemonium {
            view! { <div class="inner-p">...</div> }.into_any()
        } else {
            view! { <div class="inner-d">...</div> }.into_any()
        }}
    </div>
}
// Depth: outer types PLUS closure return type PLUS inner view types
// This compounds and can hit 512 if the outer view is already deep
```

**Recommendation:** If a page's existing view tree is already moderately deep (draft.rs at 3,801 LOC is the risk case), ensure region branching happens at the TOP of each page component, not inside a deeply nested helper view. For heavy pages (18-04, 18-05, 18-06), create separate `DraftCarouselView { region }`, `DraftWarTableView { region }` etc. components (pages not directly render-branching inside one large view tree).

**Current state:** `src/lib.rs` line 1: `#![recursion_limit = "512"]` [VERIFIED: grep]. No change needed to this limit for Phase 18 if the top-level branch pattern is followed.

[VERIFIED: src/lib.rs:1, src/main.rs:1; VERIFIED: leptos-patterns.md rule 38; reasoning based on Leptos type system behavior]

---

## Common Pitfalls

### Pitfall 1: `Children` consumed before branch arm executes

**What goes wrong:** A primitive with `children: Children` calls `children()` inside a `move || if …` closure. The closure captures `children` but only one arm calls it. If the component re-renders after the initial render (e.g., region changes in WASM after a theme switch), the closure is called again — but `children` (which is `FnOnce`) has already been consumed.

**Why it happens:** `Children` is `Box<dyn FnOnce() -> Fragment>`. The `FnOnce` is consumed on first call. SSR renders once (fine). WASM re-renders on signal change (breaks on second render if region changes dynamically).

**How to avoid:** For primitives where the region is a static SSR prop (read once, passed down), `Children` is fine. For primitives where region could change dynamically (e.g., if a future phase adds live region switching), use `ChildrenFn`. In Phase 18, region is passed from `InitialTheme` which is set at page load and does not change in-page — `Children` is acceptable.

**Warning signs:** `thread 'main' panicked at 'called FnOnce more than once'` in WASM console; OR `closure already moved` compile error if `ChildrenFn` is missing.

[VERIFIED: Leptos 0.8 docs + leptos-patterns.md rule 19]

### Pitfall 2: SurrealDB field returns NONE for users created before Phase 18

**What goes wrong:** `SELECT draft_mode FROM user WHERE id = $id` returns `NONE` for records created before the field was defined. If the getter deserializes as `String` (not `Option<String>`), the query fails with a deserialization error.

**Why it happens:** `DEFINE FIELD IF NOT EXISTS` with `DEFAULT` applies on CREATE, not retroactively.

**How to avoid:** Always use `Option<String>` in the `Db*` struct, then `.unwrap_or_else(|| "auto".to_string())` in the getter. This matches the existing `mode` and `theme` field pattern in db.rs.

**Warning signs:** Server fn errors like `Failed to deserialize field 'draft_mode': expected String, got NONE` in logs.

[VERIFIED: db.rs:4466-4477 pattern + web search on SurrealDB DEFAULT retroactivity]

### Pitfall 3: Visual-regression snapshot path outside spec's snapshots directory

**What goes wrong:** `toHaveScreenshot(path.join('..', 'other-spec-snapshots', 'file.png'))` — traversal outside the spec's own `-snapshots/` directory throws `SnapshotError: snapshot path outside of test snapshots directory`.

**Why it happens:** Playwright enforces path containment; all snapshots for a spec file must live in `{spec-file}-snapshots/`.

**How to avoid:** All 18-09 snapshot paths use `['authed-draft', 'demacia-carousel.png']` — no traversal. The `region-diff.spec.ts` snapshots go in `region-diff.spec.ts-snapshots/` (if any baselines are stored there) but the spec itself does NOT use `toHaveScreenshot` — it computes the ratio manually with `pixelmatch` against in-memory buffers.

[VERIFIED: Playwright docs via WebFetch + Context7]

### Pitfall 4: ASSERT constraint on mode fields blocks future modes

**What goes wrong:** Adding `ASSERT $value IN ['auto', 'carousel', ...]` to the 3 mode fields means any new mode added in a future phase requires a schema change with downtime.

**Why it happens:** Over-constraining at the DB layer when app-layer validation is sufficient.

**How to avoid:** No ASSERT on `draft_mode`, `team_dashboard_mode`, `solo_mode`. Validation happens in the server function before calling `set_user_*_mode`. The DB schema uses `TYPE string DEFAULT 'auto'` only.

[VERIFIED: schema.surql:13-14 shows how ASSERT is used for theme field — deliberately NOT replicated for mode fields]

### Pitfall 5: `into_any()` missing causes type error in `move ||` closure

**What goes wrong:** Without `.into_any()` on both arms, Rust infers different concrete view types for each arm of the `if` expression, causing a type mismatch compile error.

**Why it happens:** `view! { <div /> }` returns a concrete type like `HtmlElement<Div>`, not `AnyView`. The two arms must return the same type.

**How to avoid:** ALWAYS add `.into_any()` to both arms of every `move || if is_pandemonium { … } else { … }` expression. No exception.

**Warning signs:** `error[E0308]: if and else have incompatible types` during `cargo check --features hydrate`.

[VERIFIED: leptos-patterns.md rule 19; ornaments.rs:146,165]

### Pitfall 6: Playwright screenshot dimensions differ between regions for pixelmatch

**What goes wrong:** `pixelmatch` requires both images to have IDENTICAL dimensions. If Pandemonium's solo page is taller than Demacia's (different component counts), `pixelmatch` throws or produces garbage output.

**Why it happens:** `fullPage: true` captures the entire scrollable page — heights differ per region.

**How to avoid:** Use `fullPage: false` (viewport-only screenshot, default) for region-diff assertions in `region-diff.spec.ts`. Viewport dimensions are fixed by Playwright's `devices["Desktop Chrome"]` config and are identical across regions. Continue using `fullPage: true` only for baseline capture in `visual-regression.spec.ts`.

[VERIFIED: Playwright docs (dimensions constraint); ASSUMED: heights will differ between regions]

### Pitfall 7: Existing Phase-17 flat baselines conflict with new subfolder entries

**What goes wrong:** If 18-09 captures new subfolder baselines WITHOUT first deleting the old flat baselines, `visual-regression.spec.ts` has two tests for the same route — the old test passes (old flat file exists) and the new test fails (subfolder file absent). The overall suite passes incorrectly.

**Why it happens:** Phase 17 tests reference `authed-draft.png` (flat). Phase 18 tests reference `authed-draft/demacia-carousel.png` (subfolder). Both can coexist as separate test cases.

**How to avoid:** At the start of 18-09 task: (1) delete the 7 old flat scoped-route baselines, (2) remove the old test cases from `visual-regression.spec.ts` that reference them, (3) add the new subfolder test cases, (4) run with `--update-snapshots` to capture new baselines. This is explicitly planned per CONTEXT.md D-10.

[VERIFIED: CONTEXT.md D-10; visual-regression.spec.ts current content]

---

## Code Examples

### fixtures.ts — setRegion helper

```typescript
// Source: fixtures.ts pattern + theme.spec.ts:23-27 (existing toggle pattern)
// e2e/tests/fixtures.ts
export async function setRegion(
  page: Page,
  region: 'demacia' | 'pandemonium'
): Promise<void> {
  const btnText = region === 'pandemonium' ? 'Pandemonium' : 'Demacia';
  const themeAttr = await page.getAttribute('html', 'data-theme');
  if (themeAttr === region) return; // already correct region
  await page.click(`button:has-text("${btnText}")`);
  // wasm-patterns rule 56: WASM Effect fires asynchronously
  await page.waitForTimeout(700);
  // Verify
  const newTheme = await page.getAttribute('html', 'data-theme');
  if (newTheme !== region) throw new Error(`setRegion failed: expected ${region}, got ${newTheme}`);
}

export async function setMode(
  page: Page,
  mode: string
): Promise<void> {
  // Click the mode toggle button with the matching label (Demacia: title case, Pandemonium: UPPER_CASE)
  await page.click(`[data-mode-toggle="${mode}"], button:has-text("${mode.toUpperCase()}"), button:has-text("${mode}")`);
  await page.waitForTimeout(500);
}
```

### schema.surql additions (18-08)

```sql
-- In schema.surql (after existing user field definitions)
-- Source: D-03, following CLAUDE.md surreal-patterns.md rule 30
DEFINE FIELD IF NOT EXISTS draft_mode ON user TYPE string DEFAULT 'auto';
DEFINE FIELD IF NOT EXISTS team_dashboard_mode ON user TYPE string DEFAULT 'auto';
DEFINE FIELD IF NOT EXISTS solo_mode ON user TYPE string DEFAULT 'auto';
```

### resolve_mode function (18-08)

```rust
// In src/pages/ (used at page entry) or src/server/db.rs
// Source: CONTEXT.md D-04 resolver shape
pub fn resolve_mode<'a>(stored: &'a str, region: &str, route: &str) -> &'a str {
    if stored != "auto" { return stored; }
    match (route, region) {
        ("draft",         "demacia")     => "carousel",
        ("draft",         "pandemonium") => "ledger",
        ("team-dashboard","demacia")     => "dashboard",
        ("team-dashboard","pandemonium") => "brief",
        ("solo",          "demacia")     => "constellation",
        ("solo",          "pandemonium") => "forge",
        _ => "carousel",
    }
}
```

### visual-regression.spec.ts subfolder tests (18-09 structure)

```typescript
// Source: D-07/D-08/D-09 + Playwright array path syntax
test('/draft demacia carousel baseline', async ({ authedPage }) => {
  await setRegion(authedPage, 'demacia');
  await authedPage.goto('/draft');
  await setMode(authedPage, 'carousel');
  await authedPage.waitForLoadState('networkidle');
  await authedPage.waitForTimeout(500);
  await expect(authedPage).toHaveScreenshot(
    ['authed-draft', 'demacia-carousel.png'],
    VR_OPTS
  );
});
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| 5-accent theme system | 2-region (demacia / pandemonium) | Phase 17 | Token set simplified; primitives branch on region not accent |
| `CompanionSigil` as sole region-branching component | 24 region-aware primitives across 7 modules | Phase 18 | Generalized pattern; `CompanionSigil` is reference implementation |
| Color-only `[data-theme]` swap | Structural component composition per region | Phase 18 | Different component trees per region, not just different colors |
| Flat snapshot directory | Route-namespaced subfolders | Phase 18 | Required to hold (mode × region) baseline combinations |
| `toMatchSnapshot()` (old) | `toHaveScreenshot()` (current) | Playwright 1.23+ | Platform suffix auto-appended; `maxDiffPixelRatio` built-in |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | SurrealDB `DEFAULT 'auto'` is NOT retroactively applied to existing user records; getters must handle `NONE` with `.unwrap_or_else(|| "auto")` | SurrealDB Schema Migration | If DEFAULT IS retroactive, the unwrap_or is harmless but unnecessary. Risk is low — pattern is defensive. |
| A2 | Phase 18's top-level AnyView branches do NOT push recursion depth beyond 512 | Recursion Limit Analysis | If wrong: `cargo check --features hydrate` will fail with recursion overflow error. Mitigation: split large pages into sub-components early. |
| A3 | The `setRegion` fixture helper can click the region toggle button by text (`button:has-text("Pandemonium")`) as in theme.spec.ts | Code Examples | If the theme_toggle.rs button text changes during 18-08 restyle, the selector breaks. Mitigation: add `data-testid` on toggle buttons during 18-08. |

**If this table is empty:** Not empty — three assumptions documented above.

---

## Critical Finding: 18-07 Gate Already Cleared

**F-01:** All 4 sibling JSX files are already present and substantive: [VERIFIED: filesystem `ls` + `wc -l`]

| File | Lines | Status |
|------|-------|--------|
| `draft-ledger/demacia.jsx` | 298 | Substantive |
| `solo-journal/pandemonium.jsx` | 298 | Substantive |
| `solo-forge/demacia.jsx` | 275 | Substantive |
| `team-game-day-brief/pandemonium.jsx` | 269 | Substantive |

**Impact on planning:** The D-13 gate is already satisfied. Plan 18-07 does NOT need a "wait for user to complete design session" step. The plan ONLY needs:
1. A preflight `ls` check (required by D-13) — it will pass
2. A content contract derivation step for each sibling (reading the JSX and deriving the contract)
3. The port-to-Leptos task

The wave structure is NOT blocked by human design work. The SPEC's "Wave 3 parallel" slot for 18-07 is fully unblocked.

---

## Wave / Dependency Ordering

Recommended wave structure (matches SPEC implementation hint with 18-07 unblocked):

| Wave | Plans | Blocker | Status |
|------|-------|---------|--------|
| 1 | 18-01 (primitives), 18-02 (skeletons) | None | Run in parallel |
| 2 | 18-03 (no-patch ports), 18-04 (light-patch), 18-05 (medium-patch) | Wave 1 complete | Run in parallel |
| 3 | 18-06 (team-dashboard heavy), 18-07 (sibling pairs) | Wave 1 complete | Run in parallel; NOT blocked by design session |
| 4 | 18-08 (mode toggles + persistence) | Waves 2+3 complete | Sequential |
| 5 | 18-09 (baselines + region-diff), 18-10 (audit) | Wave 4 complete | Run in parallel |

**Risk call:** The heaviest plan is 18-06 (team-dashboard Pandemonium rebuild with full data surface). This is the single highest-risk plan in Phase 18 due to the complexity of the Pandemonium team-dashboard design (roster mood indicators, captain's note, reasoned bans, threat ranking). It should be sequenced FIRST in Wave 3 rather than parallel-started with 18-07.

---

## Open Questions

1. **`match_report` shared component — single or two files?**
   - What we know: Both `/match/:id` and `/post-game` surface the `match-report` design page per SPEC route mapping.
   - What's unclear: Whether the current `match_detail.rs` and `post_game.rs` are similar enough to share a single `MatchReportView { region }` component.
   - Recommendation: Planner decides during 18-03. Look at `post_game.rs` and `match_detail.rs` LOC — if they diverge significantly today, keep separate; if both are thin wrappers, extract shared component.

2. **`Region` enum vs. `String` props — planner decides in 18-01.**
   - What we know: Adding a `pub enum Region { Demacia, Pandemonium }` with `impl From<&str>` is ~15 lines and eliminates all string comparison footguns across 24 primitives.
   - What's unclear: Downside is a small refactor of existing `CompanionSigil` and the `InitialTheme` type.
   - Recommendation: Add the enum in 18-01. The cost is trivial at the start of the phase; prohibitive after 24 primitives are written with `String` props.

3. **Mode toggle UI: viewport-only vs fullPage screenshots for region-diff**
   - Recommendation: Use viewport screenshots (default) for `region-diff.spec.ts` to avoid dimension mismatch issues with pixelmatch.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cargo (nightly) | All Rust compilation | ✓ | 1.96.0-nightly | — |
| node.js | e2e tests | ✓ | 25.2.1 | — |
| Playwright test runner | 18-09, 18-10 | ✓ | 1.58.2 | — |
| pixelmatch | 18-09 (region-diff) | ✗ (not yet installed) | 7.2.0 (latest) | — |
| pngjs | 18-09 (region-diff) | ✗ (not yet installed) | 7.0.0 (latest) | — |
| tailwindcss standalone | CSS compilation | ✓ (managed by cargo-leptos) | v4 | — |
| .local-design-source/ (design bundle) | 18-01 through 18-07 | ✓ | Current extraction | — |

**Missing dependencies:** pixelmatch + pngjs are not in `e2e/package.json`. Must be installed at the start of 18-09 (`cd e2e && npm install pixelmatch pngjs`). No other blocking missing dependencies.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Playwright 1.58.2 (e2e); cargo test --features ssr --lib (unit) |
| Config file | `e2e/playwright.config.ts` |
| Quick run command | `cargo check --features ssr && cargo check --features hydrate --target wasm32-unknown-unknown` |
| Full suite command | `cargo test --features ssr --lib && cd e2e && npx playwright test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| REQ-1 | ~24 primitives compile on both targets | compile | `just check` / `cargo check --features ssr && cargo check --features hydrate --target wasm32-unknown-unknown` | ✅ (justfile check target) |
| REQ-2 | PageLoading/PageEmpty render both regions | visual | `cd e2e && npx playwright test region-diff.spec.ts` | ❌ Wave 0 (create region-diff.spec.ts in 18-09) |
| REQ-3 | 7 page pairs have both-region baselines | visual | `cd e2e && npx playwright test visual-regression.spec.ts` | ✅ (exists; needs updating) |
| REQ-4 | 4 sibling pairs have both-region baselines | visual | `cd e2e && npx playwright test visual-regression.spec.ts` | ✅ (exists; needs updating) |
| REQ-5 | Mode toggle persists across navigation | e2e | `cd e2e && npx playwright test theme.spec.ts` (extend) | ✅ (theme.spec.ts exists) |
| REQ-6 | pixelDiffRatio > 0.40 per scoped route | visual | `cd e2e && npx playwright test region-diff.spec.ts` | ❌ Wave 0 |
| REQ-7 | Utility routes have zero new region conditionals | grep check | `grep -rE "is_pandemonium|theme == \"pandemonium\"" src/pages/{auth,...}.rs` | manual / CI grep |
| REQ-8 | 18-UI-REVIEW.md exists, no FAIL | manual audit | File existence check + grep | ❌ Wave 0 (created in 18-10) |

### Sampling Rate

- **Per task commit:** `cargo check --features ssr` (fast; catches SSR regressions)
- **Per task commit (WASM changes):** `cargo check --features hydrate --target wasm32-unknown-unknown` (catches WASM-specific type errors)
- **Per wave merge:** `cargo test --features ssr --lib` (≥111 tests must pass)
- **Phase gate:** Full suite green (`just full-check`) before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `e2e/tests/region-diff.spec.ts` — new spec for REQ-2 and REQ-6 (created in plan 18-09)
- [ ] `e2e/package.json` — add `pixelmatch`, `pngjs`, `@types/pixelmatch`, `@types/pngjs` (18-09 task)
- [ ] `e2e/tests/fixtures.ts` — add `setRegion` and `setMode` helpers (18-09 task)

---

## Security Domain

The phase adds no authentication logic, no new API endpoints exposed to untrusted callers, and no new data that could contain PII beyond the three mode fields (string values like 'carousel', 'auto'). Applicable ASVS review:

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | No new auth logic |
| V3 Session Management | No | Mode stored in DB via existing authenticated server functions |
| V4 Access Control | Minimal | Mode setter server functions should verify user is authenticated before writing (follows existing `set_user_theme` pattern which reads AuthSession) |
| V5 Input Validation | Yes | Mode value validated in server function before `set_user_*_mode` call; valid set is: auto, carousel, war-table, ledger, dashboard, brief, constellation, forge, journal |
| V6 Cryptography | No | No cryptographic operations |

**Threat note:** Mode fields store routing preferences only. No cross-user data leakage risk. Validation check: server function reads `user_id` from `AuthSession` (not from client-supplied parameter).

---

## Sources

### Primary (HIGH confidence)

- `src/components/ornaments.rs:128-167` — CompanionSigil AnyView branching reference implementation [VERIFIED: file read]
- `.claude/rules/leptos-patterns.md` — Project Leptos patterns (rules 19, 38) [VERIFIED: file read]
- `.claude/rules/wasm-patterns.md` — WASM safety patterns (rules 35-43) [VERIFIED: file read]
- `src/server/db.rs:4460-4520` — get_user_mode / set_user_theme pattern [VERIFIED: file read]
- `src/server/auth.rs:15-56` — AppUser / DbUser struct pattern [VERIFIED: file read]
- `schema.surql:1-15` — Existing DEFINE FIELD patterns, ASSERT constraint on theme [VERIFIED: file read]
- `e2e/tests/visual-regression.spec.ts` — Existing snapshot path pattern [VERIFIED: file read]
- `e2e/tests/fixtures.ts` — authenticatePage + setRegion basis [VERIFIED: file read]
- `e2e/playwright.config.ts` — Playwright config (project name: chromium, no fullPage default) [VERIFIED: file read]
- `input.css:152-265` — @theme block, Demacia/Pandemonium token definitions [VERIFIED: file read]
- `/leptos-rs/leptos` (Context7) — AnyView, into_any(), component prop patterns [VERIFIED: Context7]
- `/microsoft/playwright.dev` (Context7) — toHaveScreenshot array path, subfolder behavior [VERIFIED: Context7 + WebFetch]
- Tailwind v4 docs (tailwindcss.com/docs/animation, /docs/adding-custom-styles) — @theme @keyframes, @utility syntax [VERIFIED: WebFetch]
- npm registry — pixelmatch 7.2.0, pngjs 7.0.0 [VERIFIED: npm view]

### Secondary (MEDIUM confidence)

- `.local-design-source/lol-team-companion-app/project/pages/_shared/components.jsx` — Primitive API reference; skeletons-pandemonium.jsx animation names [VERIFIED: file read]
- `.local-design-source/lol-team-companion-app/project/pages/{draft-ledger,solo-journal,solo-forge,team-game-day-brief}/{demacia,pandemonium}.jsx` — sibling JSX existence and line counts [VERIFIED: ls + wc -l]
- SurrealDB DEFINE FIELD docs — DEFAULT retroactivity behavior [MEDIUM: web search + official docs; explicit retroactivity behavior not documented]

### Tertiary (LOW confidence — flagged as ASSUMED)

- SurrealDB DEFAULT non-retroactivity (A1) — web search consensus, not officially documented
- Recursion limit safety for top-level branches (A2) — reasoning from type system, not empirically tested

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries verified from official sources or project codebase
- Architecture: HIGH — based on verified codebase patterns + Leptos docs
- Pitfalls: HIGH for code pitfalls (compiler-verified); MEDIUM for SurrealDB migration behavior
- Visual testing: HIGH — Playwright behavior verified via Context7 + official docs

**Research date:** 2026-05-14
**Valid until:** 2026-06-14 (30 days; Leptos 0.8 and Tailwind v4 are stable; pixelmatch API is stable)
