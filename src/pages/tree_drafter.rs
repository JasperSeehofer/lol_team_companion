//! Draft trees: a tree of draft plans (the root draft, then a branch for
//! each thing the opponent might do), edited on the shared draft board and
//! walked branch by branch in Live game mode.

use crate::components::champion_picker::ChampionPicker;
use crate::components::draft_board::slot_meta;
use crate::components::tree_graph::TreeGraph;
use crate::components::ui::a11y::focus_id;
use crate::components::ui::{
    slot_dom_id, Badge, BadgeTone, Button, ButtonSize, ButtonVariant, Callout, CalloutTone,
    ConfirmDialog, ConfirmTone, Density, DraftBoard, EmptySize, EmptyState, Field, HeadingLevel,
    NoTeamState, PageHeader, PageShell, SectionHeader, Segment, SegmentSize, SegmentedControl,
    Skeleton, SkeletonGroup, SkeletonShape, Switch, TextField, Toast, ToastContext,
    TOURNAMENT_ORDER,
};
use crate::models::champion::Champion;
use crate::models::draft::{DraftAction, DraftTree, DraftTreeNode};
use leptos::prelude::*;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Server functions
// ---------------------------------------------------------------------------

#[server]
pub async fn get_champions_for_tree() -> Result<Vec<Champion>, ServerFnError> {
    use crate::server::authz;
    use crate::server::data_dragon;
    data_dragon::champions().await.map_err(authz::server_err)
}

/// Whether the current user belongs to a team. Drives the empty state of the
/// tree list (no team: point to the roster page; team but no trees: point to
/// the create form).
#[server]
pub async fn user_has_team() -> Result<bool, ServerFnError> {
    use crate::server::{authz, db};

    let Some(user) = authz::optional_user().await? else {
        return Ok(false);
    };
    let db = authz::db()?;
    let team_id = db::get_user_team_id(&db, &user.id)
        .await
        .map_err(authz::server_err)?;
    Ok(team_id.is_some())
}

#[server]
pub async fn list_trees() -> Result<Vec<DraftTree>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;

    let team_id = match db::get_user_team_id(&db, &user.id)
        .await
        .map_err(authz::server_err)?
    {
        Some(id) => id,
        None => return Ok(Vec::new()),
    };

    db::list_draft_trees(&db, &team_id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn create_tree(name: String, opponent: Option<String>) -> Result<String, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;

    let team_id = db::get_user_team_id(&db, &user.id)
        .await
        .map_err(authz::server_err)?
        .ok_or_else(|| ServerFnError::new(authz::NO_TEAM))?;

    db::create_draft_tree(&db, &team_id, &user.id, name, opponent)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn delete_tree(tree_id: String) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;
    let team_id = authz::require_team(&db, &user).await?;

    db::delete_draft_tree(&db, &team_id, &tree_id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn update_tree_meta(
    tree_id: String,
    name: String,
    opponent: Option<String>,
) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;
    let team_id = authz::require_team(&db, &user).await?;

    db::update_draft_tree(&db, &team_id, &tree_id, name, opponent)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn get_tree_nodes(tree_id: String) -> Result<Vec<DraftTreeNode>, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;
    let team_id = authz::require_team(&db, &user).await?;

    db::get_tree_nodes(&db, &team_id, &tree_id)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn add_branch(
    tree_id: String,
    parent_id: Option<String>,
    label: String,
) -> Result<String, ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;
    let team_id = authz::require_team(&db, &user).await?;

    db::create_tree_node(&db, &team_id, &tree_id, parent_id, label)
        .await
        .map_err(authz::server_err)
}

#[server]
pub async fn save_node(
    node_id: String,
    label: String,
    notes: Option<String>,
    is_improvised: bool,
    actions_json: String,
) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;
    let team_id = authz::require_team(&db, &user).await?;

    let actions: Vec<DraftAction> =
        serde_json::from_str(&actions_json).map_err(authz::server_err)?;

    db::update_tree_node(
        &db,
        &team_id,
        &node_id,
        label,
        notes,
        is_improvised,
        actions,
    )
    .await
    .map_err(authz::server_err)
}

#[server]
pub async fn remove_node(node_id: String) -> Result<(), ServerFnError> {
    use crate::server::authz;
    use crate::server::db;

    let authz::Ctx { db, user } = authz::authed().await?;
    let team_id = authz::require_team(&db, &user).await?;

    db::delete_tree_node(&db, &team_id, &node_id)
        .await
        .map_err(authz::server_err)
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
                    role: None,
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

/// Champion ids (stored) to display names (shown on the board).
fn display_slots(slots: &[Option<String>], champs: &[Champion]) -> Vec<Option<String>> {
    slots
        .iter()
        .map(|s| {
            s.as_ref().map(|id| {
                champs
                    .iter()
                    .find(|c| c.id == *id || c.name == *id)
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| id.clone())
            })
        })
        .collect()
}

/// The node being edited.
#[derive(Clone, Copy)]
struct NodeEd {
    id: RwSignal<Option<String>>,
    label: RwSignal<String>,
    notes: RwSignal<String>,
    improvised: RwSignal<bool>,
    /// Champion ids per slot.
    slots: RwSignal<Vec<Option<String>>>,
    active: RwSignal<Option<usize>>,
    /// Suppress auto-save while a node is being loaded (leptos rule 55).
    loading: RwSignal<bool>,
    status: RwSignal<&'static str>,
}

impl NodeEd {
    fn new() -> Self {
        Self {
            id: RwSignal::new(None),
            label: RwSignal::new(String::new()),
            notes: RwSignal::new(String::new()),
            improvised: RwSignal::new(false),
            slots: RwSignal::new(vec![None; 20]),
            active: RwSignal::new(Some(0)),
            loading: RwSignal::new(false),
            status: RwSignal::new(""),
        }
    }

    fn load(&self, node: Option<&DraftTreeNode>) {
        self.loading.set(true);
        self.status.set("");
        match node {
            Some(n) => {
                let slots = actions_to_slots(&n.actions);
                let next = slots.iter().position(|s| s.is_none());
                self.id.set(n.id.clone());
                self.label.set(n.label.clone());
                self.notes.set(n.notes.clone().unwrap_or_default());
                self.improvised.set(n.is_improvised);
                self.slots.set(slots);
                self.active.set(next);
            }
            None => {
                self.id.set(None);
                self.label.set(String::new());
                self.notes.set(String::new());
                self.improvised.set(false);
                self.slots.set(vec![None; 20]);
                self.active.set(Some(0));
            }
        }
        // Re-enable auto-save after this batch of updates.
        let loading = self.loading;
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::prelude::*;
            let cb = Closure::once(move || loading.set(false));
            if let Some(win) = web_sys::window() {
                let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    0,
                );
            }
            cb.forget();
        }
        #[cfg(not(feature = "hydrate"))]
        loading.set(false);
    }
}

#[component]
pub fn TreeDrafterPage() -> impl IntoView {
    let auth_user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());
    Effect::new(move || {
        if let Some(Ok(None)) = auth_user.get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/auth/login");
            }
        }
    });

    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let champions_resource = Resource::new(|| (), |_| get_champions_for_tree());
    let trees = Resource::new(|| (), |_| list_trees());
    let has_team = Resource::new(|| (), |_| user_has_team());
    let selected_tree = RwSignal::new(None::<String>);
    let nodes = Resource::new(
        move || selected_tree.get(),
        |id| async move {
            match id {
                Some(id) => get_tree_nodes(id).await,
                None => Ok(Vec::new()),
            }
        },
    );
    let mode = RwSignal::new("edit".to_string());
    let ned = NodeEd::new();

    // New tree.
    let new_name = RwSignal::new(String::new());
    let new_opp = RwSignal::new(String::new());
    let new_error = RwSignal::new(None::<String>);
    let create = Callback::new(move |_: leptos::ev::MouseEvent| {
        let name = new_name.get_untracked();
        if name.trim().is_empty() {
            new_error.set(Some("Give the tree a name.".into()));
            return;
        }
        new_error.set(None);
        let opp = new_opp.get_untracked();
        let opp = (!opp.trim().is_empty()).then_some(opp);
        leptos::task::spawn_local(async move {
            match create_tree(name.trim().to_string(), opp).await {
                Ok(id) => {
                    new_name.set(String::new());
                    new_opp.set(String::new());
                    ned.load(None);
                    selected_tree.set(Some(id));
                    trees.refetch();
                    toast.push.run(Toast::success("Tree created"));
                }
                Err(e) => toast
                    .push
                    .run(Toast::error(format!("Could not create the tree: {e}"))),
            }
        });
    });

    let confirm_delete = RwSignal::new(false);
    let on_delete_tree = Callback::new(move |_| {
        let Some(id) = selected_tree.get_untracked() else {
            return;
        };
        leptos::task::spawn_local(async move {
            match delete_tree(id).await {
                Ok(()) => {
                    selected_tree.set(None);
                    ned.load(None);
                    mode.set("edit".to_string());
                    trees.refetch();
                    toast.push.run(Toast::success("Tree deleted"));
                }
                Err(e) => toast
                    .push
                    .run(Toast::error(format!("Could not delete the tree: {e}"))),
            }
        });
    });

    let select_tree = Callback::new(move |id: String| {
        ned.load(None);
        mode.set("edit".to_string());
        selected_tree.set(Some(id));
    });

    let tree_open = Memo::new(move |_| selected_tree.get().is_some());
    let tree_title = move || {
        let id = selected_tree.get()?;
        trees
            .get()
            .and_then(|r| r.ok())
            .and_then(|l| l.into_iter().find(|t| t.id.as_deref() == Some(id.as_str())))
    };

    view! {
        <PageShell density=Density::Compact attr:data-feedback-label="Tree drafter">
            <PageHeader
                title="Draft trees"
                eyebrow="Draft"
                meta=move || view! {
                    <span>"Branch a draft plan: what you do if they ban or pick something else."</span>
                }
                actions=move || view! {
                    {move || selected_tree.get().map(|_| view! {
                        <SegmentedControl
                            id="tree-mode"
                            label="Mode"
                            options=vec![Segment::new("edit", "Edit"), Segment::new("live", "Live game")]
                            value=mode
                        />
                    })}
                }
            />
            <div class="grid grid-cols-1 gap-x-10 gap-y-10 lg:grid-cols-[16rem_minmax(0,1fr)]">
                // Tree list and the new tree form.
                <section aria-labelledby="trees-heading" class="flex min-w-0 flex-col gap-6">
                    <div>
                        <SectionHeader title="Trees" id="trees-heading" level=HeadingLevel::H2 />
                        <Suspense fallback=|| view! { <SkeletonGroup label="Loading trees"><Skeleton shape=SkeletonShape::Line lines=3 /></SkeletonGroup> }>
                            {move || trees.get().map(|r| match r {
                                Err(_) => view! { <Callout tone=CalloutTone::Danger>"The trees did not load."</Callout> }.into_any(),
                                Ok(list) if list.is_empty() => {
                                    if matches!(has_team.get(), Some(Ok(false))) {
                                        view! { <NoTeamState /> }.into_any()
                                    } else {
                                        view! {
                                            <EmptyState size=EmptySize::Compact message="Trees your team creates are listed here. Name one below to start." />
                                        }.into_any()
                                    }
                                }
                                Ok(list) => view! {
                                    <ul class="flex flex-col" data-testid="tree-list">
                                        {list.into_iter().map(|t| {
                                            let id = t.id.clone().unwrap_or_default();
                                            let id_sel = id.clone();
                                            let id_cur = id.clone();
                                            let opp = t.opponent.clone();
                                            view! {
                                                <li class="border-b border-rule last:border-b-0">
                                                    <Button
                                                        variant=ButtonVariant::Quiet
                                                        full_width=true
                                                        pressed=Signal::derive(move || selected_tree.get().as_deref() == Some(id_cur.as_str()))
                                                        on_click=Callback::new(move |_| select_tree.run(id_sel.clone()))
                                                    >
                                                        <span class="flex w-full min-w-0 items-baseline justify-between gap-2 text-left">
                                                            <span class="truncate">{t.name.clone()}</span>
                                                            {opp.map(|o| view! { <span class="truncate text-caption font-normal text-muted">"vs " {o}</span> })}
                                                        </span>
                                                    </Button>
                                                </li>
                                            }
                                        }).collect_view()}
                                    </ul>
                                }.into_any(),
                            })}
                        </Suspense>
                    </div>
                    <div class="flex flex-col gap-3 border-t border-rule pt-5">
                        <h3 class="font-display text-heading font-semibold text-primary">"New tree"</h3>
                        <Field id="tree-name" label="Name" required=true error=Signal::derive(move || new_error.get())>
                            <TextField value=new_name placeholder="Blue side vs Riverside" />
                        </Field>
                        <Field id="tree-opponent" label="Opponent">
                            <TextField value=new_opp placeholder="Riverside Rooks" />
                        </Field>
                        <div><Button on_click=create>"Create tree"</Button></div>
                    </div>
                </section>

                <div class="min-w-0">
                    {move || match tree_open.get() {
                        false => view! {
                            <EmptyState
                                title="No tree open"
                                level=HeadingLevel::H2
                                message="Choose a tree on the left, or create one. A tree starts with a root draft; each branch is what you do when the opponent does something else."
                            />
                        }.into_any(),
                        true => view! {
                            <div class="flex flex-col gap-8">
                                <div class="flex flex-wrap items-baseline justify-between gap-3 border-b border-rule pb-3">
                                    <p class="flex flex-wrap items-baseline gap-x-3">
                                        <span class="font-display text-title font-semibold text-primary">
                                            {move || tree_title().map(|t| t.name).unwrap_or_default()}
                                        </span>
                                        {move || tree_title().and_then(|t| t.opponent).map(|o| view! { <span class="text-body text-secondary">"vs " {o}</span> })}
                                    </p>
                                    <Button variant=ButtonVariant::Danger size=ButtonSize::Sm on_click=Callback::new(move |_| confirm_delete.set(true))>
                                        "Delete tree"
                                    </Button>
                                </div>
                                {move || if mode.get() == "live" {
                                    view! {
                                        <LiveNavigator nodes=nodes champions=champions_resource selected_tree=selected_tree />
                                    }.into_any()
                                } else {
                                    view! {
                                        <TreeEditor nodes=nodes champions=champions_resource selected_tree=selected_tree ned=ned />
                                    }.into_any()
                                }}
                            </div>
                        }.into_any(),
                    }}
                </div>
            </div>
            <ConfirmDialog
                id="delete-tree"
                open=confirm_delete
                title="Delete this tree?"
                message="The tree and all of its branches are deleted for the whole team."
                confirm_label="Delete tree"
                tone=ConfirmTone::Destructive
                on_confirm=on_delete_tree
            />
        </PageShell>
    }
}

/// Branches (list or graph) and the editor for the selected node.
#[component]
fn TreeEditor(
    nodes: Resource<Result<Vec<DraftTreeNode>, ServerFnError>>,
    champions: Resource<Result<Vec<Champion>, ServerFnError>>,
    selected_tree: RwSignal<Option<String>>,
    ned: NodeEd,
) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let view_mode = RwSignal::new("list".to_string());
    // Collapsed nodes (everything else is open).
    let expanded = RwSignal::new(std::collections::HashSet::<String>::new());
    let adding_to = RwSignal::new(None::<Option<String>>);
    let branch_label = RwSignal::new(String::new());

    let select = Callback::new(move |n: DraftTreeNode| ned.load(Some(&n)));
    let add_branch_cb = Callback::new(move |parent: String| {
        adding_to.set(Some(Some(parent)));
        branch_label.set(String::new());
        focus_id("branch-label");
    });
    let do_add = Callback::new(move |_: leptos::ev::MouseEvent| {
        let Some(tree_id) = selected_tree.get_untracked() else {
            return;
        };
        let parent = adding_to.get_untracked().flatten();
        let label = branch_label.get_untracked();
        if label.trim().is_empty() {
            toast.push.run(Toast::error("Give the branch a label."));
            return;
        }
        leptos::task::spawn_local(async move {
            match add_branch(tree_id.clone(), parent.clone(), label.trim().to_string()).await {
                Ok(id) => {
                    adding_to.set(None);
                    if let Some(p) = parent {
                        expanded.update(|e| {
                            e.remove(&p);
                        });
                    }
                    nodes.refetch();
                    let node = DraftTreeNode {
                        id: Some(id),
                        tree_id,
                        label: label.trim().to_string(),
                        notes: None,
                        is_improvised: false,
                        parent_id: None,
                        sort_order: 0,
                        actions: Vec::new(),
                        children: Vec::new(),
                    };
                    ned.load(Some(&node));
                    toast.push.run(Toast::success("Branch added"));
                }
                Err(e) => toast
                    .push
                    .run(Toast::error(format!("Could not add the branch: {e}"))),
            }
        });
    });
    let confirm_node = RwSignal::new(None::<String>);
    let confirm_node_open = RwSignal::new(false);
    let on_delete_node = Callback::new(move |id: String| {
        confirm_node.set(Some(id));
        confirm_node_open.set(true);
    });
    let do_delete_node = Callback::new(move |_| {
        let Some(id) = confirm_node.get_untracked() else {
            return;
        };
        leptos::task::spawn_local(async move {
            match remove_node(id.clone()).await {
                Ok(()) => {
                    if ned.id.get_untracked().as_deref() == Some(id.as_str()) {
                        ned.load(None);
                    }
                    nodes.refetch();
                    toast.push.run(Toast::success("Branch deleted"));
                }
                Err(e) => toast
                    .push
                    .run(Toast::error(format!("Could not delete: {e}"))),
            }
        });
    });

    view! {
        <section aria-labelledby="branches-heading" class="flex min-w-0 flex-col gap-4">
            <SectionHeader
                title="Branches"
                id="branches-heading"
                actions=move || view! {
                    <SegmentedControl
                        id="tree-view"
                        label="Show branches as"
                        options=vec![Segment::new("list", "List"), Segment::new("graph", "Graph")]
                        value=view_mode
                        size=SegmentSize::Sm
                    />
                }
            />
            <Suspense fallback=|| view! { <SkeletonGroup label="Loading branches"><Skeleton shape=SkeletonShape::Line lines=4 /></SkeletonGroup> }>
                {move || nodes.get().map(|r| match r {
                    Err(_) => view! { <Callout tone=CalloutTone::Danger>"The branches did not load."</Callout> }.into_any(),
                    Ok(roots) if roots.is_empty() => view! {
                        <EmptyState size=EmptySize::Compact message="This tree has no nodes yet. Add the root draft and it appears here." />
                        <div>
                            <Button on_click=Callback::new(move |_| { adding_to.set(Some(None)); branch_label.set("Root".to_string()); })>"Add the root draft"</Button>
                        </div>
                    }.into_any(),
                    Ok(roots) => {
                        if view_mode.get() == "graph" {
                            let champ_map = champions.get().and_then(|r| r.ok()).map(|c| c.into_iter().map(|c| (c.id.clone(), c)).collect::<HashMap<String, Champion>>()).unwrap_or_default();
                            view! {
                                <TreeGraph
                                    roots=roots.clone()
                                    selected_node_id=ned.id.read_only()
                                    on_select=select
                                    on_add_branch=add_branch_cb
                                    champion_map=StoredValue::new(champ_map)
                                    all_nodes=StoredValue::new(roots)
                                />
                                <p class="text-caption text-muted">
                                    "Thick edges lead to a branch where our picks or bans change, thin ones where theirs do; dashed nodes are improvised. The list view has the same branches and works with the keyboard."
                                </p>
                            }.into_any()
                        } else {
                            view! {
                                <ul role="tree" aria-label="Branches" class="flex flex-col border-y border-rule">
                                    {roots.into_iter().map(|n| view! {
                                        <TreeNodeRow
                                            node=n
                                            depth=0
                                            selected=ned.id
                                            expanded=expanded
                                            on_select=select
                                            on_add=add_branch_cb
                                            on_delete=on_delete_node
                                        />
                                    }).collect_view()}
                                </ul>
                            }.into_any()
                        }
                    }
                })}
            </Suspense>
            {move || adding_to.get().map(|parent| view! {
                <div class="flex flex-col gap-3 border-t border-rule pt-4 sm:flex-row sm:items-end">
                    <div class="min-w-0 flex-1">
                        <Field id="branch-label" label=if parent.is_some() { "New branch" } else { "Root draft" }>
                            <TextField value=branch_label placeholder="If they ban Jinx" />
                        </Field>
                    </div>
                    <div class="flex gap-2">
                        <Button variant=ButtonVariant::Primary on_click=do_add>"Add branch"</Button>
                        <Button variant=ButtonVariant::Quiet on_click=Callback::new(move |_| adding_to.set(None))>"Cancel"</Button>
                    </div>
                </div>
            })}
        </section>
        <NodeEditorSection ned=ned champions=champions selected_tree=selected_tree nodes=nodes />
        <ConfirmDialog
            id="delete-node"
            open=confirm_node_open
            title="Delete this branch?"
            message="The branch and every branch under it are deleted."
            confirm_label="Delete branch"
            tone=ConfirmTone::Destructive
            on_confirm=do_delete_node
        />
    }
}

/// One node of the branch list, with its children when expanded.
#[component]
fn TreeNodeRow(
    node: DraftTreeNode,
    depth: usize,
    selected: RwSignal<Option<String>>,
    expanded: RwSignal<std::collections::HashSet<String>>,
    on_select: Callback<DraftTreeNode>,
    on_add: Callback<String>,
    on_delete: Callback<String>,
) -> impl IntoView {
    let id = node.id.clone().unwrap_or_default();
    let has_children = !node.children.is_empty();
    let children = StoredValue::new(node.children.clone());
    let is_root = node.parent_id.is_none();
    let count = node.actions.len();
    let label = node.label.clone();
    let improvised = node.is_improvised;
    let node = StoredValue::new(node);
    let id_exp = id.clone();
    let id_sel = id.clone();
    let id_add = id.clone();
    let id_del = id.clone();
    let id_kids = id.clone();
    // `expanded` holds the collapsed nodes: branches start open.
    let is_open = Memo::new(move |_| expanded.with(|e| !e.contains(&id_exp)));
    let toggle = {
        let id = id.clone();
        Callback::new(move |_: leptos::ev::MouseEvent| {
            let id = id.clone();
            expanded.update(|e| {
                if !e.remove(&id) {
                    e.insert(id);
                }
            });
        })
    };
    let pad = format!("padding-left: {}rem", depth as f64 * 1.25);
    view! {
        <li role="treeitem" aria-expanded=move || has_children.then(|| if is_open.get() { "true" } else { "false" }) aria-selected=move || if selected.get().as_deref() == Some(id_sel.as_str()) { "true" } else { "false" }>
            <div class="flex items-center gap-1 border-b border-rule py-1" style=pad>
                {if has_children {
                    view! {
                        <Button
                            variant=ButtonVariant::Quiet
                            size=ButtonSize::Sm
                            aria_label="Show or hide the branches under this one"
                            on_click=toggle
                        >
                            <span aria-hidden="true">{move || if is_open.get() { "\u{25BE}" } else { "\u{25B8}" }}</span>
                        </Button>
                    }.into_any()
                } else {
                    view! { <span class="w-7 shrink-0" aria-hidden="true"></span> }.into_any()
                }}
                <div class="min-w-0 flex-1">
                    <Button
                        variant=ButtonVariant::Quiet
                        size=ButtonSize::Sm
                        full_width=true
                        pressed=Signal::derive({
                            let id = id.clone();
                            move || selected.get().as_deref() == Some(id.as_str())
                        })
                        on_click=Callback::new(move |_| on_select.run(node.get_value()))
                    >
                        <span class="flex w-full min-w-0 items-center gap-2 text-left">
                            <span class="truncate">{label.clone()}</span>
                            {is_root.then(|| view! { <Badge>"Root"</Badge> })}
                            {improvised.then(|| view! { <Badge tone=BadgeTone::Warning>"Improvised"</Badge> })}
                            {(count > 0).then(|| view! { <span class="figures ml-auto text-caption font-normal text-muted">{format!("{count} of 20")}</span> })}
                        </span>
                    </Button>
                </div>
                <Button variant=ButtonVariant::Quiet size=ButtonSize::Sm on_click=Callback::new(move |_| on_add.run(id_add.clone()))>
                    "Branch"
                </Button>
                {(!is_root).then(|| view! {
                    <Button variant=ButtonVariant::Quiet size=ButtonSize::Sm on_click=Callback::new(move |_| on_delete.run(id_del.clone()))>
                        "Delete"
                    </Button>
                })}
            </div>
            {move || (has_children && expanded.with(|e| !e.contains(&id_kids))).then(|| view! {
                <ul role="group" class="flex flex-col">
                    {children.get_value().into_iter().map(|c| view! {
                        <TreeNodeRow node=c depth=depth + 1 selected=selected expanded=expanded on_select=on_select on_add=on_add on_delete=on_delete />
                    }).collect_view()}
                </ul>
            })}
        </li>
    }
}

/// Label, notes, the node's draft board and picker, save and branch.
#[component]
fn NodeEditorSection(
    ned: NodeEd,
    champions: Resource<Result<Vec<Champion>, ServerFnError>>,
    selected_tree: RwSignal<Option<String>>,
    nodes: Resource<Result<Vec<DraftTreeNode>, ServerFnError>>,
) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    #[allow(unused_variables)]
    let timer: RwSignal<Option<i32>> = RwSignal::new(None);
    // Auto-save 2 s after the last change (values captured eagerly).
    Effect::new(move |_| {
        let id = ned.id.get();
        let label = ned.label.get();
        let notes = ned.notes.get();
        let improvised = ned.improvised.get();
        let slots = ned.slots.get();
        let Some(id) = id else { return };
        if ned.loading.get_untracked() {
            return;
        }
        ned.status.set("unsaved");
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::prelude::*;
            if let Some(t) = timer.get_untracked() {
                if let Some(win) = web_sys::window() {
                    win.clear_timeout_with_handle(t);
                }
            }
            let notes = (!notes.trim().is_empty()).then_some(notes);
            let json = serde_json::to_string(&build_actions_from_slots(&slots)).unwrap_or_default();
            let cb = Closure::once(move || {
                leptos::task::spawn_local(async move {
                    if save_node(id, label, notes, improvised, json).await.is_ok() {
                        ned.status.set("saved");
                        nodes.refetch();
                    }
                });
            });
            if let Some(win) = web_sys::window() {
                if let Ok(t) = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    2000,
                ) {
                    timer.set(Some(t));
                }
            }
            cb.forget();
        }
        #[cfg(not(feature = "hydrate"))]
        let _ = (id, label, notes, improvised, slots);
    });

    let save = Callback::new(move |_: leptos::ev::MouseEvent| {
        let Some(id) = ned.id.get_untracked() else {
            return;
        };
        let notes = ned.notes.get_untracked();
        let notes = (!notes.trim().is_empty()).then_some(notes);
        let json = serde_json::to_string(&build_actions_from_slots(&ned.slots.get_untracked()))
            .unwrap_or_default();
        let label = ned.label.get_untracked();
        let improvised = ned.improvised.get_untracked();
        leptos::task::spawn_local(async move {
            match save_node(id, label, notes, improvised, json).await {
                Ok(()) => {
                    ned.status.set("saved");
                    nodes.refetch();
                    toast.push.run(Toast::success("Node saved"));
                }
                Err(e) => toast.push.run(Toast::error(format!("Could not save: {e}"))),
            }
        });
    });

    let fill = move |slot: usize, name: String, champs: &[Champion]| {
        let id = champs
            .iter()
            .find(|c| c.id == name || c.name == name)
            .map(|c| c.id.clone())
            .unwrap_or(name);
        if ned
            .slots
            .with_untracked(|s| s.iter().any(|c| c.as_deref() == Some(id.as_str())))
        {
            return;
        }
        ned.slots.update(|s| s[slot] = Some(id));
        let next = ned
            .slots
            .with_untracked(|s| s.iter().position(|c| c.is_none()));
        ned.active.set(next);
    };
    let on_clear = Callback::new(move |slot: usize| {
        let before = ned.slots.with_untracked(|s| s[slot].clone());
        let Some(champ) = before else { return };
        ned.slots.update(|s| s[slot] = None);
        ned.active.set(Some(slot));
        let label = TOURNAMENT_ORDER[slot].describe();
        let undo = Callback::new(move |_| {
            ned.slots.update(|s| s[slot] = Some(champ.clone()));
            let next = ned
                .slots
                .with_untracked(|s| s.iter().position(|c| c.is_none()));
            ned.active.set(next);
        });
        toast
            .push
            .run(Toast::info(format!("Cleared {label}")).with_action("Undo", undo));
    });

    // Branch: a new child node holding this draft up to the lit slot.
    let branch_here = Callback::new(move |_: leptos::ev::MouseEvent| {
        let Some(tree_id) = selected_tree.get_untracked() else {
            return;
        };
        // Everything before the lit slot (the whole draft when complete).
        let end = ned.active.get_untracked().unwrap_or(20).min(20);
        let parent = ned.id.get_untracked();
        let current = ned.slots.get_untracked();
        let mut slots = vec![None; 20];
        slots[..end].clone_from_slice(&current[..end]);
        let label = match end {
            0 => "From the start".to_string(),
            e => format!("After {}", TOURNAMENT_ORDER[e - 1].describe()),
        };
        let json = serde_json::to_string(&build_actions_from_slots(&slots)).unwrap_or_default();
        leptos::task::spawn_local(async move {
            match add_branch(tree_id.clone(), parent, label.clone()).await {
                Ok(id) => {
                    let _ = save_node(id.clone(), label.clone(), None, false, json).await;
                    nodes.refetch();
                    let node = DraftTreeNode {
                        id: Some(id),
                        tree_id,
                        label,
                        notes: None,
                        is_improvised: false,
                        parent_id: None,
                        sort_order: 0,
                        actions: build_actions_from_slots(&slots),
                        children: Vec::new(),
                    };
                    ned.load(Some(&node));
                    toast
                        .push
                        .run(Toast::success("Branch created from this point"));
                }
                Err(e) => toast
                    .push
                    .run(Toast::error(format!("Could not branch: {e}"))),
            }
        });
    });

    // Re-render only when a node is opened or closed, not on every switch
    // between nodes (the board and picker stay mounted).
    let has_node = Memo::new(move |_| ned.id.get().is_some());
    view! {
        {move || match has_node.get() {
            false => view! {
                <section aria-labelledby="node-heading" class="mt-2">
                    <h2 id="node-heading" class="sr-only">"Node"</h2>
                    <EmptyState size=EmptySize::Compact message="Choose a node under Branches to edit its draft, notes and label." />
                </section>
            }.into_any(),
            true => view! {
                <section aria-labelledby="node-heading" class="flex min-w-0 flex-col gap-6">
                    <SectionHeader
                        title="Node"
                        id="node-heading"
                        actions=move || view! {
                            <span class="text-label text-muted" data-testid="node-save-state">
                                {move || match ned.status.get() { "saved" => "Saved", "unsaved" => "Unsaved changes", _ => "" }}
                            </span>
                            <Button variant=ButtonVariant::Primary on_click=save>"Save node"</Button>
                        }
                    />
                    <div class="grid grid-cols-1 gap-x-8 gap-y-4 md:grid-cols-[minmax(0,1fr)_auto]">
                        <Field id="node-label" label="Label">
                            <TextField value=ned.label />
                        </Field>
                        <div class="flex items-end pb-1.5">
                            <Switch label="Improvised" checked=ned.improvised />
                        </div>
                        <div class="md:col-span-2">
                            <Field id="node-notes" label="Notes">
                                <TextField value=ned.notes rows=2 placeholder="What we do on this branch and why." />
                            </Field>
                        </div>
                    </div>
                    <Suspense fallback=|| view! { <SkeletonGroup label="Loading champions"><Skeleton shape=SkeletonShape::Block size="h-80 w-full" /></SkeletonGroup> }>
                        {move || champions.get().map(|r| {
                            let champs = r.unwrap_or_default();
                            let champs_sv = StoredValue::new(champs.clone());
                            let shown = Signal::derive(move || champs_sv.with_value(|c| display_slots(&ned.slots.get(), c)));
                            let on_pick = Callback::new(move |c: Champion| {
                                if let Some(slot) = ned.active.get_untracked() {
                                    champs_sv.with_value(|list| fill(slot, c.id.clone(), list));
                                }
                            });
                            let on_drop = Callback::new(move |(slot, name): (usize, String)| champs_sv.with_value(|list| fill(slot, name, list)));
                            let unavailable = Signal::derive(move || ned.slots.get().into_iter().flatten().collect::<Vec<_>>());
                            view! {
                                <div class="grid grid-cols-1 gap-8 xl:grid-cols-[minmax(0,30rem)_minmax(0,1fr)]">
                                    <div class="flex min-w-0 flex-col gap-3">
                                        <DraftBoard
                                            id="tree-board"
                                            label="Node draft board"
                                            champions=shown
                                            current=ned.active
                                            on_select=Callback::new(move |i: usize| ned.active.set(Some(i)))
                                            on_open=Callback::new(move |_| focus_id(&crate::components::champion_picker::picker_input_id("tree-picker")))
                                            on_clear=on_clear
                                            on_drop=on_drop
                                            hints=true
                                        />
                                        <div class="flex flex-wrap items-center gap-2 px-5">
                                            <Button size=ButtonSize::Sm on_click=branch_here>"Branch from the lit slot"</Button>
                                            <span class="text-caption text-muted">"Copies the draft up to the lit slot into a new branch."</span>
                                        </div>
                                    </div>
                                    <ChampionPicker
                                        id="tree-picker"
                                        champions=champs
                                        unavailable=unavailable
                                        on_select=on_pick
                                        on_escape=Callback::new(move |_| focus_id(&slot_dom_id("tree-board", ned.active.get_untracked().unwrap_or(19))))
                                        label=Signal::derive(move || match ned.active.get() {
                                            Some(i) => format!("Champion for {}", TOURNAMENT_ORDER[i].describe()),
                                            None => "Draft complete: choose a slot to change it".to_string(),
                                        })
                                        enabled=Signal::derive(move || ned.active.get().is_some())
                                    />
                                </div>
                            }
                        })}
                    </Suspense>
                </section>
            }.into_any(),
        }}
    }
}

/// Live game: walk the tree branch by branch while the real draft runs.
#[component]
fn LiveNavigator(
    nodes: Resource<Result<Vec<DraftTreeNode>, ServerFnError>>,
    champions: Resource<Result<Vec<Champion>, ServerFnError>>,
    selected_tree: RwSignal<Option<String>>,
) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let path = RwSignal::new(Vec::<String>::new());
    let current = move || -> Option<DraftTreeNode> {
        let roots = nodes.get()?.ok()?;
        let mut cur = roots.into_iter().next();
        for id in path.get() {
            cur = cur.and_then(|n| {
                n.children
                    .into_iter()
                    .find(|c| c.id.as_deref() == Some(id.as_str()))
            });
        }
        cur
    };
    view! {
        <section aria-labelledby="live-heading" class="flex min-w-0 flex-col gap-6">
            <h2 id="live-heading" class="sr-only">"Live game"</h2>
            <Suspense fallback=|| view! { <SkeletonGroup label="Loading the tree"><Skeleton shape=SkeletonShape::Line lines=4 /></SkeletonGroup> }>
                {move || {
                    let Some(node) = current() else {
                        return view! {
                            <EmptyState size=EmptySize::Compact message="This tree has no root draft yet. Add one in Edit mode, then follow it here during the game." />
                        }.into_any();
                    };
                    let roots = nodes.get().and_then(|r| r.ok()).unwrap_or_default();
                    let trail: Vec<(Vec<String>, String)> = {
                        let mut out = Vec::new();
                        let mut cur: Option<DraftTreeNode> = roots.first().cloned();
                        let p = path.get();
                        for (i, id) in p.iter().enumerate() {
                            cur = cur.and_then(|n| n.children.into_iter().find(|c| c.id.as_deref() == Some(id.as_str())));
                            if let Some(c) = &cur {
                                out.push((p[..=i].to_vec(), c.label.clone()));
                            }
                        }
                        out
                    };
                    let root_label = roots.first().map(|r| r.label.clone()).unwrap_or_else(|| "Root".to_string());
                    let champs = champions.get().and_then(|r| r.ok()).unwrap_or_default();
                    let slots = display_slots(&actions_to_slots(&node.actions), &champs);
                    let node_id = node.id.clone().unwrap_or_default();
                    let children = node.children.clone();
                    view! {
                        <nav aria-label="Path through the tree" class="flex flex-wrap items-center gap-1 text-label">
                            <Button variant=ButtonVariant::Quiet size=ButtonSize::Sm on_click=Callback::new(move |_| path.set(Vec::new()))>{root_label}</Button>
                            {trail.into_iter().map(|(p, label)| view! {
                                <span class="text-muted" aria-hidden="true">"\u{203a}"</span>
                                <Button variant=ButtonVariant::Quiet size=ButtonSize::Sm on_click=Callback::new(move |_| path.set(p.clone()))>{label}</Button>
                            }).collect_view()}
                        </nav>
                        <div class="flex flex-col gap-2">
                            <p class="flex flex-wrap items-center gap-2">
                                <span class="font-display text-title font-semibold text-primary">{node.label.clone()}</span>
                                {node.is_improvised.then(|| view! { <Badge tone=BadgeTone::Warning>"Improvised"</Badge> })}
                            </p>
                            {node.notes.clone().map(|n| view! { <p class="max-w-prose whitespace-pre-wrap text-reading text-secondary">{n}</p> })}
                        </div>
                        <div class="max-w-[30rem]">
                            <DraftBoard id="tree-live-board" label="Draft on this branch" champions=Signal::stored(slots) />
                        </div>
                        <div class="flex flex-col gap-3">
                            <h3 class="font-display text-heading font-semibold text-primary">"What happened next?"</h3>
                            {if children.is_empty() {
                                view! { <p class="text-body text-secondary">"No branches from here. Improvise one if the game went another way."</p> }.into_any()
                            } else {
                                view! {
                                    <ul class="flex flex-col border-y border-rule">
                                        {children.into_iter().map(|c| {
                                            let id = c.id.clone().unwrap_or_default();
                                            let count = c.actions.len();
                                            view! {
                                                <li class="flex flex-wrap items-center gap-3 border-b border-rule py-2 last:border-b-0">
                                                    <Button on_click=Callback::new(move |_| path.update(|p| p.push(id.clone())))>{c.label.clone()}</Button>
                                                    {c.is_improvised.then(|| view! { <Badge tone=BadgeTone::Warning>"Improvised"</Badge> })}
                                                    {c.notes.clone().map(|n| view! { <span class="min-w-0 flex-1 truncate text-label text-secondary">{n}</span> })}
                                                    {(count > 0).then(|| view! { <span class="figures text-caption text-muted">{format!("{count} of 20")}</span> })}
                                                </li>
                                            }
                                        }).collect_view()}
                                    </ul>
                                }.into_any()
                            }}
                            <div class="flex flex-wrap gap-2">
                                <Button
                                    variant=ButtonVariant::Secondary
                                    on_click=Callback::new(move |_| {
                                        let (Some(tree), nid) = (selected_tree.get_untracked(), node_id.clone()) else { return };
                                        leptos::task::spawn_local(async move {
                                            match add_branch(tree, Some(nid), "Improvised".to_string()).await {
                                                Ok(id) => {
                                                    let _ = save_node(id, "Improvised".to_string(), None, true, "[]".to_string()).await;
                                                    nodes.refetch();
                                                    toast.push.run(Toast::success("Improvised branch added"));
                                                }
                                                Err(e) => toast.push.run(Toast::error(format!("{e}"))),
                                            }
                                        });
                                    })
                                >"Improvise a branch"</Button>
                                {move || (!path.get().is_empty()).then(|| view! {
                                    <Button variant=ButtonVariant::Quiet on_click=Callback::new(move |_| path.update(|p| { p.pop(); }))>"Back"</Button>
                                })}
                            </div>
                        </div>
                    }.into_any()
                }}
            </Suspense>
        </section>
    }
}
