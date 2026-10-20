# M160 item 2a — Founder checklist order (mechanical)
File: src/models/guidance.rs. `checklist()` at ~:384 returns a positional vec of 7 steps; `next_step` at ~:470.
Change the order to exactly: 1 "Create or join your team" → 2 "Invite a teammate" → 3 "Fill the starting five" → 4 "Add a first opponent dossier" → 5 "Make a draft" → 6 "Write a game plan" → 7 "Review a game". Keep each step's text, done-rule and status unchanged except:
- Step 1 href: "/team/dashboard" (was "/team/roster"). Step 2 href: t("/team/dashboard#joining-h") (was "/team/roster#joining-h"). Step 1's done status for a team creator may stay as is (do not add region).
- src/server/guidance.rs ~:473 fallback "/team/roster" → "/team/dashboard".
Acceptance:
1. `checklist()` vec order exactly as above (7 items).
2. Update the unit tests in guidance.rs (~:727 a_new_user_starts_at_the_team_step..., ~:738 steps_tick_themselves..., ~:769 joining_an_established_team...) so their index-based asserts match the new order; add one test `invite_comes_before_the_starting_five` asserting items[1].title contains "Invite" and items[2].title contains "starting five", and that next_step for a new team with 1 member and 0 starters is Some(1).
3. tests/integration/guidance.rs ~:157 `start_href == "/team/roster"` → "/team/dashboard".
4. `CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 cargo test --features ssr --lib guidance` passes; cargo fmt clean.
