use leptos::prelude::*;

/// Fallback for unknown routes. Sets HTTP 404 during SSR and tells the user
/// plainly that the page does not exist.
#[component]
pub fn NotFound() -> impl IntoView {
    #[cfg(feature = "ssr")]
    {
        if let Some(resp) = use_context::<leptos_axum::ResponseOptions>() {
            resp.set_status(axum::http::StatusCode::NOT_FOUND);
        }
    }

    view! {
        <div class="min-h-[60vh] bg-base flex items-center justify-center">
            <div class="max-w-lg text-center px-6">
                <p class="text-muted text-sm uppercase tracking-widest mb-2">"404"</p>
                <h1 class="text-4xl font-bold text-primary mb-4">"Page not found"</h1>
                <p class="text-secondary mb-6">
                    "The page you are looking for does not exist or has moved. Check the address, or head back to the start."
                </p>
                <a
                    href="/"
                    class="inline-block bg-accent hover:bg-accent-hover text-accent-contrast font-semibold rounded-lg px-4 py-2 text-sm transition-colors focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                >
                    "Go to the start page"
                </a>
            </div>
        </div>
    }
}

#[component]
pub fn ErrorTemplate(
    #[prop(optional)] outside_errors: Option<Errors>,
    #[prop(optional)] errors: Option<RwSignal<Errors>>,
) -> impl IntoView {
    let errors = match outside_errors {
        Some(e) => RwSignal::new(e),
        None => match errors {
            Some(e) => e,
            None => panic!("No Errors given to ErrorTemplate"),
        },
    };

    let error_list = move || {
        errors
            .get()
            .into_iter()
            .map(|(_, e)| {
                let e_string = e.to_string();
                view! {
                    <li class="list-disc ml-4 text-red-400">{e_string}</li>
                }
            })
            .collect_view()
    };

    view! {
        <div class="min-h-screen bg-base flex items-center justify-center">
            <div class="max-w-lg text-center px-6">
                <h1 class="text-6xl font-bold text-accent mb-4">"Oops"</h1>
                <p class="text-secondary mb-6">"Something went wrong:"</p>
                <ul class="text-left bg-elevated border border-divider rounded p-4">
                    {error_list}
                </ul>
                <a href="/" class="mt-6 inline-block text-accent hover:text-accent-hover underline">
                    "Go Home"
                </a>
            </div>
        </div>
    }
}
