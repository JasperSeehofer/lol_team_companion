# M160 item 5 — Developer copy out of user paths (mechanical)
Helper (reuse, never duplicate): `crate::models::utils::error_text(e: &impl std::fmt::Display) -> String` (src/models/utils.rs:5) strips "error running server function: ".
1. Replace the two developer messages:
   - src/pages/match_detail.rs ~:88 "Riot API key not configured. Ask an admin to add a RIOT_API_KEY." → "Riot features are off on this server right now."
   - src/pages/stats.rs ~:96 "RIOT_API_KEY not configured. Set it in .env to enable match syncing." → "Riot features are off on this server right now. Games stored earlier still show."
   After this, `grep -rn "RIOT_API_KEY\|\.env" src/pages src/components` must find no user-facing string (comments are fine).
2. Use `error_text(&e)` instead of `e.to_string()` / `format!("{e}")` wherever a server-fn error reaches a toast or an inline error, at: src/pages/personal_learnings.rs ~:274,:280,:570; src/pages/share.rs ~:245,:259; src/pages/home.rs ~:692,:1383; src/pages/draft.rs ~:2717; src/pages/admin/invites.rs ~:148,:159; src/pages/opponents.rs ~:756,:938; src/pages/admin/traits.rs ~:266,:284; src/pages/draft_graph.rs ~:596,:754; src/components/survey_prompt.rs ~:204; and every such site in src/pages/team/dashboard.rs (~11) and src/pages/team/roster.rs (~2) (grep `to_string()` there and change only error values). Do NOT change src/pages/community/what_next.rs ~:169 (it matches on the text; leave it). Import error_text the way neighbouring files do.
3. Lint rule in scripts/lint_design.sh: add next to the last rules (faint-outline / legacy-token, ~:65), using the existing `rule name pattern exempt message` form:
   `rule raw-error-text '(Toast::error|ToastKind::Error)[^;]*\b(e|err|error)\.to_string\(\)' '' 'server errors reach users through models::utils::error_text, never e.to_string()'`
   Then `./scripts/lint_design.sh` must print OK (fix any remaining hit by using error_text).
4. cargo fmt; memory permitting: cargo check --features ssr and --features hydrate --target wasm32-unknown-unknown.
5. scripts/ssr_stress.py ~:35: the SIGNED_IN list still has "/team/roster" (now a redirect): replace it with "/team/dashboard" if that is not already in the list, else remove it.
