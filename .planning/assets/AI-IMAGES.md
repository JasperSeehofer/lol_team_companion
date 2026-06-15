# AI Image Reproducibility Log

> Phase 30 art production — immersive rooms parallax environment art.
>
> Path A (original world): all prompts use Aurelian Reach / Sundered Hollow faction
> vocabulary. No Riot proper nouns or "in the style of [product]" wording anywhere.
>
> Phase 17 plan 06 closed-beta background entries are preserved below for historical
> reference but their prompts carry legacy vocabulary — **do not regenerate via Phase 30
> scripts without first updating those prompts to Path A vocabulary**.

---

## Path A Legal Basis (D-01 / D-02)

This file records prompts for **original-world** art that establishes the app's own
visual identity. No game-IP vocabulary, no "in the style of [Riot product]" phrasing.

Decision logged in `.planning/PROJECT.md` Key Decisions (entry dated 2026-06-15).
Faction names approved by user (continuation signal "approved", 2026-06-15):

- **Warm region** (internal code ID: see `input.css` theme block) → visible/prompt name **The Aurelian Reach**
  (sun-bleached limestone ramparts, gilded heraldry, hanging banners, vaulted halls
  pierced by warm dawn light; noble, ordered, ascendant)
- **Chaos region** (internal code ID: see `input.css` theme block) → visible/prompt name **The Sundered Hollow**
  (fractured black-glass monoliths, magenta-and-cyan arc-light bleeding through
  cracks, brutalist obsidian ruin in electric storm-haze; volatile, void-touched)

Internal directory IDs, CSS token names, data-accent values, and e2e selectors
stay **unchanged** (D-04). Only FLUX prompts and visible labels use the faction names.
See `input.css` `@theme` block for the canonical internal region identifiers.

---

## Generation Pipeline (D-12)

```
scripts/gen-room-art.sh   → BFL EU FLUX.2 [pro]  (primary; EU-resident)
scripts/remove-bg.sh      → fal.ai BiRefNet v2    (mid/near/overlay planes only; EU-exception documented below)
scripts/encode-webp.sh    → ImageMagick → WebP
scripts/verify-budget.sh  → per-room/region aggregate ≤ 400 KB
```

**BFL EU FLUX.2 [pro] endpoint:** `https://api.eu.bfl.ai/v1/flux-2-pro`
- Header: `x-key: $BFL_KEY`
- Body: `{ "prompt": "...", "width": 1920, "height": 1080, "steps": 28, "seed": N }`
- Poll `polling_url` until `status == "Ready"`, then download `result.sample`

**fal.ai BiRefNet v2:** `fal-ai/birefnet/v2`
- Header: `Authorization: Key $FAL_KEY`
- Body: `{ "image_url": "<public-url>", "model": "heavy" }`
- Returns transparent PNG; apply to mid/near planes and overlays only

**ImageMagick WebP encoding:**
```bash
magick in.png -quality 80 -define webp:alpha-quality=100 out.webp
```

---

## fal.ai EU-policy exception

| Field | Value |
|-------|-------|
| **Service** | fal.ai — BiRefNet v2 (`fal-ai/birefnet/v2`) |
| **Provider domicile** | US-domiciled |
| **Data category** | Decorative parallax plane art — no personal data |
| **GDPR exposure** | Zero: the images processed through fal.ai contain no personal information, biometric data, or user-identifiable content. They are AI-generated architectural/environmental art. |
| **Justification** | GDPR Article 2 material scope requires processing of personal data; processing of non-personal decorative art falls outside scope. US-residency carries no GDPR risk for this data category. |
| **Logged by** | Phase 30 Plan 01 executor, 2026-06-15 |
| **Exception type** | One-time offline pipeline exception (not a server-runtime data flow) |

---

## Asset Budget (D-11)

Per-room/region total ≤ 400 KB. Per-file caps:

| File type | Count/room/region | Cap |
|-----------|-------------------|-----|
| `plane-sky.webp` | 1 | 120 KB |
| `plane-mid.webp` | 1 | 100 KB |
| `plane-near.webp` | 1 | 80 KB |
| `overlay-{axis}-{1,2,3}.webp` | 9 | 11 KB each |

**Total: 120 + 100 + 80 + (9 × 11) = 399 KB** — just under budget.

If FLUX planes run large, recompress to q65-70 or resize to 1600×900 before committing.

---

## Naming Convention (D-09)

Assets live at `public/img/rooms/<room>/<region>/` where:
- `<room>` ∈ `solo-journey`, `war-room`, `library`, `throne`
- `<region>` ∈ warm-region dir name (Aurelian Reach), chaos-region dir name (Sundered Hollow) — see `input.css` for exact identifiers

| File | Role | z-index | `--depth` | Alpha |
|------|------|---------|-----------|-------|
| `plane-sky.webp`              | furthest parallax plane | -40 | 6  | opaque |
| `plane-mid.webp`              | mid parallax plane      | -30 | 16 | transparent |
| `plane-near.webp`             | near/foreground plane   | -20 | 32 | transparent |
| `overlay-{axis}-1.webp`       | growth axis, tier 1 art | -18 | — | transparent |
| `overlay-{axis}-2.webp`       | growth axis, tier 2 art | -17 | — | transparent |
| `overlay-{axis}-3.webp`       | growth axis, tier 3 art | -16 | — | transparent |

Growth overlay axes per room (concrete filenames for solo-journey shown as examples):
- **solo-journey:** `banner`, `lib`, `armory` → `overlay-banner-1.webp`, `overlay-banner-2.webp`, `overlay-banner-3.webp`, `overlay-lib-1.webp`, `overlay-lib-2.webp`, `overlay-lib-3.webp`, `overlay-armory-1.webp`, `overlay-armory-2.webp`, `overlay-armory-3.webp`
- **war-room:** `tactics`, `intel`, `command`
- **library:** `records`, `tomes`, `analysis`
- **throne:** `roster`, `progression`, `crown`

Tier art reads as "place accumulates" (sparse → full):
- Tier 1 (`-1.webp`): sparse/beginning (banner half-hung, a few books, sparse weapons)
- Tier 2 (`-2.webp`): growing (more elements, partial fill)
- Tier 3 (`-3.webp`): full/flourishing (complete, richly detailed)

**Old single-overlay files** (`overlay-{axis}.webp`) are deprecated by D-09.
Remove them in the same commit that introduces per-tier overlays (D-10).

---

## Provenance Entry Template

```markdown
### `public/img/rooms/<room>/<region>/plane-sky.webp`

- **Generated:** YYYY-MM-DD
- **Surface:** `/<route>` atmosphere — sky depth plane, [Aurelian Reach / Sundered Hollow]
- **Compute path:** BFL EU FLUX.2 [pro] → ImageMagick → WebP (BiRefNet: skip, sky is opaque)
- **Runtime:** `https://api.eu.bfl.ai/v1/flux-2-pro`
- **Prompt:** "..."
- **Negative prompt:** "..."
- **Seed:** TBD (record integer assigned by BFL after first run)
- **Steps:** 28
- **Aspect / resolution:** 16:9 / 1920×1080
- **Final size:** TBD KB WebP q80
- **BiRefNet applied:** No (sky plane is opaque)
- **Reproducibility command:**
  ```bash
  # scripts/gen-room-art.sh <room> <region> plane-sky <seed>
  ```
```

For mid/near planes: `**BiRefNet applied:** Yes`
For overlays: add `**Tier:** 1 / 2 / 3` and `**BiRefNet applied:** Yes`

---

## Room Prompts — Path A (locked for generation)

### Prompt Architecture Rules (D-06)

All prompts must:
1. Use Aurelian Reach or Sundered Hollow vocabulary — no prior region names, no prior faction names from other media
2. Specify exactly one depth band per plane
3. Use generic painterly technique vocabulary: `oil painting with visible brushwork`,
   `hand-painted`, `textured`, `moody atmospheric` — never `in the style of [proper noun]`
4. For overlays: `isolated growth element on transparent background`

**Composition skeleton is identical across regions (D-08):** sky/mid/near zones align so
surface panels land correctly. Materials, architecture, and light genuinely differ — not a
flat colour-swap.

---

### Room: solo-journey

**Route:** `/journey` | **D-07 mood:** training hall — personal growth chamber

#### solo-journey · Aurelian Reach (demacia) · sky plane

- **Prompt:** "Sun-bleached limestone training hall interior, FAR background only — high
  vaulted stone ceiling pierced by warm dawn shafts, gilded heraldry banners receding
  into atmospheric haze, motes of dust in backlit air, oil painting with visible
  brushwork, moody atmospheric. No foreground objects."
- **Negative prompt:** "neon, dark, modern, photorealistic, sharp lines"
- **Seed:** TBD
- **Final size:** TBD
- **BiRefNet applied:** No

#### solo-journey · Aurelian Reach (demacia) · mid plane

- **Prompt:** "MIDGROUND only on transparent background — carved limestone pillar pair
  and armory wall bracket, warm torchlight pooling on stone, gilded edge detail, hand-
  painted textured, painterly. Transparent sky and floor."
- **Negative prompt:** "neon, modern, photorealistic"
- **Seed:** TBD
- **Final size:** TBD
- **BiRefNet applied:** Yes

#### solo-journey · Aurelian Reach (demacia) · near plane

- **Prompt:** "FOREGROUND only on transparent background — edge of a worn oak training
  table and a practice weapon rack in warm shadow, bottom of frame, oil painting with
  visible brushwork, close detail. Transparent background."
- **Negative prompt:** "neon, modern, photorealistic, full scene"
- **Seed:** TBD
- **Final size:** TBD
- **BiRefNet applied:** Yes

#### solo-journey · Sundered Hollow (pandemonium) · sky plane

- **Prompt:** "Fractured obsidian training hall interior, FAR background only — cracked
  black-glass vaulted ceiling with magenta arc-light bleeding through fissures, brutalist
  stone-and-metal structure receding into electric storm-haze, oil painting with visible
  brushwork, moody atmospheric. No foreground objects."
- **Negative prompt:** "warm, gold, classical, photorealistic, sharp"
- **Seed:** TBD
- **Final size:** TBD
- **BiRefNet applied:** No

#### solo-journey · Sundered Hollow (pandemonium) · mid plane

- **Prompt:** "MIDGROUND only on transparent background — fractured dark-glass pillar
  pair and weapon bracket in a brutalist hall, cyan arc-light tracing cracks, hand-
  painted textured, painterly. Transparent sky and floor."
- **Negative prompt:** "warm, gold, classical, photorealistic"
- **Seed:** TBD
- **Final size:** TBD
- **BiRefNet applied:** Yes

#### solo-journey · Sundered Hollow (pandemonium) · near plane

- **Prompt:** "FOREGROUND only on transparent background — edge of a dark-metal training
  stand and fractured floor tile, magenta-tinged near shadow, bottom of frame, oil
  painting with visible brushwork. Transparent background."
- **Negative prompt:** "warm, gold, classical, photorealistic, full scene"
- **Seed:** TBD
- **Final size:** TBD
- **BiRefNet applied:** Yes

#### solo-journey overlays — Aurelian Reach (demacia)

- **banner-1:** "Isolated growth element on transparent background — a half-hung gilded
  victory banner of warm-gold fabric, sun-bleached limestone wall behind, sparse,
  tier-1 sparse state, hand-painted, textured."
- **banner-2:** "Isolated growth element on transparent background — a pair of gilded
  victory banners fully hung, heraldic emblem visible, warm-gold and cream, mid-fill,
  hand-painted, textured."
- **banner-3:** "Isolated growth element on transparent background — a full wall of
  gilded victory banners, richly detailed heraldry, warm dawn light catching folds,
  tier-3 flourishing, hand-painted, textured."
- **lib-1:** "Isolated growth element on transparent background — a single shelf with
  a few worn leather-bound tomes, sparse, tier-1 state, oil painting with visible
  brushwork."
- **lib-2:** "Isolated growth element on transparent background — two shelves of
  leather-bound tomes and scrolls, growing collection, mid-fill, oil painting."
- **lib-3:** "Isolated growth element on transparent background — towering bookshelf
  packed with leather-bound tomes and scrolls, richly detailed spines, tier-3
  flourishing, oil painting with visible brushwork."
- **armory-1:** "Isolated growth element on transparent background — a sparse weapon
  rack with one practice sword and shield, tier-1 sparse, hand-painted, textured."
- **armory-2:** "Isolated growth element on transparent background — a weapon rack
  with several swords, shields, and a bow, growing armory, mid-fill, hand-painted."
- **armory-3:** "Isolated growth element on transparent background — a full gleaming
  armory rack of swords, shields, polearms, and bows, tier-3 flourishing, warm-gold
  light on polished steel, hand-painted, textured."

**All overlay seeds:** TBD | **BiRefNet applied:** Yes

#### solo-journey overlays — Sundered Hollow (pandemonium)

- **banner-1:** "Isolated growth element on transparent background — a single fractured
  obsidian banner frame with magenta-tinted fabric, sparse, tier-1 state, hand-painted."
- **banner-2:** "Isolated growth element on transparent background — two obsidian banner
  frames with crackling cyan runes visible, mid-fill, hand-painted, textured."
- **banner-3:** "Isolated growth element on transparent background — a full wall of
  obsidian banner frames blazing with magenta and cyan arc-light runes, tier-3
  flourishing, hand-painted."
- **lib-1:** "Isolated growth element on transparent background — a sparse dark-metal
  shelf with a few cracked tomes, tier-1 state, oil painting."
- **lib-2:** "Isolated growth element on transparent background — dark-metal shelves
  with tomes and void-crystal fragments, growing collection, mid-fill, oil painting."
- **lib-3:** "Isolated growth element on transparent background — towering dark-metal
  shelves crammed with fractured tomes and glowing void-crystal orbs, tier-3
  flourishing, cyan arc-light on spines, oil painting."
- **armory-1:** "Isolated growth element on transparent background — a sparse obsidian
  weapon rack with one crackling arc-light blade, tier-1 sparse, hand-painted."
- **armory-2:** "Isolated growth element on transparent background — an obsidian weapon
  rack with several void-forged blades and fractured shields, mid-fill, hand-painted."
- **armory-3:** "Isolated growth element on transparent background — a full obsidian
  armory rack of void-forged blades, shard-shields, and crystalline polearms blazing
  with cyan arc-light, tier-3 flourishing, hand-painted."

**All overlay seeds:** TBD | **BiRefNet applied:** Yes

---

### Room: war-room

**Route:** `/war-room` | **D-07 mood:** tactical war-table chamber — tense, focused light over central planning table, maps and banners

#### war-room · Aurelian Reach (demacia) · sky plane

- **Prompt:** "Sun-bleached limestone war council chamber, FAR background only — high
  vaulted stone ceiling with amber candle-sconce light, gilded heraldic banners
  descending from the rafters, atmospheric haze of torch smoke, oil painting with
  visible brushwork, moody atmospheric. No foreground objects."
- **Negative prompt:** "neon, modern, photorealistic, bright"
- **Seed:** TBD
- **Final size:** TBD
- **BiRefNet applied:** No

#### war-room · Aurelian Reach (demacia) · mid plane

- **Prompt:** "MIDGROUND only on transparent background — a large oak war table
  scattered with parchment maps, candles guttering in the draught, gilded map weights,
  warm amber light, hand-painted textured, painterly. Transparent sky."
- **Negative prompt:** "neon, modern, photorealistic"
- **Seed:** TBD
- **Final size:** TBD
- **BiRefNet applied:** Yes

#### war-room · Aurelian Reach (demacia) · near plane

- **Prompt:** "FOREGROUND only on transparent background — edge of a carved stone
  column and a hanging campaign map in amber shadow, bottom of frame, oil painting
  with visible brushwork. Transparent background."
- **Negative prompt:** "neon, modern, photorealistic, full scene"
- **Seed:** TBD
- **Final size:** TBD
- **BiRefNet applied:** Yes

#### war-room · Sundered Hollow (pandemonium) · sky plane

- **Prompt:** "Fractured obsidian war-council chamber, FAR background only — cracked
  black-glass ceiling with magenta arc-light bleeding through, brutalist iron-and-
  obsidian beams, electric storm-haze, oil painting with visible brushwork, moody
  atmospheric. No foreground objects."
- **Negative prompt:** "warm, gold, classical, photorealistic"
- **Seed:** TBD
- **Final size:** TBD
- **BiRefNet applied:** No

#### war-room · Sundered Hollow (pandemonium) · mid plane

- **Prompt:** "MIDGROUND only on transparent background — a dark-metal war table
  inlaid with glowing cyan tactical runes, fractured obsidian weights, magenta arc-
  light above, hand-painted textured, painterly. Transparent sky."
- **Negative prompt:** "warm, gold, classical, photorealistic"
- **Seed:** TBD
- **Final size:** TBD
- **BiRefNet applied:** Yes

#### war-room · Sundered Hollow (pandemonium) · near plane

- **Prompt:** "FOREGROUND only on transparent background — edge of a fractured obsidian
  column and a hanging void-sigil map in magenta-tinged shadow, bottom of frame, oil
  painting with visible brushwork. Transparent background."
- **Negative prompt:** "warm, gold, classical, photorealistic, full scene"
- **Seed:** TBD
- **Final size:** TBD
- **BiRefNet applied:** Yes

#### war-room overlays — Aurelian Reach (demacia)

- **tactics-1:** "Isolated growth element on transparent background — a sparse parchment
  campaign map pinned to a limestone board, a single marker token, tier-1 sparse,
  hand-painted."
- **tactics-2:** "Isolated growth element on transparent background — a campaign map
  board with several marker tokens and route lines drawn, mid-fill, hand-painted."
- **tactics-3:** "Isolated growth element on transparent background — a full campaign
  map densely marked with tactical tokens, route lines, and highlighted objectives,
  tier-3 flourishing, warm candlelight, hand-painted."
- **intel-1:** "Isolated growth element on transparent background — a sparse folded
  scouting report tied with gilded cord, tier-1 sparse, oil painting."
- **intel-2:** "Isolated growth element on transparent background — a stack of scouting
  reports and scrolls with wax seals, mid-fill, oil painting."
- **intel-3:** "Isolated growth element on transparent background — an overflowing
  intelligence ledger with reports, scrolls, and sealed missives, tier-3, oil painting."
- **command-1:** "Isolated growth element on transparent background — a lone gilded
  commander's seal on a campaign table, tier-1 sparse, hand-painted."
- **command-2:** "Isolated growth element on transparent background — a commander's
  seal flanked by ranked insignia tokens, mid-fill, hand-painted."
- **command-3:** "Isolated growth element on transparent background — a full command
  station with ranked insignia, a commander's seal, and ranked commendation scrolls,
  tier-3 flourishing, hand-painted."

**All overlay seeds:** TBD | **BiRefNet applied:** Yes

#### war-room overlays — Sundered Hollow (pandemonium)

- **tactics-1:** "Isolated growth element on transparent background — a sparse void-
  rune tactical display, single marker, tier-1 sparse, hand-painted."
- **tactics-2:** "Isolated growth element on transparent background — a void-rune
  tactical display board with several glowing marker tokens, mid-fill, hand-painted."
- **tactics-3:** "Isolated growth element on transparent background — a full void-rune
  tactical display blazing with cyan and magenta tactical markers, tier-3, hand-painted."
- **intel-1:** "Isolated growth element on transparent background — a single crackling
  void-crystal shard intel fragment, tier-1 sparse, oil painting."
- **intel-2:** "Isolated growth element on transparent background — a cluster of void-
  crystal intel shards and fractured sigil-tablets, mid-fill, oil painting."
- **intel-3:** "Isolated growth element on transparent background — an overflowing
  void-crystal intel array with shards, sigil-tablets, and arc-light data streams,
  tier-3 flourishing, oil painting."
- **command-1:** "Isolated growth element on transparent background — a single obsidian
  command sigil, tier-1 sparse, hand-painted."
- **command-2:** "Isolated growth element on transparent background — obsidian command
  sigils with fractured rank insignia, mid-fill, hand-painted."
- **command-3:** "Isolated growth element on transparent background — a full obsidian
  command array with ranked sigils blazing with cyan arc-light, tier-3, hand-painted."

**All overlay seeds:** TBD | **BiRefNet applied:** Yes

---

### Room: library

**Route:** `/library` | **D-07 mood:** scholarly archive — towering shelves of tomes and scrolls, quiet contemplative reading light, dust motes

#### library · Aurelian Reach (demacia) · sky plane

- **Prompt:** "Sun-bleached limestone library hall, FAR background only — towering
  vaulted stone ceiling with golden reading light streaming through high arched windows,
  gilded book-case spandrels receding in atmospheric haze, dust motes in backlit air,
  oil painting with visible brushwork, contemplative. No foreground objects."
- **Negative prompt:** "neon, modern, photorealistic, dark"
- **Seed:** TBD
- **Final size:** TBD
- **BiRefNet applied:** No

#### library · Aurelian Reach (demacia) · mid plane

- **Prompt:** "MIDGROUND only on transparent background — a grand limestone reading
  lectern and an arched bookshelf alcove packed with leather-bound tomes, golden
  reading candlelight, hand-painted textured. Transparent sky."
- **Negative prompt:** "neon, modern, photorealistic"
- **Seed:** TBD
- **Final size:** TBD
- **BiRefNet applied:** Yes

#### library · Aurelian Reach (demacia) · near plane

- **Prompt:** "FOREGROUND only on transparent background — edge of a carved oak reading
  desk with an open tome and quill, dust motes in warm shadow, bottom of frame, oil
  painting with visible brushwork. Transparent background."
- **Negative prompt:** "neon, modern, photorealistic, full scene"
- **Seed:** TBD
- **Final size:** TBD
- **BiRefNet applied:** Yes

#### library · Sundered Hollow (pandemonium) · sky plane

- **Prompt:** "Fractured obsidian archive hall, FAR background only — towering dark-
  glass ceiling with magenta arc-light filtering through crystal-lattice spandrels,
  void-crystal shelves receding in electric storm-haze, oil painting with visible
  brushwork, moody atmospheric. No foreground objects."
- **Negative prompt:** "warm, gold, classical, photorealistic"
- **Seed:** TBD
- **Final size:** TBD
- **BiRefNet applied:** No

#### library · Sundered Hollow (pandemonium) · mid plane

- **Prompt:** "MIDGROUND only on transparent background — a dark-metal reading stand
  and a void-crystal shelving alcove with fractured tomes glowing at their spines,
  cyan arc-light from above, hand-painted textured. Transparent sky."
- **Negative prompt:** "warm, gold, classical, photorealistic"
- **Seed:** TBD
- **Final size:** TBD
- **BiRefNet applied:** Yes

#### library · Sundered Hollow (pandemonium) · near plane

- **Prompt:** "FOREGROUND only on transparent background — edge of a dark-metal reading
  desk with a glowing sigil-tome open, magenta light spilling onto fractured floor,
  bottom of frame, oil painting with visible brushwork. Transparent background."
- **Negative prompt:** "warm, gold, classical, photorealistic, full scene"
- **Seed:** TBD
- **Final size:** TBD
- **BiRefNet applied:** Yes

#### library overlays — Aurelian Reach (demacia)

- **records-1:** "Isolated growth element on transparent background — a sparse bronze
  records plaque with a few entries etched, tier-1 sparse, hand-painted."
- **records-2:** "Isolated growth element on transparent background — a bronze records
  display with growing list of etched entries and a wax seal, mid-fill, hand-painted."
- **records-3:** "Isolated growth element on transparent background — a full illuminated
  records board with dense golden-etched entries and heraldic seals, tier-3, hand-
  painted."
- **tomes-1:** "Isolated growth element on transparent background — a single leather-
  bound tome on an oak shelf, tier-1 sparse, oil painting."
- **tomes-2:** "Isolated growth element on transparent background — a pair of shelves
  filling with leather-bound tomes and scrolls, mid-fill, oil painting."
- **tomes-3:** "Isolated growth element on transparent background — towering shelves
  packed densely with leather-bound tomes, scrolls, and illuminated manuscripts,
  tier-3 flourishing, warm reading light, oil painting."
- **analysis-1:** "Isolated growth element on transparent background — a sparse analysis
  diagram on parchment with a few annotated nodes, tier-1, hand-painted."
- **analysis-2:** "Isolated growth element on transparent background — an analysis
  parchment spread with growing node diagram, annotations, mid-fill, hand-painted."
- **analysis-3:** "Isolated growth element on transparent background — a richly annotated
  analysis diagram covering a large parchment, dense node-and-arrow network, gilded
  highlights, tier-3, hand-painted."

**All overlay seeds:** TBD | **BiRefNet applied:** Yes

#### library overlays — Sundered Hollow (pandemonium)

- **records-1:** "Isolated growth element on transparent background — a sparse fractured
  obsidian records plaque with a single glowing void-etched entry, tier-1, hand-painted."
- **records-2:** "Isolated growth element on transparent background — a fractured obsidian
  records display with growing cyan-etched entries, mid-fill, hand-painted."
- **records-3:** "Isolated growth element on transparent background — a full void-rune
  records board blazing with dense cyan and magenta etched entries, tier-3, hand-painted."
- **tomes-1:** "Isolated growth element on transparent background — a single cracked tome
  on a dark-metal shelf, magenta spine glow, tier-1 sparse, oil painting."
- **tomes-2:** "Isolated growth element on transparent background — dark-metal shelves
  filling with cracked tomes and void-crystal orb bookends, mid-fill, oil painting."
- **tomes-3:** "Isolated growth element on transparent background — towering dark-metal
  shelves packed with fractured tomes, void-crystal orbs, and arc-light scroll cases,
  tier-3 flourishing, cyan light on spines, oil painting."
- **analysis-1:** "Isolated growth element on transparent background — a sparse void-
  sigil analysis diagram on fractured crystal with a few glowing nodes, tier-1, hand-painted."
- **analysis-2:** "Isolated growth element on transparent background — a void-sigil
  analysis crystal display with growing arc-light node diagram, mid-fill, hand-painted."
- **analysis-3:** "Isolated growth element on transparent background — a richly detailed
  void-sigil analysis array covering a large crystal surface, dense cyan-and-magenta
  node network blazing, tier-3 flourishing, hand-painted."

**All overlay seeds:** TBD | **BiRefNet applied:** Yes

---

### Room: throne

**Route:** `/throne` | **D-07 mood:** regal ceremonial hall — throne dais, grand vertical space, heraldry, ascension

#### throne · Aurelian Reach (demacia) · sky plane

- **Prompt:** "Sun-bleached limestone ceremonial hall, FAR background only — soaring
  vaulted ceiling forty metres high, gilded heraldic banners descending in a grand
  nave, warm dawn light flooding through clerestory arches, atmospheric depth haze,
  oil painting with visible brushwork, regal, ascendant. No foreground objects."
- **Negative prompt:** "neon, modern, photorealistic, dark, gloomy"
- **Seed:** TBD
- **Final size:** TBD
- **BiRefNet applied:** No

#### throne · Aurelian Reach (demacia) · mid plane

- **Prompt:** "MIDGROUND only on transparent background — a grand stone throne dais
  with gilded stepped risers and a high-backed ceremonial seat, warm dawn light
  pooling on carved limestone, hand-painted textured. Transparent sky."
- **Negative prompt:** "neon, modern, photorealistic"
- **Seed:** TBD
- **Final size:** TBD
- **BiRefNet applied:** Yes

#### throne · Aurelian Reach (demacia) · near plane

- **Prompt:** "FOREGROUND only on transparent background — edge of a carved limestone
  column base and gilded floor inlay in warm shadow, bottom of frame, oil painting
  with visible brushwork. Transparent background."
- **Negative prompt:** "neon, modern, photorealistic, full scene"
- **Seed:** TBD
- **Final size:** TBD
- **BiRefNet applied:** Yes

#### throne · Sundered Hollow (pandemonium) · sky plane

- **Prompt:** "Fractured obsidian ceremonial hall, FAR background only — soaring
  cracked black-glass ceiling with magenta and cyan arc-light blazing through fissures,
  brutalist obsidian column nave receding in electric storm-haze, oil painting with
  visible brushwork, ominous, ascendant. No foreground objects."
- **Negative prompt:** "warm, gold, classical, photorealistic"
- **Seed:** TBD
- **Final size:** TBD
- **BiRefNet applied:** No

#### throne · Sundered Hollow (pandemonium) · mid plane

- **Prompt:** "MIDGROUND only on transparent background — a fractured obsidian throne
  dais with dark-crystal risers and a high-backed void-metal seat crackling with cyan
  arc-light, magenta haze above, hand-painted textured. Transparent sky."
- **Negative prompt:** "warm, gold, classical, photorealistic"
- **Seed:** TBD
- **Final size:** TBD
- **BiRefNet applied:** Yes

#### throne · Sundered Hollow (pandemonium) · near plane

- **Prompt:** "FOREGROUND only on transparent background — edge of a fractured obsidian
  column base and magenta-lit floor crack, bottom of frame, oil painting with visible
  brushwork. Transparent background."
- **Negative prompt:** "warm, gold, classical, photorealistic, full scene"
- **Seed:** TBD
- **Final size:** TBD
- **BiRefNet applied:** Yes

#### throne overlays — Aurelian Reach (demacia)

- **roster-1:** "Isolated growth element on transparent background — a sparse gilded
  roster plaque with one name engraved, tier-1 sparse, hand-painted."
- **roster-2:** "Isolated growth element on transparent background — a gilded roster
  plaque with growing list of names and heraldic crests, mid-fill, hand-painted."
- **roster-3:** "Isolated growth element on transparent background — a full illuminated
  roster board dense with gilded names, crests, and rank ribbons, tier-3, hand-painted."
- **progression-1:** "Isolated growth element on transparent background — a single
  gilded rank medallion on a velvet cushion, tier-1 sparse, oil painting."
- **progression-2:** "Isolated growth element on transparent background — a growing
  array of gilded rank medallions and achievement scrolls, mid-fill, oil painting."
- **progression-3:** "Isolated growth element on transparent background — a full
  display of gilded rank medallions, championship trophies, and achievement scrolls
  in warm dawn light, tier-3 flourishing, oil painting."
- **crown-1:** "Isolated growth element on transparent background — a sparse golden
  laurel crown on a stone pedestal, tier-1, hand-painted."
- **crown-2:** "Isolated growth element on transparent background — a golden laurel
  crown flanked by heraldic flags and a victory sceptre, mid-fill, hand-painted."
- **crown-3:** "Isolated growth element on transparent background — a grand gilded
  crown display with heraldic flags, a victory sceptre, and illuminated achievement
  scroll, tier-3 flourishing, warm ceremonial light, hand-painted."

**All overlay seeds:** TBD | **BiRefNet applied:** Yes

#### throne overlays — Sundered Hollow (pandemonium)

- **roster-1:** "Isolated growth element on transparent background — a sparse fractured
  obsidian roster plaque with one void-etched name glowing cyan, tier-1, hand-painted."
- **roster-2:** "Isolated growth element on transparent background — a fractured obsidian
  roster plaque with growing cyan-etched names and sigil-crests, mid-fill, hand-painted."
- **roster-3:** "Isolated growth element on transparent background — a full void-rune
  roster board blazing with dense cyan and magenta etched names and sigil-crests,
  tier-3 flourishing, hand-painted."
- **progression-1:** "Isolated growth element on transparent background — a single void-
  crystal achievement shard on a dark-metal stand, magenta glow, tier-1, oil painting."
- **progression-2:** "Isolated growth element on transparent background — a growing array
  of void-crystal achievement shards and arc-light rank insignia, mid-fill, oil painting."
- **progression-3:** "Isolated growth element on transparent background — a full display
  of void-crystal achievement shards, obsidian trophies, and arc-light rank insignia
  blazing with cyan and magenta, tier-3 flourishing, oil painting."
- **crown-1:** "Isolated growth element on transparent background — a sparse obsidian
  void-crown crackling with a single magenta arc, tier-1, hand-painted."
- **crown-2:** "Isolated growth element on transparent background — a void-crown
  flanked by obsidian standard-poles with crackling cyan sigil-flags, mid-fill,
  hand-painted."
- **crown-3:** "Isolated growth element on transparent background — a grand void-crown
  display blazing with magenta and cyan arc-light, obsidian standards, and a void-
  sceptre, tier-3 flourishing, hand-painted."

**All overlay seeds:** TBD | **BiRefNet applied:** Yes

---

## Phase 17 Legacy Assets (preserved for historical record)

> The entries below were written for Phase 17 plan 06 (closed-beta backgrounds).
> They contain legacy non-Path-A vocabulary that is NOT compliant with D-01.
> Do not feed these prompts to Phase 30 generation scripts.
> If regeneration is needed, use the Path A prompt rewrites in the sections above.

### Generation runtime decision (Task 1, Phase 17)

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
  files. **Use Path A vocabulary when doing so.**
- **Optional auth background:** generated as placeholder per Task 1
  decision (auto-mode: "yes, generate auth-bg-warm-region").
- **Composition reference image:** the upload at
  `/tmp/lol-design-handoff/lol-team-companion-app/project/uploads/draw-92acceeb-9fd2-499d-84e4-12ff75b7ab5d.png`
  (2576×1479 PNG, 116KB) was inspected. It is a wireframe sketch from
  the design handoff bundle, not a usable photographic asset. Role:
  **composition reference only** (for future fal.ai prompt tuning;
  none of the current placeholders use it as input). No bytes copied
  into `public/img/`.

### Placeholder generation method (Phase 17 binaries)

The placeholders are layered ImageMagick gradients composed to evoke
the painterly feel of the FLUX target without using any external
service. The exact ImageMagick pipeline is reproducible from this
file alone.

All three assets are 1920×1080, JPEG quality 78, blurred 0×1.2-0×1.5
to read as soft painterly underlay rather than pixel art. Final size
sits well under the 400 KB performance budget.

### Token vocabulary used in placeholders

The token hex values below are mirrored from `input.css`'s `@theme`
block (warm region approx lines 193–233, chaos region approx lines 235–268).

- Warm-region accent gold: `#d4af5a`
- Warm-region warm midtone: `#3a2a14`
- Warm-region base: `#070912`
- Chaos-region accent pink: `#ff4d8a`
- Chaos-region accent cyan: `#78dcf0`
- Chaos-region base: `#06070b`

### `public/img/beta-landing-demacia.jpg` (Phase 17 placeholder)

- **Generated:** 2026-05-07
- **Surface:** `/closed-beta` hero landing (warm-region theme variant)
- **Compute path:** placeholder (ImageMagick layered gradient + plasma noise)
- **Intended runtime:** fal.ai — `flux.1-pro`
- **Intended prompt (Path A rewrite required before use):** "Sun-bleached limestone
  citadel at dawn, oil painting with visible brushwork, warm golden light, gilded
  heraldic banners, vaulted stone architecture, atmospheric perspective, hand-painted,
  moody atmospheric"
- **Intended negative prompt:** "modern, futuristic, neon, photorealistic"
- **Intended aspect / resolution:** 16:9 / 1920×1080
- **Intended seed:** TBD on first FLUX run — use seed `2026_warm_region_01`
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
    public/img/beta-landing-<warm-region-id>.jpg
  ```
- **TODO:** Regenerate via fal.ai once `FAL_KEY` is provisioned. Use Path A vocabulary from the Phase 30 prompts above.

### `public/img/beta-landing-pandemonium.jpg` (Phase 17 placeholder)

- **Generated:** 2026-05-07
- **Surface:** `/closed-beta` hero landing (chaos-region theme variant)
- **Compute path:** placeholder (ImageMagick layered gradient + plasma noise)
- **Intended runtime:** fal.ai — `flux.1-pro`
- **Intended prompt (Path A rewrite required before use):** "Fractured obsidian ruin
  in electric storm-haze, neon magenta and cyan arc-light, brutalist architecture,
  oil painting with visible brushwork, hand-painted, moody atmospheric, painterly"
- **Intended negative prompt:** "warm, gold, classical, oil painting"
- **Intended aspect / resolution:** 16:9 / 1920×1080
- **Intended seed:** TBD on first FLUX run — use seed `2026_chaos_region_01`
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
    public/img/beta-landing-<chaos-region-id>.jpg
  ```
- **TODO:** Regenerate via fal.ai once `FAL_KEY` is provisioned. Use Path A vocabulary from the Phase 30 prompts above.

### `public/img/auth-bg-<warm-region-id>.jpg` (Phase 17 placeholder)

- **Generated:** 2026-05-07
- **Surface:** `/auth/login` and `/auth/register` background (optional;
  warm-region theme variant only — chaos-region auth uses solid bg).
- **Compute path:** placeholder (ImageMagick layered gradient)
- **Intended runtime:** fal.ai — `flux.1-pro`
- **Intended prompt (Path A rewrite required before use):** "Limestone archway in soft
  morning haze, midground, distance, oil painting with visible brushwork, less
  saturated, warm golden ambient, hand-painted"
- **Intended negative prompt:** "neon, modern, photorealistic, sharp"
- **Intended aspect / resolution:** 16:9 / 1920×1080
- **Intended seed:** TBD — use seed `2026_warm_region_auth_01`.
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
    public/img/auth-bg-<warm-region-id>.jpg
  ```
- **TODO:** Regenerate via fal.ai once `FAL_KEY` is provisioned. Use Path A vocabulary.
  Auth background is `optional` per UI-SPEC §"Asset versioning" — if the FLUX run blows
  the 400 KB budget for auth specifically, drop this file rather than the hero backgrounds.

### Phase 17 rooms parallax placeholder assets

`public/img/rooms/solo-journey/{warm-region-id,chaos-region-id}/` — 6 files each (placeholders).
- **Compute path:** ImageMagick 6.9 (`convert`), layered `gradient:` (sky) and
  blurred `xc:none` rectangles (transparent planes/overlays), WebP q68.
- **Sizes:** ~26 KB per region (12 files total ≈ 52 KB).
- **Status:** Superseded by Phase 30 FLUX generation. Replace via Phase 30 scripts.

---

## How to regenerate via Phase 30 pipeline

```bash
# Prerequisites: BFL_KEY and FAL_KEY set in .env (or environment)
source .env

# Generate one room, one region.
# <room> ∈ solo-journey | war-room | library | throne
# <region> = the internal dir-name identifier (see input.css @theme block)
./scripts/gen-room-art.sh <room> <region>

# Run background removal on mid/near/overlay planes
./scripts/remove-bg.sh <room> <region>

# Encode to WebP
./scripts/encode-webp.sh <room> <region>

# Verify budget
./scripts/verify-budget.sh

# Update this file: fill in Seed, Final size, remove TBD, remove TODO lines
```

After each room/region generation, update each asset entry above with:
- `**Seed:** <integer BFL assigned>`
- `**Final size:** <bytes> KB WebP q80`
- Remove TBD placeholders on those fields
