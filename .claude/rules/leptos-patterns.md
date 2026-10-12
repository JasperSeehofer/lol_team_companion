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

11. **DB access and auth via `authz`** — Start protected server fns with `let authz::Ctx { db, user } = authz::authed().await?;` (DB handle from context + signed-in user) and derive the team with `authz::require_team(&db, &user)` / `current_team_for`. `authz::db()` alone is for deliberately public fns. Do NOT use `axum::extract::State` (it requires `FromRef<()>` which fails). Map errors with `.map_err(authz::server_err)`.

12. **Auth extraction for login/logout** — only login, register and logout touch `AuthSession` directly: `let mut auth: AuthSession = leptos_axum::extract().await?;` — must be `mut` for `auth.login()`.

32. **Server fn args and return types must be `Serialize + Deserialize`** — All parameters and the `Ok` type cross the wire as JSON (or msgpack). Avoid types like `HashMap` with non-string keys; flatten to `Vec<(K,V)>` or a newtype if needed.

33. **Pass complex data as JSON strings** — When a server fn needs a `Vec<T>` or nested struct that may be hard to encode as a top-level query param, serialize to `String` on the client and deserialize in the fn body (see `actions_json`/`comments_json` pattern in `draft.rs`).

34. **`#[server]` ordering** — Server functions must be defined (or `use`d) before the `#[component]` that calls them in the same file, because the macro generates a client-side stub that must be in scope.

44. **Return `Ok(Vec::new())` not `Err` when an optional resource is absent** — Server functions that list data scoped to a team should return an empty list when the user has no team, not an error. Returning `Err` propagates to the Suspense and replaces the UI with an error banner instead of an empty state:
    ```rust
    let Some(team_id) = authz::current_team_for(&db, &user).await? else {
        return Ok(Vec::new());
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

61. **Window listeners kept in a `StoredValue` need a re-entry guard** — Storing a new `Closure` with `set_value` drops the previous one while the browser still calls it ("closure invoked ... after being dropped" on every later event). Return early when the mode is already on, and end the mode on route change (`bug_report_widget.rs` `start_select`, M7 review).

62. **Live regions exist before their text; focus returns to something that still exists** — An `aria-live` or `role="status"` element inserted together with its content is often not announced: keep the region mounted and change what is inside (`survey_prompt.rs`). `Dialog` returns focus to the element focused before it opened; when that was a transient control (a hint bar button), focus the stable trigger after close (`bug_report_widget.rs` `focus_trigger`).

## Auth / Routing

49. **Logout hard-navigates to `/`** — Both `nav.rs` and `profile.rs` watch `logout_action.value()` and call `window.location().set_href("/")` on success (same pattern as login, rule 8).

50. **Protected pages redirect to `/auth/login`** — All protected page components fetch `get_current_user()` on mount and redirect via `window.location().set_href()` if the user is `None`. The redirect is client-side only (`#[cfg(feature = "hydrate")]`).

51. **Registration auto-logs in** — `register_action()` calls `auth.authenticate()` + `auth.login()` after creating the user, redirects on the server (`leptos_axum::redirect("/")`, for a submit before hydration) and returns `/`, where the hydrated page hard-navigates. Sign-in does the same. Home is Match Day for team members and a start page otherwise (there is no stored mode since migration 12).

52. **Auth-aware nav** — The nav's user cluster (join requests, account menu, phone "Menu" button) derives from the `user` Resource; hub links show only when signed in. One table, `PAGES` in `nav.rs` (+ `MENU_GROUPS` for the phone menu), drives the hubs, the sub-nav, the phone menu and the command bar.

58. **Create every Resource before the first Suspense/Suspend** — A `Resource` created inside `Suspend::new(async { x.await ... })`, or in a component rendered there, gets its id after an await on the server but at hydration time in the browser. That shifts the ids of every later resource on the page and breaks hydration ("expected a marker node"), depending on timing. Create such resources in the page component before any `<Suspense>` and pass them down as props (M5: team page, roster, My games, phone menu counts).

59. **Do not read a resource inside an open `<dialog>` that lives in a Suspense** — The read suspends the enclosing Suspense when the resource refetches, which remounts the open dialog; a re-inserted `<dialog>` keeps `open` but leaves the top layer (no scrim, not modal). Copy the value into a signal with an Effect and read the signal (the phone menu counts in `nav.rs`).

60. **No server-rendered `<Redirect>`** — A Leptos `<Redirect>` rendered during SSR left its 302 status in a response context that later static-file responses on the same worker picked up (`/pkg/*.js` and `.wasm` returned 302 at random). Redirect old addresses as axum routes in `main.rs`, or from a server function with `leptos_axum::redirect`.

## Toolchain

38. **`recursion_limit = "512"` in `lib.rs` and `main.rs`** — The deeply nested Leptos view types in `post_game.rs` exceed the default limit of 128. Both files have `#![recursion_limit = "512"]`. Do not lower it.
