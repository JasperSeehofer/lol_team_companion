use leptos::prelude::*;

/// A statistics card following the Open-Design `Card.elevated` pattern.
///
/// - Surface: `bg-surface border border-outline rounded-xl p-4`
/// - Stat number: `text-2xl font-display text-primary` (Cormorant Garamond
///   for editorial gravitas)
/// - Label: `text-xs text-muted uppercase tracking-wider font-mono`
///   (overline label)
/// - Sub-label: muted secondary metadata
#[component]
pub fn StatCard(
    label: String,
    value: String,
    #[prop(optional)] sub: Option<String>,
) -> impl IntoView {
    view! {
        <div class="bg-surface border border-outline rounded-xl p-4">
            <div class="font-mono uppercase tracking-wider text-xs text-muted mb-1">{label}</div>
            <div class="font-display text-primary text-2xl tabular-nums">{value}</div>
            {sub.map(|s| view! {
                <div class="text-muted text-sm mt-1">{s}</div>
            })}
        </div>
    }
}
