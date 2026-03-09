use leptos::prelude::*;

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
        errors.get()
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
        <div class="min-h-screen bg-gray-950 flex items-center justify-center">
            <div class="max-w-lg text-center px-6">
                <h1 class="text-6xl font-bold text-yellow-400 mb-4">"Oops"</h1>
                <p class="text-gray-300 mb-6">"Something went wrong:"</p>
                <ul class="text-left bg-gray-800 border border-gray-700 rounded p-4">
                    {error_list}
                </ul>
                <a href="/" class="mt-6 inline-block text-yellow-400 hover:text-yellow-300 underline">
                    "Go Home"
                </a>
            </div>
        </div>
    }
}
