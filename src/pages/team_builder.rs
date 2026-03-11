use leptos::prelude::*;

#[component]
pub fn TeamBuilderPage() -> impl IntoView {
    // Auth redirect
    let auth_user = Resource::new(|| (), |_| crate::pages::profile::get_current_user());
    Effect::new(move || {
        if let Some(Ok(None)) = auth_user.get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/auth/login");
            }
        }
    });

    view! {
        <div class="max-w-4xl mx-auto py-8 px-6">
            <h1 class="text-3xl font-bold text-primary mb-4">"Team Builder"</h1>
            <p class="text-muted">"Composition builder coming in a future phase."</p>
        </div>
    }
}
