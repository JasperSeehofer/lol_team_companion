# Assessment: suggestion chips on the game plan (wish fy9o9cmh)

Lane 1, M141 group B, 2026-10-16. Decision: **proposal only, not implemented in R1.** Reason: it does not fit
"small" on the existing data. The ranking exists, but the plan has nowhere to keep an accepted or dismissed
suggestion, and the evaluation answers a different question from the one a plan needs answered.

## What the owner asked for (intent, from the report)

The Draft Graph already rates synergy and counters. The owner wants that evaluation to put real suggestions
into the Game Plan, each one accepted or dismissed by the team. The system should stay non-intrusive: every
decision is still made by a person, and the person learns why a suggestion is good. It must not turn into an AI
that plays the game for the team.

## What exists today (file:line)

- `server::graph::fit::rank` (`src/server/graph/fit.rs:406`) takes a `DraftState` (`src/models/graph.rs:426`)
  and ranks **candidates for the next pick**. Each candidate carries `reasons` (`Reason`, models/graph.rs:539,
  with a `ReasonSource`: a trait rule, a team note on a pair, the team's games, a scouted pool) and the
  `CompProfile` it adds (models/graph.rs:591). That is the right material, but it is built for "who do we pick
  next", not "what does this finished matchup mean for our plan".
- `CompProfile::sum` already turns five locked picks into engage, frontline, peel, wave clear, cc, the damage
  mix and the scaling curve (`plan()`). The same holds for the enemy side (`GraphResult.enemy`).
- `server::graph::rules` (`rules.rs`, 922 lines) holds the trait rules and `PairRelations` (team notes on pairs:
  synergy and counter, written by the team itself).
- The game plan already has a deterministic **Starter outline** (`generate_template`, used at
  `src/pages/game_plan.rs:~1170`). It fills only empty fields and says "rewrite it for your team". This is
  the closest thing to a suggestion today, but it is a template, not a reasoned suggestion.
- The `game_plan` table (`schema.surql:127-144`) has no field for suggestions, accepted or dismissed.

## Smallest valuable version (R2 proposal)

**"What the picks say": up to three suggestion chips under the Matchup section, once both sides have five
picks.**

1. Server: a team-scoped `plan_suggestions(plan_id)` builds a `DraftState` from the plan's ten picks, computes
   both `CompProfile`s and checks a short, fixed list of comparisons, each tied to a plan field:
   - Their engage ≥ 4 and our peel ≤ 2 → *How we win / teamfight*: "Their comp has strong engage (Leona,
     Malphite); plan who answers it." Source: trait table.
   - Our curve Late, theirs Early → *Early game*: "Their picks peak early; plan the first 15 minutes for
     safety." Source: scaling traits.
   - A team note on a pair (synergy or counter) between two picked champions → the matching lane field,
     **quoting the team's own note** (`note` style). Source: `team_relation`.
   - Damage mix ≥ 4 of one type → *Notes*: "Mostly physical damage: they can stack armour." Source: traits.
   Each chip says **why** in one line and names its source (trait table, your note, your games). That is the
   "learn why" the owner asked for. No chip proposes picks or a playstyle.
2. UI: a chip is a `Callout`-sized row: text, source, **Add to plan** (appends the sentence to the named field,
   which the person can then edit) and **Dismiss**. Nothing is written until someone presses Add.
3. Storage: `game_plan.dismissed_suggestions: option<array<string>>` (suggestion keys such as
   `engage-vs-peel`), so a dismissed chip stays gone for this plan. That is a new optional field
   (`schema.surql`, `option<T>`, no migration), the plan's save path, and the export.
   Accepting needs no storage: the text lands in a field that already exists.

## Data it would use

The plan's ten picks (already stored), the trait table (`champion_trait`, global read), the team's pair notes
(`team_relation`, team-scoped), and optionally the team's games (`graph::evidence::for_team`) for "you are 1-4
when they pick X". No Riot calls, and no new external data.

## Size

About 1.5 to 2 days of focused work, so **medium, not small**:
- the server function, the comparison rules, and their unit tests (about 250 lines); the rules are new code,
  because `fit::rank`'s reasons are per candidate and cannot be reused as they are;
- the UI chips with Add and Dismiss, plus the field mapping (about 150 lines);
- the new optional `game_plan` field through `db.rs` save and load, the model, the account export check and a
  cross-tenant integration case;
- an e2e spec and gallery or DESIGN wording for a "suggestion" pattern. DESIGN 6.11 says guidance is never
  modal and is mutable; a suggestion is a new pattern, close to `FirstUseHint`, that the atelier should
  review first.

## Risks and guardrails

- **Honesty rule:** a chip must cite real data (a trait value, a note, the team's games). It must never be a
  generic tip. With default trait rows (`GraphNotes.default_rows`), say so.
- **Non-intrusive:** at most three chips, collapsed behind "What the picks say (3)" once the plan has text
  in that field, never on load before the picks are complete, and muteable like guidance.
- **Not an AI playstyle:** the chips describe the matchup and point at a plan field. They never write a
  strategy on their own. Riot's policy has no concern here: no in-game overlay, no live-game data.

## Recommendation

Put it on the R2 list as an agent proposal: "Suggestions from the picks (smallest version: three cited chips,
Add or Dismiss)". The owner reacts on the publish card; implement after the atelier has a recipe card for the
chip.
