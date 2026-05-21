# Phase 18 Plan 07 — Content Contracts (4 New Sibling Pairs)

**Derived:** 2026-05-21
**Sources:** CONTEXT.md D-11/D-12, 18-UI-SPEC.md §Page-Pair Inventory, 18-RESEARCH.md F-01
**Method:** Phase D step 1 per CONTEXT.md D-12
**Note:** `.local-design-source/` is gitignored and not accessible in executor worktree.
Content contracts derived from CONTEXT.md aesthetic descriptions, UI-SPEC.md region grammar,
and RESEARCH.md documented findings (F-01 confirmed all 4 JSX files exist at 269–298 lines).

---

## draft-ledger

### Demacia variant (medieval double-entry ledger, NEW sibling)

Required visible elements:
- Heraldic header: team name + opponent + date, `font-display` Cormorant Garamond, centered
- `<Crown>` SVG ornament above the title
- "DRAFT LEDGER" heading in Cinzel uppercase + gilt colour
- `<HeraldicDivider>` below the header
- Two-column layout: BLUE SIDE (left) / RED SIDE (right) — each column uses a gilt Card
- Per column `<Eyebrow>` label ("BLUE SIDE" / "RED SIDE") in Cinzel uppercase
- 5 pick rows: ledger-entry style — `{n}. <champion> ........... <role>` in Cormorant Garamond
- `<HeraldicDivider>` between picks and bans block
- 5 ban rows: same ledger format with `font-display italic` strikethrough treatment
- `<GiltCorner>` ornaments at each `Card variant="gilt"` corner (rendered by Card primitive)
- Bottom journal section: gilt Card with "JOURNAL" `<Eyebrow>`, italic Cormorant body text
- Copy: section labels in Cinzel uppercase ("PICKS", "BANS", "JOURNAL")

Required data fields:
- `draft_slots: ReadSignal<Vec<Option<String>>>` — slots 6–10 (blue picks), 7–11 (red picks)
- `our_side: ReadSignal<String>` — to label "OUR SIDE" / "THEIR SIDE" variants
- Bans: slots 0–4 (blue bans), 1–5 (red bans)
- Placeholder journal text for post-game analysis

### Pandemonium variant (brutalist ledger, EXISTING)

Required visible elements:
- `<RiotTape label="DRAFT_LEDGER · v0.1">` as header strip
- Background: `bg-base bg-scanline` container
- Two `<Card variant="zine">` columns side by side
- Left column: `<Glitch>` label `"// BLUE_LOG"` as header
- Right column: `<Glitch>` label `"// RED_LOG"` as header
- Per column: 5 pick rows formatted as JetBrains Mono `"01 | Champion | Role"` entries
  - Row format: `{zero-padded index} | {champion_name} | {ROLE_TBD}` in monospace
  - Accent colour for the index number
  - `border-b border-outline/30` separator per row
- Below columns: `<Card variant="zine">` with `<Glitch>` label `"// JOURNAL_DUMP"`
- Journal body: `<pre>` block in JetBrains Mono 11px, placeholder `// Post-game analysis pending.`
- Copy: section labels in JetBrains Mono uppercase with underscores

Required data fields:
- Same draft signals as Demacia variant

### Mismatch table

| Element | Demacia | Pandemonium | Action |
|---------|---------|-------------|--------|
| Heraldic header (Crown + date + HeraldicDivider) | YES | NO | Demacia-only |
| RiotTape strip | NO | YES | Pandemonium-only |
| Ledger row formatting | gilt serif Cormorant | mono brackets `01 \| … \| …` | both region-styled |
| GiltCorner card ornaments | YES (via Card variant=gilt) | NO | Demacia via Card primitive |
| Bracket zine corners | NO | YES (via Card variant=zine) | Pandemonium via Card primitive |
| Column labels | "BLUE SIDE" / "RED SIDE" (Eyebrow) | "// BLUE_LOG" / "// RED_LOG" (Glitch) | both required |
| Footer journal | YES (gilt Card) | YES (zine Card) | both required, different styling |
| Picks+bans separation | HeraldicDivider | layout gap only | Demacia-only divider |

---

## solo-journal

### Demacia variant (parchment diary, EXISTING)

Required visible elements:
- Page header: `<Card variant="gilt">` with journal book metaphor
- `<SectionHead>` with eyebrow "CHRONICLES" and title "Journal"
- `<HeraldicDivider>` after header
- Entry list: each entry is a gilt-bordered `<Card variant="gilt">` containing:
  - Date sub-header in Cinzel small-caps
  - Body text in Cormorant Garamond italic
  - Optional mood ribbon: `bg-accent/20` left border stripe
- Empty state: `<PageEmpty kind="matches">` in Demacia grammar
- "OPEN JOURNAL" CTA: `<Btn region="demacia" variant="primary">`
- Tags: `<Badge tone="neutral">` per tag in Cormorant Garamond

Required data fields:
- Match history / personal learnings (from `dashboard_resource` or `personal_learnings` resource)
- Date strings, match outcome, champion played
- Placeholder: recent matches formatted as diary entries

### Pandemonium variant (photocopied fanzine, NEW sibling)

Required visible elements:
- `<RiotTape label="JOURNAL_RAW">` at top
- Background: `bg-base bg-halftone` (or `bg-scanline` if halftone utility not present)
- Entry list: each entry is a `<Card variant="zine">` with:
  - `<Glitch>` mono date header: `"// ENTRY_{date}"`
  - Entry body in JetBrains Mono 12px, terse note style
  - Rotated card treatment: alternating `-rotate-1` / `rotate-1` transform classes on cards
  - Bottom: `<Badge tone="accent">` per tag with mono text
- Empty state: `<PageEmpty kind="matches">` in Pandemonium grammar
- "OPEN_JOURNAL" CTA: `<Btn region="pandemonium" variant="primary">`
- Stamped style aesthetic via border decoration (no inline CSS rotation — use Tailwind rotate)

Required data fields:
- Same as Demacia variant

### Mismatch table

| Element | Demacia | Pandemonium | Action |
|---------|---------|-------------|--------|
| Page wrapper | gilt Card + parchment | RiotTape + halftone bg | both region-styled |
| Entry card | gilt Card + mood ribbon | zine Card + rotate transform | both region-styled |
| Date label | Cinzel small-caps | `// ENTRY_{date}` Glitch | both required, different copy |
| Entry body font | Cormorant Garamond italic | JetBrains Mono terse | both region-styled |
| Tags | Badge Cormorant | Badge mono | both present |
| Rotation effect | NO | YES (-rotate-1 / rotate-1) | Pandemonium-only |
| Header strip | HeraldicDivider | RiotTape "JOURNAL_RAW" | region-exclusive |
| Background | bg-surface parchment | bg-base bg-halftone | region-exclusive |

---

## solo-forge

### Demacia variant (smith's workbench, NEW sibling)

Required visible elements:
- Page header: `<Card variant="gilt">` with forge/workshop metaphor
- `<Crown>` SVG + "FORGE" heading in Cinzel uppercase
- `<SectionHead>` eyebrow "CHAMPION MASTERY"
- `<HeraldicDivider>` below header
- Central "forge" section: gilt `<Card>` containing:
  - `<RankBadge>` + `<LPProgress>` for current rank
  - "Improvement Targets": 3 gilt `<Card>` entries listing goal champions
  - Each entry: `<ChampTile name=... size=56>` + rank target in Cormorant serif
- "Tool rack" sidebar: row of champion `<ChampTile>` icons for pool champions
- Queue CTA: `<Btn region="demacia" variant="primary">` "Queue Aatrox" (dynamic champion)
- `<Badge tone="accent">` for active champion label

Required data fields:
- `ranked: Option<RankedInfo>` from dashboard_resource
- `goal_progress_resource` for goal champions
- Pool champions from match history (placeholder with TODO)

### Pandemonium variant (locker workbench, EXISTING)

Required visible elements:
- `<RiotTape label="FORGE · QUEUE_PREP">` header
- Background: `bg-base bg-scanline`
- "QUEUE_PREP" `<Card variant="zine">` containing:
  - `<Glitch>` label `"// PREP_LIST"` header
  - 3–5 prep items in JetBrains Mono: `"[ ] Aatrox — study lvl 3 jungle"` etc. format
  - Checkbox-style mono prefixes `[ ]` / `[x]`
- "POOL_STATUS" `<Card variant="zine">`:
  - `<Glitch>` label `"// POOL_STATUS"`
  - Champion tile grid: `<ChampTile>` with bracket-corner Pandemonium styling
  - Mono label per champion: `"CHAMPION / GAMES_30D / WR"`
- Rank display: flat magenta `<LPProgress>` + `<Glitch>` tier label `"// TIER"`
- "QUEUE" CTA: `<Btn region="pandemonium" variant="primary">`
- Magenta toolbox aesthetic via accent-coloured bracket accents

Required data fields:
- Same as Demacia variant

### Mismatch table

| Element | Demacia | Pandemonium | Action |
|---------|---------|-------------|--------|
| Header | Crown + Cinzel "FORGE" + HeraldicDivider | RiotTape "FORGE · QUEUE_PREP" | region-exclusive |
| Rank display | RankBadge + gilt LPProgress | Glitch "// TIER" + flat magenta LPProgress | both present, different styling |
| Goal/prep items | gilt Card + serif text | zine Card + mono checkbox format | both required, different styling |
| Champion pool display | ChampTile row (tool rack aesthetic) | ChampTile grid + mono GAMES/WR labels | both required, different layout |
| Queue CTA | "Queue {champion}" serif Btn | "QUEUE" mono Btn | both required, different copy |
| Background | bg-surface parchment | bg-base bg-scanline | region-exclusive |
| Rotation effects | NO | NO (forge is more structured) | n/a |

---

## team-game-day-brief

### Demacia variant (newspaper aesthetic, EXISTING)

Required visible elements:
- Multi-column newspaper masthead: `<Card variant="gilt">` with Cinzel serif headers
- "THE COMPANION GAZETTE" masthead in large Cinzel (font-display, bold)
- Date and edition number: `font-imperial` small-caps below masthead
- `<HeraldicDivider>` after masthead
- Three-column newspaper body:
  - Column 1: "ROSTER" article — player name, role, `<ChampTile>` 3 recent picks
  - Column 2: "STRATEGY" article — win conditions text, ban intentions, draft notes
  - Column 3: "OPPONENT INTEL" article — key threats, recent form (placeholder)
- Drop-cap styling on article first letter: `first-letter:float-left first-letter:font-display first-letter:text-5xl first-letter:text-accent`
- `<HeraldicDivider>` between article columns
- "EDITOR'S NOTE" sidebar: `<Card variant="gilt">` with captain's note (placeholder)
- Roster data: real team members from `team.members`
- Team name in masthead: `team.name`

Required data fields:
- `team: Team` with `team.name`, `team.members`
- `members: Vec<TeamMember>` for roster
- Placeholder: recent matches summary (3 matches), strategy text, opponent intel

### Pandemonium variant (xeroxed match-day zine, NEW sibling)

Required visible elements:
- `<RiotTape label="GAME_DAY · ZINE_v0.3">` at top
- Background: `bg-base bg-scanline`
- Irregularly stacked zine cards: `<Card variant="zine">` with alternating slight rotation:
  - `rotate-1` / `-rotate-1` alternating on outer wrapper divs for collage aesthetic
  - Each card "torn" from the page — use `border-t-0` or border variations per card
- Section 1: `<Glitch>` header `"// ROSTER_CARD"` + compact roster grid (real team members)
  - Each member: `<ChampTile>` + `// {ROLE_UPPERCASE}` label in JetBrains Mono
- Section 2: `<Glitch>` header `"// STRAT_NOTE"` + strategy text in mono font
  - Placeholder text with TODO comment
- Section 3: `<Glitch>` header `"// OPPONENT_INTEL"` + 3 intel lines in mono format
  - Format: `"LAST_5_BANS: Yasuo, Yone..."` etc. (placeholder with TODO)
- Section 4: `<Glitch>` header `"// THREAT_RANK"` + 3 threat entries
  - Format: `<ChampTile>` + severity `<Badge>` + "if you let it through:" warning line
- Footer: `"// SQUAD: {team_name}"` in JetBrains Mono
- Halftone / scan-glitch aesthetic via `bg-scanline` utility

Required data fields:
- `team: Team` — `team.name`, `team.members` (real data)
- Placeholder: strategy notes, opponent intel, threat rankings (all TODO)

### Mismatch table

| Element | Demacia | Pandemonium | Action |
|---------|---------|-------------|--------|
| Header | Cinzel "COMPANION GAZETTE" masthead | RiotTape "GAME_DAY · ZINE_v0.3" | region-exclusive |
| Layout | 3-column newspaper grid | Stacked rotated zine cards | region-exclusive |
| Section headers | Article headings in serif | `// SECTION_NAME` Glitch labels | region-exclusive |
| Roster display | Role + ChampTile in newspaper column | `// ROSTER_CARD` card with grid | both present, different layout |
| Drop-cap style | YES (first-letter CSS) | NO | Demacia-only |
| Card rotation | NO (formal newspaper) | YES (rotate-1 / -rotate-1) | Pandemonium-only |
| Background | bg-surface formal | bg-base bg-scanline | region-exclusive |
| Editor's note | YES (gilt Card aside) | NO (replaced by strat note section) | Demacia-only in this form |
| Footer | — (integrated in masthead) | `// SQUAD: {team_name}` mono | Pandemonium-only |
| Threat section | Integrated in "OPPONENT INTEL" column | Separate `// THREAT_RANK` card | both present, different framing |
