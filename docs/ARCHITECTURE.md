# Architecture

How Companion is built today (state after milestone M1). For goals and scope see `MISSION.md`; for the design system see `docs/DESIGN.md`.

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
2. Open SurrealDB at `SURREAL_DATA_DIR` (default `./data`) and apply `schema.surql`.
3. Export open bug reports to `BUG_REPORT_INBOX_PATH` (default `./docs/inbox/bug-reports.md`). Failures are logged, never fatal.
4. Build the session layer (`tower-sessions` 0.14 with `SurrealSessionStore`, 30-day inactivity expiry) and the `axum-login` auth layer.
5. Mount `/healthz` (JSON `{status, db}`), all Leptos routes via `leptos_routes_with_context`, and `file_and_error_handler` for static files.
6. Wrap everything in `theme_injection_middleware` (inner) and the auth layer (outer).

For each page request the context closure provides two values to Leptos: the database handle (`Arc<Surreal<Db>>`) and an `InitialTheme`. The shell in `src/app.rs` renders `<html data-theme=...>`, the stylesheet `/pkg/lol_team_companion.css`, the nav, the routed page inside `<main>`, the site footer with Riot's legal disclaimer, and the floating bug-report widget. Unknown paths render `NotFound` and set HTTP 404.

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

- `schema.surql` is included at compile time and applied on every start (`DEFINE TABLE ... IF NOT EXISTS`). Tables include `user`, `team`, `team_member`, `join_request`, `match`, `player_match`, `draft`, `draft_action`, `draft_tree`, `draft_tree_node`, `game_plan`, `post_game_learning`, `champion_pool`, `champion_note`, `opponent`, `opponent_player`, `action_item`, `team_note`, `series`, `ranked_snapshot`, `match_detail`, `match_timeline_event`, `personal_learning`, `personal_goal`, `bug_report` and `sessions`.
- `src/server/db.rs` holds every query. DB-facing structs (`Db*`, with `RecordId`) are SSR-only and convert into the shared models in `src/models/`, which compile for both targets and derive `Serialize, Deserialize, Clone`.
- On start, `migrate_champion_names` normalises stored champion names against Data Dragon; it is skipped when Data Dragon is unreachable.

Team-scoped DB functions take the team and verify that the record belongs to it (`ensure_team_record`, `ensure_team_opponent_player`, or a `WHERE team = ...` filter); user-scoped ones take the user. A record outside the scope is reported as `DbError::NotFound`, exactly like a missing one. All client values are bound; `record_key(id, table)` strips the table prefix for `type::record('table', $key)`.

There is no migration framework, no unique indexes beyond what the schema defines and few transactions yet (planned for M6). New fields on existing tables are declared `option<T>` so old rows stay valid (for example `user.is_admin`).

## Authentication, authorization and sessions

- `src/server/config.rs` reads the security configuration once at startup (`APP_ENV`/`LEPTOS_ENV` dev detection, `CLOSED_BETA`, `SESSION_SECRET`, `COOKIE_SECURE`, `ADMIN_EMAILS`, `BOOTSTRAP_INVITE_CODE`, `RATE_LIMIT_MULTIPLIER`, `TRUST_FORWARDED_FOR`). Outside dev a missing `SESSION_SECRET` stops the server.
- `src/server/auth.rs` implements `axum_login::AuthnBackend` over the `user` table. Argon2 hashing and verification run on `spawn_blocking`; unknown emails are verified against a dummy hash so timing does not reveal accounts. Emails are compared lowercase. `session_auth_hash` is HMAC-SHA256(`SESSION_SECRET`, password hash), so session rows hold no password material and a password change ends sessions.
- `src/server/authz.rs` holds the authorization helpers (`authed`, `require_user`, `require_admin`, `current_team_for`, `require_team`, `require_team_member`, `require_team_leader`), the error mapping (`server_err`, `internal_error`) and the rate-limit helpers. `src/server/rate_limit.rs` is an in-memory token-bucket limiter (login per IP and per account, register per IP, Riot-backed endpoints per IP) plus per-user cooldowns (team and solo sync, 60 s). The peer IP comes from `ConnectInfo` (`into_make_service_with_connect_info`).
- Sessions are stored in SurrealDB by `src/server/session_store.rs` (MessagePack via `rmp-serde`, plus `expires_at` in unix seconds). Expired rows load as absent and are deleted; `delete_expired` runs at startup and hourly from `main.rs`. The cookie is `HttpOnly`, `SameSite=Strict` and `Secure` unless dev (or `COOKIE_SECURE=false`).
- Registration is invite-only: `register_action` validates username, email and a 10-character password minimum, atomically consumes one use of an `invite` code (unknown, used-up, expired and disabled codes, and duplicate emails, all get the same generic message), creates the user and logs them in. `CLOSED_BETA=false` lifts the gate in dev only. The UI still redirects `/auth/register` without `?invite=` to `/closed-beta`.
- Admins (`user.is_admin`, promoted from `ADMIN_EMAILS` at login/registration) manage codes on `/admin/invites`; everyone else gets the 404 page (rendered with `SsrMode::Async` so the status is set), and the nav shows the link to admins only.
- A user belongs to at most one team: `create_team` and join requests refuse a second membership; join requests are answered only by the leader of the requested team and only while pending.
- `/healthz` returns 200 with `{"status":"ok","db":"ok"}`, or 503 when a trivial DB query fails.

Users have a `mode` (`solo` or `team`). New users start in solo mode and land on `/solo`; login sends team-mode users to `/team/dashboard`.

## Riot and other integrations

- `src/server/riot.rs` wraps the `riven` crate. Each call builds a client from `RIOT_API_KEY`; there is no shared client and the platform route is passed per call. It covers account lookup (`get_puuid`), match history, full match detail and timeline, champion masteries, ranked entries and opponent intel. Pages call it from server functions (stats, solo dashboard, roster, opponents, post-game, draft).
- `src/server/data_dragon.rs` fetches the latest Data Dragon version and `champion.json`, and normalises free-text champion names. Several components still hardcode older Data Dragon versions for images (M6).

## Theming and CSS

Tailwind CSS v4 runs through cargo-leptos (`tailwind-input-file = "input.css"`); there is no Node toolchain. `input.css` contains self-hosted `@font-face` rules (no Google Fonts CDN, enforced in CI), the `@theme` block mapping semantic tokens (`--color-base`, `--color-surface`, `--color-primary`, `--color-accent`, `--color-success`, ...) to per-theme variables, and a few component rules in `@layer components`.

The theme value travels: cookie `lol_companion_theme`, then the user's DB `theme`, then the default, resolved by `src/server/theme_layer.rs` into a task-local, then provided as `InitialTheme` and written to `<html data-theme>`. On hydrate, `App` re-reads that attribute so server and client agree. Since M0 the resolver clamps every value to `"demacia"`; the user-facing toggle is gone. The `src/components/region/` primitives (Card, Btn, SectionHead, ModeToggle, Badge, ...) still accept a `region` prop and contain unreachable Pandemonium branches; M2 and M3 replace them with the new design-system primitives.

## Components

`src/components/` holds the shared UI: `nav` (hub navigation, solo/team mode toggle, notifications, user menu), `footer`, `ui` (ErrorBanner, StatusMessage, ToastProvider, Skeleton*, EmptyState, NoTeamState), `skeleton` (PageLoading, PageEmpty), `draft_board` (20-slot pick/ban board; `slot_meta` is the canonical slot order), `champion_picker`, `champion_autocomplete`, `tree_graph` (SVG draft tree), `bug_report_widget`, `stat_card`, `icon` and the legacy `region/*` set.

## Testing

| Layer | Where | Command |
|-------|-------|---------|
| Unit | `#[cfg(test)]` modules in models and server files | `cargo test --features ssr --lib` |
| DB integration | `tests/integration/main.rs` plus one module per area, each on an in-memory SurrealDB (`kv-mem`); `security.rs` holds the cross-tenant, invite, session-expiry and injection suites | `cargo test --features ssr` |
| e2e | `e2e/tests/*.spec.ts`, Playwright 1.58, Chromium | `cd e2e && npx playwright test` |

The e2e fixtures register a fresh user per test (`authedPage`, with the invite code from `E2E_INVITE_CODE`, which the server provides through `BOOTSTRAP_INVITE_CODE`) and optionally create a team (`teamPage`). `hydration-no-panic.spec.ts` is the gate against SSR/hydrate divergence; `cross-tenant.spec.ts` drives two accounts and checks that one team cannot read or change the other's draft and game plan. CI (`.github/workflows/ci.yml`) checks both targets, runs clippy for both targets with `-D warnings`, `cargo fmt --check`, the unit and DB integration tests and the style guardrails, then builds the app, starts the binary in production mode (with `SESSION_SECRET`, a bootstrap invite and `ADMIN_EMAILS`), waits for `/healthz` and runs the smoke, hydration, auth, admin-invites and cross-tenant specs.

## Tooling

- `rust-toolchain.toml` pins stable with the `wasm32-unknown-unknown` target, clippy and rustfmt.
- `.claude/hooks/session-start.sh` prepares cloud sessions (cargo-leptos, components, e2e dependencies, Playwright browser compat directory, `.env`).
- `scripts/` holds `wait_for_server.sh`, `health.sh`, `api_test.sh`, `flow_test.sh` and `clean-orphan-targets.sh`; `justfile` wraps the common commands.
