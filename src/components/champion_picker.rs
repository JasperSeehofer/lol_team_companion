use leptos::prelude::*;
use crate::models::champion::Champion;

#[component]
pub fn ChampionPicker(
    champions: Vec<Champion>,
    on_select: Callback<Champion>,
) -> impl IntoView {
    let (query, set_query) = signal(String::new());

    let filtered = move || {
        let q = query.get().to_lowercase();
        champions
            .iter()
            .filter(|c| c.name.to_lowercase().contains(&q))
            .cloned()
            .collect::<Vec<_>>()
    };

    view! {
        <div class="flex flex-col gap-2">
            <input
                type="text"
                placeholder="Search champion..."
                class="bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white placeholder-gray-400 focus:outline-none focus:border-yellow-400"
                on:input=move |ev| set_query.set(event_target_value(&ev))
            />
            <div class="grid grid-cols-5 gap-2 max-h-64 overflow-y-auto">
                <For
                    each=filtered
                    key=|c| c.id.clone()
                    children=move |champion| {
                        let champ = champion.clone();
                        view! {
                            <button
                                class="bg-gray-800 hover:bg-gray-700 border border-gray-600 rounded p-2 text-xs text-center text-gray-200 truncate transition-colors"
                                on:click=move |_| on_select.run(champ.clone())
                            >
                                {champion.name.clone()}
                            </button>
                        }
                    }
                />
            </div>
        </div>
    }
}
