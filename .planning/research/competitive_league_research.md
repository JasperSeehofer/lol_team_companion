# Competitive League Research

Research date: 2026-03-12

---

## R1 -- PrimeLeague Integration

### What is Prime League?

Prime League is the official League of Legends esports league for Germany, Austria, and Switzerland. It is operated by **Freaks 4U Gaming** under license from Riot Games (renewed 2025-2027). The league runs seasonal splits (Winter, Summer, Fall) with multiple tiers (1. Liga, 2. Liga, etc.) and open tournaments like "Snowdown."

### Is there a public API?

**No official public API exists.** Key findings:

1. **No developer docs** -- primeleague.gg has no `/developers` page, no published API documentation, and no developer portal. Searches across primeleague.gg, Freaks4U, and Strauss (a sponsor, not an API provider) returned nothing.

2. **Undocumented internal API** -- The site does have an internal API used by the frontend. The best evidence comes from [primebot_backend](https://github.com/random-rip/primebot_backend), an unofficial Discord/Telegram notification bot. Its code reveals:
   - An internal REST API with endpoints for teams, matches, players, and leagues
   - Rate limit of **1 request per second** (given to the bot maintainers by Prime League staff)
   - **IP whitelisting required** -- the PrimeBot server IP was explicitly whitelisted; the API is not publicly accessible
   - The bot stores all API responses as local JSON files for development because external developers cannot hit the API directly

3. **Data model** (from primebot source): The internal API exposes `Player` (user ID, summoner name), `Team` (name, tag, picture), `Match` (schedule, results), and league/season structures. Match data can be enriched with opposing team information.

4. **Freaks4U insolvency risk** -- In late 2024, Freaks 4U Gaming filed for insolvency, though they subsequently renewed their operator license through 2027. This introduces organizational instability.

### Alternative data sources

- **Liquipedia** has comprehensive Prime League data (rosters, results, schedules) but requires scraping their wiki pages. Liquipedia has a [rate-limited API](https://liquipedia.net/commons/Liquipedia:API_Usage) for wiki content.
- **Riot's LoL Esports API** (lolesports.com) covers major leagues but Prime League coverage is inconsistent for lower tiers.
- **Leaguepedia/Fandom** wiki -- similar to Liquipedia, scrapeable but not a stable API.

### Feasibility assessment

| Option | Feasibility | Effort | Recommendation |
|--------|-------------|--------|----------------|
| Official Prime League API | Not possible | N/A | **Skip** -- no public API, IP whitelisted |
| Scrape primeleague.gg internal API | Fragile, ToS risk | Medium | **Skip** -- undocumented, whitelisted, could break |
| Liquipedia API for Prime League data | Possible but limited | Medium | **Wait** -- viable fallback if demand is strong |
| Manual import (CSV/JSON upload) | Always works | Low | **Build** -- let users paste/upload team data |

**Recommendation: Skip automated integration for now.** Build a manual roster/schedule import feature instead (CSV or JSON paste). If user demand materializes, revisit Liquipedia scraping as the most stable path. Do not attempt to reverse-engineer the primeleague.gg internal API.

---

## R2 -- Competitive Draft Practices & Tools

### Landscape of existing tools

| Tool | Key differentiator | Relevance to our app |
|------|-------------------|---------------------|
| [ProComps.gg](https://procomps.gg/) | AI-powered suggestions at each draft step; team roster management (starting 5, subs, coach, analyst); gamestyle analysis; integrates with Prodraft/DraftLoL for scrims | High -- closest to our team-oriented model |
| [DraftGap](https://draftgap.com/) | Open-source; syncs with League client; purely statistical (unopinionated); desktop app | Medium -- good reference for analysis features |
| [DraftGap AI](https://devpost.com/software/draftgap-ai-the-agentic-draft-strategist) | Multi-agent system (Strategist, Reasoning, Critic agents); built for Cloud9 coaches; detects team identities and opponent signatures | High -- shows what pro coaching teams want |
| [iTero](https://www.itero.gg/) | AI draft model with win-rate predictions; explains *why* a pick is good (lane pressure, scaling, composition); Overwolf integration | High -- annotation/rationale model |
| [Drafter.lol](https://drafter.lol) | One shareable link per entire series (BO3/BO5); scrim block support | Medium -- UX pattern for series-level drafts |
| [DraftVision](https://loldraftvision.fun/) | Fearless draft support; map drawing for strategy routes | Low -- map overlay is out of scope |
| [ScoutAhead.pro](https://scoutahead.pro/) | Fearless mode; live pick probability scoring; pro play data | Medium -- Fearless draft reference |
| [dr4ft.lol](https://dr4ft.lol/) | Free Fearless draft simulator with probability scoring | Low -- simple simulator |
| [Fearlessdraft.net](https://www.fearlessdraft.net/) | Dedicated Fearless draft simulator | Low -- single-purpose |

### What competitive teams want (from tool analysis)

Based on analyzing all the above tools and their feature sets, here is a ranked list of feature proposals for our Draft and Tree Drafter sections:

#### Draft Page improvements (ranked by impact)

1. **Per-pick rationale/comments** -- Allow text annotations on each pick and ban slot explaining *why* that champion was chosen. iTero's model shows coaches want to record reasoning (lane pressure, scaling, flex potential), not just the pick itself. This is the most requested coaching workflow gap.

2. **Composition analysis tags** -- Auto-tag or manually tag draft compositions with identities: "teamfight," "split-push," "poke," "pick," "early-game," "scaling." ProComps and DraftGap AI both classify compositions this way. Enables filtering saved drafts by playstyle.

3. **Win condition notes** -- A text field per completed draft for "how we win with this comp" and "what to watch out for." Distinct from per-pick comments; this is the macro summary. Maps to ProComps' "gamestyle overview, strengths and weaknesses."

4. **Matchup context** -- When a champion is picked, show basic counter/synergy info inline (not a full analysis engine, but surface data from our existing champion data). DraftGap and iTero both do this.

5. **Fearless draft mode** -- Track previously picked/banned champions across a series (BO3/BO5) and gray them out in subsequent games. ScoutAhead, dr4ft.lol, and DraftVision all support this. Relevant for competitive teams practicing series drafts.

6. **Series-level draft grouping** -- Group multiple drafts into a "series" (BO3/BO5) with a single shareable view. Drafter.lol's one-link-per-series pattern. Useful for post-series review.

7. **Side preference toggle with history** -- Our app already has blue/red side toggle. Extend with the ability to record which side was chosen and why, building a preference history over time.

#### Tree Drafter improvements (ranked by impact)

1. **Node-level comments/annotations** -- Each node in the draft tree should support rich text notes explaining the branching rationale ("if they pick Azir, we flex to mid Tristana because..."). This is the most impactful improvement for coaching workflows.

2. **Conditional pick labels on edges** -- Label the edges between nodes with the condition that triggers that branch ("enemy picks engage support," "enemy bans our ADC pool"). Currently edges show champion icons; adding text labels makes the tree self-documenting.

3. **Priority markers** -- Flag certain branches as "preferred" or "backup" with visual indicators (color, thickness, star). Helps coaches communicate the ideal draft path vs. fallback options.

4. **Opponent scouting integration** -- Link a tree to a specific opponent team and pre-populate likely picks based on their champion pool data. ProComps does this with their scouting features.

5. **Tree comparison view** -- Side-by-side two trees (e.g., "our plan vs. Team A" and "our plan vs. Team B") to spot overlapping flex picks and shared preparation.

6. **Snapshot/version history** -- Save tree versions before/after scrims so coaches can track how their draft prep evolved. A simple "duplicate tree" with timestamp is sufficient.

7. **Collapse/expand subtrees** -- For large trees, allow collapsing branches that aren't being actively worked on. Improves navigation in complex prep trees.

---

## R3 -- Champion Learning Notes

### How existing platforms structure champion knowledge

**Mobalytics** champion pages:
- Runes (primary + secondary tree, situational shards)
- Items (starter, core, situational, by game state)
- Ability order (with win rates per skill path)
- Matchups (favorable/unfavorable, with tips per matchup)
- Combos (key sequences with descriptions)
- Tips & tricks (general gameplay advice)
- Patch-over-patch trend graphs (win rate, pick rate, ban rate)

**MOBAFire** community guides add:
- Champion identity description (lane bully, scaling, roamer, etc.)
- Power spike timeline (levels 1-3, first item, level 6, 2-item, etc.)
- Teamfight role and positioning
- Macro decision-making per game phase
- Matchup-specific trade patterns
- Build path adaptation based on enemy comp

**U.GG** focuses on:
- Statistical builds (highest win rate, most popular)
- Pro player builds
- Counter stats (win rate against each champion)
- Duo synergy stats

**LoL Matchup Notes** (oracle-notes.lovable.app):
- Champion vs. champion matchup entries
- Per-matchup: strategy text, tips, difficulty rating
- Personal knowledge database (not crowd-sourced)

### What our app should track (beyond a single textarea)

The current champion pool page uses a single tier list. A "champion learning notes" feature should let players build structured personal knowledge for each champion they play. Based on the research, here is a suggested schema:

#### Proposed schema: `ChampionNote`

```
ChampionNote {
    id: String,
    user_id: String,
    champion_name: String,

    // -- Overview --
    comfort_level: u8,          // 1-5 scale (1 = learning, 5 = mastered)
    roles: Vec<String>,         // ["mid", "top"] -- roles they play this champ
    identity_tags: Vec<String>, // ["scaling", "teamfight", "zone-control"]

    // -- Build notes --
    preferred_runes: String,    // Free text: "Electrocute into melee, Comet into ranged"
    core_items: String,         // Free text: "Luden's > Shadowflame always, Zhonya's 3rd if assassins"
    situational_notes: String,  // "Banshee's vs Veigar cage, Morello vs healers"

    // -- Power spikes --
    power_spikes: Vec<PowerSpike>,
    // PowerSpike { timing: String, description: String }
    // e.g. { "Level 3", "Full combo available, look for all-in if they used escape" }
    // e.g. { "Lost Chapter", "Can perma-shove and roam" }
    // e.g. { "Level 6", "Ult enables tower dives with jungler" }

    // -- Combos --
    combos: Vec<Combo>,
    // Combo { name: String, sequence: String, notes: String }
    // e.g. { "Basic trade", "Q > AA > W > AA", "Use in short trades, walk back before they retaliate" }
    // e.g. { "All-in", "E > Flash > R > Q > W > AA", "Only when ult is up, kills at 60% HP" }

    // -- Matchups --
    matchups: Vec<MatchupNote>,
    // MatchupNote {
    //   opponent_champion: String,
    //   difficulty: u8,           // 1-5 (1 = easy, 5 = unplayable)
    //   lane_strategy: String,    // "Play safe until 6, poke with Q at max range"
    //   key_abilities_to_track: String, // "Dodge his E (14s CD lvl 1), trade when it's down"
    //   items_to_adjust: String,  // "Rush Seeker's Armguard"
    //   personal_notes: String,   // "I always die to his level 2 all-in, need to respect it"
    // }

    // -- Teamfight role --
    teamfight_role: String,     // "Stay back, wait for enemy to use gap closers, then ult the clump"
    positioning_notes: String,  // "Stand near wall for escape route, never frontline"

    // -- Synergies & weaknesses --
    strong_with: Vec<String>,   // Champion names: ["Jarvan IV", "Amumu"]
    weak_against_comps: String, // "Hard engage comps that can gap-close past my zone"

    // -- Personal journal --
    lessons_learned: Vec<LessonEntry>,
    // LessonEntry { date: String, text: String }
    // e.g. { "2026-03-10", "Realized I need to save W for disengage, not poke" }

    // -- Metadata --
    last_updated: String,
    created_at: String,
}
```

#### Key design decisions

1. **Structured fields over free text** -- Power spikes, combos, and matchups each get their own repeatable sub-records rather than one big textarea. This enables filtering ("show me all my hard matchups"), sorting, and eventually cross-referencing with draft data.

2. **Matchup notes are personal** -- Unlike Mobalytics/U.GG which are crowd-sourced statistics, these are the player's own observations. The `personal_notes` field in each matchup is the differentiator.

3. **Comfort level replaces tier** -- The existing champion pool has a tier list (S/A/B/C). Champion notes add a `comfort_level` which is about personal mastery, not meta strength. Both can coexist.

4. **Lessons learned as a journal** -- Timestamped entries that accumulate over time. Players add entries after games where they learned something new. This replaces a single textarea with a chronological log.

5. **Identity tags enable draft integration** -- If a champion is tagged "scaling" and "teamfight," the draft planner could surface this when building a scaling teamfight comp. Future feature, but the schema supports it.

6. **Team-visible vs. private** -- Some fields (comfort_level, roles, identity_tags) could be visible to teammates for draft coordination. Matchup personal_notes and lessons_learned should be private by default.

#### Implementation approach

- New SurrealDB table `champion_note` with `DEFINE TABLE champion_note SCHEMALESS` (flexible sub-records)
- New page at `/champion-notes` or extend `/champion-pool` with a detail view per champion
- Sub-records (power_spikes, combos, matchups, lessons) stored as JSON arrays in the DB record
- Start with: comfort_level, matchups (difficulty + notes), and lessons_learned as MVP fields
- Add combos, power_spikes, teamfight_role in a second pass

---

## Sources

### R1 -- PrimeLeague
- [Prime League official site](https://www.primeleague.gg/en)
- [Freaks 4U Gaming license renewal (2025-2027)](https://www.freaks4u.de/news/149493-freaks-4u-gaming-announces-renewal-of-the-league-of-legends-esports-prime-league-operator-license-for-2025-2027)
- [Freaks 4U Gaming insolvency filing](https://esportsradar.gg/freaks-4u-gaming-files-for-insolvency/)
- [primebot_backend (unofficial Prime League bot)](https://github.com/random-rip/primebot_backend)
- [PrimeBot API docs](https://primebot.org/guides/api/)
- [Prime League 2026 Winter on Liquipedia](https://liquipedia.net/leagueoflegends/Prime_League/2026/Winter)

### R2 -- Draft Tools
- [ProComps.gg](https://procomps.gg/)
- [DraftGap (open source)](https://github.com/vigovlugt/draftgap)
- [DraftGap AI (Cloud9 coaching tool)](https://devpost.com/software/draftgap-ai-the-agentic-draft-strategist)
- [iTero drafting tool](https://www.itero.gg/drafting-tool)
- [iTero Draft Review Model article](https://medium.com/the-esports-analyst-club-by-itero-gaming/the-draft-review-model-lol-57aeef1de89a)
- [Drafter.lol](https://drafter.lol)
- [DraftVision](https://loldraftvision.fun/)
- [ScoutAhead.pro](https://scoutahead.pro/)
- [dr4ft.lol](https://dr4ft.lol/)
- [Fearlessdraft.net](https://www.fearlessdraft.net/)
- [ProComps + Prodraft/DraftLoL integration](https://procomps.gg/blog/prodraft-draftlol-integration.html)

### R3 -- Champion Learning Notes
- [Mobalytics champion pages](https://mobalytics.gg/lol/champions)
- [U.GG champion builds](https://u.gg/lol/champions)
- [OP.GG](https://op.gg/)
- [MOBAFire guide builder](https://www.mobafire.com/league-of-legends/build/creating-effective-league-of-legends-champion-guides-650046)
- [LoL Matchup Notes (oracle-notes)](https://oracle-notes.lovable.app/)
- [iTero AI Coach](https://www.itero.gg/)
- [Statup.gg coaching tools roundup](https://statup.gg/post/best-league-of-legends-apps-coaching-tools)
