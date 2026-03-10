# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# LoL Team Companion

A League of Legends team management app for coordinating drafts, tracking stats, and planning games.

**Stack:** Rust nightly · Leptos 0.8 · Axum 0.8 · SurrealDB 3.x (SurrealKV) · Tailwind CSS v4

## Commands

```bash
# Dev server with live-reload (runs on :3000, reload on :3001)
cargo leptos watch

# Production build
cargo leptos build --release

# Fast type-check (SSR target, no WASM compile)
cargo check --features ssr

# Fast type-check (WASM/hydrate target)
cargo check --features hydrate --target wasm32-unknown-unknown

# Run server tests
cargo test --features ssr

# Adjust log level (default: info, app crate: debug)
RUST_LOG=debug cargo leptos watch
```

## Setup

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

### SurrealDB 3.x

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

### Leptos Reactivity

18. **Clone before multiple closures** — `Vec<T>` and `HashMap<K,V>` don't implement `Copy`. When the same value must be captured by two or more `move` closures (e.g. `class=move ||` and `on:click=move |_|`), clone before each:
    ```rust
    let role_val_for_class = role_val.clone();
    view! {
        <button class=move || { ... role_val_for_class.clone() ... }
                on:click=move |_| set_x.set(role_val.clone())>
    ```

19. **`into_any()` for divergent view branches** — When `if/else` or `match` arms inside `{move || ...}` return structurally different view types, each arm must call `.into_any()`:
    ```rust
    {move || if filled {
        view! { <img ... /> }.into_any()
    } else {
        view! { <span>...</span> }.into_any()
    }}
    ```

20. **`get_untracked()` in event handlers** — Inside `on:click`, `on:input`, etc., read signals with `get_untracked()` to avoid accidentally registering reactive dependencies in a non-tracking context:
    ```rust
    on:click=move |_| {
        let val = my_signal.get_untracked(); // not .get()
    }
    ```

21. **`prop:value` for controlled inputs** — `attr:value` only sets the initial DOM attribute. For a controlled input that reflects signal changes after render, use `prop:value`:
    ```rust
    <input prop:value=move || signal.get()
           on:input=move |ev| set_signal.set(event_target_value(&ev)) />
    ```

22. **`StoredValue` for non-reactive data shared across closures** — When you need to share a large non-`Copy` value (like a `HashMap`) across multiple closures without reactive tracking overhead, use `store_value()`:
    ```rust
    let map = store_value(champion_map); // StoredValue<HashMap<...>>
    // later in closures:
    map.with_value(|m| m.get(&name).cloned())
    ```

23. **`resource.refetch()` after mutations** — `Resource::new` does not auto-refetch after a server fn mutates data. Call `resource.refetch()` inside the `spawn_local` success branch to refresh lists:
    ```rust
    Ok(_) => { drafts.refetch(); set_save_result.set(Some("Saved!".into())); }
    ```

24. **`spawn_local` for async event handlers** — The only way to call an async server function from a sync `on:click` handler. Errors must be handled inside:
    ```rust
    on:click=move |_| {
        leptos::task::spawn_local(async move {
            match my_server_fn(args).await {
                Ok(v) => ...,
                Err(e) => ...,
            }
        });
    }
    ```

25. **`collect_view()` for iterators** — Use `.collect_view()` instead of `.collect::<Vec<_>>()` when building fragments from iterators inside `view!`. It handles the view trait conversion correctly.

26. **`<For>` key must be stable** — The `key` prop on `<For>` drives DOM diffing. Always use a stable entity ID (e.g. `c.id`), never the array index. Unstable keys cause unnecessary re-renders or DOM thrash.

### SurrealDB

27. **`.check()` on write queries** — After `CREATE`/`UPDATE`/`DELETE`, call `.check()` to surface constraint violations and query errors. Without it, a failed write silently returns `Ok`:
    ```rust
    db.query("CREATE ...").bind(...).await?.check()?;
    ```
    Read queries (`SELECT`) surface errors via `response.take()` instead.

28. **`take(0).unwrap_or_default()` for list queries** — For queries that return `Vec<T>`, use `unwrap_or_default()` rather than `?` so an empty result doesn't error:
    ```rust
    let rows: Vec<MyStruct> = result.take(0).unwrap_or_default();
    ```
    For single-record lookups returning `Option<T>`, use `result.take(0)?` instead.

29. **Batch multiple queries in one call** — Chain statements in a single `.query()` to avoid round-trips. Index results by statement order:
    ```rust
    let mut r = db.query("SELECT ...; SELECT ...;").await?;
    let teams: Vec<DbTeam> = r.take(0).unwrap_or_default();
    let members: Vec<DbTeamMember> = r.take(1).unwrap_or_default();
    ```

30. **`DEFINE FIELD IF NOT EXISTS` for all schema fields** — Schema is re-applied on every startup. `IF NOT EXISTS` makes it idempotent so existing records are never affected by schema re-runs. Never omit it.

31. **Use `BEGIN`/`COMMIT` for multi-step writes** — When multiple `CREATE`/`UPDATE` statements must succeed or fail together, wrap in a transaction:
    ```rust
    db.query("BEGIN TRANSACTION; CREATE ...; CREATE ...; COMMIT TRANSACTION;")
        .bind(...).await?.check()?;
    ```

### Server Functions

32. **Server fn args and return types must be `Serialize + Deserialize`** — All parameters and the `Ok` type cross the wire as JSON (or msgpack). Avoid types like `HashMap` with non-string keys; flatten to `Vec<(K,V)>` or a newtype if needed.

33. **Pass complex data as JSON strings** — When a server fn needs a `Vec<T>` or nested struct that may be hard to encode as a top-level query param, serialize to `String` on the client and deserialize in the fn body (see `actions_json`/`comments_json` pattern in `draft.rs`).

34. **`#[server]` ordering** — Server functions must be defined (or `use`d) before the `#[component]` that calls them in the same file, because the macro generates a client-side stub that must be in scope.

## Code Style

- **Errors:** `thiserror` for custom error types, map to `ServerFnError` via `.map_err(|e| ServerFnError::new(e.to_string()))`
- **Tailwind:** Dark theme by default (`bg-gray-950`, `text-white`), accent color `yellow-400`
- **Forms:** Tailwind utility classes inline, `bg-gray-800` inputs with `border-gray-600`
- **Naming:** snake_case files and functions, PascalCase components
