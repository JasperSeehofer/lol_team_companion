# Technology Stack

**Analysis Date:** 2025-03-14

## Languages

**Primary:**
- Rust nightly - Full-stack development (server binary and WASM client compilation)
- SurrealQL - Database query language (schema and queries in `schema.surql`, queries in `src/server/db.rs`)

**Secondary:**
- TypeScript - E2E test framework (Playwright tests in `e2e/tests/`)
- CSS - Styling via Tailwind (input.css with custom theme tokens)

## Runtime

**Environment:**
- Rust nightly (specified in `rust-toolchain.toml`)
- Tokio 1.x - Async runtime for server binary

**Package Manager:**
- Cargo - Rust package manager with workspace-like single-crate structure
- npm - JavaScript package manager for E2E tests (`e2e/package.json`)

**Lockfile:**
- `Cargo.lock` - Rust dependencies (present)
- `e2e/package-lock.json` - JavaScript dependencies (present)

## Frameworks

**Core (Shared):**
- Leptos 0.8 - Full-stack reactive UI framework with SSR and WASM hydration
  - `leptos_router` 0.8 - Routing for both server and client
  - `leptos_meta` 0.8 - Meta tag management (titles, descriptions)

**Server (SSR feature):**
- Axum 0.8 - HTTP web framework built on Tower
  - `leptos_axum` 0.8 - Leptos integration with Axum server
  - `axum_login` 0.17 - Session-based authentication middleware
  - `tower-sessions` 0.14 - Session management (version critical: must match axum-login)
  - `tower-sessions-core` 0.14 - Session storage traits

**Database:**
- SurrealDB 3.x (SurrealKV storage engine) - Multi-table relational schema with typed queries
  - `surrealdb` 3 - Rust client library
  - `surrealdb-types-derive` 3 - Macro for deriving `SurrealValue` on DB result structs

**Testing:**
- Playwright `@1.49.0` - E2E browser testing (tests in `e2e/tests/`)

**Build/Dev:**
- `cargo-leptos` - Build tool orchestrating dual SSR + WASM compilation
- Tailwind CSS v4 - Standalone binary for CSS processing (no npm required)
  - Referenced as `tailwindcss-linux-x64` binary in `[package.metadata.leptos]`
- `just` - Command runner for build/test/dev tasks (optional, fallback to cargo)

## Key Dependencies

**Critical (Direct API Integrations):**
- `riven` 2.x - Riot API client library for League of Legends stats
  - Used in `src/server/riot.rs` for match history, account lookup
  - Auth: `RIOT_API_KEY` environment variable (loaded via `dotenvy`)

**Infrastructure:**
- `reqwest` 0.12 - HTTP client with JSON support
  - Used for Data Dragon CDN (`ddragon.leagueoflegends.com`)
  - Used for Leaguepedia Cargo API (`lol.fandom.com`)
- `tokio` 1.x - Async runtime with full feature set (TLS, networking, timers)
- `serde` 1.x - Serialization/deserialization framework (derive macros for JSON)
- `serde_json` 1.x - JSON handling (queries, responses)

**Auth & Security:**
- `argon2` 0.5 - Password hashing algorithm
- `password-hash` 0.5 - Password hashing traits and utilities

**Observability:**
- `tracing` 0.1 - Structured logging framework (async compatible)
- `tracing-subscriber` 0.3 - Logging output formatting with environment-based level control

**Client (WASM):**
- `wasm-bindgen` 0.2 - Rust-to-JavaScript FFI
- `console_error_panic_hook` 0.1 - WASM panic handler (logs to browser console)
- `web-sys` 0.3 - Web APIs bindings
  - Features: `Window`, `Document`, `Element`, `HtmlElement`, `Node`, `Storage`, `DragEvent`, `DataTransfer`

**Utilities:**
- `thiserror` 2.x - Derive macro for error types (custom error implementations)
- `chrono` 0.4 - Date/time handling with serde support
- `async-trait` 0.1 - Async trait methods (used for session store trait)
- `dotenvy` 0.15 - `.env` file loading (optional, loaded on startup)
- `rmp-serde` 1.x - MessagePack serialization for session storage

## Configuration

**Environment:**
- `.env` file (loaded via `dotenvy::dotenv().ok()` in `src/main.rs`)
- Critical var: `RIOT_API_KEY` (Riot API authentication)
- Optional var: `SURREAL_DATA_DIR` (SurrealDB data directory, defaults to `./data`)
- Logging: `RUST_LOG` env var controls log level (default: `info`, app crate: `debug`)

**Build:**
- `[package.metadata.leptos]` in `Cargo.toml` specifies:
  - Output: `target/site` (compiled assets)
  - Assets directory: `public/`
  - Server address: `127.0.0.1:3002`
  - Reload port: `3003` (for live-reload during development)
  - Tailwind input: `input.css` (processed to `target/site/style.css`)
  - Standalone Tailwind: No npm required

**Features:**
- `ssr` - Server-side rendering (SSR binary target)
  - Activates: axum, tokio, surrealdb, auth, session, riot integration
  - Builds: server binary in `target/release/lol_team_companion`
- `hydrate` - WASM client hydration (browser target)
  - Activates: leptos hydration, wasm-bindgen, console panic hook
  - Builds: `.wasm` bundle served from `target/site/pkg/`
- `default` - Empty (no features enabled by default)

**Build Profiles:**
- `wasm-release` profile:
  - Opt-level: `z` (minimize size for WASM)
  - LTO: enabled
  - Codegen units: 1 (maximum optimization)

## Compiler Workarounds

**Linker:**
- `.cargo/config.toml` forces `bfd` linker (not `rust-lld`)
- Workaround for LLVM 22 lld crash on Rust nightly

**Recursion Limit:**
- `#![recursion_limit = "512"]` in `src/lib.rs` and `src/main.rs`
- Required for deeply nested Leptos view types in `src/pages/post_game.rs`

## Platform Requirements

**Development:**
- Rust nightly (from `rust-toolchain.toml`)
- Cargo + cargo-leptos plugin (`cargo install cargo-leptos`)
- WASM target: `wasm32-unknown-unknown` (installed via rustup)
- Tailwind CSS standalone binary (Linux x64)
- Optional: `just` command runner (fallback to `cargo`/`npm`)

**Production:**
- Linux server (binary expects Linux for `tailwindcss-linux-x64`)
- SurrealDB SurrealKV backend (embedded in binary via `kv-surrealkv` feature)
- No external services required (except Riot API and optional services)

---

*Stack analysis: 2025-03-14*
