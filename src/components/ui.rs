use leptos::prelude::*;

/// A styled error banner for page-level and resource errors.
#[component]
pub fn ErrorBanner(message: String) -> impl IntoView {
    view! {
        <div class="bg-red-500/10 border border-red-500/30 rounded-xl p-4 flex items-start gap-3">
            <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5 text-red-400 shrink-0 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
            </svg>
            <p class="text-red-400 text-sm">{message}</p>
        </div>
    }
}

/// A styled status message that shows success (green) or error (red) based on content.
#[component]
pub fn StatusMessage(message: String) -> impl IntoView {
    let is_err = message.starts_with("Error");
    let cls = if is_err {
        "bg-red-500/10 border border-red-500/30 text-red-400 rounded-xl px-4 py-3 text-sm"
    } else {
        "bg-emerald-500/10 border border-emerald-500/30 text-emerald-400 rounded-xl px-4 py-3 text-sm"
    };
    view! {
        <div class=cls>{message}</div>
    }
}

/// A full-width empty state placeholder.
#[component]
pub fn EmptyState(message: &'static str) -> impl IntoView {
    view! {
        <div class="text-center py-12">
            <p class="text-dimmed text-sm">{message}</p>
        </div>
    }
}
