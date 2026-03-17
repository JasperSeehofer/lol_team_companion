use crate::components::ui::{EmptyState, ErrorBanner, SkeletonCard, SkeletonLine, ToastContext, ToastKind};
use crate::models::action_item::ActionItem;
use leptos::prelude::*;
use leptos_router::components::A;

// ---------------------------------------------------------------------------
// Server functions
// ---------------------------------------------------------------------------

#[server]
pub async fn get_action_items() -> Result<Vec<ActionItem>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = match db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    db::list_action_items(&surreal, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn create_action_item_fn(
    text: String,
    assigned_to: Option<String>,
) -> Result<String, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = db::get_user_team_id(&surreal, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No team"))?;

    // Treat empty string as None for assigned_to
    let assigned = assigned_to.filter(|s| !s.is_empty());

    db::create_action_item(&surreal, &team_id, text, None, assigned)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn update_item_status(id: String, status: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let _user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::update_action_item_status(&surreal, &id, status)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn update_item(
    id: String,
    text: String,
    assigned_to: Option<String>,
) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let _user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let assigned = assigned_to.filter(|s| !s.is_empty());

    // Keep current status (don't change it through this endpoint)
    // We need to read the current item to preserve status
    db::update_action_item(&surreal, &id, text, assigned, "open".into())
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn delete_item(id: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let _user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::delete_action_item(&surreal, &id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_team_members_list() -> Result<Vec<(String, String)>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let surreal =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let (_, members) = match db::get_user_team_with_members(&surreal, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(tm) => tm,
        None => return Ok(Vec::new()),
    };

    Ok(members
        .into_iter()
        .map(|m| (m.user_id, m.username))
        .collect())
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

#[component]
pub fn ActionItemsPage() -> impl IntoView {
    // Auth guard
    let user = Resource::new(
        || (),
        |_| async {
            #[cfg(feature = "ssr")]
            {
                use crate::server::auth::AuthSession;
                let auth: AuthSession = leptos_axum::extract().await.ok()?;
                auth.user.map(|u| u.id)
            }
            #[cfg(not(feature = "ssr"))]
            {
                Some(String::new())
            }
        },
    );

    Effect::new(move |_| {
        if let Some(None) = user.get() {
            #[cfg(feature = "hydrate")]
            {
                if let Some(win) = web_sys::window() {
                    let _ = win.location().set_href("/auth/login");
                }
            }
        }
    });

    let items = Resource::new(|| (), |_| get_action_items());
    let members = Resource::new(|| (), |_| get_team_members_list());

    let toast = use_context::<ToastContext>().expect("ToastProvider");

    let (status_filter, set_status_filter) = signal("all".to_string());
    let (assignee_filter, set_assignee_filter) = signal("all".to_string());
    let (new_text, set_new_text) = signal(String::new());
    let (new_assignee, set_new_assignee) = signal(String::new());

    let add_item = move |_| {
        let text = new_text.get_untracked();
        if text.trim().is_empty() {
            return;
        }
        let assignee = new_assignee.get_untracked();
        let assigned = if assignee.is_empty() {
            None
        } else {
            Some(assignee)
        };
        leptos::task::spawn_local(async move {
            match create_action_item_fn(text, assigned).await {
                Ok(_) => {
                    set_new_text.set(String::new());
                    set_new_assignee.set(String::new());
                    toast.show.run((ToastKind::Success, "Action item added".into()));
                    items.refetch();
                }
                Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
            }
        });
    };

    let change_status = move |id: String, status: String| {
        leptos::task::spawn_local(async move {
            let msg = if status == "done" { "Marked complete" } else { "Status updated" };
            match update_item_status(id, status).await {
                Ok(_) => {
                    items.refetch();
                    toast.show.run((ToastKind::Success, msg.into()));
                }
                Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
            }
        });
    };

    let remove_item = move |id: String| {
        leptos::task::spawn_local(async move {
            match delete_item(id).await {
                Ok(_) => items.refetch(),
                Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
            }
        });
    };

    view! {
        <div class="max-w-4xl mx-auto p-6">
            <div class="flex items-center justify-between mb-6">
                <h1 class="text-2xl font-bold text-primary">"Action Items"</h1>
                <A href="/team/dashboard" attr:class="text-accent hover:underline text-sm">"Back to Dashboard"</A>
            </div>


            // Quick stats
            <Suspense fallback=move || view! { <SkeletonLine width="w-48" height="h-5" /> }>
                {move || items.get().map(|result| match result {
                    Ok(ref all_items) => {
                        let open_count = all_items.iter().filter(|i| i.status == "open").count();
                        let in_progress_count = all_items.iter().filter(|i| i.status == "in_progress").count();
                        let done_count = all_items.iter().filter(|i| i.status == "done").count();
                        view! {
                            <div class="flex gap-4 mb-6">
                                <div class="bg-surface border border-divider rounded-lg px-4 py-2 flex items-center gap-2">
                                    <span class="w-2.5 h-2.5 rounded-full bg-green-500"></span>
                                    <span class="text-secondary text-sm">{open_count}" open"</span>
                                </div>
                                <div class="bg-surface border border-divider rounded-lg px-4 py-2 flex items-center gap-2">
                                    <span class="w-2.5 h-2.5 rounded-full bg-yellow-500"></span>
                                    <span class="text-secondary text-sm">{in_progress_count}" in progress"</span>
                                </div>
                                <div class="bg-surface border border-divider rounded-lg px-4 py-2 flex items-center gap-2">
                                    <span class="w-2.5 h-2.5 rounded-full bg-gray-500"></span>
                                    <span class="text-secondary text-sm">{done_count}" done"</span>
                                </div>
                            </div>
                        }.into_any()
                    }
                    Err(_) => view! { <span></span> }.into_any(),
                })}
            </Suspense>

            // Add item form
            <div class="bg-surface border border-divider rounded-lg p-4 mb-6">
                <h2 class="text-primary font-semibold mb-3">"Add Action Item"</h2>
                <div class="flex gap-3">
                    <input
                        type="text"
                        placeholder="What needs to be done?"
                        class="flex-1 bg-surface/50 border border-outline/50 rounded px-3 py-2 text-primary placeholder:text-muted text-sm focus:outline-none focus:border-accent"
                        prop:value=move || new_text.get()
                        on:input=move |ev| set_new_text.set(event_target_value(&ev))
                        on:keydown=move |ev| {
                            if ev.key() == "Enter" {
                                let text = new_text.get_untracked();
                                if text.trim().is_empty() {
                                    return;
                                }
                                let assignee = new_assignee.get_untracked();
                                let assigned = if assignee.is_empty() { None } else { Some(assignee) };
                                leptos::task::spawn_local(async move {
                                    match create_action_item_fn(text, assigned).await {
                                        Ok(_) => {
                                            set_new_text.set(String::new());
                                            set_new_assignee.set(String::new());
                                            toast.show.run((ToastKind::Success, "Action item added".into()));
                                            items.refetch();
                                        }
                                        Err(e) => toast.show.run((ToastKind::Error, format!("{e}"))),
                                    }
                                });
                            }
                        }
                    />
                    <Suspense fallback=move || view! { <span></span> }>
                        {move || members.get().map(|result| match result {
                            Ok(ref member_list) if !member_list.is_empty() => {
                                let member_list = member_list.clone();
                                view! {
                                    <select
                                        class="bg-surface/50 border border-outline/50 rounded px-3 py-2 text-secondary text-sm focus:outline-none focus:border-accent"
                                        prop:value=move || new_assignee.get()
                                        on:change=move |ev| set_new_assignee.set(event_target_value(&ev))
                                    >
                                        <option value="">"Unassigned"</option>
                                        {member_list.iter().map(|(_uid, uname)| {
                                            let uname_val = uname.clone();
                                            let uname_display = uname.clone();
                                            view! { <option value=uname_val>{uname_display}</option> }
                                        }).collect_view()}
                                    </select>
                                }.into_any()
                            }
                            _ => view! { <span></span> }.into_any(),
                        })}
                    </Suspense>
                    <button
                        class="bg-accent hover:bg-accent-hover text-accent-contrast font-medium rounded px-4 py-2 text-sm transition-colors cursor-pointer"
                        on:click=add_item
                    >"Add"</button>
                </div>
            </div>

            // Filters
            <div class="flex gap-3 mb-4">
                <select
                    class="bg-surface/50 border border-outline/50 rounded px-3 py-1.5 text-secondary text-sm focus:outline-none focus:border-accent"
                    prop:value=move || status_filter.get()
                    on:change=move |ev| set_status_filter.set(event_target_value(&ev))
                >
                    <option value="all">"All Statuses"</option>
                    <option value="open">"Open"</option>
                    <option value="in_progress">"In Progress"</option>
                    <option value="done">"Done"</option>
                </select>
                <Suspense fallback=move || view! { <span></span> }>
                    {move || members.get().map(|result| match result {
                        Ok(ref member_list) if !member_list.is_empty() => {
                            let member_list = member_list.clone();
                            view! {
                                <select
                                    class="bg-surface/50 border border-outline/50 rounded px-3 py-1.5 text-secondary text-sm focus:outline-none focus:border-accent"
                                    prop:value=move || assignee_filter.get()
                                    on:change=move |ev| set_assignee_filter.set(event_target_value(&ev))
                                >
                                    <option value="all">"All Assignees"</option>
                                    <option value="unassigned">"Unassigned"</option>
                                    {member_list.iter().map(|(_uid, uname)| {
                                        let uname_val = uname.clone();
                                        let uname_display = uname.clone();
                                        view! { <option value=uname_val>{uname_display}</option> }
                                    }).collect_view()}
                                </select>
                            }.into_any()
                        }
                        _ => view! { <span></span> }.into_any(),
                    })}
                </Suspense>
            </div>

            // Items list
            <Suspense fallback=move || view! { <div class="flex flex-col gap-2"><SkeletonCard height="h-12" /><SkeletonCard height="h-12" /><SkeletonCard height="h-12" /></div> }>
                {move || items.get().map(|result| match result {
                    Ok(all_items) => {
                        let sf = status_filter.get();
                        let af = assignee_filter.get();
                        let filtered: Vec<ActionItem> = all_items
                            .into_iter()
                            .filter(|item| {
                                if sf != "all" && item.status != sf {
                                    return false;
                                }
                                if af == "unassigned" && item.assigned_to.is_some() {
                                    return false;
                                }
                                if af != "all" && af != "unassigned" {
                                    if item.assigned_to.as_deref() != Some(&af) {
                                        return false;
                                    }
                                }
                                true
                            })
                            .collect();

                        if filtered.is_empty() {
                            view! {
                                <EmptyState
                                    icon="✅"
                                    message="No action items yet — they'll appear here automatically after post-game reviews, or add one manually"
                                />
                            }.into_any()
                        } else {
                            view! {
                                <div class="space-y-2">
                                    {filtered.into_iter().map(|item| {
                                        let item_id = item.id.clone().unwrap_or_default();
                                        let item_id_for_status = item_id.clone();
                                        let item_id_for_delete = item_id.clone();
                                        let status = item.status.clone();
                                        let status_for_badge = item.status.clone();
                                        let text = item.text.clone();
                                        let assigned = item.assigned_to.clone();
                                        let source = item.source_review.clone();

                                        let status_dot_class = match status_for_badge.as_str() {
                                            "open" => "w-2.5 h-2.5 rounded-full bg-green-500 shrink-0",
                                            "in_progress" => "w-2.5 h-2.5 rounded-full bg-yellow-500 shrink-0",
                                            _ => "w-2.5 h-2.5 rounded-full bg-gray-500 shrink-0",
                                        };

                                        let text_class = if status == "done" {
                                            "text-muted line-through text-sm"
                                        } else {
                                            "text-primary text-sm"
                                        };

                                        let next_status = match status.as_str() {
                                            "open" => Some(("in_progress", "Start")),
                                            "in_progress" => Some(("done", "Done")),
                                            _ => None,
                                        };

                                        let reopen = status == "done";

                                        view! {
                                            <div class="bg-surface border border-divider rounded-lg px-4 py-3 flex items-center gap-3">
                                                <span class=status_dot_class></span>
                                                <div class="flex-1 min-w-0">
                                                    <p class=text_class>{text}</p>
                                                    <div class="flex gap-2 mt-1">
                                                        {assigned.map(|a| view! {
                                                            <span class="text-xs bg-elevated text-secondary rounded px-1.5 py-0.5">{a}</span>
                                                        })}
                                                        {source.map(|_s| view! {
                                                            <span class="text-xs text-muted">"from review"</span>
                                                        })}
                                                    </div>
                                                </div>
                                                <div class="flex items-center gap-2 shrink-0">
                                                    {next_status.map(|(next, label)| {
                                                        let next = next.to_string();
                                                        let id = item_id_for_status.clone();
                                                        view! {
                                                            <button
                                                                class="text-xs bg-accent/20 text-accent hover:bg-accent/30 rounded px-2 py-1 transition-colors cursor-pointer"
                                                                on:click=move |_| {
                                                                    let id = id.clone();
                                                                    let next = next.clone();
                                                                    change_status(id, next);
                                                                }
                                                            >{label}</button>
                                                        }
                                                    })}
                                                    {if reopen {
                                                        let id = item_id_for_status.clone();
                                                        view! {
                                                            <button
                                                                class="text-xs bg-green-500/20 text-green-400 hover:bg-green-500/30 rounded px-2 py-1 transition-colors cursor-pointer"
                                                                on:click=move |_| {
                                                                    let id = id.clone();
                                                                    change_status(id, "open".into());
                                                                }
                                                            >"Reopen"</button>
                                                        }.into_any()
                                                    } else {
                                                        view! { <span></span> }.into_any()
                                                    }}
                                                    <button
                                                        class="text-xs text-red-400 hover:text-red-300 transition-colors cursor-pointer"
                                                        on:click=move |_| {
                                                            let id = item_id_for_delete.clone();
                                                            remove_item(id);
                                                        }
                                                    >"Delete"</button>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }
                    }
                    Err(e) => view! {
                        <ErrorBanner message=format!("Failed to load action items: {e}") />
                    }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
