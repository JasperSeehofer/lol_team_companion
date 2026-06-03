# Pitfalls Research

**Domain:** Adding immersive 2.5D parallax rooms + AI-art pipeline + gamified growth to an existing Leptos 0.8 SSR/hydrate League companion app, then shipping to closed beta on Hetzner CAX11
**Researched:** 2026-06-03
**Confidence:** HIGH (all WASM/Leptos/performance pitfalls verified against codebase or official docs; Riot IP section is MEDIUM — policy is ambiguous and Riot has not published an explicit AI-art policy as of this date)

---

## Critical Pitfalls

Mistakes that cause rewrites, legal exposure, or hard-to-recover regressions.

---

### Pitfall 1: The Riot IP / Fan-Content-Policy Question Is Unanswered and Cannot Be Deferred Past Soft Launch

**What goes wrong:**

This project generates AI art depicting "Demacia citadel", "Pandemonium harrowing", and "League of Legends splash art style" environments, serves them to users of an app that uses the Riot API, and is heading toward a closed beta that may become a paid product (subscriptions/donations are the permitted monetisation model under the Riot API Terms). Three policies intersect here, and they are in tension:

1. **Riot Fan Content Policy** (last updated August 2018 — the policy has not been revised to address AI-generated content). It permits non-commercial fan projects but explicitly prohibits apps on commercial storefronts without a licence. The policy says fan art must "include original content" (your own unique and original ideas) — whether AI-generated art using League universe prompts qualifies as "original" is legally untested.

2. **Riot API Terms** (current). Commercial use is permitted if a free tier exists and the product has a production key. The API Terms explicitly prohibit "creating alternative ranking systems like MMR calculators" and several other categories, but do not mention AI-generated visual assets.

3. **Riot's public stance on AI art** is openly hostile. In August 2025 Riot published an AI-generated anniversary trailer for Wild Rift, faced severe community and internal backlash, deleted the video, and issued a public apology. A tool that openly describes its art as "League of Legends splash art style" AI-generated art may attract the same backlash — and potentially a DMCA or C&D even if the policy technically permits it, because the policy was written before generative AI existed.

The specific risk is: the app uses explicit League universe visual identity (Demacia architecture, Pandemonium halos) in generated art and then sells/monetises access to the app via API Terms. This is the combination the policy was not written to address.

**Why it happens:**

AI-art pipelines have moved faster than platform IP policies. The FLUX prompts locked in `AI-IMAGES.md` were written before the Wild Rift AI controversy peaked. There is a temptation to defer the IP question because the art is "placeholder" — but once FLUX-generated assets replace the ImageMagick placeholders and the beta opens, the risk crystallises.

**How to avoid:**

Two mutually exclusive paths:

- **Path A (safe, slower):** Replace League universe imagery with original-world art. Invent a fictional faction (not Demacia/Pandemonium) with similar visual DNA (stone citadel + warm gold / brutalist neon). The parallax system, growth tiers, and everything else remain identical — only the FLUX prompt vocabulary changes. The reproducibility infrastructure in `AI-IMAGES.md` already supports this; update the prompts before running FLUX. This also sidesteps the community backlash risk entirely.

- **Path B (risks accepted, faster):** Keep League universe art, add mandatory disclaimer per Fan Content Policy, keep the product free (closed beta, no paid tier yet), document the accepted risk in PROJECT.md, and plan to redo the art before any public paid launch. If taking Path B, do not describe the art as "League of Legends splash art style" in any external communication — describe it as "Runeterra-inspired" which is closer to the fan-creation framing.

Do not build the production FLUX pipeline without first deciding which path to take. Running FLUX and committing the outputs is the point of no return for the IP question.

**Warning signs:**
- FLUX prompts reference specific Riot champion names or lore locations by name (the current prompts reference "Demacia" and "Pandemonium" which are Riot-owned faction names).
- The beta invitation page or any external description mentions "League of Legends art".
- A team member asks "do we need permission from Riot for this?" and the answer is deferred.

**Phase to address:** Art pipeline phase (immediately before the first FLUX generation run). This decision must be made and logged in PROJECT.md before any production art is generated. The roadmap phase that generates FLUX assets should have this as a blocking prerequisite.

---

### Pitfall 2: Hydration Panic on Growth-Tier Branching in New Rooms

**What goes wrong:**

The Solo Journey room currently avoids structural hydration branching correctly — all three growth overlays are always present in the DOM, with tier changes expressed via `data-*` attributes and pure CSS (the Phase 18.1/18.2 lesson). The risk reappears when the War Room, Hall of Records, and Throne rooms are built, because each room has different feature surfaces, different growth axes, and different conditional content. Developers building those rooms from scratch may not know the lesson and introduce `if tier >= 2 { view! { <img ...> } }` branches that produce structural DOM differences between SSR (which renders at the SSR-computed tier) and hydrate (which re-evaluates the signal after mount). The tachys "unwrap-on-None" panic documented in the milestone context is exactly this class of failure.

**Why it happens:**

The defensive pattern (`always-present DOM + CSS data attributes`) is documented in the parallax.rs comments and `GrowthOverlay`'s doc comment, but it is not in `leptos-patterns.md` or `CLAUDE.md`, so it is invisible to any agent that only reads those files before building a new room. New room components will be built by agents reading CLAUDE.md, not by reading the parallax.rs source.

**How to avoid:**

Add rule to `leptos-patterns.md` and `CLAUDE.md` (Critical Patterns): "Growth-tier branching in rooms MUST NOT use structural `if/match` inside `view!` that change whether an element exists. Use `data-*` attributes on a permanently-present container and drive visibility/opacity via CSS. See `parallax.rs::GrowthOverlay` and the Phase 18.1/18.2 post-mortem." Every new room component must have a `hydration-no-panic` e2e gate (the per-route gate pattern already established) before the room is considered shippable. The gate should navigate to the room, let WASM hydrate, then scroll/interact and confirm no panic in the console.

**Warning signs:**
- A new room file contains `if growth.get() >= 2 { view! { <img ... /> } }` at the top level of a view.
- `cargo check --features hydrate --target wasm32-unknown-unknown` passes but `cargo leptos watch` produces a "panicked at 'tried to access None value'" in the browser console on first load.
- The e2e smoke test for the new route passes at Tier 0 but fails if you seed Tier 2 data and reload.

**Phase to address:** Each new room's implementation phase. Codify the anti-pattern in rules files during the Solo Journey spike close-out phase so it is in place before War Room and Hall of Records are built.

---

### Pitfall 3: Mousemove Listener Accumulation Across SPA Route Changes

**What goes wrong:**

`AtmosphereRoot` registers a `mousemove` listener on `window` inside an `Effect` and removes it in `on_cleanup`. This is correct for a component that mounts and unmounts cleanly. However, Leptos 0.8's router does not guarantee that `on_cleanup` fires atomically with the route unmount — there is a known issue (leptos-rs/leptos#3665) where route cleanup does not fire until the second navigation under the same parent. If a user navigates: Solo Journey → War Room → Solo Journey, the first Solo Journey's `on_cleanup` may not fire until the War Room unmount, meaning two `mousemove` listeners are active simultaneously. The second listener also schedules a second RAF per frame, causing doubled RAF churn and CSS-var writes, which manifests as slightly jittery parallax or console noise but no hard error.

**Why it happens:**

The current implementation is correctly written but relies on a Leptos cleanup timing guarantee that has a known edge case. The risk is masked during development because you typically navigate once and reload.

**How to avoid:**

Add a belt-and-suspenders guard: before attaching the listener, check if `ATMOSPHERE_ROOT_ID` already has a non-zero `--px` CSS var set (indicates another instance is already active) and skip the second registration. Alternatively, use a `static AtomicBool` (via `std::sync::atomic` — WASM is single-threaded so this is sound) as a global "listener active" flag that `on_cleanup` clears. Write an e2e test that navigates Solo Journey → War Room → Solo Journey and asserts exactly one mousemove listener via `page.evaluate("window.__raf_count")` (expose a debug counter in debug builds only). The `on_cleanup` approach is correct as documented; the guard is insurance against the router timing edge case.

**Warning signs:**
- Browser devtools Performance tab shows two RAF callbacks firing per frame during parallax.
- `getEventListeners(window).mousemove.length` in the console returns 2 after navigating back to a room.
- Parallax motion is doubled in magnitude after revisiting a room without a full page reload.

**Phase to address:** During the second room implementation (War Room), when two rooms first coexist in the router tree. Add the guard and the nav-round-trip e2e test at that point.

---

### Pitfall 4: Too Many Composited Layers Blowing GPU Memory on Low-End or Mobile Machines

**What goes wrong:**

Each `Plane` and `GrowthOverlay` is a full-bleed `position: fixed` (or `absolute`) image with `transform: translate3d(...)`. Every such element gets promoted to its own compositor layer. For one room: 3 planes + 3 overlays + 1 scrim = 7 layers. For three rooms simultaneously preloaded (if the router keeps them in the DOM): 21 layers. At 1920×1080 and RGBA, one layer is ~8 MB GPU texture. 7 layers = ~56 MB GPU VRAM, 21 layers = ~168 MB. Shared Hetzner CAX11 (ARM, 4 GB RAM, no discrete GPU) does not have a GPU at all — compositing falls back to the CPU and the "GPU memory" is system RAM. On end-user machines, integrated graphics on budget laptops have 128–512 MB VRAM shared with RAM.

**Why it happens:**

The parallax implementation correctly uses `transform3d` for GPU compositing on real GPU hardware. On CPU-composited environments (server rendering a headless browser, ARM VPS running e2e tests) or low-end integrated graphics, the layer count becomes a RAM consumer rather than a GPU consumer.

**How to avoid:**

Three mitigations: (1) Only ever mount the active room — never keep inactive rooms in the DOM. The router already unmounts on route change; verify no `<Show>` pattern keeps rooms mounted while hidden. (2) Add `will-change: transform` only to planes that are actively animating (i.e., only when the mousemove listener is active). Use a CSS class toggled at mount rather than inline `will-change` on all planes statically. (3) Cap planes at 3 per room. Growth overlays should share compositor layers by using `opacity` transitions on `<img>` elements that are already promoted — do not add `will-change: transform` to overlays. Test with `chrome://tracing` or the Layers panel in DevTools and confirm total layer count per room stays under 10. On the CI Hetzner box, run e2e tests with `--disable-gpu` in Playwright's Chromium flags to catch CPU-path regressions.

**Warning signs:**
- DevTools Layers panel shows >10 layers for a room.
- Frame rate drops below 60 fps during mousemove on a mid-range MacBook (M-series are fine; test on Intel/AMD).
- Memory usage in DevTools Task Manager climbs by >100 MB when navigating between rooms.

**Phase to address:** During the War Room implementation when you first have two rooms in the router. Add a Layers panel check as part of the phase's browser verification step.

---

### Pitfall 5: `prefers-reduced-motion` Runtime Toggle Is Not Handled — Only Mount-Time Check

**What goes wrong:**

The `AtmosphereRoot` component checks `prefers-reduced-motion` once at mount inside the `Effect` and bails early if set. It does not add a `MediaQueryList.addEventListener('change', ...)` listener. A user who enables "Reduce Motion" in their OS while the app is open (common: enabling in System Preferences during a migraine, then forgetting to re-check the app) will not have the listener removed — the parallax continues moving until they reload the page. This violates WCAG 2.3.3 (Animation from Interactions — AAA) and the intent of SC 2.2.2 (Pause, Stop, Hide). Both are listed in the project's accessibility standards (design system pointers in CLAUDE.md). More practically: a closed-beta user with vestibular disorder who experiences motion sickness mid-session cannot stop the parallax without reloading.

**Why it happens:**

Mount-time media query checks are the obvious first implementation. Runtime change listening requires a second `Closure` and cleanup that is easy to overlook when the belt-and-suspenders CSS fallback already handles the static case.

**How to avoid:**

Inside the `#[cfg(feature = "hydrate")]` block in `AtmosphereRoot`, after attaching the mousemove listener, also register a `MediaQueryList` change listener. On `change`, if `matches` is now true, call `remove_event_listener` to remove the mousemove listener and reset `--px`/`--py` to 0 on the root element. If `matches` is now false, re-register the mousemove listener. Store the MQL handle in a second `StoredValue` for cleanup. Additionally, add a user-visible "Pause atmosphere" toggle button in the room UI (a small button, not a setting buried in a menu) that writes a localStorage key and is respected on mount. This gives the user an in-app override that doesn't require OS settings knowledge.

**Warning signs:**
- Running the app with System Preferences > Accessibility > Reduce Motion enabled, enabling it mid-session, and observing parallax still moving.
- No `MediaQueryList.addEventListener` call anywhere in the parallax source.
- Accessibility audit (axe-core or wave) flags motion that cannot be paused.

**Phase to address:** Solo Journey polish phase / accessibility pass. Must be resolved before closed-beta launch, not after.

---

### Pitfall 6: Contrast Failure Over Parallax Art — WCAG AA Text Contrast Is Unverifiable Against Dynamic Backgrounds

**What goes wrong:**

The surface layer (the actual UI — stats panels, draft board, form fields) floats at `z-index: 10` over parallax art. The current scrim (`radial-gradient(ellipse at center, transparent 0%, var(--color-base) 95%)`) fades the edges to base but leaves the center semi-transparent. Text over a semi-transparent scrim over a painting has unpredictable contrast — it depends on the specific art asset, the user's accent/theme, and the screen calibration. WCAG 1.4.3 requires 4.5:1 contrast for normal text. A cream-coloured UI panel over warm gold Demacia art may pass; the same panel over a bright sky band in the mid-plane may fail. Since the parallax moves, the contrast ratio changes dynamically — which means it is impossible to measure statically.

**Why it happens:**

Designers test contrast on a static screenshot of the room at rest (pointer centred). The parallax moves the bright sky band under the text block when the user moves the mouse to the upper-left.

**How to avoid:**

Two strategies: (1) The scrim must be opaque enough to guarantee contrast regardless of what art is behind it. The current `35% multiply` opacity is not sufficient — test at the worst-case art position (point the mouse to the corner so the brightest plane element moves under the text area) and verify contrast with a real asset (not a placeholder gradient). Increase scrim opacity until it passes at worst case. (2) Surface-layer panels (stat cards, form areas, code blocks) must use `bg-surface` (`--color-surface`) with no transparency — `bg-surface/50` (the semi-transparent pattern from CLAUDE.md) should NOT be used on panels over parallax art. Use `bg-surface` (fully opaque) for all panel backgrounds in rooms.

**Warning signs:**
- `bg-surface/50` or any `bg-*/opacity` classes appear on form inputs or text panels in a room component.
- A contrast checker run on a screenshot at rest passes, but running it with the mouse at upper-left fails.
- The scrim's gradient alpha is `<= 40%`.

**Phase to address:** Each room's design implementation phase. Add a contrast verification step (test at worst-case mouse position with real FLUX art, not placeholders) to the phase's verification checklist.

---

### Pitfall 7: Focus Management Breaks When the QuickNav Overlay Summons Over Art

**What goes wrong:**

The QuickNav overlay (`src/components/room/quick_nav.rs`) summons over the parallax room. When it opens, focus must be trapped inside the overlay and returned to the trigger element when it closes. If focus is not trapped: keyboard users tab into the parallax planes (which are `aria-hidden="true"` but are real DOM elements), screen readers announce "image" for every plane, and the keyboard navigation path from the nav trigger through the overlay to a destination is non-linear. This is a WCAG 2.1 failure (SC 2.1.2 No Keyboard Trap, SC 2.4.3 Focus Order).

**Why it happens:**

Leptos has no built-in focus-trap primitive. It is easy to `show!` an overlay and forget to implement focus trapping. During development, sighted mouse users never notice.

**How to avoid:**

Implement focus trapping in `quick_nav.rs` using the web-sys DOM API: on open, find all focusable descendants, set focus to the first, and intercept `Tab`/`Shift+Tab` to cycle within the trap. On close, restore focus to the trigger element (stored in a `StoredValue<Option<web_sys::Element>>`). Additionally, the overlay must set `aria-modal="true"` and use `role="dialog"` with an `aria-labelledby` pointing to the nav title. Verify with keyboard-only navigation: Tab from the trigger, confirm focus enters the overlay, Tab through all items, confirm Tab on the last item wraps back to the first, confirm Escape returns focus to the trigger.

**Warning signs:**
- `quick_nav.rs` has no `set_focus` or `tabindex` manipulation.
- Tab key during a keyboard test causes focus to disappear behind the overlay.
- Screen reader announces the parallax image elements when the overlay is open.

**Phase to address:** QuickNav implementation phase. Focus trapping is not a polish step — it must be part of the initial implementation.

---

### Pitfall 8: FOUC of the Atmosphere Before WASM Hydrates — Art Flashes In After Load

**What goes wrong:**

The SSR render sends the `AtmosphereRoot` div with `--px:0; --py:0` and the `<img>` planes in the HTML. However, the planes are `loading="lazy"`, and on first load the browser may not start fetching them until after the WASM bundle (which is the first render-blocking asset) finishes parsing. The sequence becomes: (1) HTML arrives with static painting — no planes visible yet (lazy); (2) WASM parses and hydrates; (3) planes enter viewport/viewport-adjacent and lazy-load fires; (4) planes fade in. This creates a visible "bare room then art appears" flash, especially on slower connections. For the growth overlays, the tiers are data-async: the `get_growth_state` resource resolves after hydration, causing a second flash where tier-0 state renders then jumps to the actual tier.

**Why it happens:**

`loading="lazy"` on fixed/absolute full-bleed images defers the fetch until after layout, but these images are always in the viewport — the lazy trigger fires immediately after layout, which is after WASM hydration on first load.

**How to avoid:**

For the primary sky plane (the largest, most visible layer): use `loading="eager"` or `fetchpriority="high"`. Only the mid and near planes (which are partially transparent and less visually prominent) benefit from lazy. Add `<link rel="preload" as="image" href="/img/rooms/solo-journey/{region}/plane-sky.webp">` in the page `<head>` via a Leptos `<Meta>` component — this starts the fetch in parallel with the WASM bundle. For growth overlays: accept the tier-0-then-real-tier flash as the cost of data-driven tiers, but minimise it by setting CSS `transition: opacity 0.6s` on the overlays so the transition is a smooth fade rather than a snap. Do not preload all 6 overlay assets — that would blow the 400 KB budget on assets that may not be needed.

**Warning signs:**
- Network throttled to "Fast 3G" in DevTools shows a visible frame where the room background is solid `bg-base` before planes appear.
- Lighthouse "Largest Contentful Paint" identifies the sky plane as the LCP element with a fetch delay > 500ms.
- Growth overlays snap between tiers rather than fading.

**Phase to address:** Solo Journey polish phase and each subsequent room implementation phase. The preload `<Meta>` tag should be part of every room's template.

---

### Pitfall 9: Layer-Separation Artifacts in AI-Generated Parallax Planes Cause Halo Fringing

**What goes wrong:**

The FLUX prompts for mid and near planes request "transparent background, foreground only". FLUX does not natively generate transparent images — the workflow is: generate opaque image → run background removal pass → export WebP with alpha. Background removal tools (rembg, SAM-based models, service APIs) produce hard or soft edges that visually halos the subject against the background. When the mid plane (stone pillars) is composited over the sky plane with a parallax offset, white or grey halo fringing around the pillars is visible as a 1–3 px artifact ring that moves with parallax. This is especially visible against dark backgrounds. The artifact is often invisible at rest but appears during motion.

**Why it happens:**

Background removal from non-green-screen source images is inherently lossy. FLUX generates images with natural edge feathering that the background remover cannot cleanly isolate. The current placeholder WebPs (ImageMagick gradients with `xc:none` rectangles) do not exhibit this — so the problem is invisible until real FLUX assets replace placeholders.

**How to avoid:**

Three techniques, all should be applied: (1) Use **layer-by-layer generation** — do not generate a full scene and remove the background. Instead, generate each plane with `black background, isolated element, no other scene elements` and composite in image editing software. The sky plane is opaque so no background removal needed. The mid plane should be generated with explicit `isolated on pure black background` and converted using a luminance-key instead of an edge-detection-based background remover — luminance-key is cleaner for architectural elements. (2) Apply a 1px `erode` mask shrink before export to eat the halo fringe. (3) Export at slightly higher resolution than 1920×1080 (e.g. 2400×1350) and scale down in CSS — downsampling blends the artifact into the surrounding pixels. Test by running the parallax with motion on a dark background and inspecting plane edges at 2x zoom.

**Warning signs:**
- Mid or near plane WebPs show a visible white or grey ring around foreground elements when opened in an image viewer against a dark background.
- During parallax motion, a 1–3 px light ring follows the pillar/architecture edges.
- The background removal tool used is an online web UI (low control over edge parameters).

**Phase to address:** Art generation phase. Every generated mid/near plane must pass a visual QA step: open in browser against `bg-base`, play parallax at max amplitude, confirm no halo ring visible at 1x zoom. This is a build-time check, not a runtime check.

---

### Pitfall 10: `format!()` in `Plane` and `GrowthOverlay` Allocates a New String Per Render

**What goes wrong:**

`Plane` constructs `style=format!("--depth:{depth}; z-index:{z};")` as a static `format!` call in the component body. Since `Plane` is called from the `SoloJourneyRoom` view body which re-runs on room mount, this allocation happens on every mount. That is acceptable. The risk comes if someone wraps plane rendering in a reactive closure: `{move || view! { <Plane src=dynamic_src.get() ... /> }}` — now the `format!` inside `Plane` runs on every signal update, which during mousemove (with RAF batching) means up to 60 times per second. Each call allocates a new `String` and a new DOM style string, causing garbage collection pressure in the WASM linear memory allocator.

**Why it happens:**

The component signature looks like it takes a static `depth: i32` and `z: i32` — developers wrapping it in a reactive closure do not realise the `format!` is inside.

**How to avoid:**

Make the style a `&'static str` or a `Cow<'static, str>` computed once. For the current implementation where `depth` and `z` are compile-time constants (they are passed as literals from `SoloJourneyRoom`), use a `const` or `static` style string per plane and pass it as `style=static_style`. Alternatively, annotate `Plane` with a doc comment: "Depth and z must be compile-time constants — do not wrap this component in a reactive closure." Add a lint note in `CLAUDE.md` for future room authors: parallax plane components are not reactive — depth/z are structural constants per room, not runtime signals.

**Warning signs:**
- A `Plane` or `GrowthOverlay` call appears inside `{move || ...}` in a room component.
- Performance profiling in the browser shows repeated DOM style mutations on `room-plane` elements during mousemove (beyond the intended `--px`/`--py` writes on the root).

**Phase to address:** War Room implementation phase when a second developer writes a room component. Add the doc comment now during Solo Journey cleanup.

---

## Moderate Pitfalls

---

### Pitfall 11: The Growth Gamification Layer Interrupts Task Flow for Power Users

**What goes wrong:**

Growth tiers are the "your time shows" feedback mechanism — a good fit for exploration and motivation. The risk: if the tier-up transition (overlay fade-in, visual change) fires while the user is mid-task (filling in a game plan, editing a draft pick), the animation draws the eye, interrupts focus, and may cause form submission errors or misclicks. A power user who has memorised the keyboard flow for draft entry does not want the room to visually change when they hit 50 champion pool entries mid-session.

**Why it happens:**

Tier updates are driven by a resource that refetches periodically or after mutations. If the resource refetches too eagerly, the overlay transition fires at unpredictable times during the session.

**How to avoid:**

Tier changes should only be evaluated at page load / room mount, not mid-session. The `get_growth_state` resource should not auto-refetch on every mutation — it should refetch once at mount and once when the user explicitly "returns to the room" (i.e. navigates back to it, not mid-flow). Specifically: do not call `growth_resource.refetch()` inside the mutation success handlers for champion pool saves, learning journal saves, etc. Let the tier update be visible on the *next* visit to the room after the milestone is crossed.

**Warning signs:**
- `growth_resource.refetch()` appears in the spawn_local success branch of any form-submit server function on a room surface.
- A user can observe the overlay opacity transition while actively typing in a form field.

**Phase to address:** Growth system integration phase (whichever phase wires up tier thresholds to real data).

---

### Pitfall 12: Room Scope Creep — Each Room Grows to Hold Its Own Navigation Logic

**What goes wrong:**

The War Room is supposed to contain draft + tree-drafter + game-plan + opponents + ban-priority. In practice, each of those surfaces has its own complex state, navigation, and server functions. Without a clear boundary, the War Room component file grows to 1,000+ lines trying to orchestrate all surfaces, and the existing page components (`draft.rs`, `game_plan.rs`) get forked into room-specific versions rather than reused. The result is duplicated server functions (drift in query logic) and a maintenance burden.

**How to avoid:**

The room is purely a shell: `AtmosphereRoot` + `Plane`/`GrowthOverlay` layout + `QuickNav` + a `<Outlet>` or explicit sub-route for the active surface. The existing page components (`DraftPage`, `GamePlanPage`, etc.) render unchanged inside the room's content `<div class="relative z-10">`. The room does NOT own the surface state — it only owns the atmosphere. Each room maps to a set of routes, not a set of embedded components.

**Warning signs:**
- A room file contains `#[server]` functions for surface-level operations (saving a draft, fetching game plan).
- The room file is >200 lines of view code.
- Existing page components are duplicated with a `_room` suffix.

**Phase to address:** Architectural decision in the War Room planning phase. Define the room-as-shell pattern explicitly before implementation begins.

---

### Pitfall 13: Stale SSR HTML Served During WASM Bundle + CSS Rebuild Window

**What goes wrong:**

`cargo leptos watch` rebuilds the SSR binary and the WASM/CSS bundle separately. The SSR binary restarts first (it is faster to compile). During the window between SSR restart and WASM/CSS bundle write, the SSR server serves HTML that references the new binary's asset hashes but serves the old WASM bundle and old CSS. A browser that fetches during this window gets a hydration mismatch (new HTML structure, old WASM client code) or gets no parallax styles (CSS not yet updated). This was noted in the milestone context. On the production Hetzner box, the same window exists during `cargo leptos build --release` deployments.

**How to avoid:**

For development: accept the ~5–10 second window — it resolves on the next reload. For production deployment: use a blue-green swap. Build the release binary and CSS bundle fully before stopping the old process. Use a simple approach: build to a `next/` directory, health-check it, then `mv next current && systemctl restart lol-companion`. Never restart the running binary until both the SSR binary and the `site/` directory (WASM + CSS) are fully written. Document this in the deployment runbook.

**Warning signs:**
- After `git push && deploy.sh`, users report broken styles or JS errors for ~30 seconds.
- The deploy script does `systemctl restart` before `cargo leptos build` completes.

**Phase to address:** Deployment / production hardening phase (Phase 20).

---

### Pitfall 14: Oversized Asset Budget — Individual FLUX Planes Exceed 400 KB Total-Per-Room Budget

**What goes wrong:**

The 400 KB total budget per room is documented in `AI-IMAGES.md` and inherited from UI-SPEC. The current ImageMagick placeholder files total ~52 KB for 12 files — well under budget. FLUX generates at higher quality and detail. A single `plane-sky.webp` from FLUX at 1920×1080 with WebP quality 80 can be 150–300 KB depending on scene complexity. Three planes + three overlays at those sizes = 900 KB–1.8 MB per room, 4–6x over budget. If three rooms are built (Solo Journey + War Room + Hall of Records), the total asset download for a user who visits all three rooms is 2.7–5.4 MB of room art on top of the WASM bundle (~1–2 MB compressed) and HTML.

**How to avoid:**

Set per-file budgets before running FLUX: sky plane ≤ 100 KB, mid plane ≤ 80 KB, near plane ≤ 60 KB, each overlay ≤ 50 KB. Total per room ≤ 400 KB. Enforce via a CI check: `find public/img/rooms -name "*.webp" -size +100k` should produce zero output. If FLUX output exceeds budget, re-run with lower inference steps (24 instead of 28), resize to 1600×900 (rooms are full-bleed but CSS `object-cover` handles the scale), and compress to WebP quality 65. The parallax motion means users rarely scrutinise static detail — lower quality is visually acceptable for background art.

**Warning signs:**
- `ls -lh public/img/rooms/` shows files > 100 KB.
- Lighthouse reports "Avoid enormous network payloads" on a room page.
- Total room assets for one region exceed 400 KB.

**Phase to address:** Art generation phase (same as Pitfall 9). Budget check must be a gate before any FLUX asset is committed.

---

### Pitfall 15: Launch Work Squeezed by Immersive UI Scope Creep

**What goes wrong:**

The v1.4 milestone explicitly re-incorporates the deferred v1.3 launch work: production hardening, closed-beta gate, deploy infra, compliance/transparency, pre-launch review, soft launch. These are not optional — without them, "closed beta" means an unguarded URL on an unmonitored server with no rate limiting. The immersive rooms work is visually rewarding; the hardening work is not. The risk is that the rooms scope expands (three rooms with full FLUX art, plus the QuickNav, plus IA healing for opponents/learnings) and the launch-infra phases get pushed to "after the beta opens" — at which point they are under time pressure with beta users already on the server.

**How to avoid:**

Sequence the roadmap so the minimum viable room (Solo Journey spike, already done) unlocks the closed-beta gate and deploy infra phases next, before War Room and Hall of Records. The beta opens with one immersive room and locked-down infra. Additional rooms ship as beta updates. This also reduces the FLUX art budget risk — you only need one region's art set for launch.

**Warning signs:**
- The roadmap shows War Room and Hall of Records before Phase 20 (production hardening).
- No beta invite gate exists in the running server.
- There is no rollback procedure documented for the Hetzner deployment.

**Phase to address:** Roadmap sequencing decision. Production hardening (Phase 20) must come before any additional rooms.

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Using League universe faction names in FLUX prompts ("Demacia", "Pandemonium") | Art is immediately recognisable to target users | IP risk crystallises when product monetises; may require full art redo | Only if Path B (accepted IP risk) is explicitly logged in PROJECT.md |
| `loading="lazy"` on the sky plane | No extra preload config | Visible FOUC on first room visit; Lighthouse LCP regression | Never for the sky plane — use eager or preload |
| `bg-surface/50` on form inputs over parallax art | Visually "blends" into atmosphere | Contrast failure at worst-case art position | Never in rooms — use `bg-surface` (fully opaque) |
| Refetching growth state on every mutation success | Tiers update immediately | Animation fires mid-task; disrupts power user flow | Never — fetch only at room mount |
| Building each room as a monolith component | Simpler single file | Server functions duplicated; 1000+ line files; query drift | Never — rooms are shells over existing page components |
| Running FLUX at `num_inference_steps: 28` without budget check | Higher quality output | Files routinely exceed 100 KB; total budget blown | Acceptable if immediately followed by compression + budget CI check |

---

## Integration Gotchas

Common mistakes when connecting to external services.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| fal.ai FLUX API | Generate mid/near planes as full scene then remove background | Generate each plane isolated (`isolated on black`, `no other elements`) — luminance-key extraction is cleaner than edge-based background removal |
| fal.ai FLUX API | Re-use the same seed across rooms and regions | Use documented per-asset seeds (see `AI-IMAGES.md` schema); regenerating with wrong seed breaks reproducibility contract |
| fal.ai FLUX API | Call the API inside a Rust build script or server startup | Call offline (human in the loop) and commit the WebP outputs; never call generation APIs from server runtime |
| Riot API (existing) | None — existing integration is solid | No new Riot API surfaces needed for rooms; do not add API calls for atmosphere/growth features |

---

## Performance Traps

Patterns that work at small scale but fail as usage grows.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| All 6 room planes preloaded for all rooms | Preload covers 3 rooms = 1.2 MB of blocking prefetch | Only preload sky plane for the active room; lazy-load everything else | Any second room built |
| `will-change: transform` on all planes statically via CSS | 7+ compositor layers always promoted even without active mousemove | Apply `will-change` only while mousemove listener is active (CSS class toggled at mount) | Immediately on mobile / integrated GPU machines |
| Tier threshold computed in the view (client-side signal math) | Works in dev; passes tests | Computed on every render; for complex tier formulas this re-runs on unrelated signal changes | Any tier formula more complex than a simple comparison |
| Room art not served with far-future cache headers | Every navigation re-fetches 400 KB of art | Set `Cache-Control: public, max-age=31536000, immutable` for all `/img/rooms/` assets; use content-hash filenames for cache-busting on updates | First beta user after an art update |

---

## Security Mistakes

Domain-specific security issues.

| Mistake | Risk | Prevention |
|---------|------|------------|
| Serving room art from a path controlled by user input (e.g. `region` prop derived from untrusted data) | Path traversal — serve arbitrary files from `public/` | The `region` value used to construct image paths must come from the validated `InitialTheme` context (server-set), not from a URL query param or user-supplied string |
| Growth tier exposed as a publicly readable endpoint without auth | Leaks user activity data to unauthenticated callers | `get_growth_state` server fn must enforce `get_current_user()` check before returning tier data (confirm this is already enforced in `solo_dashboard.rs`) |
| FLUX-generated art committed to the repo without licence metadata | Ambiguous IP status at audit time | Each asset committed to `public/img/rooms/` must have a corresponding entry in `AI-IMAGES.md` with prompt, seed, and generation date; add a CI check that counts entries |

---

## UX Pitfalls

Common user experience mistakes in this domain.

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| No way to disable atmosphere in-app | Users who find parallax distracting or cognitively heavy must dig into OS accessibility settings | Add an "Atmosphere" toggle in the user preferences or as a persistent button in the room (stores preference in DB, not localStorage, so it persists across devices) |
| Growth overlays animate at the same moment the user is focused on a form | Distraction causes misclicks or lost form state | Evaluate tier only at room mount; never refetch mid-session |
| QuickNav is the only way to navigate within a room | Keyboard users cannot navigate without the overlay (if QuickNav requires mouse to open) | QuickNav must have a keyboard shortcut (e.g. `?` or `/`) that works from any focused element within the room; document it in a visible affordance |
| "Rooms" framing confuses returning users who know where features are | Users who memorised `/draft` now navigate through rooms to find it | Keep existing direct URLs (`/draft`, `/stats`) active and redirecting into the correct room's active surface — do not break bookmark-able deep links |
| Power user skips room entirely and navigates directly to `/draft` | Bypasses all atmosphere features — which is fine, but growth tiers may not update | Growth tier computation must work from direct-URL access, not only from room mount |

---

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **Riot IP decision:** Has Path A or Path B been explicitly logged in PROJECT.md before FLUX is run? Verify by checking PROJECT.md Key Decisions for an "AI art IP" entry.
- [ ] **Hydration-no-panic gate:** Does every room route have an e2e test that seeds Tier 2+ data, navigates to the room, hydrates, interacts, and confirms no panic in the console? Verify by running `npx playwright test` with Tier 2 seed data.
- [ ] **Mousemove listener count:** After navigating Solo Journey → War Room → Solo Journey, does `getEventListeners(window).mousemove.length` return 1 (not 2)? Verify in DevTools.
- [ ] **prefers-reduced-motion runtime toggle:** Enable Reduce Motion in OS while the app is running and confirm parallax stops without a page reload. Verify by toggling OS setting mid-session.
- [ ] **Contrast over art:** At worst-case mouse position (corner, brightest art band under text), does text on surface panels pass WCAG 4.5:1? Verify with a real FLUX asset (not the ImageMagick placeholder).
- [ ] **Focus trap in QuickNav:** Tab from the room page into the QuickNav overlay, then Tab through all items — does focus cycle within the overlay? Press Escape — does focus return to the trigger? Verify with keyboard only, no mouse.
- [ ] **Asset budget:** Does `find public/img/rooms -name "*.webp" -size +100k` return zero files? Verify after every FLUX generation run.
- [ ] **Layer separation QA:** Does each mid/near plane WebP show no visible halo fringe when opened in a browser against `background: #070912` and played with parallax at full amplitude?
- [ ] **Growth timing:** Does the growth overlay animate while a user is mid-form in the draft input? Verify by starting a form, triggering a tier threshold crossing server-side, and confirming no animation fires.
- [ ] **Deep link preservation:** Does `/draft` still navigate to the draft surface after rooms are implemented? Verify that the existing route still works and returns 200.
- [ ] **Production deploy window:** Is there a documented blue-green procedure for the Hetzner deployment that prevents the stale-HTML window? Verify by reading the deploy runbook before the first beta deploy.
- [ ] **Beta gate:** Is there a closed-beta invite check before production hardening is merged? Verify that `/closed-beta` exists and unauthenticated users cannot access any room route.

---

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| FLUX art uses Riot IP and a C&D arrives | HIGH | Switch to Path A: re-run FLUX with original-faction prompts (the pipeline is intact, only prompts change); swap WebP files in `public/img/rooms/`; update `AI-IMAGES.md`; one deploy cycle |
| Hydration panic discovered in a new room post-launch | MEDIUM | Apply the static-markup + `data-*` pattern; SSR renders the static fallback immediately; WASM hydration panic is non-blocking if the CSS fallback renders the correct tier visually |
| Mousemove listener stacking causing doubled RAF | LOW | Add the global-flag guard in `AtmosphereRoot` and redeploy; no data migration needed |
| FLUX assets exceed 400 KB budget already committed | MEDIUM | Recompress with `magick` at quality 55–65 + resize to 1600×900; update `AI-IMAGES.md` sizes; verify visual acceptability; redeploy |
| Layer separation halo visible in production with real assets | MEDIUM | Apply erode mask + luminance-key re-export offline; swap WebP; one deploy cycle |
| prefers-reduced-motion runtime toggle missing | LOW | Add MQL change listener to `AtmosphereRoot` + redeploy; no user data affected |
| QuickNav focus trap missing discovered in accessibility audit | MEDIUM | Implement focus trap in `quick_nav.rs`; re-test; redeploy; update test suite |

---

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Riot IP / fan-content-policy (Pitfall 1) | Art pipeline planning phase — before any FLUX run | PROJECT.md Key Decisions has "AI art IP path" entry with Path A or Path B explicitly chosen |
| Hydration panic on tier branching (Pitfall 2) | Solo Journey polish / rules codification phase | `leptos-patterns.md` updated; every room has hydration-no-panic e2e gate at Tier 2 seed |
| Mousemove listener accumulation (Pitfall 3) | War Room implementation phase (first time two rooms coexist) | DevTools `getEventListeners(window).mousemove.length === 1` after nav round-trip |
| GPU layer overload (Pitfall 4) | War Room implementation phase | DevTools Layers panel shows ≤ 10 layers per room; no `will-change` on overlays statically |
| prefers-reduced-motion runtime toggle (Pitfall 5) | Solo Journey polish / accessibility pass | MQL change listener present; mid-session OS toggle stops parallax without reload |
| Contrast failure over art (Pitfall 6) | Each room's design phase (requires real FLUX asset, not placeholder) | Contrast checker at worst-case mouse position passes WCAG 4.5:1 |
| Focus trap in QuickNav (Pitfall 7) | QuickNav implementation phase | Keyboard-only nav test passes full Tab-cycle and Escape-returns-focus |
| Atmosphere FOUC (Pitfall 8) | Solo Journey polish phase | Sky plane has `loading="eager"` or preload `<link>`; Lighthouse LCP ≤ 2.5s on Fast 3G |
| AI art halo artifacts (Pitfall 9) | Art generation phase | Visual QA checklist item before committing each WebP |
| Reactive closure around Plane (Pitfall 10) | War Room implementation phase | Code review: no `{move || view! { <Plane ...> }}` in any room file |
| Growth animation mid-task (Pitfall 11) | Growth system integration phase | Confirm no `growth_resource.refetch()` in form mutation success handlers |
| Room scope creep (Pitfall 12) | War Room planning phase (architectural decision) | War Room file ≤ 200 lines; no `#[server]` functions in room files |
| Stale SSR during deploy window (Pitfall 13) | Production hardening phase (Phase 20) | Deploy runbook includes blue-green procedure; verified with test deploy |
| Asset budget overrun (Pitfall 14) | Art generation phase | `find public/img/rooms -name "*.webp" -size +100k` returns 0; CI check added |
| Launch work squeezed by rooms scope (Pitfall 15) | Roadmap sequencing — Phase 20 before War Room phase | Phase 20 (production hardening) is ordered before any second room implementation |

---

## Sources

- Codebase: `src/components/room/parallax.rs` — current RAF pattern, cleanup implementation, reduced-motion mount-time check
- Codebase: `src/pages/rooms/solo_journey.rs` — room shell pattern, asset path construction from region
- Codebase: `.planning/assets/AI-IMAGES.md` — FLUX prompts, asset budget (400 KB), placeholder pipeline
- Codebase: `.planning/PROJECT.md` — milestone context, constraints, v1.3 deferred work now in v1.4
- Codebase: `.claude/rules/leptos-patterns.md` — rules 19 (into_any for divergent branches), 22 (StoredValue), 23 (refetch), 54 (eager capture in Effects)
- Codebase: `.claude/rules/wasm-patterns.md` — rules 35 (no unwrap), 42 (cancellable timer), 43 (allow unused_variables)
- Riot Fan Content Policy: [riotgames.com/en/legal](https://www.riotgames.com/en/legal) — MEDIUM confidence; policy last updated August 2018; does not address AI-generated art
- Riot API Terms: [developer.riotgames.com/policies/general](https://developer.riotgames.com/policies/general) — commercial use permitted with free tier; AI art not addressed
- Riot AI art controversy: [pcgamesn.com/league-of-legends/wild-rift-trailer-backlash](https://www.pcgamesn.com/league-of-legends/wild-rift-trailer-backlash) — August 2025 Wild Rift AI trailer backlash and apology
- WCAG 2.3.3 Animation from Interactions: [w3.org/WAI/WCAG21/Understanding/animation-from-interactions.html](https://www.w3.org/WAI/WCAG21/Understanding/animation-from-interactions.html) — prefers-reduced-motion applies to runtime changes, not only mount-time
- Leptos router cleanup timing: [github.com/leptos-rs/leptos/issues/3665](https://github.com/leptos-rs/leptos/issues/3665) — route cleanup does not always fire on first navigation under same parent
- Chrome GPU compositing: [developer.chrome.com/blog/performant-parallaxing](https://developer.chrome.com/blog/performant-parallaxing) — transform-only parallax keeps work on compositor thread; `will-change` promotes to layer
- AI background removal halo artifacts: [alibaba.com/product-insights/why-is-my-ai-art-app-adding-halos](https://www.alibaba.com/product-insights/why-is-my-ai-art-app-adding-halos-to-every-subject-i-upload.html) — edge-detection background removal produces halo fringe
- Hetzner CAX11 specs: [sparecores.com/server/hcloud/cax11](https://sparecores.com/server/hcloud/cax11) — 2 vCPU ARM Ampere, 4 GB RAM, no discrete GPU; compositing is CPU-side

---

*Pitfalls research for: v1.4 Immersive Companion — parallax rooms + AI-art pipeline + gamified growth*
*Researched: 2026-06-03*
