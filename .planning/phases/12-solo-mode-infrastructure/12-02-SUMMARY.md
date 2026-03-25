---
phase: 12-solo-mode-infrastructure
plan: "02"
subsystem: ui
tags: [solo-mode, nav, auth, team-gate, region, profile]
dependency_graph:
  requires:
    - db.set_user_mode
    - db.get_user_mode
    - db.set_user_region
    - model.PublicUser.mode
    - model.PublicUser.riot_region
  provides:
    - nav.ModeToggle
    - nav.set_user_mode_server_fn
    - auth.login_mode_redirect
    - auth.register_solo_redirect
    - team.dashboard_solo_gate
    - team.roster_solo_gate
    - opponents.solo_gate
    - profile.region_dropdown
    - profile.save_region_server_fn
key-files:
  created: []
  modified:
    - src/components/nav.rs
    - src/pages/auth/login.rs
    - src/pages/auth/register.rs
    - src/pages/team/dashboard.rs
    - src/pages/team/roster.rs
    - src/pages/opponents.rs
    - src/pages/profile.rs
---

## What was built

Mode toggle UI and team-only page gates for the solo mode infrastructure.

### Task 1: Mode Toggle in Nav Bar
- Added `set_user_mode` server fn in nav.rs
- ModeToggle component renders Solo/Team pill toggle with accent styling
- Only visible when user is authenticated
- Optimistic UI update + page reload on mode switch

### Task 2: Mode-aware Redirects, Team Gates, Region Dropdown
- **Login**: Returns destination URL based on user mode (`/solo` or `/team/dashboard`)
- **Register**: Always redirects to `/solo` (new users default to solo mode)
- **Team Dashboard, Roster, Opponents**: Show gate banner with "Team feature" heading and "Switch to Team Mode" CTA when user is in solo mode
- **Profile**: Region dropdown with 15 regions (EUW, NA, KR, etc.), saves on change via `save_region` server fn

### Deviations
- Opponents page gate uses `style:display` toggle instead of `Show` component or conditional `into_any()` — the view macro was 900+ lines and nested `view!` blocks inside conditionals caused Leptos macro parser errors with unmatched tags. The CSS toggle approach avoids this entirely.

## Self-Check: PASSED

All acceptance criteria verified:
- login_action returns `Result<String, ServerFnError>` with mode-aware destination
- register_action returns `"/solo"` for new users
- Team Dashboard, Roster, Opponents contain "Team feature" gate text
- Profile contains `save_region` server fn and "Select your region" dropdown
- `cargo check --features ssr` passes
- `cargo check --features hydrate --target wasm32-unknown-unknown` passes
- 90 unit tests pass, 0 failures
