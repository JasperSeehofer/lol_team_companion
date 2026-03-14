# Feature Landscape

**Domain:** Competitive League of Legends team preparation — cross-feature intelligence
**Researched:** 2026-03-14

---

## Context

This is a brownfield project. Auth, team management, drafts, tree drafts, champion pools, stats,
game plans, post-game reviews, opponent scouting, action items, and a team notebook all exist but
operate as isolated islands. This milestone connects them. The research question is: what does
"connected" mean for competitive LoL prep tools, and where is the bar set by the existing market?

**Competitive landscape surveyed:**
- Mobalytics — all-in-one coaching companion (pre-game, overlay, post-game loop)
- ProComps.gg — team-focused draft tool with champion pool integration
- iTero.gg — AI drafting coach with 500+ account statistics connected to draft
- DraftGap — draft tool with matchup-aware pick recommendations
- OP.GG — stats-first platform with esports-tier team rankings
- Meeko.ai — AI coach with session recaps and tilt detection
- HextechTools — dashboard-centric climb tracker with checklists

---

## Table Stakes

Features users expect. Missing = product feels incomplete or disconnected.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Ban recommendations from own stats | Every tool from Mobalytics to ProComps surfaces "based on your pool, consider banning X" — users expect their data to inform bans, not just generic meta | Medium | Requires joining champion pool tiers + opponent scouting profiles per-game |
| Draft pulls from team champion pools | ProComps requires it to function; users won't manually re-enter pools for each draft | Low | Pools already exist — linking them during draft is the gap |
| Post-game → action item surface | Mobalytics and Meeko both close the loop: post-game review generates to-dos | Low–Medium | Action items table exists; need to auto-create from post-game patterns |
| Empty states that guide next action | All mature tools (Mobalytics, ProComps) show contextual CTAs when data is absent, not blank pages | Low | Currently blank; should say "No drafts yet — create one to start prepping" |
| Loading skeletons / perceived performance | Industry standard; every polished web app uses them; absence signals unfinished product | Low | Leptos `Suspense` already supports fallbacks — extend with shape skeletons |
| Confirmation feedback on mutations | Save, delete, update — users need to know it worked; ProComps and Mobalytics both show toasts | Low | Some pages have it; not consistent across all mutations |
| Draft → game plan pipeline | ProComps shows pre-game strategy connected to draft locks; a planned draft feeding into a game plan is the most obvious connection | Medium | Draft IDs exist; game plan needs a `draft_id` FK and prefill logic |
| Contextual champion pool warnings in draft | When a player has no champions in their assigned lane's pool, surface a warning during draft planning | Medium | Requires joining draft slots → player roles → champion pool tiers |

---

## Differentiators

Features that set the product apart. Not expected by every user, but create genuine advantage.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Smart dashboard that surfaces "what's next" | No LoL team tool has a genuinely prioritised prep dashboard — they either show raw stats or require manual navigation. A dashboard showing "You have a game tomorrow — here's your opponent profile, open draft, and last action items" is novel | High | Requires heuristics: upcoming context, incomplete workflows, staleness detection |
| Stats-informed draft suggestions (own data, not just meta) | DraftGap and iTero use aggregate win rates; ProComps uses a team's champion pools. Using your team's *actual match history* (roles + per-champion win rates) to score draft options is rarer and more accurate | High | Joins `match_history` per-player per-champion with draft pick slots |
| Draft outcome correlation | Surfacing "your team wins 70% when you draft an engage-heavy comp" from past post-game reviews is something no tested tool does end-to-end | High | Requires tagging game plans / drafts with outcome, then aggregating |
| Opponent tendency highlighting during draft | ProComps notes it as an analyst workflow — showing opponent's historical ban/pick tendencies inline during draft planning goes beyond what any consumer tool provides | Medium | Opponent scouting profiles already exist; surface them inside draft planner |
| Post-game lesson recall in game plan creation | When creating a game plan for an opponent you've faced, surface relevant lessons from past post-game reviews for that opponent | Medium | Requires `opponent_id` linkage across post-game + game plan |
| Game day guided flow (linear prep sequence) | No tool in the market provides a linear "do these 4 things before your game" workflow that chains: opponent review → draft plan → game plan → post-game review. This is a unique structural advantage for team-oriented prep | High | Already partially built (game day checklist exists); needs cross-link routing |
| Win condition tracker across drafts | Tracking whether declared win conditions (from game plans) were achieved (from post-game reviews) over time creates compound improvement data unavailable elsewhere | High | Requires structured win condition fields and outcome tagging |
| Contextual "last time we faced this team" recall | When loading an opponent profile or starting a draft vs a known opponent, surface the most recent game plan and post-game review for that opponent | Low–Medium | Query by `opponent_id` across game plans + post-game reviews; display inline |

---

## Anti-Features

Features to explicitly NOT build in this milestone.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Real-time collaborative editing (WebSockets) | Adds significant infrastructure complexity (session broadcasting, conflict resolution) with marginal value — teams use async prep workflows, not simultaneous editing | Keep mutations server-function based; add `resource.refetch()` for freshness |
| AI-generated draft picks (LLM integration) | Mobalytics and iTero already do this with aggregate data pipelines; without a comparable data corpus, this is vaporware and not a differentiator for a team-local tool | Surface own-data win rates as the intelligence; skip LLM calls |
| Automated video analysis / VOD review | Requires video upload, storage, ML inference — a completely different product surface | Link post-game notes to timestamp markers manually if needed |
| Mobile-optimised layout redesign | Project constraint; competitive teams prep on desktop; mobile breakpoints come after intelligence | Keep desktop-first; don't add responsive rework to this milestone |
| Public leaderboards / social features | Shifts product from private team tool to public platform — different trust model and data exposure | Stay single-team scoped; multi-team is a later milestone |
| Riot API live in-game overlay | Requires Overwolf or desktop client integration; fundamentally different distribution model from a web app | Leave in-game layer to Mobalytics / Blitz; focus on pre/post game |
| Opponent stats pulled automatically via Riot API per draft | Rate limits make per-game automatic scouting expensive; also creates false sense of completeness without manual context | Keep scouting manual + structured; show API-backed profile stats where already fetched |

---

## Feature Dependencies

```
Champion pool data → Draft pool awareness (pool warnings, draft prefill from pools)
Champion pool data + Match stats → Ban recommendations (pool-aware + stats-aware)
Draft (completed) → Game plan (draft_id FK, prefill picks/bans/side)
Game plan (win conditions declared) → Post-game review (outcome tracking)
Post-game review (outcome + opponent_id) → Next game plan for same opponent (lesson recall)
Opponent scouting profile → Draft planner (tendency sidebar)
Post-game review (patterns) → Action items (auto-create from patterns)
All of the above → Smart dashboard (surfaces incomplete workflows, recent context)
```

Ordering implication: pool awareness and draft→game plan pipeline are the lowest-hanging fruit
(existing data, low new code). Smart dashboard and win condition tracking are highest leverage
but depend on the lower-level links being in place first.

---

## MVP Recommendation for This Milestone

This is not a greenfield MVP — it is a cross-feature intelligence layer on a complete app. Prioritise:

1. **Draft → game plan pipeline** — most-requested connection; single FK + prefill = high impact, low effort
2. **Pool warnings in draft** — obvious quality-of-life; pools already exist; one join query
3. **Ban recommendations from own stats** — ProComps and Mobalytics both have this; users notice its absence
4. **Empty states with contextual CTAs** — fast wins; makes the app feel finished vs alpha
5. **Loading skeletons on all Suspense fallbacks** — perceived performance; low effort with Leptos patterns
6. **Post-game → action item auto-creation** — closes the improvement loop; action items table exists
7. **Opponent tendency sidebar in draft** — scouting profiles exist; moderate join work
8. **Consistent mutation feedback (toasts)** — polish; currently inconsistent
9. **Smart dashboard** — highest value but most design work; build last when connections are live
10. **Win condition tracker** — high value for longitudinal data; needs 2–3 other connections first

Defer:
- Draft outcome correlation (needs 10+ tagged game outcomes to be meaningful)
- Lesson recall in game plan creation (useful but non-urgent; can be Phase 2)
- Full game day guided flow as a wizard (existing checklist is sufficient; wizard is scope creep)

---

## Sources

- [Mobalytics: beginners guide to features](https://mobalytics.gg/blog/lol-mobalytics-beginners-guide/) — MEDIUM confidence (official doc)
- [ProComps.gg team drafting features](https://procomps.gg/) — MEDIUM confidence (official site)
- [ProComps Hub: mega app update 2024](https://hub.procomps.gg/2024/09/mega-app-update/) — MEDIUM confidence (official changelog)
- [iTero drafting tool](https://www.itero.gg/drafting-tool) — MEDIUM confidence (official docs)
- [iTero: best companion app 2026 comparison](https://www.itero.gg/articles/what-is-the-best-league-of-legends-companion-app-in-2025) — LOW confidence (self-published)
- [DraftGap GitHub README](https://github.com/vigovlugt/draftgap/blob/main/README.md) — HIGH confidence (source)
- [Meeko.ai overview](https://meeko.ai/) — MEDIUM confidence (official site)
- [Statup.gg: top 10 coaching tools 2025](https://statup.gg/post/best-league-of-legends-apps-coaching-tools) — LOW confidence (third-party review)
- [Esportsheaven: art of the draft from a coach's perspective](https://www.esportsheaven.com/features/the-art-of-the-draft-an-in-depth-look-into-drafting-from-a-coachs-perspective/) — MEDIUM confidence (editorial)
- [BPCoach academic paper on hero drafting visual analytics](https://dl.acm.org/doi/10.1145/3637303) — HIGH confidence (peer-reviewed)
