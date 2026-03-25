use leptos::prelude::*;
use leptos::web_sys;
use leptos_router::components::A;

#[server]
pub async fn register_action(
    username: String,
    email: String,
    password: String,
) -> Result<String, ServerFnError> {
    use crate::server::auth::{hash_password, AuthSession, Credentials};
    use crate::server::db;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let db =
        use_context::<Arc<Surreal<Db>>>().ok_or_else(|| ServerFnError::new("No DB context"))?;

    let password_hash = hash_password(&password).map_err(ServerFnError::new)?;

    db::create_user(&db, username, email.clone(), password_hash)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Auto-login after registration
    let mut auth: AuthSession = leptos_axum::extract().await?;
    let creds = Credentials { email, password };
    if let Ok(Some(user)) = auth.authenticate(creds).await {
        let _ = auth.login(&user).await;
    }

    // New users default to solo mode (D-03)
    Ok("/solo".to_string())
}

#[component]
pub fn RegisterPage() -> impl IntoView {
    let register = ServerAction::<RegisterAction>::new();

    // Hard navigate after successful registration + auto-login
    Effect::new(move || {
        #[allow(unused_variables)]
        if let Some(Ok(dest)) = register.value().get() {
            #[cfg(feature = "hydrate")]
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href(&dest);
            }
        }
    });

    let error = move || {
        register
            .value()
            .get()
            .and_then(|r| r.err())
            .map(|e| e.to_string())
    };

    view! {
        <div class="max-w-md mx-auto py-16 px-6">
            <h1 class="text-3xl font-bold text-primary mb-8">"Create Account"</h1>
            <ActionForm action=register>
                <div class="flex flex-col gap-4">
                    {move || error().map(|e| view! {
                        <div class="bg-red-900 border border-red-700 text-red-200 rounded px-4 py-3 text-sm">
                            {e}
                        </div>
                    })}
                    <div>
                        <label class="block text-secondary text-sm mb-1">"Username"</label>
                        <input
                            type="text"
                            name="username"
                            required
                            class="w-full bg-elevated border border-outline rounded px-3 py-2 text-primary focus:outline-none focus:border-accent"
                        />
                    </div>
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
                            minlength="8"
                            class="w-full bg-elevated border border-outline rounded px-3 py-2 text-primary focus:outline-none focus:border-accent"
                        />
                    </div>
                    <button
                        type="submit"
                        class="bg-accent hover:bg-accent-hover text-accent-contrast font-bold rounded px-4 py-2 transition-colors"
                    >
                        "Create Account"
                    </button>
                    <p class="text-muted text-sm text-center">
                        "Already have an account? "
                        <A href="/auth/login" attr:class="text-accent hover:text-accent-hover">
                            "Sign In"
                        </A>
                    </p>
                </div>
            </ActionForm>
        </div>
    }
}
