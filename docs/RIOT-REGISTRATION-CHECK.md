# Riot registration check

Team Companion is already registered on the Riot Developer Portal. This page describes what the code does today (branch state at M8, October 2026) so the owner can compare it with the registered product description and update the registration where it differs. It replaces a fresh application. Nothing here has been sent to Riot.

How to use it: open the product on the portal next to this page and walk the sections in order. Every place where the registration says something else is either a registration update or a question for the maintainer.

## 1. Product, in the words the code supports

- **What it is:** a private coaching workspace for one amateur or semi-pro team (players, substitutes, coach): scouting an opponent, planning the draft (including a branching draft tree and the Draft Graph, which explains why picks fit from a curated trait table and the team's own notes), the game plan, the post-game review, action items and lessons.
- **Who can use it:** closed beta, registration only with an invite code. One team per account. No public pages with game data except share links a team creates on purpose (one plan, review or player card, expiring after 1, 7 or 30 days, revocable; a player can keep their card out of share links).
- **What it is not:** no public stats site, no leaderboard, no public player search, no AI pick bot, no live-game or champion-select assistance, no betting.
- **Name:** "Team Companion" (owner's decision, 12 October 2026). It contains neither "League of Legends", "LoL" nor "Riot", and no Riot IP. Public address: `https://app.team-companion.eu`.
- **Money:** using Team Companion costs nothing; support is voluntary. The support link stays off at launch (`SUPPORT_URL` unset).
  [OWNER: compare with the registration's monetisation answer. The code has no paid features; a support link appears on /transparency only once the owner sets `SUPPORT_URL` and approves the money texts on /admin/texts.]

## 2. Riot API endpoints the code calls

All calls go through one server-side client (`src/server/riot.rs`, riven 2.x). The key lives only in the server environment (`RIOT_API_KEY`); the browser never contacts Riot.

| API | Method (riven) | Endpoint | Used for | Requests | Where |
|---|---|---|---|---|---|
| ACCOUNT-V1 | `account_v1().get_by_riot_id` | `/riot/account/v1/accounts/by-riot-id/{gameName}/{tagLine}` | Link a player's own Riot ID on the profile; look up an opponent's player for a dossier | 1 per lookup | `profile.rs`, `opponents.rs` |
| MATCH-V5 | `match_v5().get_match_ids_by_puuid` (count 20, optional queue) | `/lol/match/v5/matches/by-puuid/{puuid}/ids` | Recent games of the team's own players (also offered when a review is started); recent games of a scouted opponent | 1 per sync | `me.rs`, `stats.rs`, `post_game.rs`, `opponents.rs` |
| MATCH-V5 | `match_v5().get_match` | `/lol/match/v5/matches/{matchId}` | Stats of each listed game; the full scoreboard of one game | up to 20 per sync, 1 per opened game | same, `match_detail.rs` |
| MATCH-V5 | `match_v5().get_timeline` | `/lol/match/v5/matches/{matchId}/timeline` | Kills, objectives and structures for the review of one game (Sand Table frames: storage only, no UI yet) | 1 per opened game | `match_detail.rs` |
| LEAGUE-V4 | `league_v4().get_league_entries_by_puuid` | `/lol/league/v4/entries/by-puuid/{puuid}` | Ranked standing of the signed-in player | 1 per sync | `me.rs` |
| CHAMPION-MASTERY-V4 | `champion_mastery_v4().get_all_champion_masteries_by_puuid` | `/lol/champion-mastery/v4/champion-masteries/by-puuid/{puuid}` | Champion pool of a scouted opponent's player | 1 per lookup | `opponents.rs` |

Not used: SUMMONER-V4, SPECTATOR-V5, CLASH, TOURNAMENT, Riot Sign-On. Data Dragon (patch versions, champion list, champion and item images) is fetched by the server once per patch and served from Team Companion's own domain.

Request sizes: a player sync is 1 + 20 match requests (+ 1 ranked); a team sync about five times that; an opponent lookup 1 + 20 + 1 (mastery); opening a game 2 (match + timeline), cached afterwards.

## 3. Rate budget

- Every Riot call is started by a person (sync, link, lookup, opening a game). There is no crawler and no scheduled polling.
- Per client IP: `Bucket::Riot`, 30 requests then one every 2 s. Per user: one player sync a minute; per team: one team sync a minute.
- Process-wide budget (`RiotBudget`) under the application limit: defaults 20 per second and 50 per minute with a two-minute burst (development key). For the production key the operator sets `RIOT_GLOBAL_BUDGET_PER_SEC` / `_PER_MIN` (the template suggests 45 / 2400 for limits of 500 per 10 s and 30,000 per 10 min; see `deploy/env.production.example`). A job checks the headroom first and stops with "Riot is busy, try again in N s" instead of stalling. riven also follows Riot's rate-limit headers.
- Expected volume in the closed beta: 5 to 20 teams of about 6 accounts; a few thousand requests on busy days, peaks of a few hundred per minute. [OWNER: compare with the volume stated in the registration.]

## 4. Data stored, and for how long

| Data | Stored as | Kept |
|---|---|---|
| Linked player: Riot ID, PUUID, region | `user` | until the account is deleted (or relinked) |
| Synced games of a linked player: champion, K/D/A, CS, vision, damage, result, game end | `match`, `player_match` | per player: until the account is deleted; the `match` row (id, queue, duration) is shared and stays while any player's synced games use it, then is deleted 12 months after the game |
| Opened games: all ten participants' public match data (names, PUUIDs, champions, items, stats) and the timeline events | `match_detail`, `match_timeline_event` | shared cache, **deleted 12 months after it was fetched** (nightly task, `MATCH_CACHE_RETENTION_DAYS=365`) |
| Ranked snapshots of a linked player | `ranked_snapshot` | until the account is deleted |
| Scouted opponents: player names, Riot IDs, PUUIDs, recent champions, mastery and role data | `opponent_player` | until the team deletes the dossier or the team is deleted |
| Sand Table replays (positions and events of all ten players, with PUUIDs) | `replay*` | until the team deletes it (no import UI yet) |
| Backups of the whole database | encrypted (rclone crypt) on a Hetzner Storage Box | 30 days |

Retention of the shared match caches (owner's decision, 12 October 2026): **12 months**. A task in the server deletes cached games (`match_detail` with their `match_timeline_event` rows) 12 months after they were fetched, a minute after every start and then every 24 hours, in batches of 50 games, and logs the counts; `match` rows that no player's synced games use any more go 12 months after the game (a row without a recorded end time stays). The limit is the `MATCH_CACHE_RETENTION_DAYS` setting (default 365) and the privacy page states it. A player's own synced games and ranked snapshots are kept until the account is deleted, and scouting dossiers until the team deletes them. [OWNER: if the registration states a retention period, make it match.]

Account holders can download everything about themselves and delete their account on the profile (M8); deletion removes their Riot link, synced games and ranked snapshots at once and from backups within 30 days.

## 5. Policies the code follows

- **Disclaimer:** every page footer carries Riot's legal boilerplate with the product name (`components/footer.rs`, `RIOT_DISCLAIMER`; a unit test keeps the exact wording).
- **Privacy and terms:** `/legal/datenschutz` lists what is stored, the one session cookie, the logs, retention, third parties and rights; `/legal/terms` has the rules of use. The operator's facts are filled in (no `[OWNER: ...]` left); the privacy page's legal bases and the terms' liability and law clauses are drafts pending the operator's adviser.
- **Scouting:** only public match history, looked up by the Riot IDs a team enters for an upcoming opponent; visible only to that team.
- **Isolation:** every server function derives the team from the session; cross-tenant tests in `tests/integration/security.rs` and `e2e/tests/cross-tenant.spec.ts`.
- **Game integrity:** nothing during a live game or champion select beyond what the team prepared before.
- **Branding:** no Riot trademarks or game IP in the product name or visuals.

## 6. Verification file

`public/riot.txt` is served at `https://app.team-companion.eu/riot.txt` (cargo-leptos copies `public/` into the site root; `e2e/tests/legal.spec.ts` checks it answers as plain text). The owner found no verification section for app 834192 on the portal (13 October 2026), so the file names the app and says so; it ships with the next image build. Should the portal ask for a code later:

1. Copy the verification code from the product page on the portal.
2. Replace the whole content of `public/riot.txt` with it (no other text, one line).
3. Commit, deploy, and check `curl -s https://app.team-companion.eu/riot.txt` prints exactly the code.

## 7. Production key, if the registration still needs one

State on 13 October 2026: the production key is live on the VM (`riot_key=true`, budget 45 / 2400), the owner linked an account and saw the match history and a match detail with its timeline; ranked standing and an opponent lookup are still to be checked (the lookup needs a team). The registration text was rewritten from section 1 and 2 (the old text named the product "LoL Team Companion", placed the hosting in Germany, and said champion mastery seeds the user's own pool, which the code does not do: mastery is read only for scouted opponents' players); the owner's answers for the three notes above: no monetisation (a voluntary support link may follow, off at launch); closed beta of 5 to 20 teams, all requests user-triggered, a few thousand on a busy day; retention as in section 4.

1. Make sure sections 1 to 6 match the registration; update the product description on the portal where they do not.
2. The portal asks for a live site over HTTPS with terms, a privacy policy, `riot.txt` and a working prototype: deploy per `docs/RUNBOOK.md`, have the adviser confirm the drafted legal bases and clauses on the legal pages, and give the reviewer an invite link and a test account with a linked Riot ID (or a short screencast).
3. After approval: set `RIOT_API_KEY` and the two budget variables on the VM (`docs/RUNBOOK.md` sections 2.4 and 8) and note the approval date and the limits in MISSION.md's decision log.
