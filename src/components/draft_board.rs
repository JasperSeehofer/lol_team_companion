use crate::models::champion::Champion;
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
) -> impl IntoView {
    let (first_pick_blue, set_first_pick_blue) = signal(true);
    let champion_map = StoredValue::new(champion_map);

    let render_ban_slot = move |slot_idx: usize| {
        let (_, _, label) = slot_meta(slot_idx);
        view! {
            <div
                class=move || {
                    let slots = draft_slots.get();
                    let filled = slots.get(slot_idx).and_then(|s| s.as_ref()).is_some();
                    let is_active = active_slot.get() == Some(slot_idx);
                    let is_highlighted = highlighted_slot.get() == Some(slot_idx);
                    if is_active && !filled {
                        "w-10 h-10 rounded border-2 border-accent animate-pulse bg-surface flex items-center justify-center cursor-pointer"
                    } else if filled && is_highlighted {
                        "w-10 h-10 rounded border-2 border-red-400 bg-surface flex items-center justify-center relative overflow-hidden cursor-pointer"
                    } else if filled {
                        "w-10 h-10 rounded border border-outline bg-surface flex items-center justify-center relative overflow-hidden cursor-pointer hover:opacity-75"
                    } else {
                        "w-10 h-10 rounded border border-dashed border-outline bg-surface flex items-center justify-center cursor-pointer hover:border-gray-400"
                    }
                }
                on:click=move |_| on_slot_click.run(slot_idx)
                on:dragover=move |ev: web_sys::DragEvent| ev.prevent_default()
                on:drop={
                    let on_slot_drop = on_slot_drop;
                    move |ev: web_sys::DragEvent| {
                        ev.prevent_default();
                        if let Some(dt) = ev.data_transfer() {
                            if let Ok(name) = dt.get_data("text/plain") {
                                if !name.is_empty() {
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
                                <img src=icon_url alt=champ_name class="w-full h-full object-cover opacity-50 grayscale" />
                                {if is_highlighted {
                                    view! {
                                        <button
                                            class="absolute top-0 right-0 w-4 h-4 bg-red-600 text-white text-xs rounded-bl flex items-center justify-center hover:bg-red-500 z-10 cursor-pointer"
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
                        view! { <span class="text-overlay-strong text-xs leading-none text-center">{label}</span> }.into_any()
                    }
                }}
            </div>
        }
    };

    // is_pick1_slot: slot 6 or 7 (first pick of each side)
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
                        "h-16 rounded border-2 border-accent animate-pulse bg-surface overflow-hidden cursor-pointer"
                    } else if filled && is_highlighted && is_blue {
                        "h-16 rounded border-2 border-red-400 bg-blue-950 overflow-hidden cursor-pointer relative"
                    } else if filled && is_highlighted {
                        "h-16 rounded border-2 border-red-400 bg-red-950 overflow-hidden cursor-pointer relative"
                    } else if filled && is_blue {
                        "h-16 rounded border border-blue-600 bg-blue-950 overflow-hidden cursor-pointer hover:opacity-75"
                    } else if filled {
                        "h-16 rounded border border-red-600 bg-red-950 overflow-hidden cursor-pointer hover:opacity-75"
                    } else {
                        "h-16 rounded border border-dashed border-outline bg-surface overflow-hidden cursor-pointer hover:border-gray-400"
                    }
                }
                on:click=move |_| on_slot_click.run(slot_idx)
                on:dragover=move |ev: web_sys::DragEvent| ev.prevent_default()
                on:drop={
                    let on_slot_drop = on_slot_drop;
                    move |ev: web_sys::DragEvent| {
                        ev.prevent_default();
                        if let Some(dt) = ev.data_transfer() {
                            if let Ok(name) = dt.get_data("text/plain") {
                                if !name.is_empty() {
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
                        view! {
                            <div class="relative h-full w-full">
                                <div class={if is_blue { "flex h-full" } else { "flex flex-row-reverse h-full" }}>
                                    <div class="relative flex-shrink-0 h-full">
                                        <img src=icon_url alt=champ_name.clone() class="h-full aspect-square object-cover" />
                                        {is_first_pick.then(|| view! {
                                            <div class="absolute top-0 left-0 bg-accent text-accent-contrast text-xs font-bold px-1 leading-tight rounded-br">"1st"</div>
                                        })}
                                        {warning.map(|(player_name, class_detail)| {
                                            let tooltip = format!("Not in {}'s pool. {}", player_name, class_detail);
                                            view! {
                                                <div
                                                    class="absolute top-0 right-0 bg-amber-500 text-white text-xs font-bold px-1 leading-tight rounded-bl z-10 cursor-help"
                                                    title=tooltip
                                                >
                                                    <span class="text-[10px]">{"\u{26A0}"}</span>
                                                </div>
                                            }
                                        })}
                                    </div>
                                    <div class={if is_blue { "flex-1 flex flex-col justify-center px-2 min-w-0" } else { "flex-1 flex flex-col justify-center items-end px-2 min-w-0" }}>
                                        <span class="text-primary text-sm font-medium truncate">{champ_name}</span>
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
                                {if is_highlighted {
                                    view! {
                                        <button
                                            class="absolute top-0 right-0 w-4 h-4 bg-red-600 text-white text-xs rounded-bl flex items-center justify-center hover:bg-red-500 z-10 cursor-pointer"
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
                        view! {
                            <div class="flex items-center justify-between px-2 h-full w-full">
                                <span class="text-dimmed text-sm">{label}</span>
                                {is_first_pick.then(|| view! {
                                    <span class="text-accent text-xs font-bold">"1st"</span>
                                })}
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        }
    };

    view! {
        <div class="grid grid-cols-[13rem_8rem_13rem] gap-x-4 gap-y-2">

            // Row: Headers
            <h3 class="text-blue-400 font-bold text-center text-sm">"Blue Side"</h3>
            <div></div>
            <h3 class="text-red-400 font-bold text-center text-sm">"Red Side"</h3>

            // Row: Phase 1 bans + first-pick toggle
            <div class="flex gap-1">
                {render_ban_slot(0)}
                {render_ban_slot(2)}
                {render_ban_slot(4)}
            </div>
            <div class="flex items-center justify-center">
                <div class="flex flex-col items-center gap-1">
                    <span class="text-muted text-xs font-medium">"First pick"</span>
                    <button
                        class="relative w-11 h-6 rounded-full bg-elevated border border-outline hover:border-outline cursor-pointer transition-colors"
                        title="Toggle first pick side"
                        on:click=move |_| set_first_pick_blue.update(|v| *v = !*v)
                    >
                        <span class=move || if first_pick_blue.get() {
                            "absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-blue-400 transition-all duration-300"
                        } else {
                            "absolute top-0.5 right-0.5 w-5 h-5 rounded-full bg-red-400 transition-all duration-300"
                        }></span>
                    </button>
                </div>
            </div>
            <div class="flex gap-1 justify-end">
                {render_ban_slot(1)}
                {render_ban_slot(3)}
                {render_ban_slot(5)}
            </div>

            // Row: Pick 1
            {render_pick_slot(6, true)}
            <div class="flex items-center justify-center text-dimmed font-bold text-sm">"VS"</div>
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
            <div class="flex gap-1">
                {render_ban_slot(13)}
                {render_ban_slot(15)}
            </div>
            <div></div>
            <div class="flex gap-1 justify-end">
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
