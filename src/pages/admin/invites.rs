//! Admin invite management (`/admin/invites`).
//!
//! Admin-only: every server function checks `require_admin()` on the server,
//! and non-admins (including signed-out visitors) get the regular "Page not
//! found" page with HTTP 404, so the page does not reveal that it exists. The
//! route renders with `SsrMode::Async` so the 404 status is set before the
//! response starts. The nav shows the link to admins only.

use leptos::prelude::*;

use crate::components::ui::{
    Button, ButtonSize, ButtonType, ButtonVariant, EmptySize, EmptyState, Field, InputKind,
    PageHeader, PageShell, PageWidth, SectionHeader, Skeleton, SkeletonGroup, Table, Td, TextField,
    Th, ToastContext, ToastKind, Tr,
};
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

    let max_uses = RwSignal::new("1".to_string());
    let expiry = RwSignal::new("14".to_string());
    let note = RwSignal::new(String::new());
    view! {
        <Suspense fallback=|| view! { <PageShell><SkeletonGroup label="Loading invites"><Skeleton lines=4 /></SkeletonGroup></PageShell> }>
            {move || invites.get().map(|result| match result {
                Err(_) => view! { <NotFound /> }.into_any(),
                Ok((now, rows)) => {
                    view! {
                        <PageShell width=PageWidth::Standard>
                            <PageHeader title="Beta invitations" eyebrow="Admin" meta=move || view! { <span>"Invite codes for the closed beta: mint, share the link, disable."</span> } />
                            <section aria-labelledby="mint-h">
                                <SectionHeader title="Mint a code" id="mint-h" />
                                <ActionForm action=mint>
                                    <div class="flex flex-wrap items-end gap-3">
                                        <div class="w-32">
                                            <Field id="invite-max-uses" label="Max uses">
                                                <TextField kind=InputKind::Number name="max_uses" value=max_uses required=true />
                                            </Field>
                                        </div>
                                        <div class="w-40">
                                            <Field id="invite-expiry" label="Expires in days" hint="Empty: never">
                                                <TextField kind=InputKind::Number name="expires_in_days" value=expiry />
                                            </Field>
                                        </div>
                                        <div class="min-w-48 flex-1">
                                            <Field id="invite-note" label="Note (optional)">
                                                <TextField name="note" value=note maxlength=200 placeholder="Who is this for?" />
                                            </Field>
                                        </div>
                                        <Button kind=ButtonType::Submit variant=ButtonVariant::Primary>"Mint code"</Button>
                                    </div>
                                </ActionForm>
                                {move || last_code().map(|code| {
                                    let link = format!("/auth/register?invite={code}");
                                    view! {
                                        <p class="mt-4 text-body text-secondary" data-testid="minted-code">
                                            "New code: "
                                            <code class="font-mono text-primary">{code}</code>
                                            ". Share the link "
                                            <code class="break-all font-mono text-primary">{link}</code>
                                        </p>
                                    }
                                })}
                            </section>
                            <section class="mt-12" aria-labelledby="all-invites-h">
                                <SectionHeader title="All invitations" id="all-invites-h" />
                                {if rows.is_empty() {
                                    view! { <EmptyState size=EmptySize::Compact message="No invite codes yet. Mint one above; it appears here with its uses and expiry." /> }.into_any()
                                } else {
                                    view! {
                                        <Table caption="Invite codes" caption_hidden=true>
                                            <thead>
                                                <tr>
                                                    <Th>"Code"</Th>
                                                    <Th>"Status"</Th>
                                                    <Th numeric=true>"Uses"</Th>
                                                    <Th>"Expires"</Th>
                                                    <Th>"Created"</Th>
                                                    <Th>"Note"</Th>
                                                    <Th><span class="sr-only">"Actions"</span></Th>
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
                                                        <Tr>
                                                            <Td><code class="font-mono text-caption text-primary">{inv.code.clone()}</code></Td>
                                                            <Td><span class=status_class(status)>{status}</span></Td>
                                                            <Td numeric=true>{format!("{} / {}", inv.uses, inv.max_uses)}</Td>
                                                            <Td>{inv.expires_at.as_deref().map(short_date).unwrap_or_else(|| "never".to_string())}</Td>
                                                            <Td muted=true>{created}</Td>
                                                            <Td muted=true>{inv.note.clone().unwrap_or_default()}</Td>
                                                            <Td>
                                                                {can_disable.then(|| view! {
                                                                    <Button size=ButtonSize::Sm variant=ButtonVariant::Danger on_click=Callback::new(move |_| { disable.dispatch(DisableInviteFn { id: id.clone() }); })>"Disable"</Button>
                                                                })}
                                                            </Td>
                                                        </Tr>
                                                    }
                                                }).collect_view()}
                                            </tbody>
                                        </Table>
                                    }.into_any()
                                }}
                            </section>
                        </PageShell>
                    }.into_any()
                }
            })}
        </Suspense>
    }
}
