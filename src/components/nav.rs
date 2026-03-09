use leptos::prelude::*;
use leptos_router::components::A;

use crate::pages::profile::{get_current_user, Logout};

#[component]
pub fn Nav() -> impl IntoView {
    let logout_action = ServerAction::<Logout>::new();
    let menu_open = RwSignal::new(false);

    // Refetch current user whenever logout action completes
    let logout_version = logout_action.version();
    let user = Resource::new(move || logout_version.get(), |_| get_current_user());

    // Close menu when logout finishes
    Effect::new(move || {
        if logout_version.get() > 0 {
            menu_open.set(false);
        }
    });

    view! {
        <nav class="bg-gray-900 border-b border-gray-700 px-6 py-3 flex items-center justify-between">
            <div class="flex items-center gap-6">
                <span class="text-yellow-400 font-bold text-lg tracking-wide">
                    "LoL Team Companion"
                </span>
                <div class="flex gap-4 text-sm">
                    <A href="/" attr:class="text-gray-300 hover:text-white transition-colors">
                        "Home"
                    </A>
                    <A href="/team/dashboard" attr:class="text-gray-300 hover:text-white transition-colors">
                        "Team"
                    </A>
                    <A href="/draft" attr:class="text-gray-300 hover:text-white transition-colors">
                        "Draft"
                    </A>
                    <A href="/stats" attr:class="text-gray-300 hover:text-white transition-colors">
                        "Stats"
                    </A>
                    <A href="/game-plan" attr:class="text-gray-300 hover:text-white transition-colors">
                        "Game Plan"
                    </A>
                    <A href="/post-game" attr:class="text-gray-300 hover:text-white transition-colors">
                        "Post Game"
                    </A>
                </div>
            </div>
            <div class="relative text-sm">
                <Suspense fallback=move || view! {
                    <span class="text-gray-500">"..."</span>
                }>
                    {move || Suspend::new(async move {
                        match user.await {
                            Ok(Some(u)) => view! {
                                <button
                                    on:click=move |_| menu_open.update(|v| *v = !*v)
                                    class="text-yellow-400 hover:text-yellow-300 font-semibold transition-colors cursor-pointer"
                                >
                                    {u.username}
                                </button>
                                <div
                                    class="absolute right-0 mt-2 w-40 bg-gray-800 border border-gray-700 rounded-lg shadow-lg py-1 z-50"
                                    style:display=move || if menu_open.get() { "block" } else { "none" }
                                >
                                    <A
                                        href="/profile"
                                        attr:class="block px-4 py-2 text-gray-300 hover:bg-gray-700 hover:text-white transition-colors"
                                    >
                                        "Profile Details"
                                    </A>
                                    <ActionForm action=logout_action>
                                        <button
                                            type="submit"
                                            class="block w-full text-left px-4 py-2 text-red-400 hover:bg-gray-700 hover:text-red-300 transition-colors cursor-pointer"
                                        >
                                            "Logout"
                                        </button>
                                    </ActionForm>
                                </div>
                            }.into_any(),
                            _ => view! {
                                <A href="/auth/login" attr:class="text-gray-300 hover:text-white transition-colors">
                                    "Login"
                                </A>
                                <span class="text-gray-600">"/"</span>
                                <A href="/auth/register" attr:class="text-gray-300 hover:text-white transition-colors">
                                    "Register"
                                </A>
                            }.into_any(),
                        }
                    })}
                </Suspense>
            </div>
        </nav>
    }
}
