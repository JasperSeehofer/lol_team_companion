# M160 item 6 — fix round 1 (from the Sonnet verifier; commit 7ec401c)
All sites are converted correctly. One clippy failure (both targets, -D warnings): `needless_return` at src/pages/champion_pool.rs ~:535-537.
1. In that match arm drop `return` and the trailing `;` so the arm is the expression `view! { <LoadFailed what="Stats" on_retry=Callback::new(move |_| stats.refetch()) /> }.into_any()`. Check the rest of the commit's hunks for any other `return ...;` used as the last expression of an arm or closure and fix it the same way.
2. Run `cargo fmt`, then (memory permitting, per pi-common.md) `cargo clippy --features ssr -- -D warnings` and `cargo clippy --features hydrate --target wasm32-unknown-unknown -- -D warnings`; both must pass. Change nothing else.
