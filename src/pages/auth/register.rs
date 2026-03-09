use leptos::prelude::*;
use leptos_router::components::A;

#[server]
pub async fn register_action(
    username: String,
    email: String,
    password: String,
) -> Result<(), ServerFnError> {
    use crate::server::auth::hash_password;
    use crate::server::db;
    use leptos_axum::redirect;
    use std::sync::Arc;
    use surrealdb::{engine::local::Db, Surreal};

    let db = use_context::<Arc<Surreal<Db>>>()
        .ok_or_else(|| ServerFnError::new("No DB context"))?;

    let password_hash = hash_password(&password).map_err(|e| ServerFnError::new(e))?;

    db::create_user(&db, username, email, password_hash)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    redirect("/auth/login");
    Ok(())
}

#[component]
pub fn RegisterPage() -> impl IntoView {
    let register = ServerAction::<RegisterAction>::new();
    let error = move || {
        register.value().get().and_then(|r| r.err()).map(|e| e.to_string())
    };

    view! {
        <div class="max-w-md mx-auto py-16 px-6">
            <h1 class="text-3xl font-bold text-white mb-8">"Create Account"</h1>
            <ActionForm action=register>
                <div class="flex flex-col gap-4">
                    {move || error().map(|e| view! {
                        <div class="bg-red-900 border border-red-700 text-red-200 rounded px-4 py-3 text-sm">
                            {e}
                        </div>
                    })}
                    <div>
                        <label class="block text-gray-300 text-sm mb-1">"Username"</label>
                        <input
                            type="text"
                            name="username"
                            required
                            class="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white focus:outline-none focus:border-yellow-400"
                        />
                    </div>
                    <div>
                        <label class="block text-gray-300 text-sm mb-1">"Email"</label>
                        <input
                            type="email"
                            name="email"
                            required
                            class="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white focus:outline-none focus:border-yellow-400"
                        />
                    </div>
                    <div>
                        <label class="block text-gray-300 text-sm mb-1">"Password"</label>
                        <input
                            type="password"
                            name="password"
                            required
                            minlength="8"
                            class="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-white focus:outline-none focus:border-yellow-400"
                        />
                    </div>
                    <button
                        type="submit"
                        class="bg-yellow-400 hover:bg-yellow-300 text-gray-900 font-bold rounded px-4 py-2 transition-colors"
                    >
                        "Create Account"
                    </button>
                    <p class="text-gray-400 text-sm text-center">
                        "Already have an account? "
                        <A href="/auth/login" attr:class="text-yellow-400 hover:text-yellow-300">
                            "Sign In"
                        </A>
                    </p>
                </div>
            </ActionForm>
        </div>
    }
}
