# M160 item 7 — Delete the dead server functions (mechanical)
1. Re-grep callers first: `grep -rn "fetch_champions\|get_team_champion_stats" src tests e2e` (also `FetchChampions`, `GetTeamChampionStats` — the #[server] structs). If anything but the definitions appears, STOP and report it.
2. Delete `fetch_champions` (src/pages/opponents.rs ~:146-197, incl. its doc comment and #[server] attribute) and `get_team_champion_stats` (src/pages/draft.rs ~:381-405).
3. Then for each helper they used — `db::get_team_region`, `db::update_opponent_player_champions`, the Riot client method `fetch_player_champions` (src/server/riot.rs or wherever), `db::get_champion_stats_for_user`, `db::get_user_team_with_members`, `db::ensure_team_opponent_player` — grep callers in src tests e2e; delete a helper only when it now has zero callers (and its tests then go too). Remove imports that become unused.
4. Do NOT delete `update_tree_meta` (tree_drafter.rs) — a later item wires it.
5. cargo fmt; memory permitting: cargo check --features ssr and hydrate wasm (no unused warnings left). In your summary list exactly what was deleted and what was kept and why.
