# LoL Team Companion

A League of Legends team management app for coordinating drafts, tracking stats, and planning games.

**Stack:** Rust nightly · Leptos 0.8 · Axum 0.8 · SurrealDB 2.x (SurrealKV) · Tailwind CSS v4

## Quick Start

```bash
# Prerequisites: rust nightly, cargo-leptos, tailwindcss standalone binary (no npm)
cargo install cargo-leptos

# Environment
cp .env.example .env   # Add RIOT_API_KEY, optionally SURREAL_DATA_DIR
cargo leptos watch
```

The app runs at `http://127.0.0.1:3000` with live-reload on port 3001.

## Architecture

Single crate, dual compile target:
- **`ssr` feature** → Server binary (Axum + SurrealDB + Riot API)
- **`hydrate` feature** → WASM client (browser hydration)

`cargo-leptos` builds both targets and serves the WASM bundle.

### Directory Layout

```
src/
├── main.rs              # Axum server setup, DB init, session/auth layers
├── app.rs               # Router, shell, all <Route> definitions
├── lib.rs               # Crate root, re-exports app module
├── error_template.rs    # Fallback error page
├── server/              # SSR-only (behind #[cfg(feature = "ssr")])
│   ├── db.rs            # All SurrealDB queries
│   ├── auth.rs          # AuthBackend (axum-login), Credentials, password hashing
│   ├── session_store.rs # SurrealSessionStore for tower-sessions
│   ├── riot.rs          # Riot API client (riven crate)
│   ├── data_dragon.rs   # Champion data from Data Dragon CDN
│   └── leaguepedia.rs   # Pro play data scraping
├── models/              # Shared types (compiled for both SSR and WASM)
│   ├── user.rs          # AppUser, TeamMember
│   ├── team.rs          # Team
│   ├── draft.rs         # Draft, DraftAction
│   ├── match_data.rs    # PlayerMatchStats
│   ├── game_plan.rs     # GamePlan, PostGameLearning
│   └── champion.rs      # Champion metadata
├── pages/               # Route components (each may contain #[server] fns)
│   ├── home.rs          # Landing page
│   ├── profile.rs       # Riot account linking
│   ├── auth/login.rs    # Login form
│   ├── auth/register.rs # Registration form
│   ├── team/dashboard.rs
│   ├── team/roster.rs
│   ├── team_builder.rs
│   ├── draft.rs         # Draft planner with champion picker
│   ├── stats.rs         # Match history + stats
│   ├── game_plan.rs     # Pre-game strategy
│   └── post_game.rs     # Post-game review
├── components/          # Reusable UI
│   ├── nav.rs           # Top navigation bar
│   ├── champion_picker.rs
│   ├── draft_board.rs
│   └── stat_card.rs
schema.surql             # DB schema (loaded on startup via include_str!)
```

## Routes

| Path               | Component        | Auth Required |
|--------------------|------------------|---------------|
| `/`                | HomePage         | No            |
| `/auth/login`      | LoginPage        | No            |
| `/auth/register`   | RegisterPage     | No            |
| `/profile`         | ProfilePage      | Yes           |
| `/team/dashboard`  | TeamDashboard    | Yes           |
| `/team/roster`     | RosterPage       | Yes           |
| `/team-builder`    | TeamBuilderPage  | Yes           |
| `/draft`           | DraftPage        | Yes           |
| `/stats`           | StatsPage        | Yes           |
| `/game-plan`       | GamePlanPage     | Yes           |
| `/post-game`       | PostGamePage     | Yes           |

## Critical Patterns and Gotchas

### SurrealDB 2.x

1. **`type::record()` not `type::thing()`** — `type::thing()` was removed in SurrealDB 2.x. Always use `type::record('table', $key)`.

2. **Strip the table prefix before passing keys** — `type::record('user', $key)` expects just the key part. Always strip: `user_id.strip_prefix("user:").unwrap_or(&user_id).to_string()`.

3. **RecordId deserialization** — Never deserialize SurrealDB query results as `serde_json::Value`. Create `Db*` structs with `surrealdb::RecordId` fields, then convert to app-facing structs with `String` IDs. See `DbTeam` → `Team` in `db.rs`.

4. **`.bind()` requires `'static`** — Always pass owned `String` values to `.bind()`, never `&str`.

5. **No `string()` cast in SurQL** — SurrealDB 2.x removed `string()`. Use `.to_sql()` on `RecordId` in Rust code instead.

6. **`SurrealValue` derive** — Use `#[derive(SurrealValue)]` from `surrealdb-types-derive` on DB result structs.

### Leptos 0.8

7. **`ActionForm` has no `class` prop** — Wrap it in a `<div>` for styling.

8. **Server redirect doesn't refetch resources** — After login/register, use hard navigation (`window.location().set_href()`) instead of relying on `leptos_axum::redirect` to refresh auth state.

9. **SSR imports inside `#[server]` body** — Put `use` statements for server-only crates (leptos_axum, auth types) inside the `#[server]` function body, not at the top of the file.

10. **`attr:class` on `<A>`** — Use `attr:class="..."` instead of `class="..."` on Leptos router's `<A>` component.

### Server Functions

11. **DB access via context** — Use `use_context::<Arc<Surreal<Db>>>()` inside `#[server]` functions. Do NOT use `axum::extract::State` (it requires `FromRef<()>` which fails).

12. **Auth extraction** — `let mut auth: AuthSession = leptos_axum::extract().await?;` — must be `mut` for `auth.login()`.

### Toolchain

13. **BFD linker** — `.cargo/config.toml` forces `bfd` linker to work around an LLVM 22 lld crash on nightly.

14. **Standalone Tailwind** — Uses the `tailwindcss-linux-x64` binary directly, no npm/node required. Referenced in `[package.metadata.leptos]`.

15. **tower-sessions version** — Must match the version used by axum-login (currently 0.14). A mismatch causes silent session failures.

### External APIs

16. **riven crate** — `Queue::from(u16)` not `i16`; `puuid` field is `String` not `Option<String>`.

17. **Riot API key** — Set `RIOT_API_KEY` in `.env`. Loaded via `dotenvy::dotenv()` on startup.

## Adding a New Feature

### New page

1. Create `src/pages/my_page.rs` with a `#[component] pub fn MyPage()` and any `#[server]` functions
2. Add `pub mod my_page;` to `src/pages/mod.rs`
3. Add a `<Route>` in `src/app.rs`
4. Add a nav link in `src/components/nav.rs` if needed

### New DB table

1. Add `DEFINE TABLE` / `DEFINE FIELD` statements to `schema.surql`
2. Add query functions to `src/server/db.rs` (with `Db*` struct → model conversion)
3. Add shared model struct to `src/models/`

### New model struct

- Shared structs go in `src/models/` — must compile for both SSR and WASM
- DB-facing structs with `RecordId` go in `src/server/db.rs` (SSR only)
- Derive `Serialize, Deserialize, Clone` on shared structs

## Code Style

- **Errors:** `thiserror` for custom error types, map to `ServerFnError` via `.map_err(|e| ServerFnError::new(e.to_string()))`
- **Tailwind:** Dark theme by default (`bg-gray-950`, `text-white`), accent color `yellow-400`
- **Forms:** Tailwind utility classes inline, `bg-gray-800` inputs with `border-gray-600`
- **Naming:** snake_case files and functions, PascalCase components
