# Feature Research — v1.4 Immersive Companion

**Domain:** Immersive explorable-hub UI with environment-grows-with-progress meta-progression
**Researched:** 2026-06-03
**Confidence:** HIGH for UX patterns and anti-patterns (well-documented game and web design literature);
MEDIUM for specific implementation complexity in Leptos/WASM context (extrapolated from Solo Journey spike)

---

## Context

This research covers only the **new immersive-presentation features** of v1.4. The underlying functional
features (draft planner, tree-drafter, champion pool, stats, match detail, game-plan, post-game,
opponents, analytics, learnings, goals/LP history) are already built and out of scope.

The Solo Journey room is already built as the proving spike:
- `AtmosphereRoot` with parallax planes (sky/mid/near, pointer-driven depth offsets)
- `GrowthOverlay` fading in across 4 discrete opacity tiers driven by `data-*-tier` attributes
- `GrowthState` computed from DB counts (library_tier, armory_tier, banner_tier)
- Layer-separated WebP art loaded per-region (demacia/pandemonium)
- Contrast scrim (radial-gradient vignette) keeping surfaces readable against any art

The research question: **what patterns govern immersive hub UX, meta-progression visibility, ambient
motion, command-palette fast-paths, and diegetic data display** — and what are the table stakes,
differentiators, and anti-features for v1.4?

---

## Reference Ecosystem Surveyed

**Game hub UX:**
- League of Legends client event hubs (Spirit Blossom 2020, RiotX Arcane 2021) — interactive hubs
  with region art, dialogue-driven NPC interaction, clickable scene elements, unlockable narrative
- Hades — House of Hades as the between-run hub; environment adds furniture, decoration, NPCs, ambient
  dialogue as Darkness/Gemstones accumulate; clear visual proof of meta-progression
- Cult of the Lamb — tiered building upgrades where each tier is visually distinct and space transforms;
  followers' presence and structures fill the camp progressively
- Stardew Valley — farm visual progression; crops, buildings, paths transform the same space over time

**Web parallax / ambient motion:**
- Smashing Magazine (2025) on ambient animation principles: slow duration, seamless loop,
  layered complexity, `prefers-reduced-motion` compliance, `aria-hidden` on decorative elements
- Web parallax conventions: pointer-driven layer offsets, sky layer slowest, near layer fastest;
  `transform` and `opacity` only (GPU compositing); 150ms exit / 210ms enter asymmetric timing

**Command palette:**
- VS Code, Figma, Notion, Linear — Cmd/Ctrl+K universal escape hatch
- Best practices: show recently-used commands first, display shortcut hints, make palette discoverable
  via a UI hint not only keyboard, keep search fast, confirm execution

**Diegetic data display:**
- Dead Space (RIG suit), Firewatch (physical map), Alien: Isolation — world objects as UI
- The authenticity-usability trade-off: beautiful diegetic elements fail when they prevent fast reading
- Contemporary games use "fluid mix" of diegetic + non-diegetic — never go purely diegetic
- The LoL room design decision already chosen: "atmosphere is the frame, surfaces are the work"
  = the non-diegetic surface layer (z-10) is where real work happens; parallax is purely atmospheric

---

## Feature Landscape

### Table Stakes (Users Expect These)

Features required for the immersive-room concept to feel complete and non-broken.
Missing any = the product feels like a broken prototype, not a shipped feature.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Parallax atmosphere: pointer-driven depth | Every game-client hub and premium web landing page uses pointer-driven parallax. Without it, the "2.5D room" reads as a flat static background with no depth cues. | LOW | Already built in `AtmosphereRoot` for Solo Journey. Extend to War Room and Library with same component. |
| Growth overlays fade in by discrete tier | The core "your time shows" promise. If the environment looks the same on day 1 and day 100, the concept fails entirely. Tiers must be visually distinct (not a barely-perceptible gradient). | LOW | Already built via `GrowthOverlay` + `data-*-tier` CSS. Confirm tier thresholds produce visible delta at each step. |
| Contrast scrim keeping surfaces readable | Backgrounds fight text at high art fidelity. A radial-gradient vignette is non-negotiable for readability. Without it the work layer is illegible against art. | LOW | Already built in Solo Journey. Replicate for each room. |
| Room-specific asset loading (one region only) | Loading all region variants on every page visit blows the 400 KB budget and is perceptually slower. Users expect fast load even in an immersive app. | LOW | Already enforced by solo_journey.rs (dir constructed from region prop). Replicate pattern. |
| `prefers-reduced-motion` compliance | Users with vestibular disorders MUST be able to disable parallax. Failing this is an accessibility violation, not a preference. | LOW | Add `@media (prefers-reduced-motion: reduce)` CSS block that sets `--depth: 0` on all planes and pauses ambient keyframe animations. |
| QuickNav (Cmd/Ctrl+K) from every room | Power users who use the app daily will not tolerate clicking through room art to navigate. A command palette summonable from anywhere is the expected escape hatch in modern web tools. | MEDIUM | Already built and proven on Solo Journey. Must work identically across all rooms (global component in nav or shell). |
| Room routing: each room = a stable URL | Users bookmark rooms, share links with teammates, open specific surfaces directly. Rooms without stable URLs feel like modals, not destinations. | LOW | Each room = a `<Route>` in `app.rs` (already pattern from Solo Journey: `/solo-journey`). |
| Surface layer is always reachable (z-10 stays accessible) | The work layer must never be obscured by parallax art or overlays in any tier or pointer position. Tab order, focus indicators, and click targets must all remain above the atmosphere. | LOW | Enforced by `z-index: 10` on surface div. Verify no tier-3 overlay or near-plane element occludes interactive surfaces. |
| Empty / Seed state looks intentional (not broken) | First-time users see tier-0. If tier-0 looks like a failed asset load or a half-rendered scene, the concept reads as buggy. The seed state must be a coherent, complete atmosphere with clear "this will grow" affordance. | MEDIUM | The seed-state art (placeholder gradients) must look like a sparse but intentional room, not a missing texture. Explicit "0 items" empty states on surfaces + copy that references the room ("Your armory is empty — add champions to your pool.") |

### Differentiators (Competitive Advantage)

Features that make the immersive companion meaningfully better than a plain themed web app.
Not table stakes, but the reason the "rooms" direction exists.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Per-axis growth (three independent axes per room) | Library, Armory, and Banner tiers progress independently. A player with a large champion pool but no learnings sees the armory full and the library sparse. This "profile" of growth is personal and non-generic. It's a visual fingerprint of how you use the app. | MEDIUM | Already modeled in `GrowthState` (library_tier, armory_tier, banner_tier). Extend GrowthState to cover War Room axes (draft count, opponent profiles, game plans) and Library axes (match count, post-game reviews, learnings). |
| Discrete tier art, not continuous opacity | Continuous opacity modulation (tier-0=0%, tier-3=100%) is the current approach (fast to ship). Discrete per-tier WebP overlays (`overlay-lib-1.webp`, `overlay-lib-2.webp`, `overlay-lib-3.webp`) let each tier tell a visual story (e.g., tier-1: one shelf with a few scrolls; tier-2: full shelves; tier-3: overflowing + a glowing tome on a pedestal). This makes the transition *events* feel like achievements, not animations. | HIGH | Deferred per AI-IMAGES.md: "production may split each into discrete per-tier art." Flag as a v1.4 stretch goal vs. the opacity-modulation MVP. |
| Growth tier notification / moment | When a threshold is crossed mid-session (e.g., player adds their 10th learning and the library tier advances), show a brief ambient moment: the overlay fades in with a subtle glow pulse. No modal, no XP bar — just the room changing. This is the "your time shows" payoff moment. | MEDIUM | Implement as a Leptos signal comparison: if `new_tier > old_tier`, add CSS class `tier-advanced` for 1.5s then remove. The overlay's `transition: opacity 800ms ease` already provides the fade. |
| Room-to-room transition polish | Entering the War Room feels different from entering the Library — not just different art, but a sense of moving through space (brief parallax zoom-in on room entry, planes settling into position). Makes navigation feel spatial, not like a tab switch. | MEDIUM | CSS View Transitions API (`view-transition-name` on the room root) + cross-fade or slide animation on route change. Leptos router does not currently wire this automatically. Requires `wasm_bindgen` + `document.startViewTransition()` wrapper. Check `prefers-reduced-motion` gate. |
| Diegetic-framing copy in surface headers | Surface headers inside a room speak the room's language: "Drafts & Strategy" becomes "War Room Plans." "Stats" becomes "Records." "Champion Pool" becomes "The Armory." The UI *sounds* like it lives in the world. | LOW | Pure copy change. Establish copy rules: each room has a voice/vocabulary table. Implement as a `room_label()` fn mapping route + feature name to the diegetic label. |
| Region-reactive ambient motion | Demacia rooms have slow, warm torch-flicker ambient motion (subtle opacity pulse on mid-plane, 4s loop). Pandemonium rooms have faster, cooler electric-haze flicker (2s loop). The atmosphere *breathes* differently per region. | MEDIUM | CSS keyframe animation on plane elements, gated behind `prefers-reduced-motion: no-preference`. Duration and timing-function parameterized per region via CSS custom property (`--ambient-period: 4s`). |
| Keyboard shortcut hints visible in QuickNav | QuickNav shows not only destinations but the keyboard shortcut to reach any feature directly (if assigned). Power users can scan once and learn. Casual users discover shortcuts exist. | LOW | Add a `shortcut` field to the QuickNav item struct. Render as a `<kbd>` element in the command palette row. |
| Surface tabs within a room (War Room has multiple tools) | War Room hosts draft, tree-drafter, game-plan, opponents, ban-priority. These cannot all be simultaneously visible. Tab navigation within the room (keyboard + click) with the tab bar rendered in the surface layer (not the atmosphere) lets users switch contexts without leaving the room. | HIGH | New component: `RoomSurface` with a tab strip at top of z-10 layer. Each tab renders a different feature surface. Tab state stored in URL param (`/war-room?tab=draft`) for deep-linking and browser back. |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Forced exploration: "click the bookshelf to open stats" | Thematic consistency, game-like feel | Destroys power-user speed. Users who need to check a stat quickly have to find the right clickable object first. This pattern is documented as the #1 complaint in spatial web UIs (visual style without functional clarity). The app is a productivity tool, not a game. | Keep the atmosphere purely aesthetic. All navigation happens via surface tabs, nav bar, and QuickNav. Never gate a feature behind clicking an art element. |
| Animated entrance per feature (loading screen) | "Entering the War Room" feels epic | A 2-second animated transition on every navigation is charming once and annoying forever. Power users who visit the draft 10 times per day will skip-click through any mandatory transition. | Use CSS view transitions (150-300ms cross-fade) for ambient spatial feel without blocking interaction. Never > 300ms for a navigation transition. |
| Parallax scroll (scroll-driven depth) | Scrollytelling sites look impressive | The rooms are full-viewport (no scroll needed for the atmosphere). Scroll-based parallax requires the user to be in the right scroll position to see the effect. Most users will immediately scroll down to reach content, losing the effect entirely. Pointer-driven parallax (already built) works regardless of scroll position. | Keep pointer-driven depth. Use CSS `min-h-screen` on the room root so the atmosphere fills the viewport above the fold. |
| XP bar / level counter overlaid on the room | Gamification visual language users know | Explicit progression meters (XP bars, level numbers) signal "grind" rather than "your real work shows." Hades succeeded with implicit progression (the house just fills up). An explicit XP bar cheapens the "your time shows" narrative and adds cognitive load without information value. | Growth tier advancement is the metric. Surfaces show real data (match count, learning count). The tier changes are the reward — never numerify them with an overlay counter. |
| Clickable NPCs / room characters | LoL Spirit Blossom feel | Spirit Blossom worked because it was a dedicated narrative event with dialogue trees and story investment. Adding clickable NPCs to a productivity tool creates dead-end interactions ("oh, the champion said something cool — now what?") and is extremely expensive to build well. | Use champion art as pure atmosphere (in overlay layers). No interactive NPCs in v1.4. If champion presence grows with the armory tier, that's sufficient diegetic signal. |
| Per-object tooltip showing stats ("hover the shelf to see your learning count") | "Diegetic data display" idea | The authenticity-usability trade-off from diegetic interface research: objects with information capacity limits force users to hunt for data. Reading a learning count from a tooltip on a bookshelf object is slower and less readable than showing it in the surface header. The project has already made the right call: atmosphere is the frame, surfaces are the work. | Show real counts in the surface layer (e.g., "32 learnings" as a header stat). The atmosphere reflects this implicitly through the tier of the overlay, not via interactive tooltips. |
| Continuous parallax depth-of-field / blur | Creates a sense of volume | CSS blur on background layers (`filter: blur(Xpx)`) is GPU-composited but increases memory pressure on WASM. More importantly, blurred background text (if any art has text-like elements) can trigger WCAG failures. | Apply blur in the source art (at generation time in fal.ai prompts), not at render time. The prompt already specifies "atmospheric haze" — the blur is baked in, not composited live. |
| Auto-play ambient audio (rain, forge sounds, library quiet) | Immersion | Unsolicited audio on a productivity web app is universally disliked. Users frequently have music/calls running. Autoplay audio is also blocked by most browsers unless the user has interacted. | If audio is added at all (v2+), make it opt-in via a visible toggle in the room header. Out of scope for v1.4. |
| Mobile-responsive room layout | More users | Desktop-first constraint in PROJECT.md; the parallax + layer system requires a viewport with stable pointer position. Mobile would require a complete redesign of the parallax model. | Mobile comes after competitive intelligence features per the existing constraint. Rooms stay desktop-only in v1.4. |

---

## Feature Dependencies

```
Room system (AtmosphereRoot + Plane + GrowthOverlay)  [ALREADY BUILT]
    └──consumed by──> War Room
    └──consumed by──> Hall of Records / Library
    └──consumed by──> Throne / Home
    └──requires──> Per-room GrowthState server fn (one per room)

GrowthState server fn
    └──reads──> DB counts (drafts, game plans, opponents → War Room)
    └──reads──> DB counts (matches, post-game reviews, learnings → Library)
    └──reads──> DB counts (LP entries, rank, team members → Throne)
    └──outputs──> (library_tier, armory_tier, banner_tier) per room

QuickNav (Cmd/Ctrl+K)  [ALREADY BUILT on Solo Journey]
    └──must be──> global (shell-level, not per-room)
    └──requires──> shortcut field added to nav items for keyboard hint display

RoomSurface tab component  [NEW]
    └──required by──> War Room (draft | tree-drafter | game-plan | opponents | ban-priority)
    └──required by──> Library (stats | match detail | analytics | post-game | learnings)
    └──requires──> URL param routing (?tab=draft) for deep links
    └──requires──> keyboard navigation (arrow keys between tabs)

Room transition animation  [NEW, optional]
    └──requires──> CSS View Transitions API wrapper in WASM
    └──blocked by──> prefers-reduced-motion gate (must degrade gracefully)
    └──enhances──> all room-to-room navigations

Growth tier notification moment  [NEW]
    └──requires──> GrowthState fetched on mount + refetched after mutations
    └──requires──> signal comparison (prev_tier vs new_tier) in AtmosphereRoot or per-room
    └──enhances──> all growth axes

Discrete per-tier art  [FUTURE]
    └──requires──> fal.ai key + FLUX generation pipeline
    └──requires──> art direction for each axis x tier x region combination (54 images for 3 rooms × 3 axes × 3 tiers × 2 regions)
    └──replaces──> current opacity-modulation approach
    └──deferred to──> post-v1.4 (MVP uses opacity modulation)

Region-reactive ambient motion  [NEW]
    └──implemented as──> CSS keyframes on plane elements
    └──requires──> CSS custom property `--ambient-period` set per region
    └──gated by──> prefers-reduced-motion media query
```

---

## Growth Mechanic Recommendations

These recommendations are grounded in the Hades / Stardew / Cult of the Lamb / LoL event hub patterns
observed in research. They define *what counts* toward each growth axis and why.

### What Feels Rewarding vs. Gimmicky

**Rewarding (from Hades / Stardew patterns):**
- Growth reflects **real, meaningful work** — not "log in 5 days in a row" (a retention dark pattern)
  but "you made 10 game plans, your armory grew." The metric is the actual feature use.
- Tier thresholds are **non-trivial but reachable** — tier 1 at ~5 items, tier 2 at ~20, tier 3 at ~50
  means a casual user reaches tier 1 quickly (positive first impression) and tier 3 takes real investment
  (aspirational). Adjust per feature frequency.
- Tier transitions are **noticeable without being jarring** — 800ms opacity fade, ambient glow pulse,
  no modal or sound. The room quietly changes. This is the "room fills as you work" moment.
- Growth axes are **independent** — a player who does lots of drafts but few learnings sees a full
  banner/armory and a sparse library. This creates a truthful visual fingerprint, not a generic "level 7."

**Gimmicky (patterns to avoid):**
- Growth triggered by **time, not work** (daily login streaks, session timers). Slay the Spire's
  deliberate choice to have minimal meta-progression is instructive: if progression is not tied to
  meaningful in-game decisions, it feels hollow.
- **Random/surprise unlocks** — users open a new decoration they didn't earn. Creates initial delight,
  then "what does this mean?" confusion.
- **Tier regression** — the library shrinks if you delete learnings. The environment should only grow;
  regression is punishing and demotivating.
- **Narrative without payoff** — "You've unlocked the Ancient Tome!" with no functional or visual change.
  Every tier advance must produce a visible atmospheric difference.

### Proposed Growth Thresholds (Starting Point)

| Room | Axis | Tier 0 | Tier 1 | Tier 2 | Tier 3 |
|------|------|--------|--------|--------|--------|
| War Room | Banner (drafts + game plans) | 0 | 3 | 15 | 40 |
| War Room | Armory (champion pool entries) | 0 | 5 | 20 | 50 |
| War Room | Library (opponent profiles) | 0 | 2 | 8 | 20 |
| Library | Library (learnings + post-game reviews) | 0 | 5 | 20 | 50 |
| Library | Armory (tracked matches) | 0 | 10 | 50 | 200 |
| Library | Banner (analytics / goal completions) | 0 | 1 | 5 | 15 |
| Throne | Banner (LP entries / rank progression) | 0 | 3 | 15 | 40 |
| Throne | Library (team members + join requests) | 0 | 2 | 5 | 10 |
| Throne | Armory (action items completed) | 0 | 5 | 20 | 50 |

These are starting hypotheses; calibrate during closed-beta based on observed user data volume.
The `GrowthState::from_counts` function in `src/models/growth.rs` is where these live.

---

## Power-User Speed Implications

Power users visit the app 5-10x per day. Every feature must be evaluated against the question:
"Does this slow down the 20th visit?"

| Feature | Power-User Impact | Mitigation |
|---------|-------------------|------------|
| Parallax atmosphere rendering | Negligible — CSS transform/opacity only, GPU-composited. No JS on hot path. | None needed. |
| Growth state fetch on room mount | 1 extra DB round-trip per room visit. Tier rarely changes; can be cached. | Cache `GrowthState` in `localStorage` with a 5-minute TTL. Refetch after known mutation events only. |
| Room-to-room transition animation | If > 300ms, becomes noticeable friction on rapid navigation. | Hard cap at 250ms. Respect `prefers-reduced-motion` (instant transition). |
| RoomSurface tab switching | Must be instant (no server round-trip). Tab shows the cached surface UI; data inside may load async. | Tab switch = URL param change + Leptos route match. Surface components use Suspense with skeleton loaders — tab header is always visible instantly. |
| QuickNav activation | Must open in < 50ms. Filtering must update in < 16ms (1 frame). | Keep command list in-memory (no server round-trip to open). Filter is a client-side JS/WASM string match. |
| Ambient animation CPU usage | CSS keyframe animations are compositor-thread only. No JS on the main thread. | Use `will-change: opacity` on animated overlays. Test with DevTools CPU throttle at 4x. |
| Art asset load time | 400 KB budget per region. Cold visit = network fetch. | WebP compression + 400 KB cap (enforced). Assets served from CDN. Skeleton placeholder (gradient) while art loads. |

---

## Accessibility Implications

| Feature | Accessibility Concern | Requirement |
|---------|----------------------|-------------|
| Parallax depth on pointer move | Vestibular disorders; motion sickness | `@media (prefers-reduced-motion: reduce) { --depth: 0; }` sets all plane offsets to zero. No movement. |
| Ambient keyframe animation (torch flicker) | Same vestibular concern | Entire keyframe animation wrapped in `@media (prefers-reduced-motion: no-preference)`. Static at reduced-motion. |
| Room transition animation | Same | View transition degraded to instant at reduced-motion. |
| Growth overlay fade-in | Same | 800ms fade is within safe range (WCAG 2.3.3 allows motion if < 3 flashes/second and non-essential). Still gate via prefers-reduced-motion for belt-and-suspenders. |
| Parallax planes and overlays | Screen readers will read `<img>` alt text if present | All planes and overlays: `aria-hidden="true"`. Already applied in Solo Journey. |
| Contrast scrim | Background art may reduce contrast if scrim is removed | Never remove scrim. Vignette ensures ≥ 4.5:1 contrast ratio on surface text. |
| RoomSurface tabs | Keyboard navigation required | Tab bar must support `role="tablist"`, `role="tab"`, `aria-selected`, arrow-key navigation between tabs. Follows ARIA Authoring Practices Guide tabbed-interface pattern. |
| QuickNav modal | Focus trap required | On open: focus moves to search input. On close: focus returns to trigger element. ESC closes. Standard modal focus management. |
| Growth tier notification glow | Brief flash on tier advance | Ensure pulse animation is < 3Hz. Single 1.5s pulse = safe. |

---

## MVP Definition

### v1.4 Launch With (the rooms milestone)

- [ ] War Room: `AtmosphereRoot` + parallax + growth overlays + `RoomSurface` tabs housing draft, tree-drafter, game-plan, opponents, ban-priority
- [ ] Hall of Records / Library: same structure housing stats, match detail, analytics, post-game, learnings
- [ ] Throne / Home: same structure housing identity, team dashboard, solo progression
- [ ] IA healing: opponents page folded into War Room tab; learnings folded into Library tab; old standalone routes redirect
- [ ] GrowthState server fn per room (reading appropriate DB counts)
- [ ] QuickNav working globally from all rooms (shell-level component)
- [ ] `prefers-reduced-motion` compliance across all motion (planes, overlays, transitions)
- [ ] ARIA compliance for tab bars and QuickNav modal
- [ ] Growth tier notification: signal comparison + CSS class + ambient glow pulse on tier advance
- [ ] Diegetic copy vocabulary: per-room label fn mapping feature names to room language

### Add After Validation (v1.4 stretch / v1.5)

- [ ] Room-to-room view transition animation (CSS View Transitions API) — adds polish but not required for rooms to function
- [ ] Region-reactive ambient motion (torch flicker vs electric haze per region) — atmospheric enhancement
- [ ] GrowthState localStorage caching (5-min TTL) — performance optimization; plain fetch is acceptable at launch
- [ ] Keyboard shortcut hints in QuickNav — nice polish; QuickNav works without it

### Future Consideration (v2+)

- [ ] Discrete per-tier art (54 images across rooms × axes × tiers × regions) — requires fal.ai + art direction; transformative but high effort
- [ ] Optional ambient audio with opt-in toggle — requires browser autoplay policy handling; deferred
- [ ] Mobile-responsive room variant — requires fundamentally different parallax model

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| War Room (draft + game-plan + opponents as tabs) | HIGH | MEDIUM (RoomSurface component + existing pages) | P1 |
| Hall of Records / Library (stats + learnings + post-game as tabs) | HIGH | MEDIUM | P1 |
| Throne / Home room | HIGH | LOW (simpler content, fewer tabs) | P1 |
| GrowthState per room | HIGH | LOW (extend existing growth.rs model) | P1 |
| Global QuickNav (from all rooms) | HIGH | LOW (already built; promote to shell) | P1 |
| prefers-reduced-motion compliance | HIGH | LOW | P1 |
| ARIA compliance (tabs, modal) | HIGH | LOW | P1 |
| IA healing (redirect old routes into room tabs) | MEDIUM | LOW | P1 |
| Growth tier notification moment | MEDIUM | LOW | P2 |
| Diegetic copy vocabulary | MEDIUM | LOW | P2 |
| GrowthState localStorage cache | LOW | LOW | P2 |
| QuickNav keyboard shortcut hints | LOW | LOW | P2 |
| Room-to-room view transition animation | MEDIUM | MEDIUM | P2 |
| Region-reactive ambient motion | LOW | MEDIUM | P2 |
| Discrete per-tier art (fal.ai) | HIGH | HIGH | P3 |
| Optional ambient audio | LOW | HIGH | P3 |

**Priority key:**
- P1: Required for v1.4 milestone to deliver the "immersive rooms" promise
- P2: Polish; add when P1 is stable; adds meaningfully without being required
- P3: Transformative but deferred; requires significant new resources or infrastructure

---

## Sources

- [Spirit Blossom event hub UX — Surrender at 20 coverage](https://www.surrenderat20.net/2020/07/red-post-collection-spirit-blossom.html) — MEDIUM confidence
- [RiotX Arcane interactive hub — Riot Games announcement](https://www.riotgames.com/en/news/welcome-to-riotx-arcane) — HIGH confidence (official)
- [RiotX Arcane design — Thinkingbox case study](https://thinkingbox.com/work/riotgames-arcane) — MEDIUM confidence
- [Hades meta-progression design — DMS 462 analysis](https://dms462fall2020.wordpress.com/2020/12/06/meta-is-etymologically-greek-right-meta-progression-in-hades/) — MEDIUM confidence
- [Hades game design overview — polydin.com](https://polydin.com/hades-game-design/) — MEDIUM confidence
- [Ambient animations in web design — Smashing Magazine Part 1 (2025)](https://www.smashingmagazine.com/2025/09/ambient-animations-web-design-principles-implementation/) — HIGH confidence
- [Ambient animations in web design — Smashing Magazine Part 2 (2025)](https://www.smashingmagazine.com/2025/10/ambient-animations-web-design-practical-applications-part2/) — HIGH confidence
- [Command palette UX patterns — Bootcamp / Medium](https://medium.com/design-bootcamp/command-palette-ux-patterns-1-d6b6e68f30c1) — HIGH confidence
- [Command palette best practices — Superhuman blog](https://blog.superhuman.com/how-to-build-a-remarkable-command-palette/) — HIGH confidence
- [Diegetic interfaces — Native UI Substack](https://nativeui.substack.com/p/diegetic-interfaces) — HIGH confidence
- [Types of UI in gaming — Medium](https://medium.com/@lorenzoardeni/types-of-ui-in-gaming-diegetic-non-diegetic-spatial-and-meta-5024ce6362d0) — MEDIUM confidence
- [Diegetic and non-diegetic UI — Nasty Rodent](https://nastyrodent.com/diegetic-and-non-diegetic-ui/) — MEDIUM confidence
- [Designing reward loops — Medium](https://medium.com/@rakeshroyakula/designing-reward-loops-that-keep-players-hooked-without-manipulation-58447c858d4a) — MEDIUM confidence
- [Stardew Valley design analysis — Medium / The Startup](https://medium.com/swlh/deceptively-simple-design-cabde40af87f) — MEDIUM confidence
- [Parallax mouse effects — Wix Studio blog](https://www.wix.com/studio/blog/mouse-parallax) — MEDIUM confidence
- [prefers-reduced-motion — MDN Web Docs](https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-reduced-motion) — HIGH confidence (official)
- [Animation and motion accessibility — web.dev](https://web.dev/learn/accessibility/motion) — HIGH confidence (official)
- Solo Journey room implementation — `src/pages/rooms/solo_journey.rs` (first-party, HIGH confidence)
- AI Images log — `.planning/assets/AI-IMAGES.md` (first-party, HIGH confidence)

---

*Feature research for: v1.4 Immersive Companion (immersive explorable-hub rooms, environment-grows-with-progress)*
*Researched: 2026-06-03*
