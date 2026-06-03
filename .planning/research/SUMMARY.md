# Research Summary — v1.4 Immersive Companion

**Project:** LoL Team Companion — v1.4 "Immersive Companion" milestone
**Domain:** 2.5D parallax rooms UI + environment-grows-with-progress meta-progression + AI art pipeline
**Researched:** 2026-06-03
**Confidence:** HIGH (architecture grounded in existing codebase; stack adds no new Rust deps; Riot IP pitfall is MEDIUM — policy is ambiguous)

---

## Executive Summary

The v1.4 milestone transforms the existing flat themed UI into an explorable 2.5D "rooms" experience by rolling the proven Solo Journey room system out to three new rooms — War Room (draft + strategy tools), Hall of Records / Library (stats + learnings), and Throne (identity + team). The entire extension requires zero new Rust/Cargo dependencies: the `AtmosphereRoot` / `Plane` / `GrowthOverlay` primitives already built for Solo Journey extend directly to new rooms by generalising the three atmosphere axis props and adding a parallel `TeamGrowthCounts` / `get_team_growth_counts` DB function for team-scoped growth. AI art production uses Black Forest Labs FLUX.2 [pro] via the EU endpoint (`api.eu.bfl.ai`, aligned with project EU-tool preference) for environment art generation, and fal.ai BiRefNet v2 for layer separation; ImageMagick WebP encoding is sufficient within the 400 KB per-room budget.

The recommended build sequence has clear dependency ordering. First, extend `TeamGrowthCounts` + `get_team_growth_counts` in the model and DB layers (Phase A), then generalise `AtmosphereRoot` axis props in CSS + component (Phase B). Once Phase B completes, War Room, Library, and Throne can be built in parallel (Phases C/D/E), each following the extract-then-compose pattern: existing page content is extracted into a `<Room>Surfaces` component rendered over `AtmosphereRoot`. The gating dependency for multi-surface rooms is the new `RoomSurface` tab component — intra-room navigation via `RwSignal<ActiveSurface>` + `<Show>`, NOT sub-routes (sub-routes re-mount `AtmosphereRoot` on every nav, destroying parallax continuity). Art production (Phase F) is decoupled: rooms ship with placeholder gradient assets; FLUX art swaps in at the same file paths with no code changes. QuickNav + nav updates (Phase G) are thin. All room work must be followed by the re-incorporated launch phases (20–24: hardening, closed-beta gate, deploy infra, compliance, pre-launch review, soft launch).

The top risk requiring an explicit go/no-go decision before any production FLUX generation is the Riot IP / AI-art question. Riot's Fan Content Policy (last updated August 2018) is silent on AI-generated art; Riot's public stance since the August 2025 Wild Rift AI trailer controversy is openly hostile to AI art depicting League universe assets. The current FLUX prompts reference "Demacia" and "Pandemonium" by name. This decision must be logged in PROJECT.md (Path A: invent original factions vs. Path B: accepted risk) before running FLUX on production assets. Two accessibility blockers also require resolution before closed beta: the `prefers-reduced-motion` runtime change-listener is missing from `AtmosphereRoot` (only a mount-time check exists), and QuickNav lacks a focus trap implementation. Finally, the anti-structural-branching hydration rule from Phase 18.2 must be codified in `leptos-patterns.md` and `CLAUDE.md` before any new room is built — it is the root cause of the tachys Option::unwrap panic and is currently invisible to agents that only read those files.

---

## Key Findings

### Recommended Stack

No Cargo.toml changes. All web-sys types needed by new rooms are already declared or transitively provided by Leptos (`Window`, `HtmlElement`, `CssStyleDeclaration`, `MediaQueryList`; `MouseEvent` and `KeyboardEvent` are transitively covered — verified via `cargo check --features hydrate --target wasm32-unknown-unknown`). The animation system (RAF + CSS `--px`/`--py` custom properties to GPU-composited transforms) extends to all new rooms with zero modification. GSAP, Three.js, and any JS animation library are explicitly rejected per project guardrails.

**Art pipeline additions — external tooling only, no server-side dependency:**
- **Black Forest Labs FLUX.2 [pro]** via `api.eu.bfl.ai` — environment art generation; German company (Freiburg), explicit EU endpoint, GDPR-aligned, ~$0.045/image. Supersedes FLUX.1 [pro] referenced in `AI-IMAGES.md` at the same price; locked prompts transfer directly.
- **fal.ai BiRefNet v2** (`fal-ai/birefnet/v2`) — background removal for mid/near/overlay planes producing transparent PNG with alpha; dual-pathway segmentation handles painterly edges better than rembg; no local GPU needed. US-hosted — prefer BFL for primary generation; use fal.ai only for layer separation.
- **ImageMagick** (already installed) — PNG to WebP: `-quality 80 -define webp:alpha-quality=100` for transparent planes; `-quality 80` for opaque sky plane. Sufficient at this asset count and budget; cwebp and avifenc add setup complexity with no meaningful benefit.
- **New env vars:** `BFL_KEY` (add to `.env.example`); `FAL_KEY` (already in `.env.example`). Both used only in offline shell scripts; never loaded by the server binary.

Art pipeline lives in `scripts/`: `gen-room-art.sh`, `remove-bg.sh`, `encode-webp.sh`, `verify-budget.sh` — each independently runnable so a single-plane regeneration does not re-run the full pipeline.

### Expected Features

**Must have (table stakes) — v1.4 launch:**
- Parallax atmosphere (pointer-driven depth) for all three new rooms — already built in `AtmosphereRoot`
- Growth overlays fading in by discrete tier (0–3) per axis — already built in `GrowthOverlay`
- Contrast scrim keeping surface layer readable at all art positions — already built in `AtmosphereRoot`
- Room-specific asset loading (active region only, never both regions simultaneously)
- `prefers-reduced-motion` compliance: CSS `@media` zeroing `--depth` + runtime `MediaQueryList.addEventListener('change')` in `AtmosphereRoot` — **the runtime listener is currently MISSING and must be added**
- QuickNav (Cmd/Ctrl+K) promoted to shell-level so it works from every room
- Stable URLs per room (`/war-room`, `/library`, `/throne`) with existing flat routes preserved as deep-link targets
- ARIA tablist/tab/tabpanel on the `RoomSurface` tab bar; focus trap + `aria-modal="true"` + `role="dialog"` on QuickNav overlay — **focus trap is currently MISSING**
- `GrowthState` server fn per room reading team-scoped DB counts
- IA healing: opponents page folded into War Room tab; learnings folded into Library tab; old routes redirect

**Should have (competitive differentiators):**
- `RoomSurface` tab component with `RwSignal<ActiveSurface>` + `<Show>` — **gating dependency for multi-surface rooms**; enables War Room's 5-surface strip (Draft / Tree-Drafter / Game Plan / Opponents / Ban Priority) and Library's 5-surface strip; must use ARIA tablist pattern
- Growth tier notification moment: signal comparison prev/new tier, 1.5s CSS `tier-advanced` class, ambient glow pulse — Hades-style "room quietly changes" payoff
- Diegetic copy vocabulary: per-room label fn mapping feature names to room language
- Region-reactive ambient motion: CSS keyframe torch-flicker (Demacia, 4s) vs electric-haze (Pandemonium, 2s), gated by `prefers-reduced-motion`
- Sky plane `loading="eager"` + `<link rel="preload">` in room `<head>` to prevent atmosphere FOUC

**Defer to v1.5 / v2+:**
- Room-to-room view transition animation (CSS View Transitions API wrapper in WASM)
- GrowthState localStorage cache (5-min TTL) — plain mount-time fetch is acceptable at launch
- Discrete per-tier art (54 images: 3 rooms x 3 axes x 3 tiers x 2 regions)
- Optional ambient audio
- Mobile-responsive room variant (requires fundamentally different parallax model)

**Anti-features (do not build):**
- Forced exploration ("click the bookshelf to open stats") — destroys power-user speed
- Mandatory animated entrance (>300ms transition) — charming once, annoying on the 20th visit
- XP bar / level counter overlaid on room — cheapens the "your time shows" narrative
- Clickable NPCs — dead-end interactions, expensive to build well
- Diegetic data tooltips ("hover shelf to see learning count") — slower than surface header stats

### Architecture Approach

The extension follows the **extract-then-compose** pattern from `solo_dashboard.rs` to `solo_journey.rs`: each room's page content is extracted into a `<Room>Surfaces` component that runs independently, and the room page composes it over `AtmosphereRoot`. Existing flat routes remain in `app.rs` unchanged and valid as deep-link targets. Rooms are additive new routes, not replacements.

**Major components:**

1. **`AtmosphereRoot`** (`src/components/room/parallax.rs`) — owns the pointer/CSS-var bridge; writes `--px`/`--py` to DOM; hosts planes, overlays, scrim, and the z-10 surface slot. Needs new optional tier props for each room's axis names (implemented as `#[prop(optional, into)]` with `Signal::derive(|| 0u8)` defaults).
2. **`TeamGrowthCounts` / `TeamGrowthState`** (`src/models/growth.rs`) + **`get_team_growth_counts`** (`src/server/db.rs`) — parallel to the existing user-scoped growth pair; reads team-scoped tables (`draft`, `opponent`, `ban_priority`, `post_game_learning`, `team_note`, `player_match`) in a single batched 6-statement round-trip.
3. **`RoomSurface` tab component** (new, inside each `<Room>Surfaces`) — `RwSignal<ActiveSurface>` + `<Show>` per tab; tab bar uses `role="tablist"`, `role="tab"`, `aria-selected`, `aria-controls`; does NOT use sub-routes.
4. **`<Room>Room` pages** (`src/pages/rooms/war_room.rs`, `library.rs`, `throne.rs`) — reads `InitialTheme` ONCE at top level into `region: String`; fetches team growth resource; composes `AtmosphereRoot` + `<Room>Surfaces`. Rooms are shells — no `#[server]` functions for surface-level operations; room file target ≤ 200 lines.
5. **`QuickNav`** — promoted from Solo Journey to shell-level in `app.rs`; room entries added to `HUB_ROUTES`; `hub_for_path()` returns empty sub-nav for room paths.

**AtmosphereRoot axis names per room:**

| Room | Axis A | Axis B | Axis C | data-* attributes |
|------|--------|--------|--------|-------------------|
| Solo Journey | `lib` | `armory` | `banner` | data-lib-tier, data-armory-tier, data-banner-tier |
| War Room | `tactics` | `intel` | `command` | data-tactics-tier, data-intel-tier, data-command-tier |
| Library | `records` | `tomes` | `analysis` | data-records-tier, data-tomes-tier, data-analysis-tier |
| Throne | `roster` | `progression` | `crown` | data-roster-tier, data-progression-tier, data-crown-tier |

**Critical rule — no structural branching on `region`:** `InitialTheme` (region) must be read ONCE at the room component top level and passed as a `region: String` prop to child components. It must never be read inside a reactive closure, `Signal::derive()`, or any child component. Reading context inside a reactive primitive caused the Phase 18.2 tachys `Option::unwrap()` hydration panic at `html/mod.rs:217`. This rule must be in `leptos-patterns.md` and `CLAUDE.md` before new rooms are built.

**Asset naming convention** (confirmed from `public/img/rooms/solo-journey/`):

```
public/img/rooms/<room-slug>/<region>/plane-sky.webp
public/img/rooms/<room-slug>/<region>/plane-mid.webp
public/img/rooms/<room-slug>/<region>/plane-near.webp
public/img/rooms/<room-slug>/<region>/overlay-<axis>.webp  (one per growth axis)
```

### Critical Pitfalls

1. **Riot IP / AI-generated League-universe art — go/no-go required before FLUX.** The Fan Content Policy (Aug 2018) is silent on AI art; Riot's Aug 2025 stance (Wild Rift AI trailer apology) is hostile. FLUX prompts currently name "Demacia" and "Pandemonium". Decision must be logged in PROJECT.md before any FLUX run: Path A (invent original factions — safe, only prompts change) or Path B (accept risk — keep League names, document, plan art redo before paid tier). Running FLUX and committing outputs is the point of no return.

2. **`prefers-reduced-motion` runtime listener is missing.** `AtmosphereRoot` checks the media query at mount only. A user who enables Reduce Motion mid-session cannot stop the parallax without reloading — WCAG 2.3.3 violation. Fix: add `MediaQueryList.addEventListener('change', ...)` in the `#[cfg(feature = "hydrate")]` block in `AtmosphereRoot`; store MQL handle in a `StoredValue` for cleanup. Also add an in-app "Pause atmosphere" toggle.

3. **QuickNav focus trap not implemented.** When Cmd+K overlay opens, keyboard users tab into parallax plane elements (`aria-hidden` but DOM-present). WCAG 2.1 failure (SC 2.1.2, SC 2.4.3). Fix: implement focus trap in `quick_nav.rs` using web-sys DOM API; `aria-modal="true"` + `role="dialog"`; store trigger element in `StoredValue<Option<web_sys::Element>>` for focus restoration on Escape.

4. **Anti-structural-branching rule undocumented — invisible to new room authors.** The Phase 18.2 tachys panic pattern (growth-tier or region branching inside `view!` causing SSR/hydrate DOM structure mismatch) is documented in `parallax.rs` comments but NOT in `leptos-patterns.md` or `CLAUDE.md`. Any agent building a new room from CLAUDE.md will repeat the error. Fix: add rule to both files before building War Room.

5. **Mousemove listener accumulation on SPA route round-trips.** Leptos issue #3665: `on_cleanup` may not fire atomically on first nav under same parent, leaving two simultaneous listeners. Manifests as doubled parallax amplitude after returning to a room. Fix: add a `static AtomicBool` "listener active" guard in `AtmosphereRoot`; verify with a round-trip e2e test (`getEventListeners(window).mousemove.length === 1` after Solo Journey to War Room to Solo Journey).

---

## Implications for Roadmap

Research produces a clear dependency-ordered build sequence. Phases A and B must complete before any room phase begins. Rooms C/D/E can then run in parallel. Art production (Phase F) is fully decoupled from coding. All room work must precede the launch phases (20–24).

### Phase A: Growth Model + DB Extension

**Rationale:** `TeamGrowthCounts` / `TeamGrowthState` / `get_team_growth_counts` is the data foundation all three new rooms depend on. Pure model/DB, no UI, fully unit-testable in isolation.
**Delivers:** Team-scoped growth counts query (6-statement batched round-trip); `TeamGrowthState::from_counts` with room-specific tier thresholds; unit tests mirroring existing `growth.rs` suite.
**Prerequisite:** Verify `ban_priority` and `team_note` table names in `schema.surql` before writing the query.
**Avoids:** Mixing user-scoped and team-scoped growth in one function (Architecture Anti-Pattern 4).
**Research flag:** None — pure extension of existing well-understood patterns. Skip research-phase.

### Phase B: AtmosphereRoot Axis Generalisation + Hydration Rule Codification

**Rationale:** New rooms pass room-specific tier signal names to `AtmosphereRoot`; the current fixed props must become optional with defaults. Simultaneously, the anti-structural-branching rule must be codified in `leptos-patterns.md` and `CLAUDE.md` — this gates all subsequent room work.
**Delivers:** `AtmosphereRoot` with optional `#[prop(optional, into)]` fields for all four rooms' axis names; new `data-*-tier` CSS rules in `input.css`; Solo Journey regression test; anti-structural-branching rule documented in `leptos-patterns.md` + `CLAUDE.md`.
**Avoids:** Hydration panic on tier branching (Pitfall 2); region-branching hydration panic (Phase 18.2 incident).
**Research flag:** None — mechanical prop generalisation. Skip research-phase.

### Phase C: War Room

**Rationale:** Highest-value room (most-used features). Introduces the `RoomSurface` tab component for the first time — this gating component establishes the pattern reused by Library and Throne. Build first when serialising.
**Delivers:** `WarSurfaces` component with `RwSignal<WarSurface>` + ARIA tablist (Draft / Tree-Drafter / Game Plan / Opponents / Ban Priority); `WarRoomRoom` page composing `AtmosphereRoot` over extracted surfaces; `get_team_growth_state` server fn; placeholder gradient WebP assets at correct paths; mousemove listener accumulation guard (first time two rooms coexist in router); hydration-no-panic e2e gate at Tier 2 seed data; e2e smoke coverage for `/war-room`.
**Avoids:** Room scope creep — room file ≤ 200 lines, no `#[server]` fns in room file (Pitfall 12); sub-routing inside room (Anti-Pattern 3); listener accumulation (Pitfall 3).
**Research flag:** None — all patterns validated in Solo Journey spike. Skip research-phase.

### Phase D: Library / Hall of Records

**Rationale:** Parallel to War Room once Phase B completes. Reuses the `RoomSurface` pattern from Phase C. Second most-used feature cluster.
**Delivers:** `LibSurfaces` with tab strip (Stats / Match Detail / Analytics / Post-Game / Learnings); `LibraryRoom` page; library axis growth state (`records`, `tomes`, `analysis`); placeholder assets; IA healing — personal-learnings route redirects into Library tab.
**Avoids:** Same pitfalls as Phase C. Verify `team_note` table name before writing growth query.
**Research flag:** None — War Room patterns apply directly. Skip research-phase.

### Phase E: Throne

**Rationale:** Simplest room — fewer surfaces, simpler content. Parallel to Library or after it. May be presented as panels on one screen without a full tab strip.
**Delivers:** `ThroneSurfaces`; `ThroneRoom` page; growth axes using team roster count, LP milestones, action items completed; placeholder assets; e2e smoke coverage for `/throne`.
**Research flag:** None — simpler than War Room/Library. Skip research-phase.

### Phase F: Art Production (Decoupled — Calendar-Parallel with C/D/E)

**Rationale:** Offline human-in-the-loop process with no code dependency on rooms (art swaps in at the same file paths). Run in parallel with room coding so art is ready when rooms ship.
**Delivers:** 36 WebP files (3 rooms x 2 regions x 6 planes/overlays); each within per-asset budget (sky ≤ 80 KB, mid ≤ 80 KB, near ≤ 60 KB, overlays ≤ 30 KB, total ≤ 400 KB/room/region); `AI-IMAGES.md` updated with seed + compute path + final size per asset; CI check (`find public/img/rooms -name "*.webp" -size +100k` returns 0).
**Blocking prerequisite:** Riot IP go/no-go decision logged in PROJECT.md before first FLUX run.
**Avoids:** Asset budget overrun (Pitfall 14); halo fringe artifacts — use isolated-plane generation + BiRefNet + 1px erode mask shrink (Pitfall 9); atmosphere FOUC — add sky plane `<link rel="preload">` in each room head (Pitfall 8).
**Research flag:** Riot IP go/no-go is the one externally-dependent decision research cannot resolve. Validate BiRefNet halo-mitigation on one test plane before committing to 36 assets.

### Phase G: QuickNav + Nav Update (Thin)

**Rationale:** Small and independently deployable. Slots in alongside any room phase after Phase B.
**Delivers:** QuickNav works from all rooms; `/war-room`, `/library`, `/throne` reachable from Cmd+K; `hub_for_path()` updated to suppress global sub-nav strip on room paths; QuickNav focus trap implemented (currently missing — Pitfall 7).
**Research flag:** None. Skip research-phase.

### Phases 20–24: Re-incorporated Launch Phases

**Rationale:** Deferred from v1.3. Non-optional — without them "closed beta" is an unguarded URL on an unmonitored server. Phase 20 (production hardening) must precede any second room. Minimum viable immersive beta = Solo Journey room + Phases 20–24 complete, with War Room added as a beta update.
**Delivers:** Phase 20 — production hardening + closed-beta access gate; Phase 21 — Hetzner deploy infra + blue-green deployment runbook; Phase 22 — compliance + transparency; Phase 23 — pre-launch full review; Phase 24 — soft launch + feedback loop.
**Avoids:** Launch work squeezed by rooms scope creep (Pitfall 15); stale SSR during deploy window — blue-green procedure required (Pitfall 13).
**Research flag:** None — existing v1.3 plans apply. Skip research-phase.

### Phase Ordering Rationale

- **A before all rooms:** `get_team_growth_counts` is called by every new room's growth server fn; building without it requires stubs that create rework.
- **B before all rooms:** Generalised `AtmosphereRoot` axis props are the integration contract rooms compile against; also the mandatory moment to codify the hydration rule.
- **C/D/E parallelisable:** Once A and B are complete, rooms have no cross-dependencies. War Room first when serialising (highest feature value; establishes `RoomSurface` template).
- **F decoupled from code:** Art swaps in at existing file path convention; rooms ship with placeholder gradients. Calendar-parallel with C/D/E.
- **G anytime after B:** Small and independent.
- **20–24 after C alone (minimum) or after C/D/E/F/G (full):** Do not open closed beta without Phase 20 in place.

### Research Flags

Phases requiring deeper research during planning:
- **Phase F (art production):** The Riot IP go/no-go is an externally-dependent decision research cannot resolve — project owner must make this call first. Validate BiRefNet workflow on one test plane before full run.

Phases with well-documented standard patterns (skip research-phase):
- **Phase A:** Pure extension of existing `growth.rs` pattern.
- **Phase B:** Mechanical prop generalisation of `AtmosphereRoot`.
- **Phases C, D, E:** Extract-then-compose pattern from Solo Journey spike — all patterns validated in production.
- **Phase G:** Small nav wiring.
- **Phases 20–24:** Existing v1.3 plans apply.

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Zero new Rust deps; all web-sys coverage verified against Cargo.toml; art pipeline tooling confirmed via official docs (BFL EU endpoint, fal.ai BiRefNet v2, ImageMagick WebP). |
| Features | HIGH | UX patterns grounded in game hub design literature (Hades, LoL event hubs, Smashing Magazine ambient motion 2025). Tier thresholds are starting hypotheses — calibrate during beta. |
| Architecture | HIGH | All patterns grounded in the existing codebase. `TeamGrowthCounts` shape is design work, not observation — verify `ban_priority` and `team_note` table names in `schema.surql` before Phase A. |
| Pitfalls | HIGH (technical) / MEDIUM (Riot IP) | WASM/Leptos/accessibility pitfalls verified against codebase and official docs. Riot IP: the policy is genuinely ambiguous — only the project owner can resolve it. |

**Overall confidence:** HIGH for technical execution. The single MEDIUM is the Riot IP decision, which is not a technical question.

### Gaps to Address

- **Riot IP decision (Path A vs. Path B):** Must be logged in PROJECT.md Key Decisions before art production phase. Most time-sensitive unresolved item; the only external blocker.
- **`ban_priority` and `team_note` table names:** Must be confirmed against `schema.surql` before Phase A. Adjust field names if they differ.
- **Growth tier thresholds:** Calibrated hypotheses from game design research; validate against actual user data in closed beta and adjust in `TeamGrowthState::from_counts`.
- **`prefers-reduced-motion` runtime MQL cleanup detail:** Verify the `StoredValue<Closure<...>>` pattern for MQL listener cleanup against the existing `on_cleanup` pattern in `parallax.rs` at implementation time.

---

## Sources

### Primary — HIGH confidence

- `src/components/room/parallax.rs` — AtmosphereRoot implementation, RAF pattern, asset budget warning, reduced-motion mount-time check
- `src/pages/rooms/solo_journey.rs` — room shell composition pattern, region-read-once, asset path convention
- `src/models/growth.rs` — GrowthState, tier math, unit tests
- `src/server/db.rs` — `get_growth_counts`, team_id strip patterns, batched query shape
- `src/components/room/quick_nav.rs` — QuickNav palette, HUB_ROUTES, keyboard listener
- `schema.surql` — ground truth for table names; verify `ban_priority`, `team_note` before Phase A
- `.planning/assets/AI-IMAGES.md` — asset budget (400 KB), locked FLUX prompts, placeholder pipeline
- [Black Forest Labs API docs](https://docs.bfl.ml/api_integration/integration_guidelines) — EU endpoint, polling pattern, FLUX.2 [pro] model name
- [fal.ai BiRefNet model page](https://fal.ai/models/fal-ai/birefnet) — v2 endpoint, PNG+alpha output confirmed
- [MDN: prefers-reduced-motion](https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-reduced-motion) — runtime change event pattern
- [WCAG 2.3.3 Animation from Interactions](https://www.w3.org/WAI/WCAG21/Understanding/animation-from-interactions.html) — runtime toggle requirement
- [Smashing Magazine: Ambient Animations in Web Design (2025)](https://www.smashingmagazine.com/2025/09/ambient-animations-web-design-principles-implementation/)

### Secondary — MEDIUM confidence

- [Riot Fan Content Policy](https://www.riotgames.com/en/legal) — last updated August 2018; silent on AI art
- [Riot API Terms](https://developer.riotgames.com/policies/general) — commercial use permitted; AI art not addressed
- [PCGamesN: Wild Rift AI trailer backlash (August 2025)](https://www.pcgamesn.com/league-of-legends/wild-rift-trailer-backlash) — Riot hostile stance on AI art confirmed
- [fal.ai privacy policy](https://fal.ai/legal/privacy-policy) — US-domiciled data processing confirmed
- [Leptos router cleanup timing issue #3665](https://github.com/leptos-rs/leptos/issues/3665) — `on_cleanup` timing edge case
- [Chrome performant parallaxing](https://developer.chrome.com/blog/performant-parallaxing) — transform-only parallax, `will-change` layer promotion
- Hades / Stardew Valley / Cult of the Lamb meta-progression design patterns — implicit growth, non-regression principle

### Tertiary — calibration only

- Growth tier threshold recommendations — hypotheses from game design literature; validate against real user data in beta
- Per-asset WebP size targets (sky ≤ 80 KB, etc.) — derived from 400 KB total budget divided by 6 assets; adjust per actual FLUX output quality

---

*Research completed: 2026-06-03*
*Ready for roadmap: yes*
