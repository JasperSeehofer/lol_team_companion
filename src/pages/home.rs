use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="max-w-4xl mx-auto py-16 px-6">
            <h1 class="text-4xl font-bold text-white mb-4">
                "LoL Team Companion"
            </h1>
            <p class="text-gray-300 text-lg mb-8">
                "Draft planning, player stats, and post-game learnings for amateur and semi-pro League of Legends teams."
            </p>
            <div class="grid grid-cols-2 gap-4 md:grid-cols-3">
                <A href="/team/dashboard">
                    <div class="bg-gray-800 hover:bg-gray-700 border border-gray-700 rounded-lg p-6 transition-colors cursor-pointer">
                        <div class="text-yellow-400 text-xl mb-2">"Team"</div>
                        <div class="text-gray-400 text-sm">"Manage your roster and team settings"</div>
                    </div>
                </A>
                <A href="/draft">
                    <div class="bg-gray-800 hover:bg-gray-700 border border-gray-700 rounded-lg p-6 transition-colors cursor-pointer">
                        <div class="text-yellow-400 text-xl mb-2">"Draft Planner"</div>
                        <div class="text-gray-400 text-sm">"Plan pick/ban phases against opponents"</div>
                    </div>
                </A>
                <A href="/stats">
                    <div class="bg-gray-800 hover:bg-gray-700 border border-gray-700 rounded-lg p-6 transition-colors cursor-pointer">
                        <div class="text-yellow-400 text-xl mb-2">"Stats"</div>
                        <div class="text-gray-400 text-sm">"Champion pools and match history"</div>
                    </div>
                </A>
                <A href="/game-plan">
                    <div class="bg-gray-800 hover:bg-gray-700 border border-gray-700 rounded-lg p-6 transition-colors cursor-pointer">
                        <div class="text-yellow-400 text-xl mb-2">"Game Plan"</div>
                        <div class="text-gray-400 text-sm">"Pre-game strategy and win conditions"</div>
                    </div>
                </A>
                <A href="/post-game">
                    <div class="bg-gray-800 hover:bg-gray-700 border border-gray-700 rounded-lg p-6 transition-colors cursor-pointer">
                        <div class="text-yellow-400 text-xl mb-2">"Post Game"</div>
                        <div class="text-gray-400 text-sm">"Record learnings and action items"</div>
                    </div>
                </A>
            </div>
        </div>
    }
}
