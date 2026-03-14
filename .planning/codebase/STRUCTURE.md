# Codebase Structure

## Top-Level Layout

```
lol_team_companion/
├── src/                    # Application source (single crate, dual target)
│   ├── main.rs             # Axum server setup, DB init, session/auth layers (97 lines)
│   ├── app.rs              # Router, shell, all <Route> definitions (77 lines)
│   ├── lib.rs              # Crate root, hydration entry point (20 lines)
│   ├── error_template.rs   # Fallback error page (51 lines)
│   ├── server/             # SSR-only modules (behind #[cfg(feature = "ssr")])
│   ├── models/             # Shared types (compiled for both SSR and WASM)
│   ├── pages/              # Route components with #[server] functions
│   └── components/         # Reusable UI components
├── tests/                  # Integration tests (SurrealDB in-memory)
├── e2e/                    # Playwright end-to-end tests
├── .cargo/                 # Cargo config (BFD linker override)
├── schema.surql            # SurrealDB schema (424 lines, ~10 tables)
├── input.css               # Tailwind v4 with @theme custom properties
├── Cargo.toml              # Single crate with ssr/hydrate features
├── justfile                # Dev commands
├── .env.example            # RIOT_API_KEY, SURREAL_DATA_DIR
└── rust-toolchain.toml     # Rust nightly pin
```

## Source Directory Detail

### `src/server/` — SSR-Only (8 files, 3,876 lines)

| File | Lines | Purpose |
|------|-------|---------|
| `db.rs` | 3,243 | All SurrealDB queries, `Db*` struct definitions, `DbError` type |
| `auth.rs` | 210 | Argon2 hashing, `AuthBackend` trait impl, `Credentials` |
| `session_store.rs` | 133 | `SurrealSessionStore` for tower-sessions |
| `riot.rs` | 137 | Riot API client wrapper (riven crate) |
| `data_dragon.rs` | 66 | Champion metadata from Data Dragon CDN |
| `leaguepedia.rs` | 63 | Pro play data scraping |

### `src/pages/` — Route Components (13 files, 10,511 lines)

| File | Lines | Route |
|------|-------|-------|
| `draft.rs` | 2,614 | `/draft` — Draft planner with blue/red side |
| `team/dashboard.rs` | 2,235 | `/team/dashboard` — Roster, notifications, stats |
| `tree_drafter.rs` | 1,608 | `/tree-drafter` — Tree-based draft planning |
| `game_plan.rs` | 1,515 | `/game-plan` — Pre-game strategy |
| `post_game.rs` | ~800 | `/post-game` — Post-game review |
| `stats.rs` | ~400 | `/stats` — Match history |
| `champion_pool.rs` | ~350 | `/champion-pool` — Tier-based pool management |
| `team/roster.rs` | 414 | `/team/roster` — Team creation/joining |
| `profile.rs` | ~200 | `/profile` — Riot account linking |
| `home.rs` | ~150 | `/` — Landing page |
| `auth/login.rs` | 90 | `/auth/login` — Login form |
| `auth/register.rs` | 112 | `/auth/register` — Registration form |

### `src/components/` — Reusable UI (8 files)

| File | Lines | Purpose |
|------|-------|---------|
| `nav.rs` | 748 | Top nav bar, notifications, auth state |
| `tree_graph.rs` | 709 | SVG tree visualization with champion icons |
| `draft_board.rs` | 562 | 20-slot draft board (picks + bans) |
| `champion_picker.rs` | — | Grid-based champion selection |
| `champion_autocomplete.rs` | — | Text input with champion dropdown |
| `theme_toggle.rs` | — | Dark/light mode + accent color picker |
| `stat_card.rs` | — | Stat display card |
| `ui.rs` | — | ErrorBanner, StatusMessage helpers |

### `src/models/` — Shared Types (11 files)

All models derive `Serialize, Deserialize, Clone`. Compiled for both SSR and WASM.

| File | Types |
|------|-------|
| `user.rs` | `AppUser`, `PublicUser`, `TeamMember` |
| `team.rs` | `Team`, `JoinRequest` |
| `draft.rs` | `Draft`, `DraftAction` |
| `champion.rs` | `Champion`, champion metadata |
| `game_plan.rs` | `GamePlan`, `PostGameLearning` |
| `match_data.rs` | `PlayerMatchStats` |
| `opponent.rs` | Opponent scouting data |
| `series.rs` | Series/match series tracking |
| `team_note.rs` | Team notebook entries |
| `action_item.rs` | Cross-feature action items |

## Configuration Files

| File | Purpose |
|------|---------|
| `Cargo.toml` | Dependencies, `[features]` for ssr/hydrate, `[package.metadata.leptos]` build config |
| `.cargo/config.toml` | Forces BFD linker (workaround for LLVM 22 lld crash) |
| `rust-toolchain.toml` | Pins Rust nightly version |
| `schema.surql` | SurrealDB schema loaded on startup via `include_str!` |
| `input.css` | Tailwind v4 entry with `@import "tailwindcss"` and `@theme` custom properties |
| `.env.example` | `RIOT_API_KEY`, `SURREAL_DATA_DIR`, `GITHUB_TOKEN` |
| `justfile` | Dev workflow commands (check, test, e2e, verify) |
| `e2e/playwright.config.ts` | Playwright test configuration |

## Naming Conventions

- **Files:** `snake_case.rs` (e.g., `champion_pool.rs`, `data_dragon.rs`)
- **Modules:** `snake_case` matching file names
- **Components:** `PascalCase` functions with `#[component]` (e.g., `ChampionPoolPage`)
- **Server functions:** `snake_case` with `#[server]` (e.g., `get_current_user`)
- **DB structs:** `Db` prefix (e.g., `DbTeam`, `DbDraft`) — SSR only
- **App structs:** No prefix (e.g., `Team`, `Draft`) — shared

## Adding New Code

### New Page
1. Create `src/pages/my_page.rs` with `#[component] pub fn MyPage()`
2. Add `pub mod my_page;` to `src/pages/mod.rs`
3. Add `<Route>` in `src/app.rs`
4. Add nav link in `src/components/nav.rs` if needed
5. Add e2e test entry in `e2e/tests/pages.spec.ts`

### New DB Table
1. Add `DEFINE TABLE` / `DEFINE FIELD` to `schema.surql`
2. Add `Db*` struct + query functions to `src/server/db.rs`
3. Add shared model struct to `src/models/`

### New Component
1. Create `src/components/my_component.rs`
2. Add `pub mod my_component;` to `src/components/mod.rs`
3. Import in pages as needed
