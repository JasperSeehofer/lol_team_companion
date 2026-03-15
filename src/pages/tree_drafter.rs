use crate::components::champion_picker::ChampionPicker;
use crate::components::draft_board::{slot_meta, DraftBoard};
use crate::components::tree_graph::TreeGraph;
use crate::components::ui::{ErrorBanner, StatusMessage};
use crate::models::champion::Champion;
use crate::models::draft::{DraftAction, DraftTree, DraftTreeNode};
use leptos::prelude::*;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Server functions
// ---------------------------------------------------------------------------

#[server]
pub async fn get_champions_for_tree() -> Result<Vec<Champion>, ServerFnError> {
    use crate::server::data_dragon;
    data_dragon::fetch_champions()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn list_trees() -> Result<Vec<DraftTree>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = match db::get_user_team_id(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    db::list_draft_trees(&db, &team_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn create_tree(name: String, opponent: Option<String>) -> Result<String, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let team_id = db::get_user_team_id(&db, &user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("No team"))?;

    db::create_draft_tree(&db, &team_id, &user.id, name, opponent)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn delete_tree(tree_id: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let _user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::delete_draft_tree(&db, &tree_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn update_tree_meta(
    tree_id: String,
    name: String,
    opponent: Option<String>,
) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let _user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::update_draft_tree(&db, &tree_id, name, opponent)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_tree_nodes(tree_id: String) -> Result<Vec<DraftTreeNode>, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let _user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::get_tree_nodes(&db, &tree_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn add_branch(
    tree_id: String,
    parent_id: Option<String>,
    label: String,
) -> Result<String, ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let _user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::create_tree_node(&db, &tree_id, parent_id, label)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn save_node(
    node_id: String,
    label: String,
    notes: Option<String>,
    is_improvised: bool,
    actions_json: String,
) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let _user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let actions: Vec<DraftAction> = serde_json::from_str(&actions_json)
        .map_err(|e| ServerFnError::new(format!("Invalid actions JSON: {e}")))?;

    db::update_tree_node(&db, &node_id, label, notes, is_improvised, actions)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn remove_node(node_id: String) -> Result<(), ServerFnError> {
    use crate::server::auth::AuthSession;
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let auth: AuthSession = leptos_axum::extract().await?;
    let _user = auth
        .user
        .ok_or_else(|| ServerFnError::new("Not logged in"))?;
    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    db::delete_tree_node(&db, &node_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_actions_from_slots(slots: &[Option<String>]) -> Vec<DraftAction> {
    slots
        .iter()
        .enumerate()
        .filter_map(|(i, opt)| {
            opt.as_ref().map(|champ| {
                let (side, kind, label) = slot_meta(i);
                DraftAction {
                    id: None,
                    draft_id: String::new(),
                    phase: format!("{}_{}", kind, label),
                    side: side.to_string(),
                    champion: champ.clone(),
                    order: i as i32,
                    comment: None,
                }
            })
        })
        .collect()
}

fn actions_to_slots(actions: &[DraftAction]) -> Vec<Option<String>> {
    let mut slots = vec![None::<String>; 20];
    for a in actions {
        let o = a.order as usize;
        if o < 20 {
            slots[o] = Some(a.champion.clone());
        }
    }
    slots
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

#[component]
pub fn TreeDrafterPage() -> impl IntoView {
    // Auth redirect
    let auth_user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());
    Effect::new(move || {
        if let Some(Ok(None)) = auth_user.get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/auth/login");
            }
        }
    });

    // Mode: "edit" or "live"
    let (mode, set_mode) = signal("edit".to_string());

    // Tree management
    let (new_tree_name, set_new_tree_name) = signal(String::new());
    let (new_tree_opponent, set_new_tree_opponent) = signal(String::new());
    let (selected_tree_id, set_selected_tree_id) = signal(Option::<String>::None);
    let (status_msg, set_status_msg) = signal(Option::<String>::None);

    // Node editing
    let (selected_node_id, set_selected_node_id) = signal(Option::<String>::None);
    let (node_label, set_node_label) = signal(String::new());
    let (node_notes, set_node_notes) = signal(String::new());
    let (node_improvised, set_node_improvised) = signal(false);
    let (draft_slots, set_draft_slots) = signal(vec![None::<String>; 20]);
    let (active_slot, set_active_slot) = signal(Some(0_usize));
    let (highlighted_slot, set_highlighted_slot) = signal(Option::<usize>::None);

    // Branch adding
    let (adding_branch_to, set_adding_branch_to) = signal(Option::<String>::None);
    let (new_branch_label, set_new_branch_label) = signal(String::new());

    // Expanded nodes in tree view
    let (expanded_nodes, set_expanded_nodes) = signal(std::collections::HashSet::<String>::new());

    // Tree visualization mode: "list" or "graph"
    let (tree_view_mode, set_tree_view_mode) = signal("list".to_string());

    // Live navigator state
    let (nav_path, set_nav_path) = signal(Vec::<String>::new());

    // Resources
    let champions_resource = Resource::new(|| (), |_| get_champions_for_tree());
    let trees = Resource::new(|| (), |_| list_trees());

    // Reactive resource for tree nodes — depends on selected_tree_id
    let nodes_resource = Resource::new(
        move || selected_tree_id.get(),
        move |tree_id| async move {
            match tree_id {
                Some(id) => get_tree_nodes(id).await,
                None => Ok(Vec::new()),
            }
        },
    );

    // Create tree handler
    let do_create_tree = Callback::new(move |_: ()| {
        let name = new_tree_name.get_untracked();
        if name.trim().is_empty() {
            set_status_msg.set(Some("Enter a tree name.".into()));
            return;
        }
        let opp = new_tree_opponent.get_untracked();
        let opp_opt = if opp.is_empty() { None } else { Some(opp) };
        leptos::task::spawn_local(async move {
            match create_tree(name, opp_opt).await {
                Ok(id) => {
                    set_selected_tree_id.set(Some(id));
                    set_new_tree_name.set(String::new());
                    set_new_tree_opponent.set(String::new());
                    set_status_msg.set(Some("Tree created!".into()));
                    trees.refetch();
                    nodes_resource.refetch();
                }
                Err(e) => set_status_msg.set(Some(format!("Error: {e}"))),
            }
        });
    });

    // Delete tree handler
    let do_delete_tree = move |_| {
        let tree_id = match selected_tree_id.get_untracked() {
            Some(id) => id,
            None => return,
        };
        leptos::task::spawn_local(async move {
            match delete_tree(tree_id).await {
                Ok(_) => {
                    set_selected_tree_id.set(None);
                    set_selected_node_id.set(None);
                    set_status_msg.set(Some("Tree deleted.".into()));
                    trees.refetch();
                }
                Err(e) => set_status_msg.set(Some(format!("Error: {e}"))),
            }
        });
    };

    // Auto-save node timer (only used in hydrate/WASM builds)
    #[allow(unused_variables)]
    let auto_save_node_timer: RwSignal<Option<i32>> = RwSignal::new(None);
    let (node_save_status, set_node_save_status) = signal(""); // "", "unsaved", "saved"
                                                               // Suppress auto-save during node/tree switches to avoid saving stale data
    #[allow(unused_variables)]
    let suppress_autosave: RwSignal<bool> = RwSignal::new(false);

    // Helper: cancel any pending auto-save timer
    let cancel_autosave_timer = move || {
        #[cfg(feature = "hydrate")]
        if let Some(timer_id) = auto_save_node_timer.get_untracked() {
            if let Some(win) = web_sys::window() {
                win.clear_timeout_with_handle(timer_id);
            }
            auto_save_node_timer.set(None);
        }
    };

    // Helper: clear editor state (used when switching trees)
    let clear_editor = move || {
        cancel_autosave_timer();
        set_selected_node_id.set(None);
        set_node_label.set(String::new());
        set_node_notes.set(String::new());
        set_node_improvised.set(false);
        set_draft_slots.set(vec![None; 20]);
        set_active_slot.set(None);
        set_highlighted_slot.set(None);
        set_node_save_status.set("");
    };

    // Select a node for editing
    let select_node = move |node: &DraftTreeNode| {
        // Suppress auto-save while we update all editor signals to avoid saving stale data
        suppress_autosave.set(true);
        cancel_autosave_timer();
        set_node_save_status.set("");

        set_selected_node_id.set(node.id.clone());
        set_node_label.set(node.label.clone());
        set_node_notes.set(node.notes.clone().unwrap_or_default());
        set_node_improvised.set(node.is_improvised);
        set_highlighted_slot.set(None);
        set_active_slot.set(None);
        let slots = actions_to_slots(&node.actions);
        let next = (0..20).find(|&i| slots[i].is_none());
        set_draft_slots.set(slots);
        set_active_slot.set(next);

        // Re-enable auto-save after a microtask so the Effect doesn't fire for these batch updates
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::prelude::*;
            let cb = Closure::once(move || {
                suppress_autosave.set(false);
            });
            if let Some(win) = web_sys::window() {
                let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    0,
                );
            }
            cb.forget();
        }
        #[cfg(not(feature = "hydrate"))]
        suppress_autosave.set(false);
    };

    // Save node handler
    let do_save_node = move |_| {
        let nid = match selected_node_id.get_untracked() {
            Some(id) => id,
            None => return,
        };
        let label = node_label.get_untracked();
        let notes_raw = node_notes.get_untracked();
        let notes = if notes_raw.trim().is_empty() {
            None
        } else {
            Some(notes_raw)
        };
        let improvised = node_improvised.get_untracked();
        let actions = build_actions_from_slots(&draft_slots.get_untracked());
        let actions_json = serde_json::to_string(&actions).unwrap_or_default();

        leptos::task::spawn_local(async move {
            match save_node(nid, label, notes, improvised, actions_json).await {
                Ok(_) => {
                    set_status_msg.set(Some("Node saved!".into()));
                    nodes_resource.refetch();
                }
                Err(e) => set_status_msg.set(Some(format!("Error: {e}"))),
            }
        });
    };

    Effect::new(move |_| {
        // Track these signals so the effect re-runs when they change
        // (values are used in #[cfg(feature = "hydrate")] block below)
        #[allow(unused_variables)]
        let current_slots = draft_slots.get();
        #[allow(unused_variables)]
        let current_notes = node_notes.get();
        #[allow(unused_variables)]
        let current_label = node_label.get();
        #[allow(unused_variables)]
        let current_improvised = node_improvised.get();
        let node_id = selected_node_id.get();

        #[allow(unused_variables)]
        let Some(node_id) = node_id
        else {
            return;
        };

        // Don't fire auto-save during node/tree switches
        if suppress_autosave.get_untracked() {
            return;
        }

        #[cfg(feature = "hydrate")]
        if let Some(timer_id) = auto_save_node_timer.get_untracked() {
            if let Some(win) = web_sys::window() {
                win.clear_timeout_with_handle(timer_id);
            }
        }

        set_node_save_status.set("unsaved");

        #[cfg(feature = "hydrate")]
        {
            // Capture all values eagerly — do NOT read signals lazily inside the timer callback
            let label = current_label;
            let notes = if current_notes.trim().is_empty() {
                None
            } else {
                Some(current_notes)
            };
            let improvised = current_improvised;
            let actions = build_actions_from_slots(&current_slots);
            let actions_json = serde_json::to_string(&actions).unwrap_or_default();

            use wasm_bindgen::prelude::*;
            let cb = Closure::once(move || {
                leptos::task::spawn_local(async move {
                    let _ = save_node(node_id, label, notes, improvised, actions_json).await;
                    set_node_save_status.set("saved");
                    nodes_resource.refetch();
                });
            });
            if let Some(win) = web_sys::window() {
                match win.set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    2000,
                ) {
                    Ok(timer_id) => {
                        auto_save_node_timer.set(Some(timer_id));
                    }
                    Err(_) => {}
                }
            }
            cb.forget();
        }
    });

    // Add branch handler
    let do_add_branch = move |_| {
        let tree_id = match selected_tree_id.get_untracked() {
            Some(id) => id,
            None => return,
        };
        let parent = adding_branch_to.get_untracked();
        let label = new_branch_label.get_untracked();
        if label.trim().is_empty() {
            set_status_msg.set(Some("Enter a branch label.".into()));
            return;
        }
        let tree_id_for_select = tree_id.clone();
        leptos::task::spawn_local(async move {
            match add_branch(tree_id, parent, label.clone()).await {
                Ok(new_node_id) => {
                    set_new_branch_label.set(String::new());
                    set_adding_branch_to.set(None);
                    set_status_msg.set(Some("Branch added!".into()));
                    nodes_resource.refetch();
                    // Select the newly created node
                    let new_node = DraftTreeNode {
                        id: Some(new_node_id),
                        tree_id: tree_id_for_select,
                        label,
                        notes: None,
                        is_improvised: false,
                        parent_id: None, // not needed for select_node
                        sort_order: 0,
                        actions: Vec::new(),
                        children: Vec::new(),
                    };
                    select_node(&new_node);
                }
                Err(e) => set_status_msg.set(Some(format!("Error: {e}"))),
            }
        });
    };

    // Delete node handler
    let do_delete_node = move |node_id: String| {
        leptos::task::spawn_local(async move {
            match remove_node(node_id).await {
                Ok(_) => {
                    set_selected_node_id.set(None);
                    set_status_msg.set(Some("Node deleted.".into()));
                    nodes_resource.refetch();
                }
                Err(e) => set_status_msg.set(Some(format!("Error: {e}"))),
            }
        });
    };

    // Champion slot filling
    let used_champions = move || {
        draft_slots
            .get()
            .into_iter()
            .flatten()
            .collect::<Vec<String>>()
    };

    let fill_slot = move |slot_idx: usize, champion_name: String| {
        let already_used = draft_slots
            .get_untracked()
            .iter()
            .any(|s| s.as_deref() == Some(&champion_name));
        if already_used {
            return;
        }
        set_draft_slots.update(|s| s[slot_idx] = Some(champion_name));
        set_highlighted_slot.set(None);
        let updated = draft_slots.get_untracked();
        let next = (0..20).find(|&i| updated[i].is_none());
        set_active_slot.set(next);
    };

    let on_champion_select = Callback::new(move |champ: Champion| {
        if let Some(slot) = active_slot.get_untracked() {
            fill_slot(slot, champ.id);
        }
    });

    let on_slot_drop = Callback::new(move |(slot_idx, name): (usize, String)| {
        fill_slot(slot_idx, name);
    });

    // Branch from position handler
    let do_branch_from = Callback::new(move |slot_idx: usize| {
        let tree_id = match selected_tree_id.get_untracked() {
            Some(id) => id,
            None => return,
        };
        let parent_id = selected_node_id.get_untracked();
        let current_slots = draft_slots.get_untracked();
        // Copy slots up to and including slot_idx
        let mut branch_slots: Vec<Option<String>> = vec![None; 20];
        let end = slot_idx.min(19) + 1;
        branch_slots[..end].clone_from_slice(&current_slots[..end]);
        let actions = build_actions_from_slots(&branch_slots);
        let actions_json = serde_json::to_string(&actions).unwrap_or_default();
        let (side, kind, num) = slot_meta(slot_idx);
        let label = format!("Branch after {} {} {}", side, kind, num);
        let label_for_save = label.clone();

        let branch_slots_for_select = branch_slots.clone();
        let tree_id_for_select = tree_id.clone();
        leptos::task::spawn_local(async move {
            match add_branch(tree_id, parent_id, label).await {
                Ok(new_node_id) => {
                    match save_node(
                        new_node_id.clone(),
                        label_for_save.clone(),
                        None,
                        false,
                        actions_json,
                    )
                    .await
                    {
                        Ok(_) => {
                            set_status_msg.set(Some("Branch created from position!".into()));
                            nodes_resource.refetch();
                            // Select the newly created node with its actions
                            let actions = build_actions_from_slots(&branch_slots_for_select);
                            let new_node = DraftTreeNode {
                                id: Some(new_node_id),
                                tree_id: tree_id_for_select,
                                label: label_for_save,
                                notes: None,
                                is_improvised: false,
                                parent_id: None,
                                sort_order: 0,
                                actions,
                                children: Vec::new(),
                            };
                            select_node(&new_node);
                        }
                        Err(e) => set_status_msg.set(Some(format!("Error saving branch: {e}"))),
                    }
                }
                Err(e) => set_status_msg.set(Some(format!("Error creating branch: {e}"))),
            }
        });
    });

    let on_slot_click = Callback::new(move |slot_idx: usize| {
        let slots = draft_slots.get_untracked();
        if slots.get(slot_idx).and_then(|s| s.as_ref()).is_some() {
            let currently_highlighted = highlighted_slot.get_untracked();
            if currently_highlighted == Some(slot_idx) {
                // Second click: set as active_slot for champion replacement
                set_active_slot.set(Some(slot_idx));
            } else {
                // First click: just highlight, set as active
                set_highlighted_slot.set(Some(slot_idx));
                set_active_slot.set(Some(slot_idx));
            }
        } else {
            set_highlighted_slot.set(None);
            set_active_slot.update(|a| {
                *a = if *a == Some(slot_idx) {
                    None
                } else {
                    Some(slot_idx)
                };
            });
        }
    });

    let on_slot_clear = Callback::new(move |slot_idx: usize| {
        set_draft_slots.update(|s| s[slot_idx] = None);
        set_highlighted_slot.set(None);
        set_active_slot.set(Some(slot_idx));
    });

    view! {
        <div class="max-w-[90rem] mx-auto py-8 px-6 flex flex-col gap-6">
            // Header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-primary">"Tree Drafter"</h1>
                    <p class="text-muted text-sm mt-1">"Plan branching draft strategies for every scenario"</p>
                </div>
                <div class="flex gap-2">
                    <button
                        class=move || if mode.get() == "edit" {
                            "px-4 py-2 rounded-l-lg text-sm font-medium bg-accent text-accent-contrast cursor-pointer"
                        } else {
                            "px-4 py-2 rounded-l-lg text-sm font-medium bg-overlay text-secondary hover:bg-overlay-strong transition-colors cursor-pointer"
                        }
                        on:click=move |_| set_mode.set("edit".to_string())
                    >"Edit"</button>
                    <button
                        class=move || if mode.get() == "live" {
                            "px-4 py-2 rounded-r-lg text-sm font-medium bg-emerald-500 text-white cursor-pointer"
                        } else {
                            "px-4 py-2 rounded-r-lg text-sm font-medium bg-overlay text-secondary hover:bg-overlay-strong transition-colors cursor-pointer"
                        }
                        on:click=move |_| {
                            if selected_tree_id.get_untracked().is_none() {
                                set_status_msg.set(Some("Select a tree first to enter Live Game mode.".into()));
                                return;
                            }
                            set_mode.set("live".to_string());
                            set_nav_path.set(Vec::new());
                        }
                    >"Live Game"</button>
                </div>
            </div>

            // Status message
            {move || status_msg.get().map(|msg| {
                view! { <StatusMessage message=msg /> }
            })}

            // Main layout: sidebar + content
            <div class="flex gap-6 min-h-[36rem]">
                // Left sidebar: tree list
                <div class="w-72 flex-shrink-0 flex flex-col gap-4">
                    // New tree form
                    <div class="bg-elevated/50 border border-divider/50 rounded-xl p-4 flex flex-col gap-3">
                        <h3 class="text-primary font-semibold text-sm">"New Tree"</h3>
                        <input
                            type="text"
                            placeholder="Tree name..."
                            prop:value=move || new_tree_name.get()
                            on:input=move |ev| set_new_tree_name.set(event_target_value(&ev))
                            on:keydown=move |ev: web_sys::KeyboardEvent| {
                                if ev.key() == "Enter" { do_create_tree.run(()); }
                            }
                            class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm placeholder-dimmed focus:outline-none focus:border-accent/50 transition-colors"
                        />
                        <input
                            type="text"
                            placeholder="Opponent (optional)"
                            prop:value=move || new_tree_opponent.get()
                            on:input=move |ev| set_new_tree_opponent.set(event_target_value(&ev))
                            on:keydown=move |ev: web_sys::KeyboardEvent| {
                                if ev.key() == "Enter" { do_create_tree.run(()); }
                            }
                            class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm placeholder-dimmed focus:outline-none focus:border-accent/50 transition-colors"
                        />
                        <button
                            class="bg-accent hover:bg-accent-hover text-accent-contrast font-semibold rounded-lg px-4 py-2 text-sm transition-colors"
                            on:click=move |_| do_create_tree.run(())
                        >"Create Tree"</button>
                    </div>

                    // Tree list
                    <div class="flex flex-col gap-1.5 flex-1 overflow-y-auto">
                        <h3 class="text-muted text-xs font-semibold uppercase tracking-wider px-1">"Your Trees"</h3>
                        <Suspense fallback=|| view! { <div class="text-dimmed text-sm px-1">"Loading..."</div> }>
                            {move || trees.get().map(|result| match result {
                                Ok(list) if list.is_empty() => view! {
                                    <div class="text-center py-6">
                                        <p class="text-dimmed text-sm mb-3">"No draft trees yet"</p>
                                        <p class="text-dimmed text-xs mb-4">"Create or join a team to get started."</p>
                                        <a href="/team/roster" class="bg-accent hover:bg-accent-hover text-accent-contrast font-bold rounded px-3 py-1.5 text-xs transition-colors">
                                            "Go to Team"
                                        </a>
                                    </div>
                                }.into_any(),
                                Ok(list) => view! {
                                    <div class="flex flex-col gap-1.5">
                                        {list.into_iter().map(|t| {
                                            let id = t.id.clone().unwrap_or_default();
                                            let id_for_click = id.clone();
                                            let name = t.name.clone();
                                            let opp = t.opponent.clone();
                                            view! {
                                                <button
                                                    class=move || {
                                                        let sel = selected_tree_id.get();
                                                        if sel.as_deref() == Some(&id) {
                                                            "w-full text-left bg-accent/10 border border-accent/30 rounded-lg p-3 transition-all"
                                                        } else {
                                                            "w-full text-left bg-elevated/30 border border-divider/30 rounded-lg p-3 hover:bg-overlay/30 transition-all"
                                                        }
                                                    }
                                                    on:click=move |_| {
                                                        clear_editor();
                                                        suppress_autosave.set(true);
                                                        set_selected_tree_id.set(Some(id_for_click.clone()));
                                                        set_nav_path.set(Vec::new());
                                                        // Re-enable after microtask
                                                        #[cfg(feature = "hydrate")]
                                                        {
                                                            use wasm_bindgen::prelude::*;
                                                            let cb = Closure::once(move || {
                                                                suppress_autosave.set(false);
                                                            });
                                                            if let Some(win) = web_sys::window() {
                                                                let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                                                                    cb.as_ref().unchecked_ref(), 0,
                                                                );
                                                            }
                                                            cb.forget();
                                                        }
                                                        #[cfg(not(feature = "hydrate"))]
                                                        suppress_autosave.set(false);
                                                    }
                                                >
                                                    <div class="text-primary text-sm font-medium truncate">{name}</div>
                                                    {opp.map(|o| view! {
                                                        <div class="text-muted text-xs mt-0.5">"vs " {o}</div>
                                                    })}
                                                </button>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_any(),
                                Err(e) => view! {
                                    <p class="text-red-400 text-sm">{e.to_string()}</p>
                                }.into_any(),
                            })}
                        </Suspense>
                    </div>

                    // Delete tree button
                    {move || selected_tree_id.get().map(|_| view! {
                        <button
                            class="text-red-400 hover:text-red-300 text-sm transition-colors border border-red-400/20 hover:border-red-400/40 rounded-lg px-3 py-1.5"
                            on:click=do_delete_tree
                        >"Delete Tree"</button>
                    })}
                </div>

                // Main content area
                <div class="flex-1 flex flex-col gap-4">
                    {move || {
                        if selected_tree_id.get().is_none() {
                            return view! {
                                <div class="flex-1 flex items-center justify-center">
                                    <div class="text-center">
                                        <div class="text-dimmed text-6xl mb-4">"&#x1f333;"</div>
                                        <p class="text-muted text-lg">"Select or create a tree to start planning"</p>
                                    </div>
                                </div>
                            }.into_any();
                        }

                        let current_mode = mode.get();
                        if current_mode == "live" {
                            // Live navigator mode
                            view! {
                                <LiveNavigator
                                    nodes_resource=nodes_resource
                                    nav_path=nav_path
                                    set_nav_path=set_nav_path
                                    champions_resource=champions_resource
                                    status_msg=set_status_msg
                                    selected_tree_id=selected_tree_id
                                />
                            }.into_any()
                        } else {
                            // Edit mode
                            view! {
                                <div class="flex gap-4 flex-1">
                                    // Tree visualization panel
                                    <div class=move || {
                                        if tree_view_mode.get() == "graph" {
                                            "flex-1 bg-elevated/50 border border-divider/50 rounded-xl p-4 overflow-y-auto flex flex-col gap-3"
                                        } else {
                                            "w-80 flex-shrink-0 bg-elevated/50 border border-divider/50 rounded-xl p-4 overflow-y-auto flex flex-col gap-3"
                                        }
                                    }>
                                        // Header with view toggle
                                        <div class="flex items-center justify-between">
                                            <h3 class="text-primary font-semibold text-sm">"Tree Structure"</h3>
                                            <div class="flex items-center gap-1 bg-overlay/50 rounded-lg p-0.5">
                                                <button
                                                    class=move || {
                                                        if tree_view_mode.get() == "list" {
                                                            "px-2 py-0.5 text-xs rounded-md bg-accent text-accent-contrast font-medium transition-colors cursor-pointer"
                                                        } else {
                                                            "px-2 py-0.5 text-xs rounded-md text-secondary hover:text-primary transition-colors cursor-pointer"
                                                        }
                                                    }
                                                    on:click=move |_| set_tree_view_mode.set("list".to_string())
                                                    title="List view"
                                                >
                                                    // List icon (≡)
                                                    "\u{2261}"
                                                </button>
                                                <button
                                                    class=move || {
                                                        if tree_view_mode.get() == "graph" {
                                                            "px-2 py-0.5 text-xs rounded-md bg-accent text-accent-contrast font-medium transition-colors cursor-pointer"
                                                        } else {
                                                            "px-2 py-0.5 text-xs rounded-md text-secondary hover:text-primary transition-colors cursor-pointer"
                                                        }
                                                    }
                                                    on:click=move |_| set_tree_view_mode.set("graph".to_string())
                                                    title="Graph view"
                                                >
                                                    // Graph/tree icon (⎔)
                                                    "\u{2442}"
                                                </button>
                                            </div>
                                        </div>

                                        <Suspense fallback=|| view! { <div class="text-dimmed text-sm">"Loading nodes..."</div> }>
                                            {move || nodes_resource.get().map(|result| match result {
                                                Ok(roots) if roots.is_empty() => view! {
                                                    <p class="text-dimmed text-sm">"No nodes yet."</p>
                                                }.into_any(),
                                                Ok(roots) => {
                                                    let vm = tree_view_mode.get();
                                                    if vm == "graph" {
                                                        // Build champion map for edge icons
                                                        let champ_map = champions_resource.get()
                                                            .and_then(|r| r.ok())
                                                            .map(|champs| {
                                                                champs.into_iter().map(|c| (c.id.clone(), c)).collect::<HashMap<String, Champion>>()
                                                            })
                                                            .unwrap_or_default();
                                                        let champion_map_stored = StoredValue::new(champ_map);
                                                        let all_nodes_stored = StoredValue::new(roots.clone());
                                                        view! {
                                                            <TreeGraph
                                                                roots=roots
                                                                selected_node_id=selected_node_id
                                                                on_select=Callback::new(move |n: DraftTreeNode| {
                                                                    select_node(&n);
                                                                })
                                                                on_add_branch=Callback::new(move |parent_id: String| {
                                                                    set_adding_branch_to.set(Some(parent_id));
                                                                    set_new_branch_label.set(String::new());
                                                                })
                                                                champion_map=champion_map_stored
                                                                all_nodes=all_nodes_stored
                                                            />
                                                        }.into_any()
                                                    } else {
                                                        // List view
                                                        view! {
                                                            <div class="flex flex-col gap-1">
                                                                {roots.into_iter().map(|root| {
                                                                    view! {
                                                                        <TreeNodeView
                                                                            node=root
                                                                            depth=0
                                                                            selected_node_id=selected_node_id
                                                                            expanded_nodes=expanded_nodes
                                                                            set_expanded_nodes=set_expanded_nodes
                                                                            on_select=Callback::new(move |n: DraftTreeNode| {
                                                                                select_node(&n);
                                                                            })
                                                                            on_add_branch=Callback::new(move |parent_id: String| {
                                                                                set_adding_branch_to.set(Some(parent_id));
                                                                                set_new_branch_label.set(String::new());
                                                                            })
                                                                            on_delete=Callback::new(move |nid: String| {
                                                                                do_delete_node(nid);
                                                                            })
                                                                        />
                                                                    }
                                                                }).collect_view()}
                                                            </div>
                                                        }.into_any()
                                                    }
                                                },
                                                Err(e) => view! {
                                                    <p class="text-red-400 text-sm">{e.to_string()}</p>
                                                }.into_any(),
                                            })}
                                        </Suspense>

                                        // Add branch inline form
                                        {move || adding_branch_to.get().map(|_pid| view! {
                                            <div class="mt-3 border-t border-divider/50 pt-3 flex flex-col gap-2">
                                                <label class="text-secondary text-xs font-medium">"New branch label"</label>
                                                <input
                                                    type="text"
                                                    placeholder="e.g. If they ban Jinx..."
                                                    prop:value=move || new_branch_label.get()
                                                    on:input=move |ev| set_new_branch_label.set(event_target_value(&ev))
                                                    class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-1.5 text-primary text-sm placeholder-dimmed focus:outline-none focus:border-accent/50 transition-colors"
                                                />
                                                <div class="flex gap-2">
                                                    <button
                                                        class="flex-1 bg-accent hover:bg-accent-hover text-accent-contrast text-sm font-medium rounded-lg px-3 py-1.5 transition-colors"
                                                        on:click=do_add_branch
                                                    >"Add"</button>
                                                    <button
                                                        class="flex-1 bg-overlay hover:bg-overlay-strong text-secondary text-sm rounded-lg px-3 py-1.5 transition-colors"
                                                        on:click=move |_| set_adding_branch_to.set(None)
                                                    >"Cancel"</button>
                                                </div>
                                            </div>
                                        })}
                                    </div>

                                    // Node editor panel
                                    <div class="flex-1 flex flex-col gap-4">
                                        {move || {
                                            if selected_node_id.get().is_none() {
                                                return view! {
                                                    <div class="flex-1 flex items-center justify-center bg-elevated/30 border border-divider/30 rounded-xl">
                                                        <p class="text-dimmed text-sm">"Click a node to edit its draft"</p>
                                                    </div>
                                                }.into_any();
                                            }

                                            view! {
                                                <NodeEditor
                                                    node_label=node_label
                                                    set_node_label=set_node_label
                                                    node_notes=node_notes
                                                    set_node_notes=set_node_notes
                                                    node_improvised=node_improvised
                                                    set_node_improvised=set_node_improvised
                                                    draft_slots=draft_slots
                                                    set_draft_slots=set_draft_slots
                                                    active_slot=active_slot
                                                    set_active_slot=set_active_slot
                                                    highlighted_slot=highlighted_slot
                                                    on_slot_clear=on_slot_clear
                                                    champions_resource=champions_resource
                                                    used_champions=Signal::derive(used_champions)
                                                    on_champion_select=on_champion_select
                                                    on_slot_click=on_slot_click
                                                    on_slot_drop=on_slot_drop
                                                    on_save=Callback::new(do_save_node)
                                                    on_branch_from=do_branch_from
                                                    node_save_status=node_save_status
                                                />
                                            }.into_any()
                                        }}
                                    </div>
                                </div>
                            }.into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Tree node view (recursive, indented)
// ---------------------------------------------------------------------------

#[component]
fn TreeNodeView(
    node: DraftTreeNode,
    depth: usize,
    selected_node_id: ReadSignal<Option<String>>,
    expanded_nodes: ReadSignal<std::collections::HashSet<String>>,
    set_expanded_nodes: WriteSignal<std::collections::HashSet<String>>,
    on_select: Callback<DraftTreeNode>,
    on_add_branch: Callback<String>,
    on_delete: Callback<String>,
) -> impl IntoView {
    let node_id = node.id.clone().unwrap_or_default();
    let has_children = !node.children.is_empty();
    let children = node.children.clone();
    let node_for_select = node.clone();
    let node_id_for_expand = node_id.clone();
    let node_id_for_add = node_id.clone();
    let node_id_for_delete = node_id.clone();
    let node_id_for_selected = node_id.clone();
    let is_root = node.parent_id.is_none();
    let is_improvised = node.is_improvised;
    let has_actions = !node.actions.is_empty();
    let action_count = node.actions.len();
    let label = node.label.clone();

    view! {
        <div style=format!("padding-left: {}rem", depth as f64 * 0.75)>
            <div
                class=move || {
                    let selected = selected_node_id.get().as_deref() == Some(&node_id_for_selected);
                    if selected {
                        "group flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg bg-accent/15 border border-accent/30 cursor-pointer transition-all"
                    } else {
                        "group flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg hover:bg-overlay/50 cursor-pointer transition-all"
                    }
                }
            >
                // Expand/collapse toggle
                {if has_children {
                    let nid = node_id_for_expand.clone();
                    view! {
                        <button
                            class="text-muted hover:text-primary text-xs w-4 h-4 flex items-center justify-center flex-shrink-0 transition-colors"
                            on:click=move |ev| {
                                ev.stop_propagation();
                                let nid = nid.clone();
                                set_expanded_nodes.update(|s| {
                                    if s.contains(&nid) {
                                        s.remove(&nid);
                                    } else {
                                        s.insert(nid);
                                    }
                                });
                            }
                        >
                            {move || {
                                let expanded = expanded_nodes.get();
                                if expanded.contains(&node_id_for_expand) { "\u{25BE}" } else { "\u{25B8}" }
                            }}
                        </button>
                    }.into_any()
                } else {
                    view! { <span class="w-4 flex-shrink-0"></span> }.into_any()
                }}

                // Node content — clickable
                <div
                    class="flex-1 flex items-center gap-1.5 min-w-0"
                    on:click={
                        let node_for_select = node_for_select.clone();
                        move |_| on_select.run(node_for_select.clone())
                    }
                >
                    // Icon
                    {if is_improvised {
                        view! { <span class="text-amber-400 text-xs flex-shrink-0" title="Improvised">"\u{26A1}"</span> }.into_any()
                    } else if is_root {
                        view! { <span class="text-emerald-400 text-xs flex-shrink-0">"\u{25C9}"</span> }.into_any()
                    } else {
                        view! { <span class="text-dimmed text-xs flex-shrink-0">"\u{251C}"</span> }.into_any()
                    }}

                    <span class="text-primary text-sm truncate">{label}</span>

                    // Action count badge
                    {has_actions.then(|| view! {
                        <span class="text-dimmed text-xs flex-shrink-0">{format!("({action_count})")}</span>
                    })}
                </div>

                // Action buttons (visible on hover)
                <div class="hidden group-hover:flex items-center gap-1 flex-shrink-0">
                    <button
                        class="text-muted hover:text-accent hover:bg-overlay/50 text-sm p-1.5 rounded w-7 h-7 flex items-center justify-center transition-colors cursor-pointer"
                        title="Add branch"
                        on:click={
                            let nid = node_id_for_add.clone();
                            move |ev: web_sys::MouseEvent| {
                                ev.stop_propagation();
                                on_add_branch.run(nid.clone());
                            }
                        }
                    >"+"</button>
                    {(!is_root).then(|| {
                        let nid = node_id_for_delete.clone();
                        view! {
                            <button
                                class="text-muted hover:text-red-400 hover:bg-overlay/50 text-sm p-1.5 rounded w-7 h-7 flex items-center justify-center transition-colors cursor-pointer"
                                title="Delete node"
                                on:click=move |ev: web_sys::MouseEvent| {
                                    ev.stop_propagation();
                                    on_delete.run(nid.clone());
                                }
                            >"\u{00D7}"</button>
                        }
                    })}
                </div>
            </div>

            // Children (if expanded)
            {move || {
                let expanded = expanded_nodes.get();
                if !has_children || !expanded.contains(&node_id) {
                    return view! { <div></div> }.into_any();
                }
                let children = children.clone();
                view! {
                    <div class="flex flex-col gap-0.5 mt-0.5">
                        {children.into_iter().map(|child| {
                            view! {
                                <TreeNodeView
                                    node=child
                                    depth=depth + 1
                                    selected_node_id=selected_node_id
                                    expanded_nodes=expanded_nodes
                                    set_expanded_nodes=set_expanded_nodes
                                    on_select=on_select
                                    on_add_branch=on_add_branch
                                    on_delete=on_delete
                                />
                            }
                        }).collect_view()}
                    </div>
                }.into_any()
            }}
        </div>
    }
}

// ---------------------------------------------------------------------------
// Node editor
// ---------------------------------------------------------------------------

#[component]
fn NodeEditor(
    node_label: ReadSignal<String>,
    set_node_label: WriteSignal<String>,
    node_notes: ReadSignal<String>,
    set_node_notes: WriteSignal<String>,
    node_improvised: ReadSignal<bool>,
    set_node_improvised: WriteSignal<bool>,
    draft_slots: ReadSignal<Vec<Option<String>>>,
    set_draft_slots: WriteSignal<Vec<Option<String>>>,
    active_slot: ReadSignal<Option<usize>>,
    set_active_slot: WriteSignal<Option<usize>>,
    highlighted_slot: ReadSignal<Option<usize>>,
    on_slot_clear: Callback<usize>,
    champions_resource: Resource<Result<Vec<Champion>, ServerFnError>>,
    used_champions: Signal<Vec<String>>,
    on_champion_select: Callback<Champion>,
    on_slot_click: Callback<usize>,
    on_slot_drop: Callback<(usize, String)>,
    on_save: Callback<web_sys::MouseEvent>,
    on_branch_from: Callback<usize>,
    node_save_status: ReadSignal<&'static str>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-4">
            // Node metadata
            <div class="bg-elevated/50 border border-divider/50 rounded-xl p-4 flex flex-col gap-3">
                <div class="flex items-center gap-4">
                    <div class="flex-1">
                        <label class="block text-muted text-xs font-medium mb-1">"Node Label"</label>
                        <input
                            type="text"
                            prop:value=move || node_label.get()
                            on:input=move |ev| set_node_label.set(event_target_value(&ev))
                            class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm focus:outline-none focus:border-accent/50 transition-colors"
                        />
                    </div>
                    <div class="flex items-center gap-2 pt-5">
                        <label class="relative inline-flex items-center cursor-pointer">
                            <input
                                type="checkbox"
                                class="sr-only peer"
                                prop:checked=move || node_improvised.get()
                                on:change=move |ev| {
                                    let checked = event_target_checked(&ev);
                                    set_node_improvised.set(checked);
                                }
                            />
                            <div class="w-9 h-5 bg-overlay peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-gray-300 after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-amber-500 peer-checked:after:bg-white"></div>
                        </label>
                        <span class="text-secondary text-xs">"Improvised"</span>
                    </div>
                </div>
                <div>
                    <label class="block text-muted text-xs font-medium mb-1">"Notes"</label>
                    <textarea
                        rows="2"
                        prop:value=move || node_notes.get()
                        on:input=move |ev| set_node_notes.set(event_target_value(&ev))
                        placeholder="Strategy notes for this branch..."
                        class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm placeholder-dimmed focus:outline-none focus:border-accent/50 resize-none transition-colors"
                    />
                </div>
            </div>

            // Draft board
            <div class="bg-elevated/50 border border-divider/50 rounded-xl p-4">
                <Suspense fallback=|| view! { <div class="text-dimmed text-sm text-center py-8">"Loading champions..."</div> }>
                    {move || champions_resource.get().map(|result| match result {
                        Err(e) => view! {
                            <div class="text-red-400 text-sm">"Failed to load champions: " {e.to_string()}</div>
                        }.into_any(),
                        Ok(champs) => {
                            let champion_map: HashMap<String, Champion> = champs
                                .into_iter()
                                .map(|c| (c.id.clone(), c))
                                .collect();
                            view! {
                                <DraftBoard
                                    draft_slots=draft_slots
                                    champion_map=champion_map
                                    active_slot=active_slot
                                    on_slot_click=on_slot_click
                                    on_slot_drop=on_slot_drop
                                    highlighted_slot=highlighted_slot
                                    on_slot_clear=on_slot_clear
                                />
                            }.into_any()
                        }
                    })}
                </Suspense>
            </div>

            // Branch from position
            {move || {
                active_slot.get().map(|slot_idx| {
                    let (side, kind, num) = slot_meta(slot_idx);
                    view! {
                        <div class="flex items-center gap-3 px-1">
                            <button
                                class="bg-purple-600 hover:bg-purple-500 text-white text-sm font-medium px-4 py-1.5 rounded-lg transition-colors cursor-pointer"
                                on:click=move |_| on_branch_from.run(slot_idx)
                            >
                                "Branch from here"
                            </button>
                            <span class="text-dimmed text-xs">
                                {format!("Create branch copying through {} {} {}", side, kind, num)}
                            </span>
                        </div>
                    }
                })
            }}

            // Champion picker
            <div class="bg-elevated/50 border border-divider/50 rounded-xl p-4">
                <Suspense fallback=|| view! { <div class="text-dimmed text-sm">"Loading..."</div> }>
                    {move || champions_resource.get().map(|result| match result {
                        Err(e) => view! {
                            <ErrorBanner message=format!("Failed to load champions: {e}") />
                        }.into_any(),
                        Ok(champs) => view! {
                            <ChampionPicker
                                champions=champs
                                used_champions=used_champions.get()
                                on_select=on_champion_select
                            />
                        }.into_any(),
                    })}
                </Suspense>
            </div>

            // Save + clear
            <div class="flex gap-3 items-center">
                <button
                    class="bg-accent hover:bg-accent-hover text-accent-contrast font-semibold rounded-lg px-6 py-2 text-sm transition-colors"
                    on:click=move |ev| on_save.run(ev)
                >"Save Node"</button>
                <button
                    class="bg-overlay hover:bg-overlay-strong text-secondary rounded-lg px-4 py-2 text-sm transition-colors"
                    on:click=move |_| {
                        set_draft_slots.set(vec![None; 20]);
                        set_active_slot.set(Some(0));
                    }
                >"Clear Board"</button>
                {move || {
                    let status = node_save_status.get();
                    match status {
                        "saved" => view! { <span class="text-green-400 text-xs">"✓ Saved"</span> }.into_any(),
                        "unsaved" => view! { <span class="text-amber-400 text-xs">"● Unsaved"</span> }.into_any(),
                        _ => view! { <span></span> }.into_any(),
                    }
                }}
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Live navigator
// ---------------------------------------------------------------------------

#[component]
fn LiveNavigator(
    nodes_resource: Resource<Result<Vec<DraftTreeNode>, ServerFnError>>,
    nav_path: ReadSignal<Vec<String>>,
    set_nav_path: WriteSignal<Vec<String>>,
    champions_resource: Resource<Result<Vec<Champion>, ServerFnError>>,
    status_msg: WriteSignal<Option<String>>,
    selected_tree_id: ReadSignal<Option<String>>,
) -> impl IntoView {
    // Find the current node based on nav_path
    let current_node = move || -> Option<DraftTreeNode> {
        let roots = nodes_resource.get()?.ok()?;
        let path = nav_path.get();
        if path.is_empty() {
            return roots.into_iter().next();
        }
        // Walk the tree following path
        let mut current: Option<DraftTreeNode> = roots.into_iter().next();
        for step_id in &path {
            current = current.and_then(|n| {
                n.children
                    .into_iter()
                    .find(|c| c.id.as_deref() == Some(step_id.as_str()))
            });
        }
        current
    };

    view! {
        <div class="flex-1 flex flex-col gap-4">
            // Breadcrumb path
            <div class="bg-elevated/50 border border-divider/50 rounded-xl p-4">
                <div class="flex items-center gap-2 flex-wrap">
                    <span class="text-muted text-xs font-semibold uppercase tracking-wider">"Path:"</span>
                    <button
                        class="text-emerald-400 hover:text-emerald-300 text-sm font-medium transition-colors"
                        on:click=move |_| set_nav_path.set(Vec::new())
                    >"Root"</button>
                    {move || {
                        let path = nav_path.get();
                        if path.is_empty() {
                            return view! { <span></span> }.into_any();
                        }
                        view! {
                            <div class="flex items-center gap-2">
                                {path.into_iter().enumerate().map(|(i, _step_id)| {
                                    let truncated_path: Vec<String> = nav_path.get_untracked().into_iter().take(i + 1).collect();
                                    let label = {
                                        // Find label for this step
                                        let roots = nodes_resource.get().and_then(|r| r.ok()).unwrap_or_default();
                                        let mut node: Option<&DraftTreeNode> = roots.first();
                                        for (j, sid) in truncated_path.iter().enumerate() {
                                            if j == 0 { continue; }
                                            node = node.and_then(|n| n.children.iter().find(|c| c.id.as_deref() == Some(sid.as_str())));
                                        }
                                        node.map(|n| n.label.clone()).unwrap_or_else(|| "...".to_string())
                                    };
                                    view! {
                                        <span class="text-overlay-strong">"\u{203A}"</span>
                                        <button
                                            class="text-accent hover:text-accent-hover text-sm font-medium transition-colors"
                                            on:click=move |_| {
                                                set_nav_path.set(truncated_path.clone());
                                            }
                                        >{label}</button>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }}
                </div>
            </div>

            // Current node display
            <Suspense fallback=|| view! { <div class="text-dimmed text-center py-8">"Loading..."</div> }>
                {move || {
                    let node = current_node();
                    match node {
                        None => view! {
                            <div class="flex-1 flex items-center justify-center bg-elevated/30 border border-divider/30 rounded-xl">
                                <p class="text-dimmed">"No node found at this path."</p>
                            </div>
                        }.into_any(),
                        Some(node) => {
                            let node_id = node.id.clone().unwrap_or_default();
                            let node_label = node.label.clone();
                            let node_notes = node.notes.clone();
                            let node_improvised = node.is_improvised;
                            let children = node.children.clone();
                            let has_children = !children.is_empty();
                            let actions = node.actions.clone();
                            let slots = actions_to_slots(&actions);

                            view! {
                                <div class="flex flex-col gap-4">
                                    // Node header
                                    <div class="bg-elevated/50 border border-divider/50 rounded-xl p-4">
                                        <div class="flex items-center gap-3">
                                            <h2 class="text-primary text-xl font-bold">{node_label}</h2>
                                            {node_improvised.then(|| view! {
                                                <span class="bg-amber-500/20 text-amber-400 text-xs font-semibold px-2 py-0.5 rounded-full">
                                                    "\u{26A1} Improvised"
                                                </span>
                                            })}
                                        </div>
                                        {node_notes.map(|notes| view! {
                                            <p class="text-secondary text-sm mt-2 whitespace-pre-wrap">{notes}</p>
                                        })}
                                    </div>

                                    // Draft board (read-only display)
                                    <div class="bg-elevated/50 border border-divider/50 rounded-xl p-4">
                                        <Suspense fallback=|| view! { <div class="text-dimmed text-sm">"Loading..."</div> }>
                                            {move || champions_resource.get().map(|result| match result {
                                                Err(_) => view! { <div class="text-dimmed">"Champions unavailable"</div> }.into_any(),
                                                Ok(champs) => {
                                                    let champion_map: HashMap<String, Champion> = champs
                                                        .into_iter()
                                                        .map(|c| (c.id.clone(), c))
                                                        .collect();
                                                    let (ro_slots, _) = signal(slots.clone());
                                                    let (ro_active, _) = signal(None::<usize>);
                                                    let (ro_highlighted, _) = signal(None::<usize>);
                                                    let noop_click = Callback::new(|_: usize| {});
                                                    let noop_drop = Callback::new(|_: (usize, String)| {});
                                                    let noop_clear = Callback::new(|_: usize| {});
                                                    view! {
                                                        <DraftBoard
                                                            draft_slots=ro_slots
                                                            champion_map=champion_map
                                                            active_slot=ro_active
                                                            on_slot_click=noop_click
                                                            on_slot_drop=noop_drop
                                                            highlighted_slot=ro_highlighted
                                                            on_slot_clear=noop_clear
                                                        />
                                                    }.into_any()
                                                }
                                            })}
                                        </Suspense>
                                    </div>

                                    // Branch selection
                                    <div class="bg-elevated/50 border border-divider/50 rounded-xl p-4 flex flex-col gap-3">
                                        <div class="flex items-center justify-between">
                                            <h3 class="text-primary font-semibold">"Choose Path"</h3>
                                            // Improvise button
                                            <button
                                                class="bg-amber-500/20 hover:bg-amber-500/30 text-amber-400 text-sm font-medium px-3 py-1.5 rounded-lg border border-amber-500/30 transition-colors"
                                                on:click={
                                                    let nid = node_id.clone();
                                                    let tree_id = selected_tree_id.get_untracked();
                                                    move |_| {
                                                        let nid = nid.clone();
                                                        let tree_id = tree_id.clone();
                                                        if let Some(tree_id) = tree_id {
                                                            let status_msg = status_msg;
                                                            leptos::task::spawn_local(async move {
                                                                match add_branch(tree_id, Some(nid), "Improvised".to_string()).await {
                                                                    Ok(new_id) => {
                                                                        let _ = save_node(new_id, "Improvised".to_string(), None, true, "[]".to_string()).await;
                                                                        status_msg.set(Some("Improvised branch created!".into()));
                                                                        nodes_resource.refetch();
                                                                    }
                                                                    Err(e) => status_msg.set(Some(format!("Error: {e}"))),
                                                                }
                                                            });
                                                        }
                                                    }
                                                }
                                            >
                                                "\u{26A1} Improvise"
                                            </button>
                                        </div>
                                        {if has_children {
                                            let children = children.clone();
                                            view! {
                                                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2">
                                                    {children.into_iter().map(|child| {
                                                        let child_id = child.id.clone().unwrap_or_default();
                                                        let child_label = child.label.clone();
                                                        let child_notes = child.notes.clone();
                                                        let child_improvised = child.is_improvised;
                                                        let child_action_count = child.actions.len();
                                                        view! {
                                                            <button
                                                                class="text-left bg-overlay/50 hover:bg-overlay border border-outline/50 hover:border-accent/30 rounded-lg p-3 transition-all group"
                                                                on:click=move |_| {
                                                                    set_nav_path.update(|p| p.push(child_id.clone()));
                                                                }
                                                            >
                                                                <div class="flex items-center gap-2">
                                                                    <span class="text-primary text-sm font-medium group-hover:text-accent transition-colors">{child_label}</span>
                                                                    {child_improvised.then(|| view! {
                                                                        <span class="text-amber-400 text-xs">"\u{26A1}"</span>
                                                                    })}
                                                                </div>
                                                                {child_notes.map(|n| view! {
                                                                    <p class="text-muted text-xs mt-1 truncate">{n}</p>
                                                                })}
                                                                {(child_action_count > 0).then(|| view! {
                                                                    <p class="text-dimmed text-xs mt-1">{format!("{child_action_count} picks/bans")}</p>
                                                                })}
                                                            </button>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <p class="text-dimmed text-sm">"No branches yet. Click Improvise to create one."</p>
                                            }.into_any()
                                        }}
                                    </div>

                                    // Back button
                                    {move || {
                                        let path = nav_path.get();
                                        if path.is_empty() {
                                            return view! { <div></div> }.into_any();
                                        }
                                        view! {
                                            <button
                                                class="bg-overlay hover:bg-overlay-strong text-secondary rounded-lg px-4 py-2 text-sm transition-colors self-start"
                                                on:click=move |_| {
                                                    set_nav_path.update(|p| { p.pop(); });
                                                }
                                            >"\u{2190} Back"</button>
                                        }.into_any()
                                    }}
                                </div>
                            }.into_any()
                        }
                    }
                }}
            </Suspense>
        </div>
    }
}
