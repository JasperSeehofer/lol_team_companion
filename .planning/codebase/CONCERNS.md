# Concerns

## Technical Debt

### Large File Complexity
| File | Lines | Risk |
|------|-------|------|
| `src/server/db.rs` | 3,243 | Monolithic DB layer — all queries for every domain in one file. Should be split by domain (draft, team, user, etc.) |
| `src/pages/draft.rs` | 2,614 | Complex state management with multiple interconnected signals |
| `src/pages/team/dashboard.rs` | 2,235 | Many responsibilities (roster, notifications, stats, join requests) |
| `src/pages/tree_drafter.rs` | 1,608 | SVG tree rendering + node editing + auto-save in one component |
| `src/pages/game_plan.rs` | 1,515 | Pre-game planning with champion autocomplete + checklist |

These files mix presentation, state management, and server functions. Extracting sub-components and moving server functions to dedicated modules would improve maintainability.

### Schema Evolution
- `schema.surql` (424 lines) is re-applied on every startup with `IF NOT EXISTS` guards
- No migration system — schema changes must be backward-compatible
- Adding/removing fields requires manual coordination between schema, `Db*` structs, and app models

## Security Concerns

### No Rate Limiting
- Login and registration endpoints have no rate limiting
- Riot API calls have no client-side throttling (relies on riven crate's built-in handling)
- Brute force attacks on `/auth/login` are not mitigated

### No Explicit CSRF Protection
- Leptos `ActionForm` uses POST with serialized data, providing some implicit protection
- No explicit CSRF token generation or validation
- Relies on same-origin policy only

### Auth Implementation
- Password hashing uses Argon2 with random salt (strong)
- `#[serde(skip_serializing)]` on `password_hash` prevents leakage (good)
- Session expiry: 30-day inactivity timeout
- No account lockout after failed login attempts
- No password complexity enforcement beyond `minlength=8`

### API Key Exposure
- `RIOT_API_KEY` loaded from `.env` via dotenvy
- No hardcoded secrets found in source
- `.env` is gitignored

## Performance Concerns

### Database
- `db.rs` uses individual queries where batch queries would be more efficient
- No connection pooling configuration (SurrealDB embedded mode may not need it)
- Full tree reassembly on every load (`get_tree_nodes` fetches all nodes, assembles in Rust)
- No query result caching

### Frontend
- Large pages (2000+ lines) compile to substantial WASM bundles
- No code splitting — entire app is one WASM binary
- Tree SVG rendering recalculates layout on every node change (O(N) per update)
- Champion picker loads full champion list on every mount

## Fragile Areas

### Reactive Signal Lifecycle (tree_drafter.rs)
- Auto-save Effect with debounced timer must capture values eagerly
- Node switching requires suppressing auto-save during batch signal updates
- Stale closure captures can save data to wrong node if not handled correctly
- `suppress_autosave` guard pattern is easy to forget when adding new signals

### Draft Board State (draft.rs)
- 20 slots with complex pick/ban ordering
- Blue/red side toggle rebuilds entire state
- Multiple interconnected signals for selections, hover states, confirmations

### Parent-Child Consistency (tree nodes)
- Tree assembly uses `children_of` HashMap for DFS traversal
- Deleting a parent node must cascade to children
- `sort_order` conflicts between siblings can cause rendering issues

### Session/Auth State
- Hard navigation (`window.location().set_href()`) required after login/logout to refresh auth
- WASM Effects fire asynchronously — race conditions possible between auth redirect and page load
- `tower-sessions` version must match `axum-login` exactly (currently 0.14) or sessions silently fail

## Scaling Limitations

- **SurrealDB (SurrealKV):** Embedded, single-node — no horizontal scaling
- **Riot API:** Rate limited per API key — concurrent team usage will hit limits
- **WASM bundle:** Single binary, no lazy loading — grows with every new page
- **No CDN/caching layer:** Static assets served directly by Axum

## Missing Features (Infrastructure)

- No logging aggregation (uses `tracing` crate but stdout only)
- No health monitoring beyond `/healthz` endpoint
- No backup/restore for SurrealKV data directory
- No CI/CD pipeline defined
- No deployment configuration (Docker, systemd, etc.)

## Code Quality

### Positive
- Zero dangerous `.unwrap()` in WASM/event handler code
- No TODO/FIXME/HACK comments in source
- Comprehensive CLAUDE.md with 55+ documented patterns
- Consistent error handling with `thiserror` + `ServerFnError`
- Parameterized queries throughout (no SQL injection risk)

### Needs Improvement
- `db.rs` monolith should be split into domain modules
- Large page components should extract sub-components
- Missing input validation beyond HTML attributes
- No structured logging (log levels exist but no structured format)
