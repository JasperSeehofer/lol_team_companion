use leptos::prelude::*;

#[component]
pub fn StatCard(label: String, value: String, #[prop(optional)] sub: Option<String>) -> impl IntoView {
    view! {
        <div class="bg-gray-800 border border-gray-700 rounded-lg p-4">
            <div class="text-gray-400 text-xs uppercase tracking-wider mb-1">{label}</div>
            <div class="text-white text-2xl font-bold">{value}</div>
            {sub.map(|s| view! {
                <div class="text-gray-400 text-sm mt-1">{s}</div>
            })}
        </div>
    }
}
