# Architecture

How Companion is built today (state after milestone M6). For goals and scope see `MISSION.md`; for the design system see `docs/DESIGN.md`.

## One crate, two targets

The repository is a single Rust crate (`lol_team_companion`, edition 2021, stable toolchain) compiled twice by `cargo-leptos`:

| Target | Feature | Output | Contains |
|--------|---------|--------|----------|
| Server | `ssr` | native binary `lol_team_companion` | Axum, SurrealDB, sessions, auth, Riot API, server-side rendering |
| Client | `hydrate` | `target/site/pkg/*.wasm` + JS glue | the same Leptos components, hydrated in the browser |

`src/lib.rs` is the shared root. It exposes `app`, `components`, `models`, `pages` and `error_template` to both targets, and `server` only under `ssr`. The WASM entry point `hydrate()` installs a panic hook and calls `leptos::mount::hydrate_body(App)`. A second binary, `src/bin/seed.rs`, fills the database with demo teams.

Leptos 0.8 runs without its `nightly` feature, so the crate builds on stable Rust. The dev profile keeps line tables for this crate only (no dependency debug info) to keep binaries small enough to link in a small container. `#![recursion_limit = "512"]` is required in `lib.rs` and `main.rs` because some view trees are deeply nested.

## Request flow

`src/main.rs` builds the server:

1. Load `.env` (`dotenvy`), set up `tracing` (`RUST_LOG`, default `info,lol_team_companion=debug`).
2. Open SurrealDB at `SURREAL_DATA_DIR` (default `./data`), apply `schema.surql` and run pending migrations (`db::init_db` → `db::prepare`). A failed migration stops the server.
3. Load the Data Dragon cache (stored copy, then a live refresh), normalise stored champion names against it, and start the 6-hour refresh task. Build the shared `RiotClient` and start the rate-limiter purge task.
4. Export open bug reports to `BUG_REPORT_INBOX_PATH` (default `./docs/inbox/bug-reports.md`). Failures are logged, never fatal.
5. Build the session layer (`tower-sessions` 0.14 with `SurrealSessionStore`, 30-day inactivity expiry) and the `axum-login` auth layer.
6. Mount `/healthz` (JSON `{status, db}`), all Leptos routes via `leptos_routes_with_context`, and `file_and_error_handler` for static files.
7. Wrap everything in `theme_injection_middleware` (inner) and the auth layer (outer).

For each page request (and server function call) the context closure provides three values to Leptos: the database handle (`Arc<Surreal<Db>>`), the shared `Arc<RiotClient>` and an `InitialTheme`. The shell in `src/app.rs` renders `<html data-theme=...>`, the stylesheet `/pkg/lol_team_companion.css`, the nav, the routed page inside `<main>`, the site footer with Riot's legal disclaimer, and the floating bug-report widget. Unknown paths render `NotFound` and set HTTP 404.

## Pages and server functions

Each route component lives in `src/pages/` (one file per page, sub-folders for `auth`, `team`, `admin`, `legal`). Pages define their own `#[server]` functions next to the component that calls them; there are about 140. A server function:

- calls `authz::authed()` (signed-in user + DB handle from context) or, for the few deliberately public functions (champion lists, `get_current_user`, login, register), `authz::db()`,
- derives the user's team on the server (`require_team` / `current_team_for` / `require_leader`); it never trusts a team ID from the client,
- calls a query in `src/server/db.rs`, passing the team or user as the scope,
- maps errors with `authz::server_err`, which logs the real error and returns a short generic message (`DbError::Invalid` messages pass through, `NotFound` becomes "Not found").

Team-scoped lists return `Ok(Vec::new())` when the user has no team so the page can show an empty state instead of an error banner. Some server functions are imported across pages (for example `pages::profile::get_current_user`); moving them into a dedicated API layer is planned.

Client code calls server functions through `Resource`s (inside `<Suspense>`), `Action`s, or `spawn_local` from event handlers. Login and registration finish with a hard navigation (`window.location().set_href`) so every resource refetches with the new session. Protected pages redirect to `/auth/login` client-side when `get_current_user()` returns `None`.

## Data layer

SurrealDB 3 runs embedded with the SurrealKV engine, namespace `lol_companion`, database `app`. The process holds an exclusive lock on the data directory, so the server and the seed binary cannot run at the same time.

- `schema.surql` is included at compile time and applied on every start. It is the baseline for a fresh database and contains only idempotent statements (`DEFINE ... IF NOT EXISTS`). Tables include `user`, `team`, `team_member`, `join_request`, `match`, `player_match`, `draft`, `draft_action`, `draft_tree`, `draft_tree_node`, `game_plan`, `post_game_learning`, `champion_pool`, `champion_note`, `opponent`, `opponent_player`, `action_item`, `team_note`, `series`, `ranked_snapshot`, `match_detail`, `match_timeline_event`, `personal_learning`, `personal_goal`, `bug_report`, `sessions`, `invite`, `schema_migration`, `ddragon_cache`, `patch_change` and the Sand Table tables (`replay`, `replay_frame`, `replay_event`, `board`, `board_version`).
- Migrations (`src/server/migrations.rs`) run right after the baseline; see "Migrations" below.
- `src/server/db.rs` holds every query. DB-facing structs (`Db*`, with `RecordId`) are SSR-only and convert into the shared models in `src/models/`, which compile for both targets and derive `Serialize, Deserialize, Clone`. Datetime fields load as `surrealdb::types::Datetime` and become RFC 3339 strings through `db::dt_to_string` / `opt_dt`; queries do not cast with `<string>` (a datetime loaded into a `String` field fails to deserialize, and a list query then silently returns nothing).
- Record links: fields that point at other records are `record<...>` (since M6 also `draft.series_id`, `game_plan.draft`, `post_game_learning.game_plan_id` and `draft_id`, `series.opponent_id`, `action_item.source_review`, `checklist_instance.game_plan_id` and `template_id`). Writes bind the key through `type::record('table', $key)` (`db::opt_key` for optional client IDs); reads turn the `RecordId` back into the string ID the models use. Fields holding external IDs stay strings: Riot match IDs (`match.match_id`, `match_detail.match_id`, `post_game_learning.match_riot_id`, `replay.match_id`), PUUIDs, and `action_item.assigned_to` (the assignee's username as picked on the page; converting it would change the page).
- Indexes: every `team`/`user` scope field that tenant-scoped reads filter on is indexed (listed at the end of `schema.surql`), plus the link fields above. Unique: `user.email` (stored lowercase), `user.username_lc`, `team_member.user` (one team per user), `invite.code`, `match.match_id`, `match_detail.match_id`, `personal_goal(user, goal_type)`, `replay(team, match_id)`, `board_version(board, version)`. Check with `INFO FOR TABLE <name>`; `tests/integration/migrations.rs` asserts the list.
- Transactions: every write path that touches more than one record runs in one `BEGIN ... COMMIT` (drafts with their actions, reviews with their action items, opponents with their players, trees with nodes, team creation, deletes with their children, the match-detail cache, replay imports, board versions, join-request acceptance). `db::check_tx` returns the real cause of a failed transaction (a `THROW` tag or a constraint violation) instead of SurrealDB's "not executed due to a failed transaction"; `db::thrown_tag` reads the tag. Lists of child rows are bound as arrays and written with `FOR $x IN $rows { CREATE ... }`, so query text stays constant.
- On start, `migrate_champion_names` normalises stored champion names against the Data Dragon cache; it is skipped when no copy is available.

Team-scoped DB functions take the team and verify that the record belongs to it (`ensure_team_record`, `ensure_team_opponent_player`, or a `WHERE team = ...` filter); user-scoped ones take the user. A record outside the scope is reported as `DbError::NotFound`, exactly like a missing one. All client values are bound; `record_key(id, table)` strips the table prefix for `type::record('table', $key)`.

New fields on existing tables are declared `option<T>` in the baseline so old rows stay valid; a migration can backfill them and tighten the type (example: `user.is_admin`, migration 3).

### Migrations

`server::migrations::run` applies every entry of `MIGRATIONS` whose `version` is not yet recorded in `schema_migration` (version, name, applied_at), in order, once each. It runs on every start and in the test helpers (`db::prepare`), and also on a fresh database right after the baseline, so each migration must be harmless there.

- A **SurrealQL** migration (`Step::Surql`) runs in one transaction together with the insert of its `schema_migration` row: all or nothing.
- A **Rust** migration (`Step::Rust`) is for per-row work (parsing legacy values, resolving duplicates). It spans several queries, so it cannot be one transaction; it must be idempotent so an interrupted run can simply be repeated. Its row is written after it succeeds.
- A failed migration stops startup (the error names the migration). A database that records a version this build does not know (migrated by a newer build) is refused too.

Applied so far: 1 `lowercase_emails` (case collisions are logged and left alone), 2 `username_lc_unique` (backfill, numeric suffix for pre-M1 case collisions, unique index), 3 `is_admin_not_null`, 4 `drop_legacy_sessions`, 5 `string_ids_to_record_links`, 6 `one_team_per_user_unique` (keeps the led team, else the oldest membership).

**How to add a migration**

1. Decide whether the baseline alone is enough. A new table, a new `option<T>` field or a plain (non-unique) index goes in `schema.surql` with `IF NOT EXISTS`; nothing else is needed.
2. Anything that changes existing data or needs clean data first (backfills, type changes, unique indexes, renames) is a migration. Add a `Migration { version, name, step }` at the end of `MIGRATIONS` with the next version number. Never edit, reorder or renumber a migration that has shipped.
3. Prefer `Step::Surql` (one transaction). Use `Step::Rust` for per-row logic and make it idempotent (select only rows still in the old form).
4. If the baseline declares the field too, declare it in its loose form there (`option<T>`) and tighten it in the migration with `DEFINE FIELD OVERWRITE`; leave a comment in `schema.surql` saying which migration tightened it.
5. Test it in `tests/integration/migrations.rs`: build the pre-migration shape with `baseline_only()` plus legacy rows, run `migrations::run`, assert the result. Then run the seed tool against a copy of a real `./data` (`SURREAL_DATA_DIR=/tmp/copy cargo run --features ssr --bin seed`) to see it apply to existing data.

## Authentication, authorization and sessions

- `src/server/config.rs` reads the security configuration once at startup (`APP_ENV`/`LEPTOS_ENV` dev detection, `CLOSED_BETA`, `SESSION_SECRET`, `COOKIE_SECURE`, `ADMIN_EMAILS`, `BOOTSTRAP_INVITE_CODE`, `RATE_LIMIT_MULTIPLIER`, `TRUST_FORWARDED_FOR`). Outside dev a missing `SESSION_SECRET` stops the server.
- `src/server/auth.rs` implements `axum_login::AuthnBackend` over the `user` table. Argon2 hashing and verification run on `spawn_blocking`; unknown emails are verified against a dummy hash so timing does not reveal accounts. Emails are compared lowercase. `session_auth_hash` is HMAC-SHA256(`SESSION_SECRET`, password hash), so session rows hold no password material and a password change ends sessions.
- `src/server/authz.rs` holds the authorization helpers (`authed`, `require_user`, `require_admin`, `current_team_for`, `require_team`, `require_team_member`, `require_team_leader`), the error mapping (`server_err`, `internal_error`), the Riot client accessor (`riot()`) and the rate-limit helpers. `src/server/rate_limit.rs` is an in-memory token-bucket limiter (login per IP and per account, register per IP, Riot-backed endpoints per IP) plus cooldowns (solo sync per user, team sync per team, 60 s) and the process-wide `RiotBudget`. Idle entries are purged by a one-minute timer, not on the request path. The peer IP comes from `ConnectInfo` (`into_make_service_with_connect_info`).
- Sessions are stored in SurrealDB by `src/server/session_store.rs` (MessagePack via `rmp-serde`, plus `expires_at` in unix seconds). Expired rows load as absent and are deleted; `delete_expired` runs at startup and hourly from `main.rs`. The cookie is `HttpOnly`, `SameSite=Strict` and `Secure` unless dev (or `COOKIE_SECURE=false`).
- Registration is invite-only: `register_action` validates username, email and a 10-character password minimum, atomically consumes one use of an `invite` code (unknown, used-up, expired and disabled codes, and duplicate emails, all get the same generic message), creates the user and logs them in. `CLOSED_BETA=false` lifts the gate in dev only. The UI still redirects `/auth/register` without `?invite=` to `/closed-beta`.
- Admins (`user.is_admin`, promoted from `ADMIN_EMAILS` at login/registration) manage codes on `/admin/invites`; everyone else gets the 404 page (rendered with `SsrMode::Async` so the status is set), and the nav shows the link to admins only.
- A user belongs to at most one team: `create_team` and join requests refuse a second membership; join requests are answered only by the leader of the requested team and only while pending.
- `/healthz` returns 200 with `{"status":"ok","db":"ok"}`, or 503 when a trivial DB query fails.

- Usernames are unique case-insensitively through `user.username_lc` (unique index); emails are stored lowercase and looked up with `email = $email` (unique index). `create_user` / `update_username` rely on the indexes and map their violations to the friendly messages.

Users have a `mode` (`solo` or `team`). New users start in solo mode and land on `/solo`; login sends team-mode users to `/team/dashboard`.

## Riot and other integrations

- `src/server/riot.rs` holds one shared `RiotClient` (riven), built once in `main.rs` from `RIOT_API_KEY`, kept in the app state and provided to server functions as context (`authz::riot()`). Sharing it lets riven's own rate limiter see every request. It covers account lookup (`get_puuid`), match history, full match detail and timeline, champion masteries, ranked entries and opponent intel.
  - **Routing:** `riot::platform_for(&[...])` takes the first region code that is set: the user's profile region (`user.riot_region`, chosen on /profile from `REGION_CODES`) for personal calls, then the team's region for team-wide syncs, and the team's region for opponent scouting; EUW is the fallback. Match detail routes by the platform in the match ID (`EUW1_...`).
  - **Budget:** on top of the per-IP `Bucket::Riot` limit, every request takes a token from the process-wide `RiotBudget` (`rate_limit.rs`): 20 per second and 50 per minute with a two-minute burst (a development key's 20/1 s and 100/2 min), configurable with `RIOT_GLOBAL_BUDGET_PER_SEC` / `RIOT_GLOBAL_BUDGET_PER_MIN`. A request waits at most 5 s for a token; multi-request jobs call `ensure_headroom(n)` first. Either way the user gets "Riot is busy, try again in N s." instead of a stalled job.
  - **No key / bad key:** without a key the client reports "not configured" and pages show that Riot features are off; a dummy key reaches Riot and gets 401/403, which `authz::server_err` turns into "The Riot API rejected our key".
- `src/server/data_dragon.rs` is a process-wide cache of the live patch version, the champion list and the numeric key → champion map. `init` loads the last good copy from `ddragon_cache` (so a cold start without network still works), then fetches `versions.json` and `champion.json`; a tokio interval refreshes every 6 hours. Each successful refresh is persisted; a new version writes a `patch_change` row (old, new, detected_at) and is logged. The public champion-list server functions read the cache. Masteries resolve champion keys through the live map first. It also normalises free-text champion names.
- Image URLs use the live version on both targets: the shell writes it into `<meta name="ddragon-version">`, and `models::ddragon::{champion_icon_url, item_icon_url}` read it from the server cache (SSR) or that tag (browser).

## Sand Table storage

Storage only so far (no UI). `server::sand_table::import_timeline` converts a Riot Match-V5 timeline JSON into per-minute frames (every participant's position, total and current gold, xp, CS, level, and an estimated `alive`) and positioned events (kills, objectives, structures, plates, wards, level-ups, game end; unknown types are counted, not fatal). `db::store_replay` writes a team's replay with all frames and events in one transaction (re-importing a match replaces the team's copy); `get_replay_frames` / `get_replay_events` read them scoped to the team. Boards are saved as numbered `board_version` scene documents (`save_board_version`, `get_board_version`). The test fixture `tests/fixtures/riot_match_v5_timeline.json` was written from the documented shape; its `_fixture_notes` list what could not be verified.

## Theming and CSS

Tailwind CSS v4 runs through cargo-leptos (`tailwind-input-file = "input.css"`); there is no Node toolchain. `input.css` contains self-hosted `@font-face` rules (no Google Fonts CDN, enforced in CI), the `@theme` block mapping semantic tokens (`--color-base`, `--color-surface`, `--color-primary`, `--color-accent`, `--color-success`, ...) to per-theme variables, and a few component rules in `@layer components`.

The theme value travels: cookie `lol_companion_theme`, then the user's DB `theme`, then the default, resolved by `src/server/theme_layer.rs` into a task-local, then provided as `InitialTheme` and written to `<html data-theme>`. On hydrate, `App` re-reads that attribute so server and client agree. Since M0 the resolver clamps every value to `"demacia"`; the user-facing toggle is gone. The `src/components/region/` primitives (Card, Btn, SectionHead, ModeToggle, Badge, ...) still accept a `region` prop and contain unreachable Pandemonium branches; M2 and M3 replace them with the new design-system primitives.

## Components

`src/components/` holds the shared UI: `nav` (hub navigation, solo/team mode toggle, notifications, user menu), `footer`, `ui` (ErrorBanner, StatusMessage, ToastProvider, Skeleton*, EmptyState, NoTeamState), `skeleton` (PageLoading, PageEmpty), `draft_board` (20-slot pick/ban board; `slot_meta` is the canonical slot order), `champion_picker`, `champion_autocomplete`, `tree_graph` (SVG draft tree), `bug_report_widget`, `stat_card`, `icon` and the legacy `region/*` set.

## Testing

| Layer | Where | Command |
|-------|-------|---------|
| Unit | `#[cfg(test)]` modules in models and server files | `cargo test --features ssr --lib` |
| DB integration | `tests/integration/main.rs` plus one module per area, each on an in-memory SurrealDB (`kv-mem`) prepared with the baseline and all migrations; `security.rs` holds the cross-tenant, invite, session-expiry and injection suites, `migrations.rs` the migration and index checks, `transactions.rs` the rollback checks | `cargo test --features ssr` |
| e2e | `e2e/tests/*.spec.ts`, Playwright 1.58, Chromium | `cd e2e && npx playwright test` |

The e2e fixtures register a fresh user per test (`authedPage`, with the invite code from `E2E_INVITE_CODE`, which the server provides through `BOOTSTRAP_INVITE_CODE`) and optionally create a team (`teamPage`). `hydration-no-panic.spec.ts` is the gate against SSR/hydrate divergence; `cross-tenant.spec.ts` drives two accounts and checks that one team cannot read or change the other's draft and game plan. CI (`.github/workflows/ci.yml`) checks both targets, runs clippy for both targets with `-D warnings`, `cargo fmt --check`, the unit and DB integration tests and the style guardrails, then builds the app, starts the binary in production mode (with `SESSION_SECRET`, a bootstrap invite and `ADMIN_EMAILS`), waits for `/healthz` and runs the smoke, hydration, auth, admin-invites and cross-tenant specs.

## Tooling

- `rust-toolchain.toml` pins stable with the `wasm32-unknown-unknown` target, clippy and rustfmt.
- `.claude/hooks/session-start.sh` prepares cloud sessions (cargo-leptos, components, e2e dependencies, Playwright browser compat directory, `.env`).
- `scripts/` holds `wait_for_server.sh`, `health.sh`, `api_test.sh`, `flow_test.sh` and `clean-orphan-targets.sh`; `justfile` wraps the common commands.
