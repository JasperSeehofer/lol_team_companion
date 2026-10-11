---
paths: ["**/src/**/*.rs", "**/e2e/**", "**/*.spec.ts"]
description: WASM safety, browser testing gotchas, and tooling patterns for lol_team_companion
---

# WASM & Browser Patterns

## WASM Safety

35. **Never `.unwrap()` in event handlers or WASM code** — A panic in WASM crashes the entire runtime, freezing all subsequent user interactions. Use `if let Some(...)`, `let Some(...) = ... else { return }`, or `.unwrap_or_default()`:
    ```rust
    // BAD: crashes WASM runtime if window() returns None
    let window = web_sys::window().unwrap();
    // GOOD:
    if let Some(window) = web_sys::window() { ... }
    ```

36. **`Callback::new()` for closures shared across reactive contexts** — Regular closures are not `Copy`. When a closure must be used in multiple `move` closures, wrap it in `Callback::new()` which is `Copy`.

37. **`wasm_bindgen::closure::Closure` for JS timers** — Use `Closure::once` + `web_sys::window().set_timeout_*` instead of depending on `gloo_timers`. Always guard with `#[cfg(feature = "hydrate")]`:
    ```rust
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::prelude::*;
        let cb = Closure::once(move || { ... });
        if let Some(win) = web_sys::window() {
            let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(), 150,
            );
        }
        cb.forget();
    }
    ```

42. **Debounced auto-save with cancellable timer** — Store the JS timer handle in a `RwSignal<Option<i32>>`. In the `Effect`, cancel any pending timer before scheduling a new one:
    ```rust
    #[allow(unused_variables)]
    let auto_save_timer: RwSignal<Option<i32>> = RwSignal::new(None);

    Effect::new(move |_| {
        let _ = some_signal.get(); // track

        #[cfg(feature = "hydrate")]
        if let Some(id) = auto_save_timer.get_untracked() {
            if let Some(win) = web_sys::window() { win.clear_timeout_with_handle(id); }
        }

        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::prelude::*;
            let cb = Closure::once(move || { /* do save */ });
            if let Some(win) = web_sys::window() {
                if let Ok(id) = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(), 2000,
                ) { auto_save_timer.set(Some(id)); }
            }
            cb.forget();
        }
    });
    ```

43. **`#[allow(unused_variables)]` on hydrate-only signals** — A `RwSignal` only read/written inside `#[cfg(feature = "hydrate")]` blocks triggers unused-variable warnings in SSR builds. Suppress it on the `let` declaration.

## Browser Testing Gotchas

45. **`WebFetch` cannot reach localhost** — It auto-upgrades HTTP to HTTPS, which fails for `127.0.0.1`. Use `curl` via Bash for fetching local pages, or agent-browser / e2e tests for browser interaction.

46. **Extracting text from Leptos SSR HTML** — The HTML contains large inline `<script>` blocks (hot-reload, hydration). Use Python's `HTMLParser` to strip scripts and extract visible text content. Raw `sed` approaches break on multi-line scripts.

47. **No `style-file` next to `tailwind-input-file`** — `input.css` starts with `@import "tailwindcss"`, which only the Tailwind CLI understands. When `Cargo.toml` also listed it as `style-file`, cargo-leptos shipped the raw import and the browser requested `/pkg/tailwindcss` (404 on every page). Keep only `tailwind-input-file`. `smoke.spec.ts` asserts there is no `/pkg/` 404.

48. **E2e auth fixture registers and lands on `/solo`** — Registration requires `?invite=...` (otherwise it redirects to `/closed-beta`), auto-logs in and hard-navigates to `/solo` because new users start in solo mode. Login goes to `/solo`, or `/team/dashboard` when the user's mode is `team`. The `authedPage` fixture waits for `**/solo`; `teamPage` then creates a team via `/team/roster`.

53. **`just` may not be installed** — `just` is a dev dependency. Fall back to running cargo/npx commands directly if `just` is not on PATH.

56. **E2e WASM Effect settle delay** — After registration or login, the hard-nav Effect (`window.location().set_href()`) fires asynchronously via WASM hydration. In Playwright tests, subsequent `page.goto()` calls can be interrupted by this delayed redirect. Fix: add `await page.waitForTimeout(500)` after `waitForURL` to let the Effect fire before navigating.
