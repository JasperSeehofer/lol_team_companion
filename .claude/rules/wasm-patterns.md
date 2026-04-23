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

47. **Tailwind v4 `@import "tailwindcss"` 404** — `input.css` starts with `@import "tailwindcss"` which is Tailwind v4 build-time syntax. When not fully processed by the tailwind CLI, the browser resolves it as a relative URL → 404. This is harmless but causes console errors. E2e tests must filter out `404 (Not Found)` console errors.

48. **E2e auth fixture registers then logs in** — Registration now auto-logs in and redirects to `/team/dashboard`. The `authedPage` fixture still does both steps for robustness. After login, `waitForURL("**/team/dashboard")` ensures the hard navigation completes before visiting other pages.

53. **`just` may not be installed** — `just` is a dev dependency. Fall back to running cargo/npx commands directly if `just` is not on PATH.

56. **E2e WASM Effect settle delay** — After registration or login, the hard-nav Effect (`window.location().set_href()`) fires asynchronously via WASM hydration. In Playwright tests, subsequent `page.goto()` calls can be interrupted by this delayed redirect. Fix: add `await page.waitForTimeout(500)` after `waitForURL` to let the Effect fire before navigating.
