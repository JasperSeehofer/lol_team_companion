use leptos::prelude::*;

const ACCENTS: &[(&str, &str, &str)] = &[
    ("yellow", "Yellow", "bg-yellow-400"),
    ("blue", "Blue", "bg-blue-400"),
    ("purple", "Purple", "bg-violet-400"),
    ("emerald", "Green", "bg-emerald-400"),
    ("rose", "Rose", "bg-rose-400"),
];

#[component]
pub fn ThemeToggle() -> impl IntoView {
    let (is_light, set_is_light) = signal(false);
    let (accent_open, set_accent_open) = signal(false);
    let (current_accent, set_current_accent) = signal(String::new());

    // Initialize from localStorage on hydration
    #[cfg(feature = "hydrate")]
    {
        Effect::new(move |_| {
            if let Some(win) = web_sys::window() {
                if let Ok(Some(storage)) = win.local_storage() {
                    if let Ok(Some(theme)) = storage.get_item("theme") {
                        if theme == "light" {
                            set_is_light.set(true);
                        }
                    }
                    if let Ok(Some(accent)) = storage.get_item("accent") {
                        set_current_accent.set(accent);
                    }
                }
            }
        });
    }

    let toggle_theme = move |_| {
        let new_light = !is_light.get_untracked();
        set_is_light.set(new_light);

        #[cfg(feature = "hydrate")]
        {
            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                if let Some(root) = doc.document_element() {
                    if new_light {
                        let _ = root.set_attribute("data-theme", "light");
                    } else {
                        let _ = root.remove_attribute("data-theme");
                    }
                }
            }
            if let Some(win) = web_sys::window() {
                if let Ok(Some(storage)) = win.local_storage() {
                    let _ = storage.set_item("theme", if new_light { "light" } else { "dark" });
                }
            }
        }
    };

    // Callback is Copy, safe to use in iterator closures
    let set_accent = Callback::new(move |accent: String| {
        set_current_accent.set(accent.clone());
        set_accent_open.set(false);

        #[cfg(feature = "hydrate")]
        {
            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                if let Some(root) = doc.document_element() {
                    if accent.is_empty() || accent == "yellow" {
                        let _ = root.remove_attribute("data-accent");
                    } else {
                        let _ = root.set_attribute("data-accent", &accent);
                    }
                }
            }
            if let Some(win) = web_sys::window() {
                if let Ok(Some(storage)) = win.local_storage() {
                    if accent.is_empty() || accent == "yellow" {
                        let _ = storage.remove_item("accent");
                    } else {
                        let _ = storage.set_item("accent", &accent);
                    }
                }
            }
        }
    });

    view! {
        <div class="flex items-center gap-1">
            // Theme toggle (moon/sun)
            <button
                class="p-2 rounded-lg text-muted hover:text-primary hover:bg-overlay transition-colors cursor-pointer"
                on:click=toggle_theme
                title=move || if is_light.get() { "Switch to dark mode" } else { "Switch to light mode" }
            >
                {move || if is_light.get() {
                    // Sun icon (currently light, click to go dark)
                    view! {
                        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                            <circle cx="12" cy="12" r="5" />
                            <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" />
                        </svg>
                    }.into_any()
                } else {
                    // Moon icon (currently dark, click to go light)
                    view! {
                        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                            <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
                        </svg>
                    }.into_any()
                }}
            </button>

            // Accent color picker
            <div class="relative">
                <button
                    class="p-2 rounded-lg text-muted hover:text-primary hover:bg-overlay transition-colors cursor-pointer"
                    on:click=move |_| set_accent_open.update(|v| *v = !*v)
                    title="Change accent color"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                        <path d="M12 2.69l5.66 5.66a8 8 0 1 1-11.31 0z" />
                    </svg>
                </button>

                {move || accent_open.get().then(|| {
                    let cur = current_accent.get();
                    view! {
                        <div class="absolute right-0 top-full mt-2 bg-surface border border-divider rounded-lg shadow-xl p-2 flex gap-1.5 z-[60]">
                            {ACCENTS.iter().map(|&(key, label, swatch_class)| {
                                let is_selected = cur == key || (cur.is_empty() && key == "yellow");
                                let key_str = key.to_string();
                                view! {
                                    <button
                                        class=format!(
                                            "w-7 h-7 rounded-full cursor-pointer transition-transform {} {}",
                                            swatch_class,
                                            if is_selected { "ring-2 ring-primary ring-offset-2 ring-offset-surface scale-110" } else { "hover:scale-110" }
                                        )
                                        title=label
                                        on:click=move |_| set_accent.run(key_str.clone())
                                    />
                                }
                            }).collect_view()}
                        </div>
                    }
                })}
            </div>
        </div>
    }
}
