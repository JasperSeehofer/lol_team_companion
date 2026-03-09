use leptos::prelude::*;
use crate::models::draft::DraftAction;

#[component]
pub fn DraftBoard(actions: Vec<DraftAction>) -> impl IntoView {
    let blue_bans: Vec<_> = actions
        .iter()
        .filter(|a| a.side == "blue" && a.phase.starts_with("ban"))
        .cloned()
        .collect();
    let red_bans: Vec<_> = actions
        .iter()
        .filter(|a| a.side == "red" && a.phase.starts_with("ban"))
        .cloned()
        .collect();
    let blue_picks: Vec<_> = actions
        .iter()
        .filter(|a| a.side == "blue" && a.phase.starts_with("pick"))
        .cloned()
        .collect();
    let red_picks: Vec<_> = actions
        .iter()
        .filter(|a| a.side == "red" && a.phase.starts_with("pick"))
        .cloned()
        .collect();

    view! {
        <div class="grid grid-cols-3 gap-4">
            // Blue side
            <div class="flex flex-col gap-2">
                <h3 class="text-blue-400 font-bold text-center">"Blue Side"</h3>
                <div class="flex gap-1 flex-wrap justify-center">
                    {blue_bans.into_iter().map(|a| view! {
                        <span class="bg-red-900 border border-red-700 rounded px-2 py-1 text-xs text-red-300 line-through">
                            {a.champion}
                        </span>
                    }).collect_view()}
                </div>
                <div class="flex flex-col gap-1">
                    {blue_picks.into_iter().map(|a| view! {
                        <div class="bg-blue-900 border border-blue-700 rounded px-3 py-2 text-sm text-white">
                            {a.champion}
                        </div>
                    }).collect_view()}
                </div>
            </div>

            // Center divider
            <div class="flex items-center justify-center text-gray-500 text-sm font-bold">
                "VS"
            </div>

            // Red side
            <div class="flex flex-col gap-2">
                <h3 class="text-red-400 font-bold text-center">"Red Side"</h3>
                <div class="flex gap-1 flex-wrap justify-center">
                    {red_bans.into_iter().map(|a| view! {
                        <span class="bg-red-900 border border-red-700 rounded px-2 py-1 text-xs text-red-300 line-through">
                            {a.champion}
                        </span>
                    }).collect_view()}
                </div>
                <div class="flex flex-col gap-1">
                    {red_picks.into_iter().map(|a| view! {
                        <div class="bg-red-900 border border-red-700 rounded px-3 py-2 text-sm text-white">
                            {a.champion}
                        </div>
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
}
