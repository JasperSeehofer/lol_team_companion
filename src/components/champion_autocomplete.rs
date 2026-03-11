use leptos::prelude::*;
use crate::models::champion::Champion;

#[component]
pub fn ChampionAutocomplete(
    champions: Vec<Champion>,
    value: RwSignal<String>,
    #[prop(optional)] placeholder: &'static str,
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

    let select_champion = move |name: String| {
        value.set(name.clone());
        set_filter_text.set(name);
        set_open.set(false);
    };

    // Sync filter_text when value changes externally
    Effect::new(move |_| {
        let v = value.get();
        if v != filter_text.get_untracked() {
            set_filter_text.set(v);
        }
    });

    let placeholder = if placeholder.is_empty() { "Champion..." } else { placeholder };

    view! {
        <div class="relative">
            <input
                type="text"
                class="w-full bg-gray-900/50 border border-gray-600/50 rounded-lg px-3 py-2 text-white text-sm placeholder-gray-500 focus:outline-none focus:border-yellow-400/50 transition-colors"
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
                    <div class="absolute z-50 mt-1 w-full bg-gray-800 border border-gray-700 rounded-lg shadow-xl overflow-hidden max-h-56 overflow-y-auto">
                        {items.into_iter().map(|c| {
                            let name = c.name.clone();
                            let name_for_click = name.clone();
                            let img = c.image_full.clone();
                            view! {
                                <button
                                    class="w-full flex items-center gap-2 px-3 py-2 hover:bg-gray-700 transition-colors text-left cursor-pointer"
                                    on:mousedown=move |ev| {
                                        ev.prevent_default();
                                        select_champion(name_for_click.clone());
                                    }
                                >
                                    <img src=img alt=name.clone() class="w-6 h-6 rounded object-cover" />
                                    <span class="text-white text-sm">{name}</span>
                                </button>
                            }
                        }).collect_view()}
                    </div>
                }.into_any()
            }}
        </div>
    }
}
