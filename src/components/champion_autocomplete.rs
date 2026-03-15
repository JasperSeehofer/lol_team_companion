use crate::models::champion::Champion;
use leptos::prelude::*;

#[component]
pub fn ChampionAutocomplete(
    champions: Vec<Champion>,
    value: RwSignal<String>,
    #[prop(optional)] placeholder: &'static str,
    #[prop(optional)] on_select: Option<Callback<String>>,
) -> impl IntoView {
    let (open, set_open) = signal(false);
    let (filter_text, set_filter_text) = signal(String::new());
    let champions = StoredValue::new(champions);

    let filtered = move || {
        let text = filter_text.get().to_lowercase();
        if text.is_empty() {
            return Vec::new();
        }
        champions.with_value(|champs| {
            champs
                .iter()
                .filter(|c| c.name.to_lowercase().contains(&text))
                .take(8)
                .cloned()
                .collect::<Vec<_>>()
        })
    };

    let select_champion = move |champ: Champion| {
        value.set(champ.id.clone());           // store canonical Data Dragon ID
        set_filter_text.set(champ.name.clone()); // display human-readable name
        set_open.set(false);
        if let Some(cb) = on_select {
            cb.run(champ.id);   // callback receives canonical ID
        }
    };

    // Sync filter_text when value changes externally.
    // If value is a canonical ID, look up and display the human name instead.
    Effect::new(move |_| {
        let v = value.get();
        if v != filter_text.get_untracked() {
            let display = champions.with_value(|champs| {
                champs.iter().find(|c| c.id == v).map(|c| c.name.clone())
            });
            set_filter_text.set(display.unwrap_or(v));
        }
    });

    let placeholder = if placeholder.is_empty() {
        "Champion..."
    } else {
        placeholder
    };

    view! {
        <div class="relative">
            <input
                type="text"
                class="w-full bg-surface/50 border border-outline/50 rounded-lg px-3 py-2 text-primary text-sm placeholder-dimmed focus:outline-none focus:border-accent/50 transition-colors"
                placeholder=placeholder
                prop:value=move || filter_text.get()
                on:input=move |ev| {
                    let val = event_target_value(&ev);
                    set_filter_text.set(val.clone());
                    value.set(val);
                    set_open.set(true);
                }
                on:focus=move |_| {
                    if !filter_text.get_untracked().is_empty() {
                        set_open.set(true);
                    }
                }
                on:blur=move |_| {
                    // Delay close to allow mousedown on dropdown items to fire first.
                    // Dropdown items use on:mousedown + prevent_default to keep focus,
                    // but we still need a short delay as a fallback for normal blur.
                    #[cfg(feature = "hydrate")]
                    {
                        use wasm_bindgen::prelude::*;
                        let cb = Closure::once(move || set_open.set(false));
                        if let Some(win) = web_sys::window() {
                            let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                                cb.as_ref().unchecked_ref(), 150,
                            );
                        }
                        cb.forget();
                    }
                    #[cfg(not(feature = "hydrate"))]
                    set_open.set(false);
                }
            />
            {move || {
                let items = filtered();
                if !open.get() || items.is_empty() {
                    return view! { <div></div> }.into_any();
                }
                view! {
                    <div class="absolute z-50 mt-1 w-full bg-elevated border border-divider rounded-lg shadow-xl overflow-hidden max-h-56 overflow-y-auto">
                        {items.into_iter().map(|c| {
                            let name = c.name.clone();
                            let img = c.image_full.clone();
                            let c_for_click = c.clone();
                            view! {
                                <button
                                    class="w-full flex items-center gap-2 px-3 py-2 hover:bg-overlay transition-colors text-left cursor-pointer"
                                    on:mousedown=move |ev| {
                                        ev.prevent_default();
                                        select_champion(c_for_click.clone());
                                    }
                                >
                                    <img src=img alt=name.clone() class="w-6 h-6 rounded object-cover" />
                                    <span class="text-primary text-sm">{name}</span>
                                </button>
                            }
                        }).collect_view()}
                    </div>
                }.into_any()
            }}
        </div>
    }
}
