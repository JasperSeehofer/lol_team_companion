use leptos::prelude::*;
use leptos::web_sys;
use leptos_router::components::A;

#[server]
pub async fn login_action(email: String, password: String) -> Result<(), ServerFnError> {
    use crate::server::auth::{AuthSession, Credentials};
    use leptos_axum::redirect;

    let mut auth: AuthSession = leptos_axum::extract().await?;
    let creds = Credentials { email, password };
    match auth.authenticate(creds).await {
        Ok(Some(user)) => {
            auth.login(&user)
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?;
            redirect("/team/dashboard");
            Ok(())
        }
        Ok(None) => Err(ServerFnError::new("Invalid email or password")),
        Err(e) => Err(ServerFnError::new(e.to_string())),
    }
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let login = ServerAction::<LoginAction>::new();

    // Hard navigate after successful login so the nav refetches auth state
    Effect::new(move || {
        if let Some(Ok(())) = login.value().get() {
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/team/dashboard");
            }
        }
    });

    let error = move || {
        login
            .value()
            .get()
            .and_then(|r| r.err())
            .map(|e| e.to_string())
    };

    view! {
        <div class="max-w-md mx-auto py-16 px-6">
            <h1 class="text-3xl font-bold text-primary mb-8">"Sign In"</h1>
            <ActionForm action=login>
                <div class="flex flex-col gap-4">
                    {move || error().map(|e| view! {
                        <div class="bg-red-900 border border-red-700 text-red-200 rounded px-4 py-3 text-sm">
                            {e}
                        </div>
                    })}
                    <div>
                        <label class="block text-secondary text-sm mb-1">"Email"</label>
                        <input
                            type="email"
                            name="email"
                            required
                            class="w-full bg-elevated border border-outline rounded px-3 py-2 text-primary focus:outline-none focus:border-accent"
                        />
                    </div>
                    <div>
                        <label class="block text-secondary text-sm mb-1">"Password"</label>
                        <input
                            type="password"
                            name="password"
                            required
                            class="w-full bg-elevated border border-outline rounded px-3 py-2 text-primary focus:outline-none focus:border-accent"
                        />
                    </div>
                    <button
                        type="submit"
                        class="bg-accent hover:bg-accent-hover text-accent-contrast font-bold rounded px-4 py-2 transition-colors"
                    >
                        "Sign In"
                    </button>
                    <p class="text-muted text-sm text-center">
                        "No account? "
                        <A href="/auth/register" attr:class="text-accent hover:text-accent-hover">
                            "Register"
                        </A>
                    </p>
                </div>
            </ActionForm>
        </div>
    }
}
