use leptos::prelude::*;

/// Persist the user's theme preference to the DB user record.
/// Mirrors `set_user_mode` in nav.rs (Phase 12 precedent, D-04 in 16-CONTEXT.md).
/// Pre-validates the theme string before the DB roundtrip; the SurrealDB
/// schema also enforces `ASSERT $value IN ['demacia', 'pandemonium']` for
/// defense-in-depth.
#[server]
pub async fn set_user_theme(theme: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    if theme != "demacia" && theme != "pandemonium" {
        return Err(ServerFnError::new("Invalid theme"));
    }

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;
    db::set_user_theme(&db, &user.id, &theme)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

/// 2-state theme toggle (demacia / pandemonium). Replaces the old 5-accent
/// picker (D-04 retired). The user's preference is persisted on the `user`
/// DB record and survives hard navigation; SSR sets `<html data-theme>`
/// authoritatively (no FOUC).
///
/// Optimistic UX: clicking flips `data-theme` on the document element
/// immediately, then `spawn_local`s the server fn for DB persistence. Per
/// wasm-patterns rule 35, no panicking unwraps — every web_sys access is
/// guarded with `if let Some(...)` chains.
#[component]
pub fn ThemeToggle(
    #[prop(optional, default = String::from("demacia"))] initial_theme: String,
) -> impl IntoView {
    let current_theme = RwSignal::new(initial_theme);

    // Callback is Copy-safe to use in iterator-style closures
    let set_theme = Callback::new(move |theme: String| {
        // Reject invalid client-side too
        if theme != "demacia" && theme != "pandemonium" {
            return;
        }

        let theme_for_signal = theme.clone();
        current_theme.set(theme_for_signal);

        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsCast;

            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                // Phase 18.1: write the lol_companion_theme cookie
                // SYNCHRONOUSLY in the same JS turn as the optimistic
                // data-theme attribute update. This must happen BEFORE
                // the `spawn_local` for the DB persist, so even a
                // sub-100ms navigation after click honours the new
                // value (closes the first-toggle race documented in
                // 18.1-CONTEXT.md risk surface).
                //
                // Cookie attributes are LOCKED per D-01:
                //   Path=/; Max-Age=31536000; SameSite=Lax
                // NO HttpOnly — the client must read this cookie for
                // the SSR-authoritative initial paint after reload.
                // Per wasm-patterns rule 35: no `.unwrap()` in any
                // hydrate code; every step is guarded.
                if let Ok(html_doc) = doc.clone().dyn_into::<web_sys::HtmlDocument>() {
                    let _ = html_doc.set_cookie(&format!(
                        "lol_companion_theme={}; Path=/; Max-Age=31536000; SameSite=Lax",
                        theme
                    ));
                }

                // Optimistic DOM update so the swap is instant.
                if let Some(root) = doc.document_element() {
                    let _ = root.set_attribute("data-theme", &theme);
                }
            }

            // Persist to DB so the choice survives hard navigation + logout.
            let theme_for_server = theme.clone();
            leptos::task::spawn_local(async move {
                let _ = set_user_theme(theme_for_server).await;
            });
        }

        // Suppress unused warning when not in hydrate
        #[cfg(not(feature = "hydrate"))]
        let _ = theme;
    });

    view! {
        <div
            class="inline-flex items-center gap-0 rounded-full border border-outline/40 bg-surface/60 p-0.5"
            role="group"
            aria-label="Theme toggle"
        >
            <button
                type="button"
                class=move || {
                    let active = current_theme.get() == "demacia";
                    if active {
                        "px-3 py-1 rounded-full font-imperial text-[10px] uppercase tracking-[0.18em] bg-accent text-accent-contrast font-semibold cursor-pointer transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                    } else {
                        "px-3 py-1 rounded-full font-imperial text-[10px] uppercase tracking-[0.18em] text-muted hover:text-secondary cursor-pointer transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                    }
                }
                aria-pressed=move || (current_theme.get() == "demacia").to_string()
                title="Demacia — heraldic, illuminated"
                on:click=move |_| set_theme.run(String::from("demacia"))
            >
                "Demacia"
            </button>
            <button
                type="button"
                class=move || {
                    let active = current_theme.get() == "pandemonium";
                    if active {
                        "px-3 py-1 rounded-full font-glitch text-[11px] uppercase tracking-[0.18em] bg-accent text-accent-contrast font-semibold cursor-pointer transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                    } else {
                        "px-3 py-1 rounded-full font-glitch text-[11px] uppercase tracking-[0.18em] text-muted hover:text-secondary cursor-pointer transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                    }
                }
                aria-pressed=move || (current_theme.get() == "pandemonium").to_string()
                title="Pandemonium — fractured, kinetic"
                on:click=move |_| set_theme.run(String::from("pandemonium"))
            >
                "Pandemonium"
            </button>
        </div>
    }
}
