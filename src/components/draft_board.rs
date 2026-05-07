use crate::components::ornaments::FleurDeLis;
use crate::models::champion::Champion;
use crate::models::draft::role_icon_url;
use leptos::prelude::*;
use std::collections::HashMap;

/// Returns (side, kind, label) for each of the 20 draft slots in official LoL order.
pub fn slot_meta(idx: usize) -> (&'static str, &'static str, &'static str) {
    match idx {
        0 => ("blue", "ban", "Ban 1"),
        1 => ("red", "ban", "Ban 1"),
        2 => ("blue", "ban", "Ban 2"),
        3 => ("red", "ban", "Ban 2"),
        4 => ("blue", "ban", "Ban 3"),
        5 => ("red", "ban", "Ban 3"),
        6 => ("blue", "pick", "Pick 1"),
        7 => ("red", "pick", "Pick 1"),
        8 => ("red", "pick", "Pick 2"),
        9 => ("blue", "pick", "Pick 2"),
        10 => ("blue", "pick", "Pick 3"),
        11 => ("red", "pick", "Pick 3"),
        12 => ("red", "ban", "Ban 4"),
        13 => ("blue", "ban", "Ban 4"),
        14 => ("red", "ban", "Ban 5"),
        15 => ("blue", "ban", "Ban 5"),
        16 => ("red", "pick", "Pick 4"),
        17 => ("blue", "pick", "Pick 4"),
        18 => ("blue", "pick", "Pick 5"),
        19 => ("red", "pick", "Pick 5"),
        _ => ("blue", "ban", "?"),
    }
}

#[component]
pub fn DraftBoard(
    draft_slots: ReadSignal<Vec<Option<String>>>,
    champion_map: HashMap<String, Champion>,
    active_slot: ReadSignal<Option<usize>>,
    on_slot_click: Callback<usize>,
    on_slot_drop: Callback<(usize, String)>,
    highlighted_slot: ReadSignal<Option<usize>>,
    on_slot_clear: Callback<usize>,
    #[prop(optional)] slot_comments: Option<ReadSignal<Vec<Option<String>>>>,
    #[prop(optional)] warning_slots: Option<Signal<Vec<Option<(String, String)>>>>,
    #[prop(optional)] role_assignments: Option<ReadSignal<Vec<Option<String>>>>,
    #[prop(optional)] role_auto_guessed: Option<ReadSignal<Vec<bool>>>,
    #[prop(optional)] on_role_set: Option<Callback<(usize, String)>>,
) -> impl IntoView {
    let (first_pick_blue, set_first_pick_blue) = signal(true);
    let champion_map = StoredValue::new(champion_map);
    let role_popover_open: RwSignal<Option<usize>> = RwSignal::new(None);

    // ── Ban slot: 64×64 circular gilt tile (Demacia "Forsworn") ──────────────
    let render_ban_slot = move |slot_idx: usize| {
        view! {
            <div class="flex flex-col items-center gap-1">
                <div
                    class=move || {
                        let slots = draft_slots.get();
                        let filled = slots.get(slot_idx).and_then(|s| s.as_ref()).is_some();
                        let is_active = active_slot.get() == Some(slot_idx);
                        let is_highlighted = highlighted_slot.get() == Some(slot_idx);
                        if is_active && !filled {
                            // Empty + active phase: gilt outline + pulse halo
                            "relative w-16 h-16 rounded-full flex items-center justify-center cursor-pointer overflow-hidden focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-all bg-elevated border-2 animate-pulse ring-2 ring-accent/40"
                        } else if filled && is_highlighted {
                            "relative w-16 h-16 rounded-full flex items-center justify-center cursor-pointer overflow-hidden focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-all bg-surface ring-2 ring-danger"
                        } else if filled {
                            "relative w-16 h-16 rounded-full flex items-center justify-center cursor-pointer overflow-hidden focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-all bg-surface hover:opacity-80"
                        } else {
                            // Empty default ban slot — dashed outline
                            "relative w-16 h-16 rounded-full flex items-center justify-center cursor-pointer overflow-hidden focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-all bg-elevated border-2 border-dashed border-outline/50 hover:border-accent/40"
                        }
                    }
                    style=move || {
                        let slots = draft_slots.get();
                        let filled = slots.get(slot_idx).and_then(|s| s.as_ref()).is_some();
                        let is_highlighted = highlighted_slot.get() == Some(slot_idx);
                        if filled && is_highlighted {
                            "border: 2px solid var(--gold-3, var(--color-accent)); box-shadow: 0 0 0 1px var(--color-danger) inset;"
                                .to_string()
                        } else if filled {
                            "border: 2px solid var(--gold-3, var(--color-accent));".to_string()
                        } else {
                            "".to_string()
                        }
                    }
                    draggable="true"
                    on:dragstart=move |ev: web_sys::DragEvent| {
                        let slots = draft_slots.get_untracked();
                        if let Some(Some(champ_name)) = slots.get(slot_idx) {
                            if let Some(dt) = ev.data_transfer() {
                                let _ = dt.set_data("text/plain", champ_name);
                                let _ = dt.set_data("text/x-source-slot", &slot_idx.to_string());
                            }
                        }
                    }
                    on:click=move |_| on_slot_click.run(slot_idx)
                    on:dragover=move |ev: web_sys::DragEvent| ev.prevent_default()
                    on:drop={
                        let on_slot_drop = on_slot_drop;
                        move |ev: web_sys::DragEvent| {
                            ev.prevent_default();
                            if let Some(dt) = ev.data_transfer() {
                                // Read source slot first — if this is a slot-to-slot move,
                                // clear the source before filling the target so the "already used"
                                // guard in fill_slot does not block the move (BUG-05).
                                let source_slot: Option<usize> = dt
                                    .get_data("text/x-source-slot")
                                    .ok()
                                    .and_then(|s| s.parse::<usize>().ok());
                                if let Ok(name) = dt.get_data("text/plain") {
                                    if !name.is_empty() {
                                        if let Some(src) = source_slot {
                                            if src != slot_idx {
                                                on_slot_clear.run(src);
                                            }
                                        }
                                        on_slot_drop.run((slot_idx, name));
                                    }
                                }
                            }
                        }
                    }
                >
                    {move || {
                        let slots = draft_slots.get();
                        let is_highlighted = highlighted_slot.get() == Some(slot_idx);
                        if let Some(Some(champ_name)) = slots.get(slot_idx) {
                            let champ_name = champ_name.clone();
                            let icon_url = champion_map.with_value(|m| {
                                m.get(&champ_name).map(|c| c.image_full.clone()).unwrap_or_default()
                            });
                            let on_slot_clear = on_slot_clear;
                            view! {
                                <div class="relative w-full h-full">
                                    <img
                                        src=icon_url
                                        alt=champ_name
                                        class="w-full h-full object-cover grayscale brightness-50 rounded-full"
                                    />
                                    // Diagonal red ban-line overlay (the "Forsworn" mark)
                                    <div
                                        class="absolute inset-0 pointer-events-none flex items-center justify-center"
                                        aria-hidden="true"
                                    >
                                        <div
                                            class="absolute left-0 right-0 h-0.5 bg-danger rotate-45"
                                            style="top: 50%; transform-origin: center;"
                                        ></div>
                                    </div>
                                    // Wax-seal fleur ornament (Demacia)
                                    <div class="absolute -bottom-0.5 -right-0.5 opacity-70 pointer-events-none">
                                        <FleurDeLis size=12 />
                                    </div>
                                    {if is_highlighted {
                                        view! {
                                            <button
                                                class="absolute top-0 right-0 w-4 h-4 bg-danger text-white text-xs rounded-bl flex items-center justify-center hover:opacity-80 z-10 cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                on:click=move |ev| {
                                                    ev.stop_propagation();
                                                    on_slot_clear.run(slot_idx);
                                                }
                                            >"\u{00D7}"</button>
                                        }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                </div>
                            }.into_any()
                        } else {
                            // Empty ban slot: faint seal icon
                            view! {
                                <span
                                    class="text-dimmed opacity-60"
                                    style="font-size: 18px; line-height: 1;"
                                    aria-hidden="true"
                                >"\u{2698}"</span>
                            }.into_any()
                        }
                    }}
                </div>
                <span
                    class="text-[10px] text-dimmed uppercase tracking-[0.18em] font-imperial leading-none"
                >"Forsworn"</span>
            </div>
        }
    };

    // ── Pick slot: 64×64 square gilt tile + role/name row + on-deck halo ────
    let render_pick_slot = move |slot_idx: usize, is_blue: bool| {
        let (_, _, label) = slot_meta(slot_idx);
        let is_pick1_slot = slot_idx == 6 || slot_idx == 7;
        view! {
            <div
                class=move || {
                    let slots = draft_slots.get();
                    let filled = slots.get(slot_idx).and_then(|s| s.as_ref()).is_some();
                    let is_active = active_slot.get() == Some(slot_idx);
                    let is_highlighted = highlighted_slot.get() == Some(slot_idx);
                    if is_active && !filled {
                        // On-deck halo: gold ring + offset + glow
                        "relative h-16 rounded-md bg-elevated overflow-hidden cursor-pointer ring-2 ring-accent ring-offset-2 ring-offset-base shadow-[0_0_14px_var(--color-accent)] focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-all"
                    } else if filled && is_highlighted {
                        "relative h-16 rounded-md bg-surface border-2 border-danger overflow-hidden cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-all"
                    } else if filled {
                        "relative h-16 rounded-md bg-surface border border-outline/30 overflow-hidden cursor-pointer hover:border-accent/40 focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-all"
                    } else {
                        "relative h-16 rounded-md bg-elevated border border-dashed border-outline/50 overflow-hidden cursor-pointer hover:border-accent/40 focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none transition-all"
                    }
                }
                draggable="true"
                on:dragstart=move |ev: web_sys::DragEvent| {
                    let slots = draft_slots.get_untracked();
                    if let Some(Some(champ_name)) = slots.get(slot_idx) {
                        if let Some(dt) = ev.data_transfer() {
                            let _ = dt.set_data("text/plain", champ_name);
                            let _ = dt.set_data("text/x-source-slot", &slot_idx.to_string());
                        }
                    }
                }
                on:click=move |_| on_slot_click.run(slot_idx)
                on:dragover=move |ev: web_sys::DragEvent| ev.prevent_default()
                on:drop={
                    let on_slot_drop = on_slot_drop;
                    move |ev: web_sys::DragEvent| {
                        ev.prevent_default();
                        if let Some(dt) = ev.data_transfer() {
                            // Read source slot first — if this is a slot-to-slot move,
                            // clear the source before filling the target so the "already used"
                            // guard in fill_slot does not block the move (BUG-05).
                            let source_slot: Option<usize> = dt
                                .get_data("text/x-source-slot")
                                .ok()
                                .and_then(|s| s.parse::<usize>().ok());
                            if let Ok(name) = dt.get_data("text/plain") {
                                if !name.is_empty() {
                                    if let Some(src) = source_slot {
                                        if src != slot_idx {
                                            on_slot_clear.run(src);
                                        }
                                    }
                                    on_slot_drop.run((slot_idx, name));
                                }
                            }
                        }
                    }
                }
            >
                {move || {
                    let slots = draft_slots.get();
                    let is_highlighted = highlighted_slot.get() == Some(slot_idx);
                    let is_first_pick = is_pick1_slot && (
                        (slot_idx == 6 && first_pick_blue.get()) ||
                        (slot_idx == 7 && !first_pick_blue.get())
                    );
                    if let Some(Some(champ_name)) = slots.get(slot_idx) {
                        let champ_name = champ_name.clone();
                        let icon_url = champion_map.with_value(|m| {
                            m.get(&champ_name).map(|c| c.image_full.clone()).unwrap_or_default()
                        });
                        let on_slot_clear = on_slot_clear;
                        let warning = warning_slots.and_then(|ws| {
                            let warnings = ws.get();
                            warnings.get(slot_idx).cloned().flatten()
                        });
                        let role = role_assignments.and_then(|ra| {
                            let roles = ra.get();
                            roles.get(slot_idx).cloned().flatten()
                        });
                        let is_auto = role_auto_guessed.map(|ag| {
                            let guessed = ag.get();
                            guessed.get(slot_idx).copied().unwrap_or(true)
                        }).unwrap_or(true);
                        let role_for_badge = role.clone();
                        let role_for_popover = role.clone();
                        view! {
                            <div class="relative h-full w-full">
                                <div class={if is_blue { "flex h-full" } else { "flex flex-row-reverse h-full" }}>
                                    <div class="relative flex-shrink-0 h-full">
                                        <img src=icon_url alt=champ_name.clone() class="h-full aspect-square object-cover" />
                                        {is_first_pick.then(|| view! {
                                            <div class="absolute top-0 left-0 bg-accent text-accent-contrast text-[10px] font-bold px-1 leading-tight rounded-br font-imperial uppercase tracking-wider">"1st"</div>
                                        })}
                                        {warning.map(|(player_name, class_detail)| {
                                            let tooltip = format!("Not in {}'s pool. {}", player_name, class_detail);
                                            view! {
                                                <div
                                                    class="absolute top-0 right-0 bg-warning text-base text-xs font-bold px-1 leading-tight rounded-bl z-10 cursor-help"
                                                    title=tooltip
                                                >
                                                    <span class="text-[10px]">{"\u{26A0}"}</span>
                                                </div>
                                            }
                                        })}
                                    </div>
                                    <div class={if is_blue { "flex-1 flex flex-col justify-center px-2 min-w-0" } else { "flex-1 flex flex-col justify-center items-end px-2 min-w-0" }}>
                                        <span class="text-primary text-sm font-semibold font-display italic truncate">{champ_name}</span>
                                        {move || {
                                            slot_comments.and_then(|sc| {
                                                let comments = sc.get();
                                                comments.get(slot_idx).cloned().flatten().map(|c| {
                                                    let truncated = if c.len() > 20 {
                                                        format!("{}...", &c[..20])
                                                    } else {
                                                        c
                                                    };
                                                    view! { <span class="text-muted text-xs truncate">{truncated}</span> }
                                                })
                                            })
                                        }}
                                    </div>
                                </div>
                                // Role badge — bottom-right corner of the pick slot
                                {
                                    let badge_title = if let Some(ref r) = role_for_badge {
                                        if is_auto {
                                            "Auto-guessed from champion class \u{2014} click to change".to_string()
                                        } else {
                                            format!("{} \u{2014} click to change", r)
                                        }
                                    } else {
                                        "Assign role".to_string()
                                    };
                                    let badge_class = if role_for_badge.is_some() && !is_auto {
                                        "absolute bottom-0 right-0 w-5 h-5 bg-base/80 rounded-full flex items-center justify-center cursor-pointer z-10 border border-solid border-accent focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                    } else {
                                        "absolute bottom-0 right-0 w-5 h-5 bg-base/80 rounded-full flex items-center justify-center cursor-pointer z-10 opacity-50 border border-dashed border-outline focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                    };
                                    let icon_url_opt = role_for_badge.as_deref().map(role_icon_url).unwrap_or("").to_string();
                                    let role_label = role_for_badge.clone().unwrap_or_default();
                                    view! {
                                        <button
                                            class=badge_class
                                            title=badge_title.clone()
                                            aria-label=badge_title
                                            on:click=move |ev| {
                                                ev.stop_propagation();
                                                role_popover_open.update(|v| {
                                                    *v = if *v == Some(slot_idx) { None } else { Some(slot_idx) };
                                                });
                                            }
                                        >
                                            {if !icon_url_opt.is_empty() {
                                                view! { <img src=icon_url_opt class="w-4 h-4" alt=role_label /> }.into_any()
                                            } else {
                                                view! { <span class="text-[8px] text-dimmed">"?"</span> }.into_any()
                                            }}
                                        </button>
                                        // Popover (5 role buttons in a row)
                                        {move || {
                                            let is_open = role_popover_open.get() == Some(slot_idx);
                                            if is_open {
                                                let roles_list: &[(&str, &str)] = &[
                                                    ("top", "Top"),
                                                    ("jungle", "Jng"),
                                                    ("mid", "Mid"),
                                                    ("bot", "Bot"),
                                                    ("support", "Sup"),
                                                ];
                                                view! {
                                                    <div
                                                        class="absolute bottom-full mb-1 right-0 z-50 bg-surface border border-divider rounded-xl p-2 shadow-lg flex gap-1"
                                                        on:click=move |ev| ev.stop_propagation()
                                                    >
                                                        {roles_list.iter().map(|&(r, label)| {
                                                            let icon = role_icon_url(r);
                                                            let is_current = role_for_popover.as_deref() == Some(r);
                                                            let r_string = r.to_string();
                                                            view! {
                                                                <button
                                                                    class=if is_current {
                                                                        "flex flex-col items-center gap-1 p-1 rounded-lg bg-overlay text-primary cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                                    } else {
                                                                        "flex flex-col items-center gap-1 p-1 rounded-lg hover:bg-overlay cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                                                    }
                                                                    title=r
                                                                    on:click=move |ev| {
                                                                        ev.stop_propagation();
                                                                        if let Some(cb) = on_role_set {
                                                                            cb.run((slot_idx, r_string.clone()));
                                                                        }
                                                                        role_popover_open.set(None);
                                                                    }
                                                                >
                                                                    <img src=icon class="w-6 h-6" alt=r />
                                                                    <span class="text-[10px] text-muted capitalize">{label}</span>
                                                                </button>
                                                            }
                                                        }).collect_view()}
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! { <span></span> }.into_any()
                                            }
                                        }}
                                    }
                                }
                                {if is_highlighted {
                                    view! {
                                        <button
                                            class="absolute top-0 right-0 w-4 h-4 bg-danger text-white text-xs rounded-bl flex items-center justify-center hover:opacity-80 z-10 cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                            on:click=move |ev| {
                                                ev.stop_propagation();
                                                on_slot_clear.run(slot_idx);
                                            }
                                        >"\u{00D7}"</button>
                                    }.into_any()
                                } else {
                                    view! { <span></span> }.into_any()
                                }}
                            </div>
                        }.into_any()
                    } else {
                        // Empty pick slot — dashed outline + faint role icon at 50% opacity
                        let role_for_empty = role_assignments.and_then(|ra| {
                            let roles = ra.get();
                            roles.get(slot_idx).cloned().flatten()
                        });
                        let placeholder_role_icon = role_for_empty
                            .as_deref()
                            .map(role_icon_url)
                            .unwrap_or("");
                        view! {
                            <div class="flex items-center justify-between px-2 h-full w-full gap-2">
                                {if !placeholder_role_icon.is_empty() {
                                    view! {
                                        <img
                                            src=placeholder_role_icon
                                            alt=""
                                            class="w-5 h-5 opacity-50 flex-shrink-0"
                                        />
                                    }.into_any()
                                } else {
                                    view! { <span class="w-5 h-5 flex-shrink-0"></span> }.into_any()
                                }}
                                <span class="text-dimmed text-xs font-imperial uppercase tracking-[0.14em] flex-1 truncate">
                                    {label}
                                </span>
                                {is_first_pick.then(|| view! {
                                    <span class="text-accent text-[10px] font-bold font-imperial uppercase tracking-wider flex-shrink-0">"1st"</span>
                                })}
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        }
    };

    view! {
        <div class="grid grid-cols-[14rem_8rem_14rem] gap-x-4 gap-y-3">

            // Row: Headers — Demacia House styling
            <h3 class="font-imperial uppercase tracking-[0.18em] text-center text-xs text-info">"House Northwind"</h3>
            <div></div>
            <h3 class="font-imperial uppercase tracking-[0.18em] text-center text-xs text-danger">"House Frostbyte"</h3>

            // Row: Phase 1 bans + first-pick toggle
            <div class="flex gap-2 justify-end">
                {render_ban_slot(0)}
                {render_ban_slot(2)}
                {render_ban_slot(4)}
            </div>
            <div class="flex items-center justify-center">
                <div class="flex flex-col items-center gap-1">
                    <span class="text-dimmed text-[10px] font-imperial uppercase tracking-[0.18em]">"First pick"</span>
                    <button
                        class="relative w-11 h-6 rounded-full bg-elevated border border-outline/50 hover:border-accent/40 cursor-pointer transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                        title="Toggle first pick side"
                        on:click=move |_| set_first_pick_blue.update(|v| *v = !*v)
                    >
                        <span class=move || if first_pick_blue.get() {
                            "absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-info transition-all duration-300"
                        } else {
                            "absolute top-0.5 right-0.5 w-5 h-5 rounded-full bg-danger transition-all duration-300"
                        }></span>
                    </button>
                </div>
            </div>
            <div class="flex gap-2 justify-start">
                {render_ban_slot(1)}
                {render_ban_slot(3)}
                {render_ban_slot(5)}
            </div>

            // Row: Pick 1
            {render_pick_slot(6, true)}
            <div class="flex items-center justify-center font-imperial italic text-dimmed font-semibold text-base tracking-[0.18em]">"vs"</div>
            {render_pick_slot(7, false)}

            // Row: Pick 2
            {render_pick_slot(9, true)}
            <div></div>
            {render_pick_slot(8, false)}

            // Row: Pick 3
            {render_pick_slot(10, true)}
            <div></div>
            {render_pick_slot(11, false)}

            // Row: Phase 2 bans
            <div class="flex gap-2 justify-end">
                {render_ban_slot(13)}
                {render_ban_slot(15)}
            </div>
            <div></div>
            <div class="flex gap-2 justify-start">
                {render_ban_slot(12)}
                {render_ban_slot(14)}
            </div>

            // Row: Pick 4
            {render_pick_slot(17, true)}
            <div></div>
            {render_pick_slot(16, false)}

            // Row: Pick 5
            {render_pick_slot(18, true)}
            <div></div>
            {render_pick_slot(19, false)}

        </div>
    }
}
