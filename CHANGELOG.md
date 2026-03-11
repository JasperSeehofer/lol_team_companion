# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- WASM runtime crash caused by `.unwrap()` in event handlers (nav Escape listener, drag-and-drop) that froze all subsequent user interactions

### Added

- SVG-based tree graph visualization (`src/components/tree_graph.rs`) with top-down layout algorithm
- Champion icons on tree graph edges showing picks/bans diff between parent and child nodes
- Ban indicators (red border + cross overlay) vs pick indicators (green border) on edge icons
- List/graph view toggle in tree structure panel header
- CSS custom property-based theming system with semantic color tokens (`input.css`)
- Dark/light mode toggle (moon/sun icon) with localStorage persistence
- Anti-FOUC inline script in HTML head to prevent flash on page load
- Accent color picker with 5 palettes (yellow, blue, purple, emerald, rose)
- Theme toggle component (`src/components/theme_toggle.rs`)

### Changed

- Replaced ~285 hardcoded Tailwind color classes with semantic tokens across all pages and components
- Body class changed from `bg-gray-950` to `bg-base text-primary` for theme support

## [0.7.0] - 2026-03-11

### Added

- Nav dropdowns close on outside click (transparent backdrop), Escape key, and link click
- Notification dropdown with inline accept/decline for join requests
- Team dashboard: coach role slots (2 slots below starting 5)
- Team dashboard: "Leave Team" button for non-leaders with confirmation
- Team dashboard: leader badge (star icon) next to team leader
- Drafts: blue/red side toggle on draft form
- Drafts: auto-populate game plan champion inputs from selected draft
- Champion autocomplete component with Data Dragon icons (`src/components/champion_autocomplete.rs`)
- Champion pool standalone page (`/champion-pool`) with tier system (comfort, match ready, scrim ready, practicing, should be practiced) and notes
- Profile: champion pool summary with link to full pool page
- Tree drafter: "Branch from here" button to create branch from a selected draft position
- Tree drafter: enlarged node +/x buttons with improved hover states

### Fixed

- Removed duplicate "Team Settings" from profile dropdown menu
- Fixed game plan save "Connection uninitialised" error (missing `.check()` on CREATE query)
- Fixed Tree drafter Live Game button not activating immediately

## [0.6.0] - 2026-03-10

### Added

- Dashboard page: team summary, draft/plan/review counts, recent game stats, win rate
- Landing page for unauthenticated users with call-to-action
- Alert banners: pending join requests, no team, missing API key
- Consistent sticky header with backdrop blur, notifications dropdown, user avatar menu
- Mobile-responsive navigation with hamburger menu
- Reusable `ErrorBanner` and `StatusMessage` UI components
- Consistent error display across all pages (standardized to ErrorBanner)

## [0.5.0] - 2026-03-10

### Added

- Post-game review page with structured feedback fields and open-ended notes
- Link post-game reviews to actual matches from stats
- Link post-game reviews to original game plan and draft
- Pattern analysis for recurring insights

## [0.4.0] - 2026-03-10

### Added

- Game plan system for pre-game strategy tied to specific matchups (your 5 champs vs enemy 5)
- Macro strategy section (team-wide) and 5 role-specific sections
- Link game plans to drafts
- Template-based auto-generation of strategy suggestions

## [0.3.0] - 2026-03-09

### Added

- Match history sync from Riot API (manual refresh)
- Filter for games with all 5 roster players
- Stats dashboard with date range and opponent filters
- Clear messaging when RIOT_API_KEY is missing

## [0.2.0] - 2026-03-09

### Added

- Draft tree system at `/tree-drafter` (separate from existing `/draft`)
- Tree data model: `DraftTree` + `DraftTreeNode` with parent/child relationships
- Tree visualization: indented list with expand/collapse
- Node editor: full draft board per node with notes
- Live game navigator: step-by-step branch selection
- Improvisation mode: create branches mid-game, tagged as improvised
- Join request system: players request to join, leader accepts/declines
- Nav badge: red dot on Team link shows pending request count (leaders only)
- Substitute roster: new members land on the bench after joining
- Starter slots: 5 role positions (Top/Jungle/Mid/Bot/Support) with drag-and-drop assignment
- Leader can kick members and edit team name/region
- Role dropdown per bench member

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
