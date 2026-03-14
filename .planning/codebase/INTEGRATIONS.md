# External Integrations

**Analysis Date:** 2025-03-14

## APIs & External Services

**Riot API:**
- Service: Riot Games League of Legends official API
- What it's used for: Player account lookup, match history retrieval, summoner statistics
- SDK/Client: `riven` 2.x crate
- Auth: `RIOT_API_KEY` environment variable
- Key functions in `src/server/riot.rs`:
  - `get_puuid(game_name, tag_line)` - Resolve Riot account to PUUID via account-v1 endpoint
  - `fetch_match_history(puuid, queue_id)` - Fetch recent matches via match-v5 endpoint
  - Data returned: match ID, queue, duration, champion, K/D/A, CS, vision score, damage, win status
  - Regional: EUROPE (hardcoded in endpoints)
- Client initialization: `riven::RiotApi::new(api_key)` per request
- Error handling: Custom `RiotError` type with `thiserror`

**Data Dragon CDN:**
- Service: Riot Games champion data and assets CDN
- What it's used for: Champion metadata (name, title, tags, image URLs)
- SDK/Client: `reqwest` 0.12 HTTP client (direct CDN fetch)
- Auth: None (public CDN)
- URL: `https://ddragon.leagueoflegends.com/`
- Key functions in `src/server/data_dragon.rs`:
  - `fetch_latest_version()` - Get latest patch version from versions.json
  - `fetch_champions()` - Get all champion data from champion.json for a version
  - Image URLs: `https://ddragon.leagueoflegends.com/cdn/{version}/img/champion/{filename}`
- Error handling: Custom `DataDragonError` type with `thiserror`

**Leaguepedia Cargo API:**
- Service: Pro play data and tournament statistics via Leaguepedia (lol.fandom.com)
- What it's used for: Recent tournament games, team picks/bans, match outcomes (non-critical feature)
- SDK/Client: `reqwest` 0.12 HTTP client (direct API fetch)
- Auth: None (public API)
- URL: `https://lol.fandom.com/api.php` (Cargo query interface)
- Key functions in `src/server/leaguepedia.rs`:
  - `fetch_recent_games(tournament, limit)` - Query PicksAndBansS7 table for tournament games
  - Returns: ProGame struct with blue/red team names, picks, winner
  - Error handling: Returns empty list on error (non-critical, errors logged but not surfaced)

## Data Storage

**Database:**
- Provider: SurrealDB 3.x (embedded, no external server)
- Storage engine: SurrealKV (embedded key-value store)
- Connection: Local file-based or in-memory (configured via `SURREAL_DATA_DIR`)
  - Default: `./data` directory
  - Alternative: In-memory for testing (feature: `kv-mem`)
- Client: Rust `surrealdb` 3.x crate
- URL: Built-in, no network required (local file or memory)

**Schema (src/server/db.rs queries + schema.surql):**
- Tables:
  - `user` - App users with Riot account linking (username, email, password_hash, riot_puuid, riot_summoner_name)
  - `team` - Team records (name, region, created_by, created_at)
  - `team_member` - Team roster (team, user, role, roster_type)
  - `join_request` - Team join requests (team, user, status, created_at)
  - `match` - Cached match metadata (match_id, queue_id, game_duration, game_end, team_id)
  - `player_match` - Per-player match stats (match, user, champion, K/D/A, CS, vision, damage, win)
  - `draft` - Draft plans (name, team, created_by, opponent, notes, comments, rating, our_side, tags, win_conditions, watch_out, series_id, game_number, created_at)
  - `draft_action` - Pick/ban actions in a draft (draft, phase, side, champion, order, comment)
  - `game_plan` - Pre-game strategy plans (team, created_by, opponent, notes, champions, created_at)
  - `post_game_learning` - Post-game review (team, created_by, game_plan, outcome, key_learnings, what_went_well, what_failed, action_items, created_at)
  - `tree_draft` - Tree-based draft structure (name, team, created_by, created_at)
  - `tree_node` - Nodes in a draft tree (tree, label, parent_node, sort_order, created_at)
  - `opponent` - Opponent team profiles (name, region, last_seen, created_at)

**File Storage:**
- Not used - All data persisted to SurrealDB

**Caching:**
- Not used - All data is live from SurrealDB

## Authentication & Identity

**Auth Provider:**
- Custom implementation (no third-party IdP)
- Implementation approach:
  - Username/password based (stored in `user` table)
  - Custom `AuthBackend` in `src/server/auth.rs` implementing `axum_login::AuthnBackend`
  - Password hashing: Argon2 (`argon2` 0.5 + `password-hash` 0.5)
  - Session management: `tower-sessions` 0.14 with custom `SurrealSessionStore` (SurrealDB backend)
  - Session serialization: MessagePack via `rmp-serde` (compact storage)
  - Session duration: 30 days on inactivity (configured in `src/main.rs` line 56-58)

**Riot Account Linking (Optional):**
- Users can link Riot account via `game_name#tag_line`
- Stored as `riot_puuid` and `riot_summoner_name` in `user` table
- Used to fetch player stats from Riot API without separate auth

## Monitoring & Observability

**Error Tracking:**
- Not detected - No external error tracking service (Sentry, Rollbar, etc.)

**Logs:**
- Approach: Structured logging via `tracing` 0.1 crate
- Output: stdout with `tracing-subscriber` 0.3
- Formatting: Configured in `src/main.rs` lines 32-38
  - Env-based level control via `RUST_LOG` variable
  - Default level: `info` for dependencies, `debug` for app crate (`lol_team_companion`)
- No external log aggregation (all logs to console)

**Health Check:**
- Endpoint: `GET /healthz` (defined in `src/main.rs`)
- Response: JSON with status and DB health (`{"status": "ok", "db": "ok"|"error"}`)
- Used for liveness checks in production deployments

## CI/CD & Deployment

**Hosting:**
- Not detected - Application is a standalone web server
- Deployment target: Linux (requires `tailwindcss-linux-x64` binary)
- Binary: Built as `target/release/lol_team_companion`
- Assets: Served from `target/site/` directory

**CI Pipeline:**
- Not detected - No GitHub Actions, GitLab CI, or other CI service configured

**Build Process:**
- Development: `cargo leptos watch` (dual SSR + WASM compilation, live-reload on port 3003)
- Production: `cargo leptos build --release` (optimized artifacts)
- Tailwind: Standalone binary build (`tailwindcss-linux-x64`)
- Test: `cargo test --features ssr` (runs all unit + integration tests)

## Environment Configuration

**Required env vars:**
- `RIOT_API_KEY` - Riot API authentication token (for player lookup and match history)

**Optional env vars:**
- `SURREAL_DATA_DIR` - SurrealDB data directory (defaults to `./data` if unset)
- `RUST_LOG` - Log level control (e.g., `debug,lol_team_companion=trace`)
- `BASE_URL` - E2E test base URL (defaults to `http://127.0.0.1:3002`)

**Secrets location:**
- `.env` file (loaded via `dotenvy` on startup)
- Never committed to git (`.gitignore` prevents accidental leaks)

## Webhooks & Callbacks

**Incoming:**
- Not detected - Application does not expose webhook endpoints for external services

**Outgoing:**
- Not detected - Application does not send webhooks to external services

## Additional Integrations

**GitHub (Optional):**
- For MCP (Claude Code extension) access to GitHub API
- Requires: `GITHUB_TOKEN` environment variable in `.env.example`
- Usage: Via Playwright MCP server for PR/issue management
- Not required for core application functionality

**Playwright (E2E Testing):**
- Version: `@playwright/test` 1.49.0
- Purpose: Browser-based E2E tests (smoke tests, feature validation)
- Configuration: `e2e/playwright.config.ts`
- Execution: Manual via `npm test` in `e2e/` directory or `just e2e` from root
- Tests: Located in `e2e/tests/` (not auto-run in CI)
- Features used: Chromium browser, baseURL pointing to localhost dev server

---

*Integration audit: 2025-03-14*
