use leptos::prelude::*;

#[component]
pub fn StatCard(
    label: String,
    value: String,
    #[prop(optional)] sub: Option<String>,
) -> impl IntoView {
    view! {
        <div class="bg-elevated border border-divider rounded-lg p-4">
            <div class="text-muted text-xs uppercase tracking-wider mb-1">{label}</div>
            <div class="text-primary text-2xl font-bold">{value}</div>
            {sub.map(|s| view! {
                <div class="text-muted text-sm mt-1">{s}</div>
            })}
        </div>
    }
}
