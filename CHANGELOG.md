# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-03-09

### Added

- Project scaffold with Leptos 0.8, Axum 0.8, and SurrealDB 2.x (SurrealKV)
- User authentication (register, login, sessions) with axum-login and argon2 password hashing
- User profile page with Riot account linking via PUUID
- Team creation, dashboard, and roster management
- Team builder page
- Draft planner with champion picker component and draft board UI
- Match stats page with player match history from Riot API (riven crate)
- Game plan page for pre-game strategy (win conditions, objective priority, teamfight strategy)
- Post-game review page (what went well, improvements, action items)
- Data Dragon integration for champion metadata and images
- Leaguepedia module for pro play data (not yet wired to UI)
- Tailwind CSS v4 with dark theme (standalone binary, no npm)
- Environment config via dotenvy (.env file support)
- Full SurrealDB schema with tables: user, team, team_member, match, player_match, draft, draft_action, game_plan, post_game_learning, sessions

### Fixed

- SurrealDB 2.x compatibility: migrated from `type::thing()` to `type::record()` throughout all queries
- SurrealDB 2.x compatibility: removed `string()` casts, use `RecordId::to_sql()` in Rust instead
- SurrealDB RecordId deserialization: introduced `Db*` intermediate structs to avoid `serde_json::Value` failures
- Linker configuration: added `.cargo/config.toml` with bfd linker to work around LLVM 22 lld crash on nightly
- Tailwind binary: switched to standalone `tailwindcss-linux-x64` to eliminate npm dependency
- Auth state refresh: use hard navigation (`window.location().set_href()`) after login/register instead of server redirect
- ActionForm styling: wrap in `<div>` since Leptos 0.8 `ActionForm` does not support `class` prop
- Session store: implement `SessionStore::create` to properly handle ID collisions instead of relying on default `save`-based fallback
