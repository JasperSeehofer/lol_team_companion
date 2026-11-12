# M160 item 6 — LoadFailed instead of hand-written or empty states (mechanical)
Primitive (reuse): `LoadFailed(what: String, on_retry: Callback<()>)` in src/components/ui/status.rs:292, re-exported from crate::components::ui. Example: src/pages/champion_pool.rs ~:512 `<LoadFailed what="Your pool" on_retry=Callback::new(move |_| pool.refetch()) />`.
1. Replace these hand-written error arms with `<LoadFailed what=".." on_retry=Callback::new(move |_| <the resource that failed>.refetch()) />` (use `.into_any()` on match arms as the file does):
   - src/pages/opponents.rs ~:514 ("The opponents did not load.") what="The opponents"; ~:570 ("The dossier did not load.") what="The dossier".
   - src/pages/stats.rs ~:362 what="The team's games".
   - src/pages/personal_learnings.rs ~:308 what="Your learnings".
   - src/pages/post_game.rs ~:1033 what="The reviews"; ~:1556 what="The linked plan".
   - src/pages/draft.rs ~:2343 (notes) what="The notes"; ~:3001 what="Ban priorities"; ~:3056 what="Tendencies"; ~:3152 what="Outcomes".
2. Silent swallows that render an empty list on a server error — make the Err case render LoadFailed instead (keep Ok behaviour identical):
   - src/pages/champion_pool.rs ~:378 (champions), ~:525 (stats), ~:1072 (notes list).
   - src/pages/personal_learnings.rs ~:598 (champions list).
   If a site cannot get at its resource for refetch without restructuring, leave it and list it in your summary.
3. Do NOT touch game_plan.rs, the other post_game.rs/draft.rs `unwrap_or_default` sites, dashboard.rs, roster.rs or nav.rs (another mission edits them; they are listed for a later round).
4. cargo fmt; memory permitting: cargo check --features ssr and hydrate wasm; ./scripts/lint_design.sh.
