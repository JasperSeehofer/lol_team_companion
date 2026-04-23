---
paths: ["**/src/pages/**", "**/src/components/**", "**/src/app.rs", "**/src/lib.rs"]
description: Leptos 0.8, server functions, reactivity, and auth patterns for lol_team_companion
---

# Leptos Patterns

## Leptos 0.8 Gotchas

7. **`ActionForm` has no `class` prop** — Wrap it in a `<div>` for styling.

8. **Server redirect doesn't refetch resources** — After login/register, use hard navigation (`window.location().set_href()`) instead of relying on `leptos_axum::redirect` to refresh auth state.

9. **SSR imports inside `#[server]` body** — Put `use` statements for server-only crates (leptos_axum, auth types) inside the `#[server]` function body, not at the top of the file.

10. **`attr:class` on `<A>`** — Use `attr:class="..."` instead of `class="..."` on Leptos router's `<A>` component.

## Server Functions

11. **DB access via context** — Use `use_context::<Arc<Surreal<Db>>>()` inside `#[server]` functions. Do NOT use `axum::extract::State` (it requires `FromRef<()>` which fails).

12. **Auth extraction** — `let mut auth: AuthSession = leptos_axum::extract().await?;` — must be `mut` for `auth.login()`.

32. **Server fn args and return types must be `Serialize + Deserialize`** — All parameters and the `Ok` type cross the wire as JSON (or msgpack). Avoid types like `HashMap` with non-string keys; flatten to `Vec<(K,V)>` or a newtype if needed.

33. **Pass complex data as JSON strings** — When a server fn needs a `Vec<T>` or nested struct that may be hard to encode as a top-level query param, serialize to `String` on the client and deserialize in the fn body (see `actions_json`/`comments_json` pattern in `draft.rs`).

34. **`#[server]` ordering** — Server functions must be defined (or `use`d) before the `#[component]` that calls them in the same file, because the macro generates a client-side stub that must be in scope.

44. **Return `Ok(Vec::new())` not `Err` when an optional resource is absent** — Server functions that list data scoped to a team should return an empty list when the user has no team, not an error. Returning `Err` propagates to the Suspense and replaces the UI with an error banner instead of an empty state:
    ```rust
    let team_id = match db::get_user_team_id(...).await? {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };
    ```

57. **Leptos 0.8 server fn URLs have hash suffixes** — `#[server]` generates URLs like `/api/save_draft9651846416477648343` (not `/api/save_draft`). The hash changes on recompile. curl-based tests must discover real URLs from the WASM binary: `strings target/site/pkg/lol_team_companion.wasm | grep -oP "^/api/${fn_name}[0-9]+"`.

## Reactivity

18. **Clone before multiple closures** — `Vec<T>` and `HashMap<K,V>` don't implement `Copy`. When the same value must be captured by two or more `move` closures, clone before each:
    ```rust
    let role_val_for_class = role_val.clone();
    view! {
        <button class=move || { ... role_val_for_class.clone() ... }
                on:click=move |_| set_x.set(role_val.clone())>
    ```

19. **`into_any()` for divergent view branches** — When `if/else` or `match` arms inside `{move || ...}` return structurally different view types, each arm must call `.into_any()`:
    ```rust
    {move || if filled {
        view! { <img ... /> }.into_any()
    } else {
        view! { <span>...</span> }.into_any()
    }}
    ```

20. **`get_untracked()` in event handlers** — Inside `on:click`, `on:input`, etc., read signals with `get_untracked()` to avoid accidentally registering reactive dependencies in a non-tracking context.

21. **`prop:value` for controlled inputs** — `attr:value` only sets the initial DOM attribute. For a controlled input that reflects signal changes after render, use `prop:value`:
    ```rust
    <input prop:value=move || signal.get()
           on:input=move |ev| set_signal.set(event_target_value(&ev)) />
    ```

22. **`StoredValue` for non-reactive data shared across closures** — When you need to share a large non-`Copy` value (like a `HashMap`) across multiple closures without reactive tracking overhead, use `StoredValue::new()`.

23. **`resource.refetch()` after mutations** — `Resource::new` does not auto-refetch after a server fn mutates data. Call `resource.refetch()` inside the `spawn_local` success branch to refresh lists.

24. **`spawn_local` for async event handlers** — The only way to call an async server function from a sync `on:click` handler. Errors must be handled inside.

25. **`collect_view()` for iterators** — Use `.collect_view()` instead of `.collect::<Vec<_>>()` when building fragments from iterators inside `view!`.

26. **`<For>` key must be stable** — Always use a stable entity ID (e.g. `c.id`), never the array index. Unstable keys cause unnecessary re-renders or DOM thrash.

54. **Auto-save Effects must capture values eagerly** — Capture ALL signal values in the Effect body (eagerly), not inside the timer callback (lazily). Lazy reads inside the callback run after a delay and may read signals updated by a node/tree switch:
    ```rust
    Effect::new(move |_| {
        let val = my_signal.get();   // eagerly tracked + captured
        let id = selected_id.get();  // eagerly tracked + captured
        let cb = Closure::once(move || {
            spawn_local(async move { save(id, val).await; });
        });
    });
    ```

55. **Suppress auto-save during batch signal updates** — When switching nodes/trees, multiple signals update in sequence. Use a `suppress_autosave: RwSignal<bool>` guard: set `true` before batch updates, re-enable after a `setTimeout(0)` microtask.

## Auth / Routing

49. **Logout hard-navigates to `/`** — Both `nav.rs` and `profile.rs` watch `logout_action.value()` and call `window.location().set_href("/")` on success (same pattern as login, rule 8).

50. **Protected pages redirect to `/auth/login`** — All protected page components fetch `get_current_user()` on mount and redirect via `window.location().set_href()` if the user is `None`. The redirect is client-side only (`#[cfg(feature = "hydrate")]`).

51. **Registration auto-logs in** — `register_action()` calls `auth.authenticate()` + `auth.login()` after creating the user, then redirects to `/team/dashboard`.

52. **Auth-aware nav links** — The nav only shows Team/Draft/Tree Drafter/Stats/Game Plan/Post Game links when the user is authenticated. The `is_authed` signal derives from the `user` Resource.

## Toolchain

38. **`recursion_limit = "512"` in `lib.rs` and `main.rs`** — The deeply nested Leptos view types in `post_game.rs` exceed the default limit of 128. Both files have `#![recursion_limit = "512"]`. Do not lower it.
