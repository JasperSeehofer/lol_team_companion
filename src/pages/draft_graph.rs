//! The Draft Graph view of the draft page (M4 stage A): the ranked list of
//! candidates for the current pick (the graph's accessible twin), the "Why
//! this pick" panel, the comp profile, and the team's notes on pairs.
//!
//! Stage B adds the force-directed canvas
//! ([`DraftGraphCanvas`](crate::components::graph_canvas::DraftGraphCanvas))
//! at the top of [`DraftGraphPanel`]: it draws the same [`GraphResult`]
//! (`locked` as pinned nodes, the first `CANVAS_CANDIDATES` candidates as
//! free nodes, `edges`) and shares the panel's `focused` signal (a champion
//! id), so the canvas, the list and the Why panel follow one focus.

use std::collections::HashMap;

use leptos::prelude::*;

use crate::components::graph_canvas::DraftGraphCanvas;
use crate::components::ui::a11y::{focus_id, roving_index};
use crate::components::ui::{
    AvatarSize, Button, ButtonSize, ButtonVariant, Callout, CalloutTone, ChampionAvatar, Delta,
    Dialog, DialogSize, DraftSide, EmptySize, EmptyState, Field, Keycap, LegendBlock, LegendEntry,
    LineStyle, LineWeight, ReadoutSize, Role, RoleLabel, Segment, SegmentSize, SegmentedControl,
    Sentiment, SideMarker, SideText, Skeleton, SkeletonGroup, SkeletonShape, StatReadout, Swatch,
    SwatchInk, TextField, Toast, ToastContext, Trend, TOURNAMENT_ORDER,
};
use crate::models::champion::Champion;
use crate::models::graph::{
    Candidate, CompProfile, Component, DraftState, GraphResult, NewRelation, Reason, ReasonSource,
    RelationKind, Scaling, TeamRelation,
};

// ---------------------------------------------------------------------------
// Server functions
// ---------------------------------------------------------------------------

/// Rank the candidates for the draft's current step. Team notes, pools,
/// games and the opponent dossier are the signed-in user's team's (none
/// without a team: the trait rules alone). `state_json` is a
/// [`DraftState`].
#[server]
pub async fn graph_candidates(state_json: String) -> Result<GraphResult, ServerFnError> {
    use crate::server::graph::{evidence, fit, traits};
    use crate::server::{authz, db, rate_limit::Bucket};

    let ctx = authz::authed().await?;
    authz::rate_limit(Bucket::Graph).await?;
    let state: DraftState = serde_json::from_str(&state_json)
        .map_err(|_| ServerFnError::new("The draft could not be read."))?;
    if state.slots.len() > 20
        || state.roles.len() > 20
        || state.roles_guessed.len() > 20
        || state.tags.len() > 20
        || state.unavailable.len() > 200
        || state.current.is_some_and(|c| c >= 20)
    {
        return Err(ServerFnError::new("The draft could not be read."));
    }
    let started = std::time::Instant::now();
    let table = traits::ensure_loaded(&ctx.db)
        .await
        .map_err(authz::server_err)?;
    let team = ctx.team().await?;

    let mut relations = Vec::new();
    let mut pools = Vec::new();
    let mut scouted = Vec::new();
    let mut ev = std::sync::Arc::new(evidence::MatchEvidence::default());
    if let Some(team_id) = &team {
        relations = db::list_team_relations(&ctx.db, team_id)
            .await
            .map_err(authz::server_err)?;
        let mut members = db::team_pool_members(&ctx.db, team_id)
            .await
            .map_err(authz::server_err)?;
        // Starters first, so a substitute never shadows the starter.
        members.sort_by_key(|m| m.roster_type != "starter");
        for m in members {
            let entries = m
                .entries
                .iter()
                .filter_map(|(champion, tier)| {
                    table
                        .get(champion)
                        .map(|r| (r.traits.champion.clone(), tier.clone()))
                })
                .collect();
            pools.push(fit::PoolPlayer {
                username: m.username,
                role: Role::parse(&m.role),
                entries,
            });
        }
        ev = evidence::for_team(&ctx.db, team_id)
            .await
            .map_err(authz::server_err)?;
        if let Some(opp) = state.opponent_id.as_deref().filter(|s| !s.is_empty()) {
            // A dossier outside the team reads as "none", like a missing one.
            if let Ok(Some((_, players))) = db::get_opponent(&ctx.db, team_id, opp).await {
                for p in players {
                    let role = Role::parse(&p.role);
                    let name = if p.name.trim().is_empty() {
                        p.riot_summoner_name.clone().unwrap_or_default()
                    } else {
                        p.name.clone()
                    };
                    let mut seen = std::collections::HashSet::new();
                    for c in &p.recent_champions {
                        if seen.insert(c.clone()) {
                            scouted.push(fit::Scouted {
                                player: name.clone(),
                                role,
                                champion: c.clone(),
                            });
                        }
                    }
                }
            }
        }
    }
    let result = fit::rank(
        &table,
        &state,
        &fit::TeamKnowledge {
            team: team.is_some(),
            relations: &relations,
            pools: &pools,
            evidence: &ev,
            scouted: &scouted,
        },
    );
    tracing::debug!(
        elapsed_us = started.elapsed().as_micros() as u64,
        candidates = result.candidates.len(),
        "graph_candidates"
    );
    Ok(result)
}

/// The team's notes on pairs (empty without a team).
#[server]
pub async fn list_relations() -> Result<Vec<TeamRelation>, ServerFnError> {
    use crate::server::{authz, db};
    let ctx = authz::authed().await?;
    let Some(team_id) = ctx.team().await? else {
        return Ok(Vec::new());
    };
    db::list_team_relations(&ctx.db, &team_id)
        .await
        .map_err(authz::server_err)
}

/// Add a team note on a pair. `relation_json` is a [`NewRelation`];
/// champions may be ids or display names. Returns the new id.
#[server]
pub async fn add_relation(relation_json: String) -> Result<String, ServerFnError> {
    use crate::server::graph::traits;
    use crate::server::{authz, db};
    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    let mut rel: NewRelation = serde_json::from_str(&relation_json)
        .map_err(|_| ServerFnError::new("The note could not be read."))?;
    let table = traits::ensure_loaded(&ctx.db)
        .await
        .map_err(authz::server_err)?;
    let resolve = |s: &str| table.get(s).map(|r| r.traits.champion.clone());
    match (resolve(&rel.a), resolve(&rel.b)) {
        (Some(a), Some(b)) => {
            rel.a = a;
            rel.b = b;
        }
        _ => return Err(ServerFnError::new("Choose two champions from the list.")),
    }
    db::create_team_relation(&ctx.db, &team_id, &ctx.user.id, &ctx.user.username, rel)
        .await
        .map_err(authz::server_err)
}

/// Remove one of the team's notes on pairs.
#[server]
pub async fn remove_relation(id: String) -> Result<(), ServerFnError> {
    use crate::server::{authz, db};
    let ctx = authz::authed().await?;
    let team_id = ctx.require_team().await?;
    db::delete_team_relation(&ctx.db, &team_id, &id)
        .await
        .map_err(authz::server_err)
}

/// Roles per champion from the trait table, keyed by display name and id:
/// the draft page guesses a pick's role from it (open roles first).
#[server]
pub async fn trait_roles() -> Result<Vec<(String, String, Vec<Role>)>, ServerFnError> {
    use crate::server::{authz, graph::traits};
    let ctx = authz::authed().await?;
    let table = traits::ensure_loaded(&ctx.db)
        .await
        .map_err(authz::server_err)?;
    Ok(table
        .rows
        .iter()
        .map(|r| {
            (
                r.traits.champion.clone(),
                r.name.clone(),
                r.traits.roles.clone(),
            )
        })
        .collect())
}

// ---------------------------------------------------------------------------
// Pure helpers (shared by both targets, unit-tested)
// ---------------------------------------------------------------------------

/// How a reason's line is drawn, the same language as the canvas edges:
/// synergy solid, counter dashed, anything that works against dotted.
pub fn reason_line(r: &Reason) -> LineStyle {
    if r.strength < 0.0 {
        LineStyle::Dotted
    } else if r.component == Component::Counter {
        LineStyle::Dashed
    } else {
        LineStyle::Solid
    }
}

/// "+12", "0", "−3" (a real minus sign).
pub fn signed(points: i32) -> String {
    match points {
        p if p > 0 => format!("+{p}"),
        p if p < 0 => format!("\u{2212}{}", -p),
        _ => "0".to_string(),
    }
}

/// The part of fit in words: "Synergy +12 of ±20".
pub fn part_text(c: Component, points: i32, band: i32) -> String {
    format!("{} {} of \u{b1}{band}", c.label(), signed(points))
}

/// The slot the graph ranks for, in words: "Blue pick 3".
pub fn slot_text(slot: usize) -> String {
    TOURNAMENT_ORDER
        .get(slot)
        .map(|s| s.describe())
        .unwrap_or_default()
}

/// Roles as a phrase: "Jungle, Mid or Support".
pub fn roles_text(roles: &[Role]) -> String {
    let words: Vec<&str> = roles.iter().map(|r| r.label()).collect();
    match words.as_slice() {
        [] => String::new(),
        [one] => one.to_string(),
        [rest @ .., last] => format!("{} or {last}", rest.join(", ")),
    }
}

/// The pool standing in a few words, for the list: "Comfort · Pell".
pub fn pool_text(c: &Candidate) -> Option<String> {
    let p = c.pool.as_ref()?;
    let player = p.player.as_ref()?;
    Some(match &p.tier {
        Some(t) => format!(
            "{} \u{b7} {player}",
            capitalise(crate::models::graph::tier_label(t))
        ),
        None => format!("Not in {player}'s pool"),
    })
}

fn capitalise(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// The weight behind a reason, in words: "weight 3", "against, weight 2",
/// "n=7, corroborates +1", "n=6, evidence only", "+4 fit" (pool).
pub fn strength_text(r: &Reason) -> String {
    let w = r.strength.abs();
    let num = if (w - w.round()).abs() < 0.05 {
        format!("{}", w.round() as i32)
    } else {
        format!("{w:.1}")
    };
    match r.source {
        ReasonSource::Pool => format!("{} fit", signed((r.strength * 3.0).round() as i32)),
        ReasonSource::MatchData => {
            let n = r.games.unwrap_or(0);
            if r.strength == 0.0 {
                format!("n={n}, evidence only")
            } else if r.strength > 0.0 {
                format!("n={n}, corroborates +{num}")
            } else {
                format!("n={n}, weakens it by {num}")
            }
        }
        _ if r.strength < 0.0 => format!("against, weight {num}"),
        _ => format!("weight {num}"),
    }
}

/// One readout of the comp profile, with the change if the focused
/// candidate is picked.
#[derive(Clone, Debug, PartialEq)]
pub struct ProfileRow {
    pub label: &'static str,
    pub value: String,
    pub unit: Option<&'static str>,
    /// Signed change (or the new value for words), when it changes.
    pub change: Option<(String, Trend, Sentiment)>,
}

/// The profile readouts of one side, with the change `adds` would make.
pub fn profile_rows(p: &CompProfile, adds: Option<&CompProfile>) -> Vec<ProfileRow> {
    let num = |label: &'static str, cur: u8, add: Option<u8>, max: u8, low: bool| ProfileRow {
        label,
        value: cur.to_string(),
        unit: Some(match max {
            15 => "of 15",
            _ => "of 10",
        }),
        change: add.filter(|a| *a > 0).map(|a| {
            (
                format!("+{a}"),
                Trend::Up,
                if low {
                    Sentiment::Good
                } else {
                    Sentiment::Neutral
                },
            )
        }),
    };
    let n = p.picks.max(1);
    let mut rows = Vec::new();
    let mix = adds.map(|a| p.plus(a));
    rows.push(ProfileRow {
        label: "AD / AP",
        value: if p.picks == 0 {
            "0 / 0".into()
        } else {
            p.damage_mix()
        },
        unit: None,
        change: mix
            .as_ref()
            .map(|m| (m.damage_mix(), Trend::Flat, Sentiment::Neutral)),
    });
    rows.push(num(
        "Engage",
        p.engage,
        adds.map(|a| a.engage),
        10,
        p.picks >= 2 && p.engage < 2,
    ));
    rows.push(num(
        "Front line",
        p.frontline,
        adds.map(|a| a.frontline),
        10,
        p.picks >= 2 && p.frontline < 2,
    ));
    rows.push(num(
        "Crowd control",
        p.cc,
        adds.map(|a| a.cc),
        15,
        p.picks >= 2 && p.cc < 2 * n,
    ));
    rows.push(num(
        "Wave clear",
        p.wave_clear,
        adds.map(|a| a.wave_clear),
        10,
        p.picks >= 2 && p.wave_clear < n,
    ));
    let plan_word = |s: Option<Scaling>| match s {
        Some(s) => s.label().to_string(),
        None => "Even".to_string(),
    };
    let new_plan = mix.as_ref().map(|m| m.plan());
    rows.push(ProfileRow {
        label: "Strongest",
        value: if p.picks == 0 {
            "\u{2014}".into()
        } else {
            plan_word(p.plan())
        },
        unit: None,
        change: new_plan
            .filter(|np| *np != p.plan() || p.picks == 0)
            .map(|np| (plan_word(np), Trend::Flat, Sentiment::Neutral)),
    });
    rows
}

/// The pair "Argue with it" prefills: the candidate and the champion of its
/// strongest pair reason, with the relation that reason states.
pub fn argue_prefill(c: &Candidate) -> (String, String, &'static str) {
    let top = c.reasons.iter().find(|r| {
        r.with.is_some() && matches!(r.component, Component::Synergy | Component::Counter)
    });
    match top {
        Some(r) => {
            let with = r.with.clone().unwrap_or_default();
            let kind = match (r.component, r.strength < 0.0) {
                (Component::Synergy, _) => "synergy",
                (_, false) => "beats",
                (_, true) => "loses",
            };
            (c.champion.clone(), with, kind)
        }
        None => (c.champion.clone(), String::new(), "synergy"),
    }
}

/// The relation the dialog's fields describe.
pub fn relation_from_form(
    a: &str,
    b: &str,
    kind: &str,
    weight: &str,
    reason: &str,
) -> Result<NewRelation, String> {
    let weight: u8 = weight
        .parse()
        .map_err(|_| "Choose how strong it is.".to_string())?;
    let (a, b, kind) = match kind {
        "synergy" => (a, b, RelationKind::Synergy),
        "beats" => (a, b, RelationKind::Counter),
        "loses" => (b, a, RelationKind::Counter),
        _ => return Err("Choose what the relation is.".into()),
    };
    NewRelation {
        a: a.to_string(),
        b: b.to_string(),
        kind,
        weight,
        reason: reason.to_string(),
    }
    .normalised()
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// What the panel reads from the draft editor.
#[derive(Clone, Copy)]
pub struct GraphInputs {
    pub slots: RwSignal<Vec<Option<String>>>,
    pub roles: RwSignal<Vec<Option<String>>>,
    pub guessed: RwSignal<Vec<bool>>,
    pub active: RwSignal<Option<usize>>,
    pub side: RwSignal<String>,
    pub tags: RwSignal<Vec<String>>,
    /// The linked opponent dossier's id, when there is one.
    pub opponent: Signal<Option<String>>,
    /// Champions unavailable besides the board (fearless).
    pub unavailable: Signal<Vec<String>>,
}

/// A pick made from the graph.
#[derive(Clone, Debug)]
pub struct GraphPick {
    pub slot: usize,
    /// Display name.
    pub name: String,
    pub role: Option<Role>,
    pub role_guessed: bool,
}

/// DOM id of a candidate row.
pub fn candidate_dom_id(i: usize) -> String {
    format!("graph-cand-{i}")
}

/// Ranked candidates for the current pick, the Why panel, the comp profile
/// and the team's notes on pairs.
///
/// A11y: the candidates are a `role="listbox"` named after the slot, one
/// Tab stop (roving tabindex): Up/Down move the focus (Home/End to the
/// ends), the Why panel follows the focused candidate, Enter or a double
/// click picks it into the current slot, and so does the Why panel's
/// "Pick" button; Escape returns to the board. Each option reads its rank, name, fit, role (and "role
/// guessed"), pool standing and strongest reasons; line glyphs repeat the
/// relation in a legend and are hidden from screen readers. Results arrive
/// in a polite live region ("12 candidates for Blue pick 3").
#[component]
pub fn DraftGraphPanel(
    inputs: GraphInputs,
    champions: StoredValue<Vec<Champion>>,
    on_pick: Callback<GraphPick>,
    /// Escape in the list: back to the board.
    on_escape: Callback<()>,
) -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastProvider");
    let result: RwSignal<Option<Result<GraphResult, String>>> = RwSignal::new(None);
    let focused: RwSignal<Option<String>> = RwSignal::new(None);
    let role_choice = RwSignal::new("open".to_string());
    let show_all = RwSignal::new(false);
    // Where each champion sat on the canvas last time (warm start).
    let memory: StoredValue<HashMap<String, (f32, f32)>> = StoredValue::new(HashMap::new());
    let refresh = RwSignal::new(0u32);
    let generation = StoredValue::new(0u32);
    #[allow(unused_variables)]
    let timer: RwSignal<Option<i32>> = RwSignal::new(None);

    // Ask the server again 250 ms after the draft stops changing; answers
    // to older questions are dropped.
    Effect::new(move |_| {
        let _ = refresh.get();
        let state = DraftState {
            slots: inputs.slots.get(),
            roles: inputs.roles.get(),
            roles_guessed: inputs.guessed.get(),
            current: inputs.active.get(),
            our_side: inputs.side.get(),
            tags: inputs.tags.get(),
            opponent_id: inputs.opponent.get(),
            unavailable: inputs.unavailable.get(),
            role: Role::parse(&role_choice.get()),
        };
        let json = serde_json::to_string(&state).unwrap_or_default();
        let gen = generation.get_value() + 1;
        generation.set_value(gen);
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::prelude::*;
            if let Some(id) = timer.get_untracked() {
                if let Some(win) = web_sys::window() {
                    win.clear_timeout_with_handle(id);
                }
            }
            let cb = Closure::once(move || {
                leptos::task::spawn_local(async move {
                    let r = graph_candidates(json).await.map_err(|e| e.to_string());
                    if generation.get_value() == gen {
                        result.set(Some(r));
                    }
                });
            });
            if let Some(win) = web_sys::window() {
                if let Ok(id) = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    250,
                ) {
                    timer.set(Some(id));
                }
            }
            cb.forget();
        }
        #[cfg(not(feature = "hydrate"))]
        let _ = json;
    });

    let candidates = Memo::new(move |_| {
        result.with(|r| match r {
            Some(Ok(g)) => g.candidates.clone(),
            _ => Vec::new(),
        })
    });
    // Keep the focus on a candidate that is still listed.
    Effect::new(move |_| {
        let list = candidates.get();
        let keep = focused
            .get_untracked()
            .filter(|f| list.iter().any(|c| &c.champion == f));
        if keep.is_none() {
            focused.set(list.first().map(|c| c.champion.clone()));
        }
    });
    // A candidate focused on the canvas below the top 10: show the whole list
    // so its row (the text twin) is there too.
    Effect::new(move |_| {
        let Some(f) = focused.get() else { return };
        let beyond = candidates.with(|l| {
            l.iter()
                .position(|c| c.champion == f)
                .is_some_and(|i| i >= 10)
        });
        if beyond && !show_all.get_untracked() {
            show_all.set(true);
        }
    });
    let focused_candidate = Memo::new(move |_| {
        let f = focused.get()?;
        candidates.with(|l| l.iter().find(|c| c.champion == f).cloned())
    });

    let pick = Callback::new(move |c: Candidate| {
        let Some(Ok(g)) = result.get_untracked() else {
            return;
        };
        let Some(target) = g.target else { return };
        on_pick.run(GraphPick {
            slot: target.slot,
            name: c.name.clone(),
            role: c.role,
            role_guessed: c.role_guessed,
        });
    });

    // Team notes on pairs: the dialog and the list.
    let relations = Resource::new(move || refresh.get(), |_| list_relations());
    let dialog_open = RwSignal::new(false);
    let rel_a = RwSignal::new(String::new());
    let rel_b = RwSignal::new(String::new());
    let rel_kind = RwSignal::new("synergy".to_string());
    let rel_weight = RwSignal::new("2".to_string());
    let rel_reason = RwSignal::new(String::new());
    let rel_error = RwSignal::new(None::<String>);
    let rel_busy = RwSignal::new(false);
    let open_dialog = Callback::new(move |prefill: Option<(String, String, &'static str)>| {
        let (a, b, k) = prefill.unwrap_or_default();
        rel_a.set(a);
        rel_b.set(b);
        rel_kind.set(if k.is_empty() { "synergy" } else { k }.to_string());
        rel_weight.set("2".into());
        rel_reason.set(String::new());
        rel_error.set(None);
        dialog_open.set(true);
    });
    let save_relation = Callback::new(move |_: leptos::ev::MouseEvent| {
        let rel = match relation_from_form(
            &rel_a.get_untracked(),
            &rel_b.get_untracked(),
            &rel_kind.get_untracked(),
            &rel_weight.get_untracked(),
            &rel_reason.get_untracked(),
        ) {
            Ok(r) => r,
            Err(e) => {
                rel_error.set(Some(e));
                return;
            }
        };
        rel_busy.set(true);
        let json = serde_json::to_string(&rel).unwrap_or_default();
        leptos::task::spawn_local(async move {
            match add_relation(json).await {
                Ok(_) => {
                    dialog_open.set(false);
                    toast
                        .push
                        .run(Toast::success("Team note saved; the ranking uses it now"));
                    refresh.update(|n| *n += 1);
                }
                Err(e) => rel_error.set(Some(e.to_string())),
            }
            rel_busy.set(false);
        });
    });
    let remove = Callback::new(move |r: TeamRelation| {
        let Some(id) = r.id.clone() else { return };
        leptos::task::spawn_local(async move {
            match remove_relation(id).await {
                Ok(()) => {
                    refresh.update(|n| *n += 1);
                    let back = NewRelation {
                        a: r.a.clone(),
                        b: r.b.clone(),
                        kind: r.kind,
                        weight: r.weight,
                        reason: r.reason.clone(),
                    };
                    let undo = Callback::new(move |_| {
                        let json = serde_json::to_string(&back).unwrap_or_default();
                        leptos::task::spawn_local(async move {
                            if add_relation(json).await.is_ok() {
                                refresh.update(|n| *n += 1);
                            }
                        });
                    });
                    toast
                        .push
                        .run(Toast::info("Team note removed").with_action("Undo", undo));
                }
                Err(e) => toast
                    .push
                    .run(Toast::error(format!("Could not remove the note: {e}"))),
            }
        });
    });

    let name_of = move |id: &str| -> String {
        champions.with_value(|l| {
            l.iter()
                .find(|c| c.id == id || c.name == id)
                .map(|c| c.name.clone())
                .unwrap_or_else(|| id.to_string())
        })
    };

    let on_list_key = move |ev: leptos::ev::KeyboardEvent| {
        let list = candidates.get_untracked();
        let shown = if show_all.get_untracked() {
            list.len()
        } else {
            list.len().min(10)
        };
        let cur = focused
            .get_untracked()
            .and_then(|f| list.iter().position(|c| c.champion == f));
        let key = ev.key();
        if let Some(next) = roving_index(&key, cur, shown, true) {
            ev.prevent_default();
            focused.set(Some(list[next].champion.clone()));
            focus_id(&candidate_dom_id(next));
            return;
        }
        match key.as_str() {
            "Enter" => {
                if let Some(c) = cur.and_then(|i| list.get(i).cloned()) {
                    ev.prevent_default();
                    pick.run(c);
                }
            }
            "Escape" => {
                ev.prevent_default();
                on_escape.run(());
            }
            _ => {}
        }
    };

    view! {
        <section aria-labelledby="draft-graph-heading" class="@container flex min-w-0 flex-col gap-4" data-draft-graph="">
            <div class="flex flex-wrap items-start justify-between gap-x-4 gap-y-2">
                <div class="flex min-w-0 flex-col gap-1">
                    <h2 id="draft-graph-heading" class="font-display text-heading font-semibold text-primary">"Draft Graph"</h2>
                    <p class="text-label text-secondary" aria-live="polite" data-graph-status="">
                        {move || match result.get() {
                            None => "Ranking candidates\u{2026}".to_string(),
                            Some(Err(_)) => "The ranking could not be loaded.".to_string(),
                            Some(Ok(g)) => match g.target {
                                None => "Nothing to rank on this step.".to_string(),
                                Some(t) => format!(
                                    "{} candidates for {} \u{b7} {}{}",
                                    g.candidates.len(),
                                    slot_text(t.slot),
                                    roles_text(&t.roles),
                                    if t.role_chosen { "" } else { " (open roles)" },
                                ),
                            },
                        }}
                    </p>
                </div>
                {move || result.with(|r| matches!(r, Some(Ok(g)) if g.notes.team)).then(|| view! {
                    <Button
                        variant=ButtonVariant::Quiet
                        size=ButtonSize::Sm
                        on_click=Callback::new(move |_| open_dialog.run(None))
                        attr:data-testid="graph-add-note"
                    >"Add what you know"</Button>
                })}
            </div>
            <div class="overflow-x-auto">
                <SegmentedControl
                    id="graph-role"
                    label="Role to rank for"
                    options=vec![
                        Segment::new("open", "Open roles"),
                        Segment::new("top", "Top"),
                        Segment::new("jungle", "Jungle"),
                        Segment::new("mid", "Mid"),
                        Segment::new("bot", "Bot"),
                        Segment::new("support", "Support"),
                    ]
                    value=role_choice
                    size=SegmentSize::Sm
                />
            </div>

            {move || match result.get() {
                None => view! {
                    <SkeletonGroup label="Ranking candidates">
                        <Skeleton shape=SkeletonShape::Line lines=6 width="w-full" />
                    </SkeletonGroup>
                }.into_any(),
                Some(Err(e)) => view! {
                    <Callout tone=CalloutTone::Danger title="The ranking could not be loaded">
                        {e}
                    </Callout>
                }.into_any(),
                Some(Ok(g)) if g.target.is_none() => view! {
                    <EmptyState
                        size=EmptySize::Compact
                        message=g.empty_reason.clone().unwrap_or_default()
                    />
                }.into_any(),
                Some(Ok(g)) if g.candidates.is_empty() => view! {
                    <EmptyState
                        size=EmptySize::Compact
                        message="No available champion plays this role in the trait table. Choose another role above."
                    />
                }.into_any(),
                Some(Ok(g)) => {
                    let target = g.target.clone().unwrap_or_else(unreachable_target);
                    let label = format!("Candidates for {}", slot_text(target.slot));
                    let side = DraftSide::parse(&target.side).unwrap_or(DraftSide::Blue);
                    let theirs = !target.ours;
                    let notes = g.notes.clone();
                    let own = g.own.clone();
                    let enemy = g.enemy.clone();
                    let total = g.candidates.len();
                    view! {
                        {theirs.then(|| view! {
                            <Callout tone=CalloutTone::Info>
                                "This is their pick: ranked for their side (synergy with their picks, counters to yours). Pools apply to your picks only."
                            </Callout>
                        })}
                        <DraftGraphCanvas
                            result=g.clone()
                            focused=focused
                            on_pick=pick
                            on_escape=on_escape
                            memory=memory
                            slot_label=slot_text(target.slot)
                        />
                        <div class="grid grid-cols-1 gap-x-8 gap-y-6 @2xl:grid-cols-[minmax(0,1fr)_minmax(0,19rem)]">
                            <div class="flex min-w-0 flex-col gap-3">
                                <p class="eyebrow">"Ranked by fit \u{b7} the graph as a list"</p>
                                <ol
                                    role="listbox"
                                    aria-label=label
                                    class="flex flex-col border-t border-rule"
                                    on:keydown=on_list_key
                                    data-graph-list=""
                                >
                                    {move || {
                                        let list = candidates.get();
                                        let n = if show_all.get() { list.len() } else { list.len().min(10) };
                                        list.into_iter().take(n).enumerate().map(|(i, c)| {
                                            let id = c.champion.clone();
                                            let id_sel = id.clone();
                                            let id_click = id.clone();
                                            let selected = Memo::new(move |_| focused.get().as_deref() == Some(id_sel.as_str()));
                                            let c_pick = c.clone();
                                            view! {
                                                <CandidateRow
                                                    rank=i + 1
                                                    candidate=c
                                                    selected=selected
                                                    on_focus=Callback::new(move |_| focused.set(Some(id_click.clone())))
                                                    on_pick=Callback::new(move |_| pick.run(c_pick.clone()))
                                                    name_of=Callback::new(move |s: String| name_of(&s))
                                                />
                                            }
                                        }).collect_view()
                                    }}
                                </ol>
                                {(total > 10).then(|| view! {
                                    <div>
                                        <Button
                                            variant=ButtonVariant::Quiet
                                            size=ButtonSize::Sm
                                            on_click=Callback::new(move |_| show_all.update(|v| *v = !*v))
                                        >
                                            {move || if show_all.get() { "Show the top 10".to_string() } else { format!("Show all {total}") }}
                                        </Button>
                                    </div>
                                })}
                                <p class="flex flex-wrap items-center gap-x-3 gap-y-1 text-caption text-muted">
                                    <span class="inline-flex items-center gap-1"><Keycap keys="\u{2191}" /><Keycap keys="\u{2193}" />" move"</span>
                                    <span class="inline-flex items-center gap-1"><Keycap keys="Enter" />" or double-click: pick"</span>
                                    <span class="inline-flex items-center gap-1"><Keycap keys="Esc" />" back to the board"</span>
                                </p>
                                <LegendBlock
                                    id="graph-reasons"
                                    title="Reason lines"
                                    entries=vec![
                                        LegendEntry::new(Swatch::Line { style: LineStyle::Solid, weight: LineWeight::Medium, ink: SwatchInk::Muted, arrow: false }, "Synergy, playstyle, timing"),
                                        LegendEntry::new(Swatch::Line { style: LineStyle::Dashed, weight: LineWeight::Medium, ink: SwatchInk::Muted, arrow: false }, "Counters"),
                                        LegendEntry::new(Swatch::Line { style: LineStyle::Dotted, weight: LineWeight::Medium, ink: SwatchInk::Muted, arrow: false }, "Works against the pick"),
                                    ]
                                />
                            </div>
                            <div class="flex min-w-0 flex-col gap-6">
                                {move || focused_candidate.get().map(|c| view! {
                                    <WhyPanel
                                        candidate=c
                                        team=notes.team
                                        on_pick=pick
                                        on_argue=open_dialog
                                        on_remove=remove
                                        relations=relations
                                        name_of=Callback::new(move |s: String| name_of(&s))
                                    />
                                })}
                                <CompProfilePanel
                                    side=side
                                    own=own.clone()
                                    enemy=enemy.clone()
                                    focused=focused_candidate
                                />
                            </div>
                        </div>
                        <GraphNotesLine notes=g.notes.clone() />
                    }.into_any()
                }
            }}

            <TeamNotesList relations=relations name_of=Callback::new(move |s: String| name_of(&s)) on_remove=remove />

            <Dialog
                id="graph-note"
                open=dialog_open
                title="Add what you know"
                description="A team note on a pair of champions. Every member sees it, and the ranking uses it with the strength you give it."
                size=DialogSize::Md
                footer=move || view! {
                    <Button variant=ButtonVariant::Quiet on_click=Callback::new(move |_| dialog_open.set(false))>"Cancel"</Button>
                    <Button
                        variant=ButtonVariant::Primary
                        on_click=save_relation
                        loading=Signal::derive(move || rel_busy.get())
                        attr:data-testid="graph-note-save"
                    >"Save note"</Button>
                }
            >
                <div class="flex flex-col gap-4">
                    <Field id="graph-note-a" label="Champion">
                        <ChampionField champions=champions.get_value() value=rel_a />
                    </Field>
                    <div class="flex flex-col gap-1.5">
                        <span class="text-label font-medium text-secondary" id="graph-note-kind-label">"Relation"</span>
                        <div class="overflow-x-auto">
                            <SegmentedControl
                                id="graph-note-kind"
                                label="Relation"
                                options=vec![
                                    Segment::new("synergy", "works with"),
                                    Segment::new("beats", "beats"),
                                    Segment::new("loses", "loses to"),
                                ]
                                value=rel_kind
                                size=SegmentSize::Sm
                            />
                        </div>
                    </div>
                    <Field id="graph-note-b" label="Other champion">
                        <ChampionField champions=champions.get_value() value=rel_b />
                    </Field>
                    <div class="flex flex-col gap-1.5">
                        <span class="text-label font-medium text-secondary">"How strong"</span>
                        <SegmentedControl
                            id="graph-note-weight"
                            label="How strong"
                            options=vec![
                                Segment::new("1", "Slight"),
                                Segment::new("2", "Clear"),
                                Segment::new("3", "Strong"),
                            ]
                            value=rel_weight
                            size=SegmentSize::Sm
                        />
                    </div>
                    <Field
                        id="graph-note-reason"
                        label="Because"
                        hint="One sentence a teammate can argue with. Up to 200 characters."
                        required=true
                        error=Signal::derive(move || rel_error.get())
                    >
                        <TextField value=rel_reason placeholder="Lantern saves Jinx after she dives in" maxlength=200 />
                    </Field>
                </div>
            </Dialog>
        </section>
    }
}

/// A target is always present on this branch; kept total for the compiler.
fn unreachable_target() -> crate::models::graph::GraphTarget {
    crate::models::graph::GraphTarget {
        slot: 0,
        side: "blue".into(),
        ours: true,
        roles: Vec::new(),
        role_chosen: false,
    }
}

/// The champion search inside the note dialog.
#[component]
fn ChampionField(champions: Vec<Champion>, value: RwSignal<String>) -> impl IntoView {
    view! {
        <crate::components::champion_autocomplete::ChampionAutocomplete
            champions=champions
            value=value
            placeholder="Search champions"
        />
    }
}

/// A short drawn line in a reason's style (decorative).
#[component]
fn ReasonLine(style: LineStyle) -> impl IntoView {
    let dash = match style {
        LineStyle::Solid => None,
        LineStyle::Dashed => Some("5 3"),
        LineStyle::Dotted => Some("1 4"),
    };
    view! {
        <svg viewBox="0 0 22 8" class="h-2 w-[22px] shrink-0 overflow-visible" aria-hidden="true">
            <line
                x1="1" y1="4" x2="21" y2="4"
                class="stroke-secondary"
                stroke-width="2"
                stroke-linecap="round"
                stroke-dasharray=dash
            />
        </svg>
    }
}

/// One ranked candidate.
#[component]
fn CandidateRow(
    rank: usize,
    candidate: Candidate,
    selected: Memo<bool>,
    on_focus: Callback<()>,
    on_pick: Callback<()>,
    name_of: Callback<String, String>,
) -> impl IntoView {
    let c = candidate;
    let dom_id = candidate_dom_id(rank - 1);
    let top: Vec<Reason> = c
        .reasons
        .iter()
        .filter(|r| r.component != Component::Pool)
        .take(2)
        .cloned()
        .collect();
    let pool = pool_text(&c);
    let pool_bad = c.pool.as_ref().is_some_and(|p| p.points < 0);
    let name = c.name.clone();
    let fit = c.fit;
    let role = c.role;
    let guessed = c.role_guessed;
    let champion = c.champion.clone();
    view! {
        <li
            id=dom_id
            role="option"
            aria-selected=move || if selected.get() { "true" } else { "false" }
            tabindex=move || if selected.get() { "0" } else { "-1" }
            data-candidate=champion
            data-fit=fit
            class=move || {
                if selected.get() {
                    "grid cursor-pointer grid-cols-[1.5rem_2rem_minmax(0,1fr)_auto] items-center gap-x-3 gap-y-1 border-b border-rule bg-ink-2 px-2 py-2.5 outline outline-1 -outline-offset-1 outline-accent"
                } else {
                    "grid cursor-pointer grid-cols-[1.5rem_2rem_minmax(0,1fr)_auto] items-center gap-x-3 gap-y-1 border-b border-rule px-2 py-2.5 hover:bg-ink-2"
                }
            }
            on:click=move |_| on_focus.run(())
            on:focus=move |_| on_focus.run(())
            on:dblclick=move |_| on_pick.run(())
        >
            <span class="figures text-caption text-muted">{rank}</span>
            <ChampionAvatar name=name.clone() size=AvatarSize::Md decorative=true />
            <span class="flex min-w-0 flex-col">
                <span class="flex min-w-0 flex-wrap items-baseline gap-x-2">
                    <span class="truncate text-body font-semibold text-primary">{name.clone()}</span>
                    {role.map(|r| view! { <RoleLabel role=r guessed=guessed /> })}
                </span>
                {pool.map(|p| view! {
                    <span class=if pool_bad { "text-caption text-warning" } else { "text-caption text-muted" }>{p}</span>
                })}
            </span>
            <span class="figures text-right text-heading font-semibold text-primary">
                <span class="sr-only">"fit "</span>{fit}
            </span>
            <ul class="col-span-3 col-start-2 flex flex-col gap-1">
                {top.into_iter().map(|r| {
                    let style = reason_line(&r);
                    let prefix = match (r.component, r.strength < 0.0) {
                        (_, true) => "Against: ",
                        (Component::Synergy, _) => "Synergy: ",
                        (Component::Counter, _) => "Counter: ",
                        (Component::Playstyle, _) => "Playstyle: ",
                        (Component::Timing, _) => "Timing: ",
                        (Component::Pool, _) => "Pool: ",
                    };
                    let text = match r.source {
                        ReasonSource::TeamNote => format!(
                            "{} + {}: team note by {}",
                            name.clone(),
                            name_of.run(r.with.clone().unwrap_or_default()),
                            r.author.clone().unwrap_or_default()
                        ),
                        _ => r.text.clone(),
                    };
                    view! {
                        <li class="grid grid-cols-[22px_minmax(0,1fr)] items-center gap-2 text-caption text-secondary">
                            <ReasonLine style=style />
                            <span><span class="sr-only">{prefix}</span>{text}</span>
                        </li>
                    }
                }).collect_view()}
            </ul>
        </li>
    }
}

/// "Why this pick": fit by part, then every reason grouped by source.
#[component]
fn WhyPanel(
    candidate: Candidate,
    team: bool,
    on_pick: Callback<Candidate>,
    on_argue: Callback<Option<(String, String, &'static str)>>,
    on_remove: Callback<TeamRelation>,
    relations: Resource<Result<Vec<TeamRelation>, ServerFnError>>,
    name_of: Callback<String, String>,
) -> impl IntoView {
    let c = candidate;
    let c_pick = c.clone();
    let prefill = argue_prefill(&c);
    let groups: Vec<(ReasonSource, Vec<Reason>)> = [
        ReasonSource::Rule,
        ReasonSource::TeamNote,
        ReasonSource::MatchData,
        ReasonSource::Scouting,
        ReasonSource::Pool,
    ]
    .into_iter()
    .map(|s| {
        (
            s,
            c.reasons
                .iter()
                .filter(|r| r.source == s)
                .cloned()
                .collect::<Vec<_>>(),
        )
    })
    .filter(|(_, v)| !v.is_empty())
    .collect();
    let name = c.name.clone();
    let parts = c.parts.clone();
    let trait_note = match (c.trait_default, c.trait_low_confidence) {
        (true, true) => Some("Its traits are a shipped default with low confidence: neutral values where the kit was unclear."),
        (true, false) => Some("Its traits are the shipped default, not yet reviewed by the maintainer."),
        (false, true) => Some("Its traits are marked low confidence."),
        _ => None,
    };
    view! {
        <section aria-labelledby="graph-why-heading" class="flex flex-col gap-4 border-t border-rule pt-4" data-graph-why="">
            <div class="flex items-center gap-3">
                <ChampionAvatar name=name.clone() size=AvatarSize::Lg decorative=true />
                <div class="flex min-w-0 flex-1 flex-col">
                    <h3 id="graph-why-heading" class="font-display text-heading font-semibold text-primary">
                        {format!("Why {name}")}
                    </h3>
                    <span class="text-caption text-muted">
                        <span class="figures text-label font-semibold text-primary">{format!("fit {}", c.fit)}</span>
                        " of 100, from a neutral 50"
                    </span>
                </div>
            </div>
            <div class="flex flex-wrap gap-2">
                <Button
                    size=ButtonSize::Sm
                    shortcut="Enter"
                    on_click=Callback::new(move |_| on_pick.run(c_pick.clone()))
                    attr:data-testid="graph-pick"
                >{format!("Pick {name}")}</Button>
                {team.then(|| {
                    let prefill = prefill.clone();
                    view! {
                        <Button
                            variant=ButtonVariant::Quiet
                            size=ButtonSize::Sm
                            on_click=Callback::new(move |_| on_argue.run(Some(prefill.clone())))
                            attr:data-testid="graph-argue"
                        >"Argue with it: add a team note"</Button>
                    }
                })}
            </div>
            <dl class="grid grid-cols-[minmax(0,1fr)_auto] gap-x-4 gap-y-1 text-label" data-graph-parts="">
                {parts.into_iter().map(|p| {
                    let label = if p.component == Component::Pool {
                        "Player pool".to_string()
                    } else {
                        format!("{} (weight {})", p.component.label(), p.component.weight())
                    };
                    view! {
                        <dt class="text-secondary">{label}</dt>
                        <dd class="figures text-right text-primary">
                            {signed(p.points)}
                            <span class="text-muted">{format!(" of \u{b1}{}", p.band)}</span>
                        </dd>
                    }
                }).collect_view()}
            </dl>
            {groups.into_iter().map(|(source, list)| {
                view! {
                    <div class="flex flex-col gap-2">
                        <h4 class="eyebrow">{source.label()}</h4>
                        <ul class="flex flex-col gap-2">
                            {list.into_iter().map(|r| reason_item(r, &name, name_of, on_remove, relations)).collect_view()}
                        </ul>
                    </div>
                }
            }).collect_view()}
            {trait_note.map(|t| view! { <p class="text-caption text-muted">{t}</p> })}
        </section>
    }
}

fn reason_item(
    r: Reason,
    candidate: &str,
    name_of: Callback<String, String>,
    on_remove: Callback<TeamRelation>,
    relations: Resource<Result<Vec<TeamRelation>, ServerFnError>>,
) -> AnyView {
    let style = reason_line(&r);
    let strength = strength_text(&r);
    match r.source {
        ReasonSource::TeamNote => {
            let pair = format!(
                "{candidate} and {}",
                name_of.run(r.with.clone().unwrap_or_default())
            );
            let note_id = r.note_id.clone();
            let author = r.author.clone().unwrap_or_default();
            view! {
                <li class="grid grid-cols-[22px_minmax(0,1fr)] items-start gap-2">
                    <span class="pt-1.5"><ReasonLine style=style /></span>
                    <span class="flex flex-col gap-0.5">
                        <span class="note text-reading text-primary">{format!("\u{201c}{}\u{201d}", r.text)}</span>
                        <span class="flex flex-wrap items-center gap-x-2 text-caption text-muted">
                            <span>{format!("team note by {author} on {pair} \u{b7} {strength}")}</span>
                            <Button
                                variant=ButtonVariant::Quiet
                                size=ButtonSize::Sm
                                aria_label=format!("Remove the team note on {pair}")
                                on_click=Callback::new(move |_| {
                                    let found = relations.get_untracked().and_then(|l| l.ok()).and_then(|l| {
                                        l.into_iter().find(|x| x.id == note_id)
                                    });
                                    if let Some(rel) = found {
                                        on_remove.run(rel);
                                    }
                                })
                            >"Remove"</Button>
                        </span>
                    </span>
                </li>
            }
            .into_any()
        }
        _ => {
            let meta = match (&r.rule_id, r.source) {
                (Some(id), _) => format!("{id} \u{b7} {strength}"),
                _ => strength,
            };
            view! {
                <li class="grid grid-cols-[22px_minmax(0,1fr)] items-start gap-2">
                    <span class="pt-1.5"><ReasonLine style=style /></span>
                    <span class="flex flex-col gap-0.5">
                        <span class="text-body text-primary">{r.text}</span>
                        <span class="figures text-caption text-muted">{meta}</span>
                    </span>
                </li>
            }
            .into_any()
        }
    }
}

/// Each side's comp profile, with the change the focused candidate makes
/// to its side.
#[component]
fn CompProfilePanel(
    side: DraftSide,
    own: CompProfile,
    enemy: CompProfile,
    focused: Memo<Option<Candidate>>,
) -> impl IntoView {
    let own_sv = StoredValue::new(own);
    let enemy_rows = profile_rows(&enemy, None);
    view! {
        <section aria-labelledby="graph-profile-heading" class="flex flex-col gap-3 border-t border-rule pt-4" data-graph-profile="">
            <h3 id="graph-profile-heading" class="font-display text-heading font-semibold text-primary">"Comp profile"</h3>
            <div class="grid grid-cols-2 gap-x-6 gap-y-4">
                <div class="flex min-w-0 flex-col gap-3">
                    <span class="text-label font-semibold"><SideMarker side=side text=SideText::Long /></span>
                    {move || {
                        let f = focused.get();
                        let against = f.as_ref().map(|c| format!("if {}", c.name)).unwrap_or_default();
                        own_sv.with_value(|own| {
                            profile_rows(own, f.as_ref().map(|c| &c.adds))
                        })
                        .into_iter()
                        .map(|row| readout(row, against.clone()))
                        .collect_view()
                    }}
                </div>
                <div class="flex min-w-0 flex-col gap-3">
                    <span class="text-label font-semibold"><SideMarker side=side.other() text=SideText::Long /></span>
                    {enemy_rows.into_iter().map(|row| readout(row, String::new())).collect_view()}
                </div>
            </div>
            <p class="text-caption text-muted">"Sums over the locked picks: each champion adds up to 2 for engage, front line and wave clear, and up to 3 for crowd control."</p>
        </section>
    }
}

fn readout(row: ProfileRow, against: String) -> AnyView {
    let delta = row
        .change
        .map(|(v, trend, sentiment)| Delta::new(v, trend, sentiment).against(against));
    match (row.unit, delta) {
        (Some(u), Some(d)) => view! { <StatReadout label=row.label value=row.value unit=u delta=d size=ReadoutSize::Sm /> }.into_any(),
        (Some(u), None) => view! { <StatReadout label=row.label value=row.value unit=u size=ReadoutSize::Sm /> }.into_any(),
        (None, Some(d)) => view! { <StatReadout label=row.label value=row.value delta=d size=ReadoutSize::Sm /> }.into_any(),
        (None, None) => view! { <StatReadout label=row.label value=row.value size=ReadoutSize::Sm /> }.into_any(),
    }
}

/// What the ranking stands on.
#[component]
fn GraphNotesLine(notes: crate::models::graph::GraphNotes) -> impl IntoView {
    let mut parts = vec![format!(
        "the trait rules ({} rows still shipped defaults, {} low confidence)",
        notes.default_rows, notes.low_confidence_rows
    )];
    if notes.team {
        parts.push(format!(
            "{} team note{}",
            notes.relations,
            if notes.relations == 1 { "" } else { "s" }
        ));
        parts.push(format!(
            "your stored games ({} so far; evidence shows from {} games)",
            notes.games, notes.min_sample
        ));
        if notes.scouting {
            parts.push("the opponent dossier".into());
        }
    }
    view! {
        <p class="text-caption text-muted" data-graph-notes="">
            {format!("Built from {}. Fit is explained, not a chance to win.", parts.join(", "))}
            {(!notes.team).then_some(" Join a team to add team notes and see your pools and games here.")}
        </p>
    }
}

/// The team's notes on pairs, newest first.
#[component]
fn TeamNotesList(
    relations: Resource<Result<Vec<TeamRelation>, ServerFnError>>,
    name_of: Callback<String, String>,
    on_remove: Callback<TeamRelation>,
) -> impl IntoView {
    view! {
        <Transition fallback=|| ()>
            {move || relations.get().map(|r| {
                let list = r.unwrap_or_default();
                let count = list.len();
                view! {
                    <section aria-labelledby="graph-notes-heading" class="flex flex-col gap-2 border-t border-rule pt-4" data-graph-team-notes="">
                        <h3 id="graph-notes-heading" class="font-display text-heading font-semibold text-primary">
                            {format!("Team notes on pairs ({count})")}
                        </h3>
                        {if list.is_empty() {
                            view! {
                                <EmptyState
                                    size=EmptySize::Compact
                                    message="No notes yet. Add what you know (X works with Y, X loses to Y, and why) and the ranking uses it, with your name on the reason."
                                />
                            }.into_any()
                        } else {
                            view! {
                                <ul class="flex flex-col">
                                    {list.into_iter().map(|rel| {
                                        let a = name_of.run(rel.a.clone());
                                        let b = name_of.run(rel.b.clone());
                                        let what = match rel.kind {
                                            RelationKind::Synergy => format!("{a} works with {b}"),
                                            RelationKind::Counter => format!("{a} beats {b}"),
                                        };
                                        let strength = match rel.weight { 1 => "slight", 2 => "clear", _ => "strong" };
                                        let date = rel.created_at.clone().unwrap_or_default().chars().take(10).collect::<String>();
                                        let label = format!("Remove the note \u{201c}{what}\u{201d}");
                                        let r2 = rel.clone();
                                        view! {
                                            <li class="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1 border-b border-rule py-2">
                                                <span class="flex min-w-0 flex-col gap-0.5">
                                                    <span class="text-body font-semibold text-primary">{what}</span>
                                                    <span class="note text-reading text-secondary">{format!("\u{201c}{}\u{201d}", rel.reason)}</span>
                                                    <span class="text-caption text-muted">{format!("{strength} \u{b7} {} \u{b7} {date}", rel.author_name)}</span>
                                                </span>
                                                <Button
                                                    variant=ButtonVariant::Quiet
                                                    size=ButtonSize::Sm
                                                    aria_label=label
                                                    on_click=Callback::new(move |_| on_remove.run(r2.clone()))
                                                >"Remove"</Button>
                                            </li>
                                        }
                                    }).collect_view()}
                                </ul>
                            }.into_any()
                        }}
                    </section>
                }
            })}
        </Transition>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::graph::{FitPart, PoolStatus, CANVAS_CANDIDATES};

    fn cand() -> Candidate {
        Candidate {
            champion: "Janna".into(),
            name: "Janna".into(),
            fit: 79,
            role: Some(Role::Support),
            role_guessed: true,
            parts: vec![FitPart {
                component: Component::Synergy,
                points: 17,
                band: 20,
            }],
            reasons: vec![
                Reason {
                    component: Component::Playstyle,
                    source: ReasonSource::Rule,
                    rule_id: Some("fit.tag_match".into()),
                    text: "Fits".into(),
                    with: None,
                    author: None,
                    note_id: None,
                    games: None,
                    wins: None,
                    strength: 1.0,
                },
                Reason {
                    component: Component::Counter,
                    source: ReasonSource::Rule,
                    rule_id: Some("ctr.engage_denied".into()),
                    text: "Syndra can undo Janna's engage".into(),
                    with: Some("Syndra".into()),
                    author: None,
                    note_id: None,
                    games: None,
                    wins: None,
                    strength: -2.0,
                },
            ],
            pool: Some(PoolStatus {
                player: Some("Pell".into()),
                tier: Some("match_ready".into()),
                points: 4,
            }),
            adds: CompProfile::default(),
            trait_default: true,
            trait_low_confidence: false,
        }
    }

    #[test]
    fn words_for_parts_slots_roles_and_pools() {
        assert_eq!(signed(12), "+12");
        assert_eq!(signed(-3), "\u{2212}3");
        assert_eq!(signed(0), "0");
        assert_eq!(
            part_text(Component::Synergy, 12, 20),
            "Synergy +12 of \u{b1}20"
        );
        assert_eq!(slot_text(10), "Blue pick 3");
        assert_eq!(
            roles_text(&[Role::Jungle, Role::Mid, Role::Support]),
            "Jungle, Mid or Support"
        );
        assert_eq!(roles_text(&[Role::Top]), "Top");
        assert_eq!(pool_text(&cand()).unwrap(), "Match ready \u{b7} Pell");
        let mut c = cand();
        c.pool.as_mut().unwrap().tier = None;
        assert_eq!(pool_text(&c).unwrap(), "Not in Pell's pool");
    }

    #[test]
    fn strengths_are_said_in_words() {
        let c = cand();
        assert_eq!(strength_text(&c.reasons[0]), "weight 1");
        assert_eq!(strength_text(&c.reasons[1]), "against, weight 2");
        let mut m = c.reasons[0].clone();
        m.source = ReasonSource::MatchData;
        m.games = Some(7);
        assert_eq!(strength_text(&m), "n=7, corroborates +1");
        m.strength = 0.0;
        assert_eq!(strength_text(&m), "n=7, evidence only");
        let mut p = c.reasons[0].clone();
        p.source = ReasonSource::Pool;
        p.strength = 4.0 / 3.0;
        assert_eq!(strength_text(&p), "+4 fit");
    }

    #[test]
    fn reason_lines_follow_the_canvas_language() {
        let c = cand();
        assert_eq!(reason_line(&c.reasons[0]), LineStyle::Solid);
        assert_eq!(reason_line(&c.reasons[1]), LineStyle::Dotted);
        let mut r = c.reasons[1].clone();
        r.strength = 2.0;
        assert_eq!(reason_line(&r), LineStyle::Dashed);
    }

    #[test]
    fn argue_prefills_the_strongest_pair() {
        let (a, b, kind) = argue_prefill(&cand());
        assert_eq!((a.as_str(), b.as_str(), kind), ("Janna", "Syndra", "loses"));
    }

    #[test]
    fn form_maps_loses_to_a_reversed_counter() {
        let r = relation_from_form("Janna", "Syndra", "loses", "3", "stunned first").unwrap();
        assert_eq!((r.a.as_str(), r.b.as_str()), ("Syndra", "Janna"));
        assert_eq!(r.kind, RelationKind::Counter);
        assert!(relation_from_form("A", "B", "maybe", "2", "why not").is_err());
        assert!(relation_from_form("A", "B", "synergy", "x", "why not").is_err());
    }

    #[test]
    fn profile_rows_show_what_a_pick_adds() {
        let own = CompProfile {
            picks: 2,
            physical: 2.0,
            magic: 0.0,
            cc: 2,
            engage: 0,
            frontline: 2,
            peel: 0,
            wave_clear: 2,
            early: 0,
            mid: 1,
            late: 1,
        };
        let adds = CompProfile {
            picks: 1,
            magic: 1.0,
            engage: 2,
            late: 1,
            ..CompProfile::default()
        };
        let rows = profile_rows(&own, Some(&adds));
        let engage = rows.iter().find(|r| r.label == "Engage").unwrap();
        assert_eq!(engage.value, "0");
        let (v, _, s) = engage.change.clone().unwrap();
        assert_eq!(v, "+2");
        assert_eq!(s, Sentiment::Good, "the comp had no engage");
        let mix = rows.iter().find(|r| r.label == "AD / AP").unwrap();
        assert_eq!(mix.value, "2 / 0");
        assert_eq!(mix.change.clone().unwrap().0, "2 / 1");
        let plan = rows.iter().find(|r| r.label == "Strongest").unwrap();
        assert_eq!(plan.value, "Even");
        assert_eq!(plan.change.clone().unwrap().0, "Late");
        let empty = profile_rows(&CompProfile::default(), None);
        assert!(empty.iter().all(|r| r.change.is_none()));
    }

    const _: () = assert!(CANVAS_CANDIDATES <= crate::models::graph::MAX_CANDIDATES);
}
