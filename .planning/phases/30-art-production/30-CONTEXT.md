# Phase 30: Art Production - Context

**Gathered:** 2026-06-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Produce the real, layer-separated FLUX environment art that replaces the ImageMagick
placeholders for the immersive rooms, establishing the project's own legally-safe
visual identity (original-world, Arcane-adjacent, not Riot IP). Both regions, all
applicable rooms.

**Scope note — this phase is NO LONGER art-only.** During discussion the user chose
discrete per-tier growth overlays and Solo Journey inclusion (see D-08/D-09/D-13),
which pull a room-component **code change** and a **budget rework** into Phase 30.
Phase 30 = production of art **plus** the wiring/CI needed to consume it. The
roadmap's "art swaps in at existing file paths with no code changes" premise is
**superseded** by these decisions.

Rooms in scope: **War Room, Library (Hall of Records), Throne, AND Solo Journey**
(4 rooms × 2 regions). SC2/SC4 name only the first three; Solo Journey is added by
user decision (D-13) to avoid a visually inconsistent room shipping into beta.

</domain>

<decisions>
## Implementation Decisions

### IP / Legal Identity
- **D-01:** **Path A (original world).** Art establishes an original visual identity —
  NO "League of Legends splash art style" or any Riot proper nouns in committed
  prompts/assets. This is the locked answer to SC1 and ART-01.
- **D-02:** Decision is grounded in a first-party read of Riot's current terms
  (API Terms, Legal page, Developer General Policies — see canonical refs). Key
  finding that drove it: the real IP-risk axis is **art that mimics Riot's
  copyrighted style**, not lore names. Original-world art removes the
  copyright-derivative exposure and the "products cannot closely resemble Riot's
  games or products in style" clause, and sheds the revocable-API + AI-backlash
  risk. (Faction *names* alone are low-risk under the fan-content policy given
  registration + free tier + disclaimer — but the user chose to invent names
  anyway for maximum cleanliness.)
- **D-03:** **Invent fictional faction names** (not "keep LoL names as flavor text").
  Draft identities **pending user approval at planning** (writer should put these in
  `AI-IMAGES.md` and surface for sign-off, per D-04):
  - Order / warm-gold faction (replaces *Demacia* vocabulary) → **"The Aurelian Reach"**
    — sun-bleached limestone ramparts, gilded heraldry and hanging banners, vaulted
    halls pierced by warm dawn light; noble, ordered, ascendant.
  - Chaos / neon faction (replaces *Pandemonium* vocabulary) → **"The Sundered Hollow"**
    — fractured black-glass monoliths, magenta-and-cyan arc-light bleeding through
    cracks, brutalist ruin in electric storm-haze; volatile, void-touched.
- **D-04:** **Region IDs stay `demacia`/`pandemonium` as internal code identifiers**
  (directory names, `region` prop values, `input.css` token names, `data-accent`,
  e2e selectors). They never appear in committed art or user-facing text. Only the
  FLUX prompts and any visible labels change. This is what keeps the code-change
  blast radius limited to the overlay system (D-08), not a cross-cutting rename.
- **D-05:** **`/consult legal` is still a mandatory gate** before the first real FLUX
  generation run, and the Path A decision MUST be logged in `PROJECT.md` Key
  Decisions before that run (SC1). The web read above is the substantive basis; the
  `/consult legal` run + PROJECT.md entry are the formal record.

### Art Direction
- **D-06:** **Painterly, Arcane-adjacent** fidelity target — hand-painted, textured,
  moody (oil-painting-with-visible-brushwork feeling), described with **generic
  technique vocabulary only** (no "in the style of [proper noun]"). Reads as a
  crafted place, hides AI tells at parallax scale, matches the existing token palette.
- **D-07:** **Per-room moods (locked):**
  - War Room — tactical war-table chamber; tense, focused light over a central
    planning table; maps and banners.
  - Library (Hall of Records) — scholarly archive; towering shelves of tomes and
    scrolls; quiet contemplative reading light; dust motes.
  - Throne — regal ceremonial hall; throne dais; grand vertical space; heraldry;
    ascension.
- **D-08 (region divergence):** **Restyled, same composition.** Each room keeps the
  same compositional skeleton across regions (sky/mid/near zones align so the
  surface panels land correctly), but materials, architecture detail, and light
  genuinely differ — Aurelian Reach = intact warm limestone; Sundered Hollow =
  fractured neon-lit obsidian. Must read as visually distinct (SC4) without breaking
  parallax alignment. Explicitly NOT a flat color-swap (the Phase 18 critique).

### Growth Overlays + Scope Expansion
- **D-09 (overlay granularity):** **Discrete per-tier overlay art.** Produce
  `overlay-{axis}-{1,2,3}.webp` per growth axis for a "shelves visibly filling"
  effect — NOT a single overlay modulated by opacity. The rooms do not consume
  per-tier files today, so this requires a code change (D-10).
- **D-10 (code change absorbed into Phase 30):** Update each room's overlay
  markup/CSS from **one `<img>` faded by opacity** to **3 stacked tier images
  toggled by the existing `data-{axis}-tier` attribute**. Applies to all four rooms
  (D-13). This is in-scope for Phase 30. Researcher/planner must scope it explicitly
  and respect the ROOM-03 hydration rule (region read once at room top, passed as
  prop — never inside reactive closures; see canonical refs).
- **D-11 (budget rework absorbed into Phase 30):** Per-tier overlays break the
  current budget at today's caps (3 planes ≈ 220 KB + 9 overlays × 30 KB = 490 KB >
  400 KB). Recompute so **3 planes + 9 per-tier overlays stay ≤ 400 KB/room/region**
  — e.g. a tighter per-tier-overlay cap (~13 KB each) — final numbers are the
  planner's to set. The CI budget check must be extended from the per-file
  `find ... -size +100k` heuristic to also enforce the **per-room/region aggregate
  ≤ 400 KB** sum (per-file caps alone no longer guarantee the total).
- **D-12 (pipeline):** Generation = **FLUX.2 [pro] via BFL EU endpoint**
  (`https://api.eu.bfl.ai/v1/flux-2-pro`, `BFL_KEY`) — primary, EU-resident. Layer
  separation = **fal.ai BiRefNet v2** (`fal-ai/birefnet/v2`, `FAL_KEY`) as a
  **documented EU-policy exception** — justified because the parallax plane images
  contain no personal data, so US processing of decorative art carries no GDPR
  exposure. Log the exception in `AI-IMAGES.md` (and per the values-charter exception
  process). Post-process = ImageMagick → WebP. BiRefNet must be validated on one test
  plane for halo-fringe before the full run (SC3).
- **D-13 (Solo Journey in scope):** Regenerate Solo Journey (`/journey`) art AND
  migrate its overlays (banner/lib/armory) to the per-tier model, so all four rooms
  are consistent and none ships with placeholder/legacy-overlay art into beta. Total
  asset count rises from 36 → ~96 (4 rooms × 2 regions × [3 planes + 9 per-tier
  overlays = 12 files]).

### Claude's Discretion
- Exact FLUX prompt wording per room × region × plane (write into `AI-IMAGES.md`
  following D-06/D-07/D-08 and the Aurelian Reach / Sundered Hollow DNA; surface
  faction names for user sign-off per D-03/D-04).
- Final per-file budget caps and the CI aggregate-check mechanism (D-11).
- Seeds, inference steps, compute-path recording per asset (SC3 reproducibility).
- The exact stacking/toggling markup for per-tier overlays (D-10), subject to ROOM-03.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Art pipeline + asset contract (authoritative)
- `.planning/assets/AI-IMAGES.md` — reproducibility log + locked naming convention,
  per-region 400 KB budget, current placeholder strategy, and the (to-be-revised)
  FLUX prompts. **Prompts must be rewritten for Path A / original-world (D-01).**
- `.planning/research/STACK.md` — FLUX.2 [pro] BFL EU endpoint, fal.ai BiRefNet v2,
  ImageMagick → WebP, env vars (`BFL_KEY`/`FAL_KEY`), planned `scripts/` layout,
  EU-residency rationale. Authoritative for the pipeline (D-12).
- `.planning/research/PITFALLS.md` — the IP/AI-art risk analysis and the Path A/B
  framing; asset-budget and hydration pitfalls.

### Growth + room system (the code D-10 touches)
- `src/models/growth.rs` — `GrowthState::from_counts`, tier thresholds (0..3) that
  drive `data-{axis}-tier` attrs the per-tier overlays toggle on.
- `src/components/room/parallax.rs` — `AtmosphereRoot`, parallax plane z-index/depth
  model; where overlay `<img>` elements live.
- `src/pages/rooms/solo_journey.rs` (`:18`) — region-read-once pattern (ROOM-03
  worked example); regression target for the Solo Journey overlay migration (D-13).
- `.planning/phases/27-war-room/27-CONTEXT.md`,
  `.planning/phases/28-library-ia-healing/28-CONTEXT.md`,
  `.planning/phases/29-throne-quicknav-room-a11y/29-CONTEXT.md` — room RoomSurface
  contract, `panels: Vec<(SurfaceTab, PanelFn)>`, eager-mount + `hidden` toggling
  conventions that the per-tier overlay change must not regress.

### Hydration rule (mandatory before any room code change)
- `CLAUDE.md` gotcha #58 and `.claude/rules/leptos-patterns.md` (ROOM-03) — region
  (`InitialTheme`) read ONCE at room top, passed as `region: String` prop; never
  inside reactive closures/child components. D-10's markup change must obey this.

### Riot legal sources (basis for D-01/D-02; cite in the `/consult legal` log)
- https://developer.riotgames.com/terms — API Terms: prior-written-approval to charge
  for Game Information; Riot Marks / endorsement prohibition; IP ownership.
- https://www.riotgames.com/en/legal — Legal hub + fan-content ("Legal Jibber
  Jabber") posture.
- https://developer.riotgames.com/policies/general — General Policies: free-tier +
  "transformative" monetization rule, Approved/Acknowledged registration, "cannot
  closely resemble Riot's games or products in style," no blockchain/NFT,
  logos-only-where-inevitable, mandatory disclaimer.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- All 36 placeholder WebP files already sit at the canonical paths
  (`public/img/rooms/<room>/<region>/{plane-*,overlay-*}.webp`) — real art overwrites
  them in place. Per-tier (D-09) adds new `overlay-{axis}-{1,2,3}.webp` paths.
- ImageMagick is already installed and used for the placeholder pipeline (the
  PNG→WebP post-process step in D-12).
- The growth tier system (`growth.rs` + `data-{axis}-tier` attrs + CSS) already
  exists; D-10 changes only how overlays are rendered against those tiers, not the
  tier computation.

### Established Patterns
- Each room loads ONLY the active region's assets (path built from the `region`
  prop) — CSS-hiding the other region would still fetch it and blow budget. Per-tier
  art must preserve this (no eager-loading all 3 tiers if avoidable, or count all
  loaded tiers against the budget).
- ROOM-03 region-read-once rule (canonical refs) is a hard constraint on D-10.
- Rooms use eager-mount + `hidden` toggling (RoomSurface `panels` contract); the
  per-tier overlay toggle should follow the same data-attr-driven CSS approach, not
  reactive remounting.

### Integration Points
- D-10 touches the overlay-rendering markup in each room's atmosphere/surface
  composition (War/Library/Throne + Solo Journey).
- D-11 touches the CI budget check (currently `find public/img/rooms -name "*.webp"
  -size +100k`; add a per-room/region aggregate sum).

</code_context>

<specifics>
## Specific Ideas

- Faction visual DNA the user approved as a starting point: **Aurelian Reach**
  (warm limestone, gilded heraldry, dawn light) vs **Sundered Hollow** (fractured
  neon-obsidian, magenta/cyan arc-light, storm-haze). Names are drafts for sign-off.
- "Shelves visibly filling" is the explicit motivation for choosing per-tier overlays
  over opacity fades (D-09) — the growth must feel like the *place* accumulates, not
  just brightens.
- Region divergence must NOT read as "just a color swap" (D-08) — this was a named
  failure mode from Phase 18.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within the art/identity domain. The two scope expansions
(per-tier overlays + Solo Journey) were deliberately pulled INTO Phase 30 by user
decision (D-09/D-10/D-13) rather than deferred.

</deferred>

---

*Phase: 30-art-production*
*Context gathered: 2026-06-15*
