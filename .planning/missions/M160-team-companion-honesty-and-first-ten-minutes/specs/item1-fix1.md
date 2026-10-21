# M160 item 1 — fix round 1 (from CI run 39616282863, Check SSR target)
CI error on commit 05ddead:
```
error[E0525]: expected a closure that implements the `Fn` trait, but this closure only implements `FnOnce`
   --> src/components/nav.rs:548:5   (the `view! { <Dialog id="phone-menu" ...> ... </Dialog> }` of the phone menu)
560 | {model.groups.into_iter().map(|(heading, pages)| {
    |  ------------ closure is `FnOnce` because it moves the variable `model.groups` out of its environment
```
`Dialog`'s children must be `Fn` (called more than once), so they may not move `model.groups` out.
Fix:
1. In the phone-menu component in src/components/nav.rs (~:520-636), make the children not consume captured values: e.g. `let groups = StoredValue::new(model.groups);` before `view!` and iterate `groups.get_value().into_iter()` (or clone inside: `model.groups.clone()` with `model` captured by reference is NOT enough; the closure must not move a non-Copy value out). Apply the same treatment to any other non-Copy value moved out inside that `Dialog`'s children (inner `pages.into_iter()` inside the map is fine because it is owned by that iteration).
2. Read the other components in nav.rs that were changed in commit 05ddead (`git show 05ddead -- src/components/nav.rs`): the desktop hub bar, the sub-nav, the command bar and the "Find or create a team" link. Wherever a value computed outside is moved into a closure that Leptos calls repeatedly (children of a component such as Dialog/Menu/Show/Suspense/Transition, or a `move ||` reactive closure), fix it the same way. Also check src/components/team_gate.rs: `children: Children` is `FnOnce` and is called once inside `Transition` — Transition's children must be `Fn`, so `children()` inside `<Transition>` will fail the same way: change the prop to `ChildrenFn` and update callers in src/pages/draft.rs, game_plan.rs, post_game.rs only if needed (they pass `view!` children, which works with ChildrenFn).
3. If the host load allows (see pi-common.md), run `cargo check --features ssr` and then `cargo check --features hydrate --target wasm32-unknown-unknown`; fix any further errors in the files changed by 05ddead. Otherwise read carefully.
Change nothing else.
