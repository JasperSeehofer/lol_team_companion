# Feature Research

**Domain:** Solo queue player tracking and match intelligence for League of Legends
**Researched:** 2026-03-25
**Confidence:** MEDIUM (competitive landscape well-documented; gap analysis from community spreadsheets and app reviews)

---

## Context

This is a **brownfield solo mode addition** to a team-scoped app. The existing app has team management,
drafts, game plans, post-game reviews, champion pools, opponent scouting, and action items — all
team-scoped. v1.2 opens the app to solo players who want to track personal improvement, ranked climb,
and personal learnings without needing a team.

**Competitive landscape surveyed for this research:**
- **OP.GG** — stats-first tracker; match history, champion stats, KDA, LP history, win streaks
- **U.GG** — build-focused; summoner profiles showing trend analysis, matchup data, clean UI
- **Porofessor.gg** — in-game overlay + pre/post game analysis; gold gaps, CS charts, matchup win rates
- **Mobalytics** — GPI skill profiling (8 axes), personal LP goal-setting, ranked climb graphs, in-game overlay
- **Blitz.gg** — auto rune import, live match analysis, match review
- **tracker.gg** — LP climb tracking, Tracker Score, real-time performance
- **Community spreadsheets** — multiple GitHub projects (5,000+ downloads) for match logging, matchup notes, post-game journaling

---

## Table Stakes

Features solo queue players assume any tracking tool will have. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Match history list (per-player) | Every tracker from OP.GG to tracker.gg shows this as the entry point. Users expect to see their last 20 games with champion, KDA, CS, result | LOW | Riot match-v5 API already called in `riot.rs`; basic `MatchData` struct already populated |
| Win/loss per-game indicator | Baseline stat; missing it makes history unreadable | LOW | `MatchData.win` already returned from Riot API |
| Game duration and CS/min | OP.GG, U.GG, Blitz all show this. CS/min = 1-second derived stat, universally expected | LOW | `game_duration` and `cs` already in `MatchData`; CS/min = `cs / (duration / 60.0)` |
| Champion KDA aggregated over time | U.GG summoner profiles surface per-champion KDA trends. Users check this to know their "real" performance on a champion beyond last session | MEDIUM | Requires aggregating existing match history records in DB by champion |
| Ranked LP and current tier/rank | Every tracker shows current rank. Users expect to see Iron → Challenger progression | MEDIUM | Riot League API (`league-v4`) provides rank/LP/tier; not yet called in `riot.rs` |
| Solo vs flex queue filter | Community specifically requests this (Facebook LoL group discussion found in research). OP.GG lumps them together; users want to see solo-only metrics | LOW | `queue_id` already in `MatchData`; ranked solo = queue 420 |
| Per-match detail view (all 10 players) | Mobalytics, Blitz, and Porofessor all offer this. Users click a match to see full scoreboard: items, damage, gold, vision — not just their own stats | HIGH | Requires fetching all 10 participants from match-v5; not currently done (only fetches the tracked player) |
| Personal champion win rate | OP.GG shows win rate per champion in the summoner profile. Users consider this table stakes when evaluating what to play | MEDIUM | Derivable from aggregating match history by champion |

---

## Differentiators

Features that set this app apart from existing tools. Not assumed, but create genuine value.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Personal learnings journal (tagged by champion/matchup) | No public tool has this. Community builds Google Sheets and Excel spreadsheets to fill the gap (5,000+ downloads for lol-spreadsheet on GitHub). In-app journaling tied to matches closes this gap entirely | MEDIUM | New `personal_learning` table; tag by champion + matchup; one-click flow from match detail → add learning |
| Ranked LP history graph (personal climb tracker) | Mobalytics has LP progression graphs behind a login/subscription wall. tracker.gg has a basic version. Neither is private, self-hosted, or integrated with personal notes. An LP history graph + milestone markers ("hit Plat today") is meaningfully better in context of personal goals | MEDIUM | Requires `lp_history` table; populate on manual entry or after each tracked match; Leptos SVG chart (reuse `tree_graph.rs` SVG patterns) |
| Goal setting with progress tracking | Mobalytics lets you set a rank goal and shows probability. But goals tied to *specific* metrics (CS/min target, 0 deaths in laning phase) that auto-track against match history are absent from any tool | MEDIUM | `solo_goal` table with target type (rank, cs_per_min, death_threshold), target value, and auto-progress query |
| Matchup notes surfaced in draft | Personal notes on a specific matchup (e.g. "vs Darius top: rush Plated Steelcaps, don't fight pre-6") shown as a tooltip in the draft picker when that champion is hovered. No competitor connects personal game experience back into the draft tool | MEDIUM | `matchup_note` table (champion + opponent_champion + notes); surface in `champion_picker.rs` and `champion_autocomplete.rs` on hover |
| Champion performance trends over time | OP.GG shows rolling win rate per champion. No tool shows CS/min trend, vision score trend, or death rate trend broken out by champion over configurable windows (last 20, last 50, this patch) | MEDIUM | Aggregate from existing match history table; configurable window via server fn parameter |
| Match → learning one-click flow | No tool lets you click a bad game and immediately add a personal learning note tagged to it. Community workaround is switching to a spreadsheet mid-session | LOW | Button on match detail view → pre-populates learning form with champion + opponent + match ID |
| Practice list with auto-tracked progress | Players manually write "practice Thresh support roaming" in Notion or a doc. Auto-surfacing "you played 8 games on Thresh this week, CS/min improved from 4.2 to 5.1" against a named goal is absent from every tested tool | HIGH | `practice_item` table; auto-query match history for champion played; derive metric delta over time window |
| Solo/team context toggle (dashboard mode) | The existing app is entirely team-scoped. A solo player without a team has nothing. A context toggle between "team mode" and "solo mode" lets the app serve both audiences with different dashboard focus | MEDIUM | Toggle stored in user preferences; solo dashboard shows LP tracker, champion trends, recent learnings; team dashboard shows current team prep pipeline |
| Post-game learning from match (solo loop) | Post-game review currently requires a game plan and is team-oriented. A lightweight "how did this game go?" reflection form tied directly to a match (not requiring a team or game plan) closes the improvement loop for solo players | LOW | Reuse `post_game_learning` concept; simplified form: result, champion, what went well, what to improve, linked match_id |
| Champion pool gap detection from solo history | "You've played 47 ranked games this season, 82% on 3 champions. Your champion pool has no carry options for jungle. Consider adding..." — no tool combines personal play data with champion pool tiers to surface preparation gaps | HIGH | Cross-join match history champion frequency with `champion_pool` tiers; surface on pool page + solo dashboard |

---

## Anti-Features

Features that seem like good ideas for solo mode but should be explicitly avoided.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Live in-game overlay | Requires Overwolf or desktop client; fundamentally different distribution and trust model; Blitz, Porofessor, Mobalytics all do it better with native integration | Focus on pre-game and post-game; the web app has no in-game slot |
| AI-generated improvement tips | Without a coaching-grade data corpus and ML pipeline, LLM tips are generic ("farm more, die less"). Mobalytics has GPI backed by real ML. Without that, this is vaporware that erodes trust | Surface own-data insights: "your CS/min on Irelia improved from 5.1 to 6.4 this month" — specific and trustworthy |
| Automated opponent scouting per solo queue game | Rate limits make scanning all 9 opponents per game expensive. Players dodging on scouting data is a pattern that creates frustration and longer queue times | Keep opponent scouting manual and profile-based; don't auto-fetch for every queue pop |
| Leaderboards / public profiles | Shifts product from personal improvement tool to social comparison — different privacy model and user anxiety. OP.GG handles public stats; the value here is private journaling | Stay private and team-scoped; no public "compare to other users" features |
| VOD / replay review integration | Requires replay file upload, storage, and video serving infrastructure. The product value is notes + data, not video | Let players link external replay links as a text field in learnings; don't host video |
| Full GPI-style ML skill profiling | Mobalytics built GPI with data science infrastructure and years of data. Faking it with simple averages labeled as "skill scores" will be less accurate and misleading | Show raw metrics (CS/min, vision score, death rate, kill participation) as trends over time; let players interpret them |
| Real-time LP polling | Polling the Riot API constantly to auto-update LP creates rate-limit pressure and adds noise; real value is in the pattern over time | Let users manually log LP after games or trigger a sync; batch-fetch on demand |

---

## Feature Dependencies

```
Riot API (league-v4 rank endpoint)
    └──required by──> LP tracker / ranked climb chart
    └──required by──> Current rank display on solo dashboard

Match history list (existing)
    └──extended by──> Per-match detail view (all 10 participants from match-v5)
    └──aggregated into──> Champion performance trends (win rate, CS/min, KDA per champion)
    └──aggregated into──> Practice list auto-progress tracking

Personal learnings journal (new table)
    └──linked to──> Match history (match_id FK)
    └──linked to──> Champion + opponent champion (for matchup surface)
    └──surfaced in──> Draft picker (matchup notes on hover)
    └──surfaced in──> Solo dashboard (recent learnings panel)

Goal setting (new table)
    └──requires──> Champion performance trends (to show progress toward metric goals)
    └──requires──> LP tracker (to show progress toward rank goals)

Solo/team context toggle
    └──requires──> Solo dashboard (what to show when in solo mode)
    └──requires nothing from team (team features remain unchanged)

Practice list (new table)
    └──requires──> Match history aggregation (to auto-derive progress)
    └──enhanced by──> Champion performance trends (shows metric delta)

Champion pool gap detection
    └──requires──> Champion performance trends (frequency + win rate from history)
    └──requires──> Champion pool tiers (existing feature)
```

### Dependency Notes

- **Per-match detail view requires no new DB layer** — it extends the Riot API fetch in `riot.rs` to capture all 10 participants and additional fields (items, gold, damage breakdown) that are already available in match-v5 participants but not currently stored.
- **Matchup notes require the draft picker to know the hovered champion** — `champion_picker.rs` already renders champion tiles; adding an on-hover tooltip that queries matchup notes is a contained change.
- **LP tracker is an independent feature** — it does not depend on any existing feature except the user's linked Riot account (PUUID, which is already stored via `profile.rs`).
- **Goal setting is valuable without LP tracker** but works better with it. Build goals first (metric-based), then add rank goals once LP data flows.

---

## MVP Definition

This milestone has a clear progression: infrastructure first, personal tools second, cross-feature connections third.

### Launch With (v1.2 core)

- [ ] Solo/team context toggle + solo dashboard shell — without this, solo players have no entry point
- [ ] Match detail view (all 10 participants, items, damage, gold) — most requested single feature; extends existing Riot API integration
- [ ] Personal learnings journal (post-game reflection, tagged by champion/matchup) — closes the improvement loop; fills the biggest gap in existing tools; minimal DB schema
- [ ] Champion performance trends (win rate, KDA, CS/min per champion over last N games) — derives from match history already stored; medium effort, high value
- [ ] Ranked LP history entry + simple graph — manual LP entry is acceptable for v1.2; auto-fetch from Riot API is a nice-to-have

### Add After Validation (v1.2 extensions)

- [ ] Goal setting (rank target, CS/min, death threshold) — needs performance trends live first
- [ ] Matchup notes surfaced in draft picker — needs learnings journal live first; adds cross-feature connection
- [ ] Practice list with manual entry — auto-progress can come later; manual list first
- [ ] Match → learning one-click flow — quick win once both match detail and journal exist

### Future Consideration (v1.3+)

- [ ] Practice list auto-progress tracking — requires reliable match history aggregation patterns to be stable
- [ ] Champion pool gap detection from solo history — high value but complex cross-join; defer until pool + match history integration is proven
- [ ] Auto LP sync via Riot API (league-v4 polling) — rate limit management needed; manual entry is good enough for v1.2
- [ ] Match timeline (objectives, events per-minute) — available in Riot match-v5 timeline endpoint; adds depth to match detail but high complexity; no competitor surfaces it well

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Match detail view (all 10 participants) | HIGH | MEDIUM | P1 |
| Personal learnings journal | HIGH | LOW | P1 |
| Solo dashboard + context toggle | HIGH | MEDIUM | P1 |
| Champion performance trends | HIGH | MEDIUM | P1 |
| LP history graph | HIGH | MEDIUM | P1 |
| Match → learning one-click | HIGH | LOW | P1 |
| Goal setting (metric targets) | MEDIUM | MEDIUM | P2 |
| Matchup notes surfaced in draft | MEDIUM | LOW | P2 |
| Practice list (manual) | MEDIUM | LOW | P2 |
| Practice list auto-progress | MEDIUM | HIGH | P3 |
| Champion pool gap detection | HIGH | HIGH | P3 |
| Match timeline (objectives/events) | MEDIUM | HIGH | P3 |

**Priority key:**
- P1: Must have for v1.2 milestone to deliver on solo mode promise
- P2: Should have; add before milestone closes
- P3: Nice to have; deferred to v1.3+

---

## Competitor Feature Analysis

| Feature | OP.GG | U.GG | Mobalytics | Porofessor | Our approach |
|---------|-------|------|------------|------------|--------------|
| Match history | YES — public, no login | YES — public | YES — requires account | YES — via app | YES — private, account-scoped |
| LP history / climb graph | Basic history | No | YES — behind premium | No | YES — private LP journal + graph |
| Champion performance trends | Win rate only | Win rate, pick rate | Win rate + GPI axes | Win rate | CS/min trend, KDA trend, win rate per configurable window |
| Personal notes / journal | NO | NO | NO | NO | YES — core differentiator |
| Matchup notes | NO | Shows meta matchup data only | NO | Shows meta matchup win rates | YES — personal notes surfaced in draft |
| Goal setting | NO | NO | YES — rank goal slider | NO | YES — rank + metric goals |
| Practice list | NO | NO | NO | NO | YES — manual with auto-progress |
| Per-match full detail | YES — all 10 players | YES — all 10 players | YES — with GPI per player | YES — gold/CS charts | YES — planned for v1.2 |
| Match timeline (objectives) | NO | NO | Partial — key events | Partial — in-game only | Deferred to v1.3 |
| Solo/team context | NO (solo only) | NO (solo only) | NO (solo only) | NO (solo only) | YES — unique; bridges both modes |

**Key gap this app fills:** Personal journaling, matchup notes, and the cross-feature connection between personal history → draft decisions. No competitor connects a player's personal game learnings back into the draft tool. This is the unique value proposition.

---

## Implementation Notes for v1.2

### Riot API Extensions Needed

The current `riot.rs` only fetches a single participant's data per match. For match detail view:

1. **All 10 participants** — iterate all `m.info.participants` instead of filtering by PUUID; store extra fields: `items` (0–6 slots), `gold_earned`, `damage_breakdown`, `vision_wards_bought`, `position` (role).
2. **Ranked tier/LP** — call `league-v4` endpoint `get_league_entries_for_summoner` using `summoner-v4` lookup by PUUID. Returns current tier, rank, LP, wins, losses, mini-series if in promo.
3. **Match timeline** — `match-v5 /timeline/{matchId}` is available in riven (`get_timeline`). Contains per-minute snapshots (gold, CS, XP, position) and event log (kills, objectives, items). Deferred to v1.3 due to complexity.

### New DB Tables Required

```sql
-- Personal improvement learnings (solo loop)
DEFINE TABLE IF NOT EXISTS personal_learning SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS user_id        ON personal_learning TYPE record(user);
DEFINE FIELD IF NOT EXISTS match_id       ON personal_learning TYPE option<string>; -- Riot match ID
DEFINE FIELD IF NOT EXISTS champion       ON personal_learning TYPE option<string>;
DEFINE FIELD IF NOT EXISTS opponent_champion ON personal_learning TYPE option<string>;
DEFINE FIELD IF NOT EXISTS content        ON personal_learning TYPE string;
DEFINE FIELD IF NOT EXISTS tags           ON personal_learning TYPE array<string>;
DEFINE FIELD IF NOT EXISTS created_at     ON personal_learning TYPE datetime;

-- LP history entries
DEFINE TABLE IF NOT EXISTS lp_entry SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS user_id        ON lp_entry TYPE record(user);
DEFINE FIELD IF NOT EXISTS tier           ON lp_entry TYPE string; -- "GOLD"
DEFINE FIELD IF NOT EXISTS rank_division  ON lp_entry TYPE string; -- "II"
DEFINE FIELD IF NOT EXISTS lp             ON lp_entry TYPE int;
DEFINE FIELD IF NOT EXISTS recorded_at   ON lp_entry TYPE datetime;
DEFINE FIELD IF NOT EXISTS note          ON lp_entry TYPE option<string>;

-- Solo goals
DEFINE TABLE IF NOT EXISTS solo_goal SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS user_id        ON solo_goal TYPE record(user);
DEFINE FIELD IF NOT EXISTS goal_type      ON solo_goal TYPE string; -- "rank" | "cs_per_min" | "death_limit" | "kda"
DEFINE FIELD IF NOT EXISTS target_value   ON solo_goal TYPE string; -- "PLATINUM II" | "7.0" | "3" | "3.0"
DEFINE FIELD IF NOT EXISTS champion       ON solo_goal TYPE option<string>; -- null = all champs
DEFINE FIELD IF NOT EXISTS active         ON solo_goal TYPE bool;
DEFINE FIELD IF NOT EXISTS created_at     ON solo_goal TYPE datetime;

-- Practice list
DEFINE TABLE IF NOT EXISTS practice_item SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS user_id        ON practice_item TYPE record(user);
DEFINE FIELD IF NOT EXISTS description    ON practice_item TYPE string;
DEFINE FIELD IF NOT EXISTS champion       ON practice_item TYPE option<string>;
DEFINE FIELD IF NOT EXISTS metric_type    ON practice_item TYPE option<string>; -- "cs_per_min" | "deaths" | "vision"
DEFINE FIELD IF NOT EXISTS target_value   ON practice_item TYPE option<float>;
DEFINE FIELD IF NOT EXISTS completed      ON practice_item TYPE bool;
DEFINE FIELD IF NOT EXISTS created_at     ON practice_item TYPE datetime;

-- Matchup notes (personal, not meta — distinct from opponent scouting)
DEFINE TABLE IF NOT EXISTS matchup_note SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS user_id          ON matchup_note TYPE record(user);
DEFINE FIELD IF NOT EXISTS champion         ON matchup_note TYPE string; -- the champion you play
DEFINE FIELD IF NOT EXISTS opponent_champion ON matchup_note TYPE string; -- the opponent you face
DEFINE FIELD IF NOT EXISTS content          ON matchup_note TYPE string;
DEFINE FIELD IF NOT EXISTS updated_at       ON matchup_note TYPE datetime;
```

### Existing Features to Extend

- `src/server/riot.rs` — extend `MatchData` struct + add `fetch_full_match` for all-participant data; add `fetch_ranked_info` for LP/tier
- `src/pages/stats.rs` — add per-match detail view routing; add champion trend aggregation server fn
- `src/pages/profile.rs` — add LP entry widget; show current rank from Riot API
- `src/components/champion_picker.rs` — add matchup note tooltip on hover
- `src/components/champion_autocomplete.rs` — same tooltip pattern

---

## Sources

- [OP.GG features overview](https://happysmurf.com/blog/what-is-opgg/) — MEDIUM confidence (third-party review)
- [Mobalytics GPI](https://mobalytics.gg/gpi/) — HIGH confidence (official)
- [Mobalytics LP goal tracking](https://mobalytics.gg/blog/how-to-track-lp-by-game) — HIGH confidence (official)
- [Porofessor features overview](https://wecoach.gg/blog/article/all-you-need-to-know-about-porofessorgg-a-complete-guide) — MEDIUM confidence
- [iTero companion app comparison 2025](https://www.itero.gg/articles/what-is-the-best-league-of-legends-companion-app-in-2025) — LOW confidence (self-published review)
- [tracker.gg LoL features](https://tracker.gg/lol/articles/trns-league-stats-tracker-can-help-you-climb-ranks-in-lol-and-heres-why) — MEDIUM confidence
- [Excel Match History Analyzer (5k+ downloads)](https://github.com/ikhebgeenaccount/Excel-Match-History-Analyzer) — HIGH confidence (community evidence of gap)
- [LoL Game Review Spreadsheet](https://github.com/ForeverTalone/Game-Review-Spreadsheet) — HIGH confidence (community evidence of gap)
- [Mobalytics: how to track stats to improve](https://mobalytics.gg/blog/how-to-track-your-stats-to-improve-in-league-of-legends/) — MEDIUM confidence
- [Riot match-v5 API overview](https://riot-watcher.readthedocs.io/en/latest/riotwatcher/LeagueOfLegends/MatchApiV5.html) — HIGH confidence (official)
- [Riven Rust crate](https://docs.rs/riven/latest/riven/) — HIGH confidence (official crate docs)
- [2026 ranked system changes (LP protection, Aegis of Valor)](https://esports.gg/news/league-of-legends/lol-ranked-system-2026/) — MEDIUM confidence
- [Community spreadsheet for self-improvement (EUW boards)](https://boards.euw.leagueoflegends.com/en/c/off-topic-en/iEx9yYY6-fancy-spreadsheet-for-self-improvement) — MEDIUM confidence

---

*Feature research for: Solo queue tracking and match intelligence (v1.2 milestone)*
*Researched: 2026-03-25*
