# Architecture

**Analysis Date:** 2026-03-14

## Pattern Overview

**Overall:** Leptos full-stack with dual compilation targets (SSR + WASM hydration)

**Key Characteristics:**
- Single Rust crate compiled to both server (Axum) and client (WASM) via `cargo-leptos`
- Server functions act as RPC layer between WASM client and Axum handlers
- SurrealDB as single source of truth, accessed exclusively from server code
- Session-based authentication with password hashing (Argon2)
- Type-safe reactivity via Leptos signals and resources

## Layers

**Server Layer:**
- Location: `src/server/`
- Contains: SSR-only code compiled with `feature = "ssr"`
- Includes: DB access (`db.rs`), authentication (`auth.rs`), external APIs (`riot.rs`, `data_dragon.rs`, `leaguepedia.rs`), session management (`session_store.rs`)
- Depends on: SurrealDB, riven crate (Riot API), axum-login for auth
- Used by: Server functions in page components

**Page Layer:**
- Location: `src/pages/`
- Contains: Route components with embedded `#[server]` functions
- Pattern: Each page file contains both server functions (SSR-only) and component (Leptos UI)
- Depends on: Server layer for database access, models for data structures
- Used by: Router in `app.rs` which defines all `<Route>` components

**Component Layer:**
- Location: `src/components/`
- Contains: Reusable UI components (champion picker, draft board, nav, theme toggle, tree graph, etc.)
- Pattern: Pure Leptos components, no direct DB access; receive data via props
- Depends on: Models for data structures, Tailwind CSS for styling
- Used by: Page components for UI construction

**Model Layer:**
- Location: `src/models/`
- Contains: Shared data structures compiled for both SSR and WASM targets
- Includes: User, Team, Draft, DraftAction, GamePlan, Champion, etc.
- Derives: `Serialize`, `Deserialize`, `Clone`, often `PartialEq`
- Used by: All layers for type-safe data passing

**Server Function Bridge:**
- Server functions marked with `#[server]` macro compile to:
  - SSR: Axum handler + SurrealDB query inside function body
  - WASM: RPC stub that sends request to server endpoint
- Arguments and return types must be `Serialize + Deserialize`
- Accessed via `use_context::<Arc<Surreal<Db>>>()` for DB, `leptos_axum::extract()` for auth

## Data Flow

**Request → Response:**

1. **Browser** sends HTTP request (page load or server fn call)
2. **Axum** (via `cargo-leptos`) routes to server or serves WASM bundle
3. **Server Function** executes (SSR target):
   - Extract auth session via `leptos_axum::extract()`
   - Get DB context via `use_context::<Arc<Surreal<Db>>>()`
   - Execute SurrealDB query via `db::*` functions
   - Return result as JSON to WASM client or render HTML
4. **WASM Client** receives JSON, updates signals, re-renders
5. **Leptos Reactivity** detects signal changes, patches DOM

**Resource Fetching:**

- `Resource::new(deps, fetcher_fn)` creates reactive data source
- `fetcher_fn` calls server function, returns `Result<T>`
- Resource auto-refetches when deps change
- Wrap in `<Suspense>` to handle loading/error states

**State Management:**

- **Signals** (`RwSignal<T>`) hold client-side UI state (form inputs, toggles, selections)
- **Resources** fetch server data and cache it reactively
- **Server Actions** (`ServerAction::<FnName>`) track pending mutations (save, delete, update)
- Auto-save patterns: Effect watches signal, debounces, calls mutation server fn via `spawn_local`

## Key Abstractions

**Server Functions:**
- Purpose: Bridge between WASM client and Axum server
- Examples: `src/pages/draft.rs::save_draft()`, `src/pages/profile.rs::get_current_user()`, `src/pages/team/roster.rs::join_team()`
- Pattern: Function body runs on server; macro generates client-side stub that sends JSON request

**Database Functions:**
- Purpose: Encapsulate SurrealDB queries into reusable operations
- Location: `src/server/db.rs` (3243 lines)
- Examples: `get_user_team_id()`, `save_draft()`, `get_champion_pool()`, `list_drafts()`
- Pattern: Accept `&Surreal<Db>` + parameters, return `DbResult<T>` or `DbResult<Vec<T>>`
- Conversion: `DbUser` (with `RecordId`) → `AppUser` (with `String` id)

**Auth Backend:**
- Purpose: Implement `AuthnBackend` trait from axum-login
- Location: `src/server/auth.rs`
- Methods: `authenticate(email, password)`, `get_user(user_id)`
- Session: Stored in SurrealDB via `SurrealSessionStore`

**Models:**
- Purpose: Type-safe data structures shared across layers
- Location: `src/models/*.rs`
- Pattern: Derive `Serialize`, `Deserialize`, `Clone`, `PartialEq`
- SSR-only derives: `#[cfg_attr(feature = "ssr", derive(SurrealValue))]` for DB deserialization

**Components:**
- Purpose: Reusable UI pieces (Champion picker, draft board, nav, etc.)
- Location: `src/components/*.rs`
- Pattern: Pure Leptos components with `#[component]` macro, receive data via props
- Styling: Tailwind utility classes, semantic tokens from CSS custom properties

## Entry Points

**Server Entry:**
- Location: `src/main.rs`
- Triggers: `cargo leptos watch` or binary startup
- Responsibilities:
  1. Load `.env` file
  2. Initialize SurrealDB at `SURREAL_DATA_DIR` (default: `./data`)
  3. Apply schema from `schema.surql`
  4. Set up session store and auth backend
  5. Start Axum on configured addr (default: `127.0.0.1:3002`)

**Client Entry:**
- Location: `src/lib.rs::hydrate()` (WASM only)
- Triggers: Browser loads WASM bundle
- Responsibilities:
  1. Set panic hook for better error messages
  2. Mount Leptos `App` component into DOM

**Application Root:**
- Location: `src/app.rs::App` component
- Renders: `<Router>` with all `<Route>` definitions
- Children: `<Nav>` (top nav bar) + `<Routes>` (page content)
- Fallback: `<ErrorTemplate>` for 404s

**Page Routes:**
- Public: `/` (home), `/auth/login`, `/auth/register`
- Authenticated: All team/draft/stats/game-plan pages
- Auth check: `get_current_user()` Resource in each protected page, hard-navigate to `/auth/login` if `None`

## Error Handling

**Strategy:** Server-side errors map to `ServerFnError`, client-side displays via error banners

**Patterns:**

- **Database Errors:** `DbError` enum with variants (Surreal, NotFound, Other)
  - Mapped to `ServerFnError::new(error.to_string())` in server functions
  - Client catches via `Result<T, ServerFnError>` in Resource/Action handlers

- **Auth Errors:** `AuthError` enum (Db, Hash)
  - Authentication failure returns `Ok(None)` not error (expected condition)
  - Password hash errors propagate as `ServerFnError`

- **API Errors:** `RiotError` from riven crate or custom
  - Wrapped in `ServerFnError::new()` when called from server functions
  - No internet = graceful degradation (stats page shows empty state)

- **Client Error Display:**
  - `<ErrorBanner>` component in `src/components/ui.rs` renders red banner with icon
  - `Resource` errors captured in `<Suspense fallback=...>` error view
  - Server action errors stored in `action.value()` signal for reactive display

## Cross-Cutting Concerns

**Logging:**
- Tool: `tracing` subscriber
- Levels: Default `info`, app crate forced to `debug`
- Override: Set `RUST_LOG` environment variable

**Validation:**
- Client-side: HTML5 validation on form inputs
- Server-side: Explicit checks in server functions
  - User auth: `auth.user.ok_or_else(ServerFnError::new(...))?`
  - Team membership: Query DB to verify user belongs to team before allowing mutations
  - Required fields: Deserialize fails naturally for missing JSON fields

**Authentication:**
- Session: Tower-sessions with SurrealDB backend, 30-day inactivity expiry
- User identity: `AppUser` struct extracted via `leptos_axum::extract()`
- Auth check: Protected pages call `get_current_user()` on load, redirect to login if `None`
- Logout: Hard-navigate to `/` via `window.location().set_href()` to clear session

**Theming:**
- CSS custom properties defined in `input.css`
- Dark theme default, light via `[data-theme="light"]`
- 5 accent palettes (yellow, blue, purple, emerald, rose) via `[data-accent="..."]`
- FOUC prevention: Anti-FOUC script in HTML reads localStorage before render

**API Rate Limiting:**
- Riot API: Handled by riven crate (respects rate headers)
- Server functions: No explicit rate limiting; relies on Axum middleware if needed

---

*Architecture analysis: 2026-03-14*
