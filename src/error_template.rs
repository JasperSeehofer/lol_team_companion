use leptos::prelude::*;

use crate::components::ui::{Button, ButtonVariant, PageShell, PageWidth};

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
        <PageShell width=PageWidth::Reading class="pt-16">
            <p class="eyebrow">"Error 404"</p>
            <h1 class="mt-3 font-display text-headline font-semibold tracking-[-0.02em] text-primary">"Page not found"</h1>
            <p class="mt-4 max-w-prose text-reading text-secondary">
                "The page you are looking for does not exist or has moved. Check the address, or head back to the start."
            </p>
            <div class="mt-8">
                <Button href="/" variant=ButtonVariant::Primary>"Go to the start page"</Button>
            </div>
        </PageShell>
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
                    <li class="ml-4 list-disc text-danger">{e_string}</li>
                }
            })
            .collect_view()
    };

    view! {
        <PageShell width=PageWidth::Reading class="pt-16">
            <p class="eyebrow">"Error"</p>
            <h1 class="mt-3 font-display text-headline font-semibold text-primary">"Something went wrong"</h1>
            <ul class="mt-6 border-y border-rule py-4 text-body">
                {error_list}
            </ul>
            <div class="mt-8">
                <Button href="/" variant=ButtonVariant::Secondary>"Go to the start page"</Button>
            </div>
        </PageShell>
    }
}
