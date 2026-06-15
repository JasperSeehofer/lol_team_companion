# Phase 30: Art Production - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-15
**Phase:** 30-art-production
**Areas discussed:** IP path & faction identity, Art direction & mood, Overlay growth granularity, EU runtime strictness, Solo Journey scope

---

## IP path & faction identity

### IP path (initial)

| Option | Description | Selected |
|--------|-------------|----------|
| Path A — original world | Invent fictional faction, no Riot names/style in committed art | ✓ (later reaffirmed) |
| Path B — accepted-risk LoL names | Keep Demacia/Pandemonium + League style, accept IP/community risk | |

**User's choice:** Path A — but then paused to re-evaluate Path B against Riot's actual terms.

**Notes:** User second-guessed and asked for a first-party read of
https://developer.riotgames.com/terms, https://www.riotgames.com/en/legal, and
the Developer General Policies before committing. Research finding: monetization
IS permitted for registered API products (free tier + "transformative" content);
faction NAMES are low-risk under the fan-content policy; the genuine risk is
**art that mimics Riot's copyrighted style** (copyright-derivative + "cannot
closely resemble in style" clause + revocable API access + AI backlash). Reframed
the decision as two axes: art-style (Path A safe) vs names (latitude).

### Region identifiers under Path A

| Option | Description | Selected |
|--------|-------------|----------|
| Keep as internal IDs | demacia/pandemonium stay as code identifiers only | ✓ |
| Rename to fictional slugs | Cross-cutting rename of paths/tokens/InitialTheme | |
| Keep IDs, rename visible strings | Audit user-facing labels only | |

**User's choice:** Keep as internal IDs.
**Notes:** Preserves the no-rename premise; limits blast radius to overlay system.

### IP decision (re-confirmed after legal read)

| Option | Description | Selected |
|--------|-------------|----------|
| Path A art + invent names | Original-world art AND fictional faction names | ✓ |
| Path A art + keep LoL names as flavor text | Original art, retain Demacia/Pandemonium labels | |
| Full Path B — names + Riot-style art | Genuine risk path; not recommended for monetized product | |

**User's choice:** Path A art + invent names.
**Notes:** Maximum safety; scrubs League names from committed assets entirely.

### Faction definition method

| Option | Description | Selected |
|--------|-------------|----------|
| I'll define them | User supplies names + DNA | |
| I draft proposals, you approve later | Claude drafts 2 factions into AI-IMAGES.md for approval | ✓ |
| Generic descriptors, no proper nouns | No named factions | |

**User's choice:** Claude drafts proposals.
**Notes:** Drafts — "The Aurelian Reach" (order/warm-gold) and "The Sundered Hollow"
(chaos/neon) — pending sign-off at planning.

---

## Art direction & mood

### Style fidelity

| Option | Description | Selected |
|--------|-------------|----------|
| Painterly Arcane-adjacent | Hand-painted, textured, moody; generic technique vocab | ✓ |
| Cleaner stylized illustration | Flatter/graphic; cheaper, less depth | |
| Semi-realistic atmospheric | Concept-art realism; budget/artifact risk | |

**User's choice:** Painterly Arcane-adjacent.

### Per-room moods

| Option | Description | Selected |
|--------|-------------|----------|
| Lock these three | War Room (tactical), Library (scholarly), Throne (regal) | ✓ |
| Lock, but tweak details | Adjust specifics | |
| You decide during planning | Hand detailing to Claude | |

**User's choice:** Lock these three.

### Region divergence

| Option | Description | Selected |
|--------|-------------|----------|
| Restyled, same composition | Same skeleton, different materials/light per region | ✓ |
| Color/light swap only | Same painting recolored | |
| Fully divergent per region | Different architecture/composition; alignment risk | |

**User's choice:** Restyled, same composition.
**Notes:** Must avoid the Phase 18 "just a color swap" critique; keep parallax zones aligned.

---

## Overlay growth granularity

### Granularity

| Option | Description | Selected |
|--------|-------------|----------|
| One overlay per axis, opacity-driven | Matches current room code; no code change | |
| Discrete per-tier art | overlay-{axis}-{1,2,3}.webp; richer; requires code change | ✓ |
| Defer per-tier, ship single now | Single now, per-tier later phase | |

**User's choice:** Discrete per-tier art.

### Scope handling for per-tier (code + budget)

| Option | Description | Selected |
|--------|-------------|----------|
| Absorb both into Phase 30 | Code change + budget rework inside Phase 30 | ✓ |
| Split: code phase first, then art | Separate code phase before art run | |
| Per-tier only where it reads best | Mixed model per axis | |

**User's choice:** Absorb both into Phase 30.
**Notes:** Phase 30 becomes art + the wiring/CI to consume it; supersedes the
roadmap's "no code change" premise. Budget recompute needed (3 planes + 9 overlays
≤ 400 KB/room/region); CI extended to aggregate sum check.

---

## EU runtime strictness

| Option | Description | Selected |
|--------|-------------|----------|
| BFL EU gen + fal.ai BiRefNet w/ justification | EU generation, US separation as documented exception | ✓ |
| Strict EU-only | Local separation; adds Python dependency | |
| fal.ai for both | One-vendor US pipeline; weaker on charter | |

**User's choice:** BFL EU for generation, fal.ai BiRefNet with documented exception.
**Notes:** Exception justified — parallax plane images contain no personal data.

---

## Solo Journey scope

| Option | Description | Selected |
|--------|-------------|----------|
| Include Solo Journey | Regenerate art + migrate overlays to per-tier | ✓ |
| Exclude — leave as-is | Keep placeholders + single-overlay model | |
| Include art, defer per-tier migration | Real art, keep single-overlay model | |

**User's choice:** Include Solo Journey (full per-tier migration).
**Notes:** All four rooms consistent; asset count 36 → ~96.

---

## Claude's Discretion

- Exact FLUX prompt wording per room × region × plane (into AI-IMAGES.md).
- Final per-file budget caps + CI aggregate-check mechanism.
- Seeds, inference steps, per-asset compute-path recording.
- Per-tier overlay stacking/toggling markup (subject to ROOM-03).

## Deferred Ideas

None — the two scope expansions (per-tier overlays, Solo Journey) were pulled into
Phase 30 by user decision rather than deferred.
