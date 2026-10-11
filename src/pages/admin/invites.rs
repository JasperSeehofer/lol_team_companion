//! Admin invite management (`/admin/invites`).
//!
//! Admin-only: every server function checks `require_admin()` on the server,
//! and non-admins (including signed-out visitors) get the regular "Page not
//! found" page with HTTP 404, so the page does not reveal that it exists. The
//! route renders with `SsrMode::Async` so the 404 status is set before the
//! response starts. The nav shows the link to admins only.

use leptos::prelude::*;

use crate::components::ui::{ToastContext, ToastKind};
use crate::error_template::NotFound;
use crate::models::invite::Invite;

/// All invites plus the server's current time (RFC 3339), used to show
/// which codes have expired. (The clock is read on the server: `Utc::now()`
/// is not available in the browser build.)
#[server]
pub async fn list_invites() -> Result<(String, Vec<Invite>), ServerFnError> {
    use crate::server::{authz, db};

    authz::require_admin().await?;
    let db = authz::db()?;
    let invites = db::list_invites(&db).await.map_err(authz::server_err)?;
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    Ok((now, invites))
}

/// Mint a new random code. An empty `expires_in_days` means it never
/// expires. Returns the code.
#[server]
pub async fn mint_invite(
    max_uses: i64,
    expires_in_days: String,
    note: String,
) -> Result<String, ServerFnError> {
    use crate::server::{authz, db};

    let admin = authz::require_admin().await?;
    let db = authz::db()?;
    if !(1..=1000).contains(&max_uses) {
        return Err(ServerFnError::new("Max uses must be between 1 and 1000."));
    }
    let expires_in_days = match expires_in_days.trim() {
        "" => None,
        v => Some(
            v.parse::<i64>()
                .map_err(|_| ServerFnError::new("Expiry must be a number of days."))?,
        ),
    };
    let expires_at = match expires_in_days.filter(|d| *d != 0) {
        Some(d) if (1..=365).contains(&d) => Some(chrono::Utc::now() + chrono::Duration::days(d)),
        Some(_) => {
            return Err(ServerFnError::new(
                "Expiry must be between 1 and 365 days (or empty for none).",
            ))
        }
        None => None,
    };
    let note = Some(note.trim().chars().take(200).collect::<String>()).filter(|n| !n.is_empty());
    let code = db::generate_invite_code();
    db::create_invite(
        &db,
        Some(&admin.id),
        code.clone(),
        max_uses,
        expires_at,
        note,
    )
    .await
    .map_err(authz::server_err)?;
    tracing::info!(admin = %admin.id, "minted invite code");
    Ok(code)
}

#[server]
pub async fn disable_invite_fn(id: String) -> Result<(), ServerFnError> {
    use crate::server::{authz, db};

    let admin = authz::require_admin().await?;
    let db = authz::db()?;
    db::disable_invite(&db, &id)
        .await
        .map_err(authz::server_err)?;
    tracing::info!(admin = %admin.id, invite = %id, "disabled invite code");
    Ok(())
}

fn short_date(value: &str) -> String {
    value.get(..10).unwrap_or(value).to_string()
}

fn status_class(status: &str) -> &'static str {
    match status {
        "open" => "text-success",
        "used up" => "text-muted",
        "expired" => "text-warning",
        _ => "text-danger",
    }
}

#[component]
pub fn AdminInvitesPage() -> impl IntoView {
    let toast = use_context::<ToastContext>();
    let invites = Resource::new(|| (), |_| list_invites());

    let mint = ServerAction::<MintInvite>::new();
    let disable = ServerAction::<DisableInviteFn>::new();

    // Refetch after mint/disable and report the outcome.
    Effect::new(move |_| {
        if let Some(result) = mint.value().get() {
            match result {
                Ok(code) => {
                    if let Some(t) = toast {
                        t.show.run((ToastKind::Success, format!("Minted {code}")));
                    }
                }
                Err(e) => {
                    if let Some(t) = toast {
                        t.show.run((ToastKind::Error, e.to_string()));
                    }
                }
            }
            invites.refetch();
        }
    });
    Effect::new(move |_| {
        if let Some(result) = disable.value().get() {
            if let Err(e) = result {
                if let Some(t) = toast {
                    t.show.run((ToastKind::Error, e.to_string()));
                }
            }
            invites.refetch();
        }
    });

    let last_code = move || mint.value().get().and_then(|r| r.ok());

    view! {
        <Suspense fallback=|| view! { <div class="min-h-screen bg-base"></div> }>
            {move || invites.get().map(|result| match result {
                Err(_) => view! { <NotFound /> }.into_any(),
                Ok((now, rows)) => {
                    view! {
                        <div class="bg-base min-h-screen px-4 sm:px-8 py-6">
                            <div class="max-w-5xl mx-auto flex flex-col gap-6">
                                <header>
                                    <div class="font-mono uppercase tracking-[0.18em] text-caption text-muted">
                                        "Invite management"
                                    </div>
                                    <h1 class="font-display text-display leading-tight text-primary mt-2">
                                        "Beta invitations"
                                    </h1>
                                    <div class="mt-3"><div class="h-px w-full bg-rule" aria-hidden="true"></div></div>
                                </header>

                                <section class="bg-surface border border-divider rounded-xl p-6">
                                    <h2 class="font-mono uppercase tracking-[0.18em] text-caption text-muted mb-3">
                                        "Mint a code"
                                    </h2>
                                    <ActionForm action=mint>
                                        <div class="flex flex-wrap gap-3 items-end">
                                            <div>
                                                <label for="invite-max-uses" class="block text-xs text-muted uppercase tracking-wider mb-1.5">
                                                    "Max uses"
                                                </label>
                                                <input
                                                    id="invite-max-uses"
                                                    type="number"
                                                    name="max_uses"
                                                    min="1"
                                                    max="1000"
                                                    value="1"
                                                    required
                                                    class="w-28 bg-surface/50 border border-outline rounded-lg px-3 py-3 text-primary text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                />
                                            </div>
                                            <div>
                                                <label for="invite-expiry" class="block text-xs text-muted uppercase tracking-wider mb-1.5">
                                                    "Expires in days (empty: never)"
                                                </label>
                                                <input
                                                    id="invite-expiry"
                                                    type="number"
                                                    name="expires_in_days"
                                                    min="1"
                                                    max="365"
                                                    value="14"
                                                    class="w-28 bg-surface/50 border border-outline rounded-lg px-3 py-3 text-primary text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                />
                                            </div>
                                            <div class="flex-1 min-w-[12rem]">
                                                <label for="invite-note" class="block text-xs text-muted uppercase tracking-wider mb-1.5">
                                                    "Note (optional)"
                                                </label>
                                                <input
                                                    id="invite-note"
                                                    type="text"
                                                    name="note"
                                                    maxlength="200"
                                                    placeholder="Who is this for?"
                                                    class="w-full bg-surface/50 border border-outline rounded-lg px-3 py-3 text-primary text-sm focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                />
                                            </div>
                                            <button
                                                type="submit"
                                                class="bg-accent text-accent-contrast font-semibold rounded-lg px-4 py-3 text-sm hover:bg-accent-hover focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-colors"
                                            >
                                                "Mint code"
                                            </button>
                                        </div>
                                    </ActionForm>
                                    {move || last_code().map(|code| {
                                        let link = format!("/auth/register?invite={code}");
                                        view! {
                                            <p class="mt-4 text-sm text-secondary" data-testid="minted-code">
                                                "New code: "
                                                <code class="font-mono text-primary">{code}</code>
                                                " \u{2014} share the link "
                                                <code class="font-mono text-primary break-all">{link}</code>
                                            </p>
                                        }
                                    })}
                                </section>

                                <section class="bg-surface border border-divider rounded-xl overflow-hidden">
                                    <div class="px-6 py-4 border-b border-divider">
                                        <h2 class="font-display text-heading text-primary">
                                            "All invitations"
                                        </h2>
                                    </div>
                                    {if rows.is_empty() {
                                        view! {
                                            <p class="px-6 py-6 text-sm text-muted">
                                                "No invite codes yet. Mint one above; it appears here with its uses and expiry."
                                            </p>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="overflow-x-auto">
                                                <table class="w-full">
                                                    <thead class="bg-elevated">
                                                        <tr class="text-left">
                                                            <th class="px-4 py-3 text-xs text-muted uppercase tracking-wider">"Code"</th>
                                                            <th class="px-4 py-3 text-xs text-muted uppercase tracking-wider">"Status"</th>
                                                            <th class="px-4 py-3 text-xs text-muted uppercase tracking-wider">"Uses"</th>
                                                            <th class="px-4 py-3 text-xs text-muted uppercase tracking-wider">"Expires"</th>
                                                            <th class="px-4 py-3 text-xs text-muted uppercase tracking-wider">"Created"</th>
                                                            <th class="px-4 py-3 text-xs text-muted uppercase tracking-wider">"Note"</th>
                                                            <th class="px-4 py-3"><span class="sr-only">"Actions"</span></th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        {rows.into_iter().map(|inv| {
                                                            let status = inv.status(&now);
                                                            let can_disable = !inv.disabled;
                                                            let id = inv.id.clone();
                                                            let created = match &inv.created_by {
                                                                Some(by) => format!("{} by {by}", short_date(&inv.created_at)),
                                                                None => short_date(&inv.created_at),
                                                            };
                                                            view! {
                                                                <tr class="border-t border-divider/50">
                                                                    <td class="px-4 py-3">
                                                                        <code class="font-mono text-caption text-primary">{inv.code.clone()}</code>
                                                                    </td>
                                                                    <td class=format!("px-4 py-3 text-sm {}", status_class(status))>{status}</td>
                                                                    <td class="px-4 py-3 text-sm text-secondary tabular-nums">
                                                                        {format!("{} / {}", inv.uses, inv.max_uses)}
                                                                    </td>
                                                                    <td class="px-4 py-3 text-sm text-secondary tabular-nums">
                                                                        {inv.expires_at.as_deref().map(short_date).unwrap_or_else(|| "never".to_string())}
                                                                    </td>
                                                                    <td class="px-4 py-3 text-sm text-secondary">{created}</td>
                                                                    <td class="px-4 py-3 text-sm text-secondary">{inv.note.clone().unwrap_or_default()}</td>
                                                                    <td class="px-4 py-3 text-right">
                                                                        {can_disable.then(|| view! {
                                                                            <button
                                                                                type="button"
                                                                                class="text-sm text-danger hover:underline focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none rounded"
                                                                                on:click=move |_| {
                                                                                    disable.dispatch(DisableInviteFn { id: id.clone() });
                                                                                }
                                                                            >
                                                                                "Disable"
                                                                            </button>
                                                                        })}
                                                                    </td>
                                                                </tr>
                                                            }
                                                        }).collect_view()}
                                                    </tbody>
                                                </table>
                                            </div>
                                        }.into_any()
                                    }}
                                </section>
                            </div>
                        </div>
                    }.into_any()
                }
            })}
        </Suspense>
    }
}
