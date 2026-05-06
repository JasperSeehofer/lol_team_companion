# Phase 19 — Production Hardening (SPEC seed)

**Status:** SEED — produced by the v1.2 → v1.3 pivot on 2026-05-06. Run `/gsd-spec-phase 19` to expand.

**Milestone:** v1.3 Launch Readiness

## Goal (one sentence)

Close all production-readiness gaps surfaced by the audit so the binary is safe to deploy behind HTTPS on Hetzner.

## Why this phase exists

A pre-pivot audit (see `/home/jasper/.claude/plans/i-want-to-pause-groovy-widget.md`) identified seven hard blockers between the current code and a production deploy. None of them are large in isolation; together they justify a dedicated phase so they don't get scattered across other phases or skipped.

## In-scope (the seven hard blockers)

| # | Blocker | Likely fix |
|---|---|---|
| 1 | `site-addr = "127.0.0.1:3020"` baked into `Cargo.toml` metadata — no runtime override | Externalize to env var (`LEPTOS_SITE_ADDR` or read from app config); update `Cargo.toml` to use a dev default that the prod env can override |
| 2 | `cookies.with_secure(false)` in `main.rs:55` — must flip to `true` behind HTTPS | Make secure flag conditional on a new env var (`COOKIE_SECURE=true` for prod). Document in `.env.example` |
| 3 | Riot API resilience — no timeout/retry/cache | Wrap `riven` calls with timeout (5s default), retry-with-backoff (3 attempts, exponential), small in-memory cache for Data Dragon (champion list, version) keyed by patch |
| 4 | Hardcoded Data Dragon patch `15.6.1` (will break on game update) | Fetch latest patch from Data Dragon `versions.json` at startup; cache for process lifetime; refresh on next boot. Also handles CR-Info-1 from Phase 15 review |
| 5 | Graceful shutdown hook | Trap SIGTERM in `main.rs`, drain in-flight requests (axum already supports this), flush SurrealDB if needed, exit within 30s |
| 6 | Backup strategy for SurrealKV `./data` directory | Standalone shell script (`scripts/backup-surreal.sh`) that takes a tarball snapshot, rotates last N (default 7), suitable for cron. Document in deploy phase how to wire to systemd timer |
| 7 | Production log config | `RUST_LOG` defaults to `info,lol_team_companion=debug` for prod; structured tracing fields (request id, user id when available); file-based log rotation via `tracing-appender` (or rely on systemd journal — decide in discuss-phase) |

## Out of scope

- Sentry / OpenTelemetry / Prometheus metrics (deferred to v1.4 — log-based observability is sufficient for closed beta)
- CSRF middleware (Leptos `<Form>` macro provides token; defer explicit middleware to post-launch)
- Rate limiting on app endpoints (closed beta scope is small; defer)
- HTTPS termination (Caddy handles in Phase 20)

## Success criteria (verify with `/gsd-verify-work 19`)

1. Same binary runs locally (`cargo leptos watch`) and on prod (env vars only differ; no recompile required between environments)
2. Riot API outage produces graceful UI fallback (skeleton + "Riot API unreachable, retrying" toast), not a 500
3. Data Dragon version is fetched at startup; restart picks up a new patch
4. Backup script tested locally — restoring a tarball into a clean `./data` rebuilds DB cleanly (tested with a roundtrip test)
5. Server processes SIGTERM and exits within 30s with no in-flight request loss (manual check or integration test)
6. `cargo check --features ssr` and `cargo check --features hydrate --target wasm32-unknown-unknown` both pass
7. `.env.example` updated with all new variables and sane defaults

## Required reading before discuss-phase

1. `src/main.rs` (current Axum + sessions setup)
2. `src/server/riot.rs` and `src/server/data_dragon.rs` (riven client wiring)
3. `Cargo.toml` `[package.metadata.leptos]` block (where site-addr lives)
4. `[[cross-project-memory]]` — feynman-lookup wasm-opt + glibc + nonce learnings
5. `[[cross-project-incidents]]` 2026-04-16 — what kinds of failures we want to catch before deploying

## Plans

TBD — produced by `/gsd-plan-phase 19`. Likely 2-3 plans:
- 19-01: env-driven config (site-addr, cookie secure, log level) + `.env.example`
- 19-02: Riot API resilience (timeout/retry/cache) + dynamic Data Dragon patch
- 19-03: Graceful shutdown + backup script + log structure

## Notes

- DO NOT use `--no-verify` to push through pre-commit hooks (project hard NO per CLAUDE.md / vault `[[guardrails]]`)
- Prefer EU/local tools (per `[[values-charter]]`) — for log shipping later, prefer Loki / Grafana on the same Hetzner over US-based services

---

This SPEC was seeded by the pivot.
