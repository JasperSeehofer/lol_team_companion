use crate::models::champion::Champion;
use leptos::prelude::*;

fn role_tags(role: &str) -> &'static [&'static str] {
    match role {
        "Top" => &["Fighter", "Tank"],
        "Jungle" => &["Fighter", "Assassin", "Tank"],
        "Mid" => &["Mage", "Assassin"],
        "ADC" => &["Marksman"],
        "Support" => &["Support"],
        _ => &[],
    }
}

fn role_icon_url(role: &str) -> &'static str {
    match role {
        "Top"     => "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-shared-components/global/default/svg/position-top.svg",
        "Jungle"  => "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-shared-components/global/default/svg/position-jungle.svg",
        "Mid"     => "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-shared-components/global/default/svg/position-middle.svg",
        "ADC"     => "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-shared-components/global/default/svg/position-bottom.svg",
        "Support" => "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-shared-components/global/default/svg/position-utility.svg",
        _         => "",
    }
}

#[component]
pub fn ChampionPicker(
    champions: Vec<Champion>,
    used_champions: Vec<String>,
    on_select: Callback<Champion>,
) -> impl IntoView {
    let (query, set_query) = signal(String::new());
    let (role_filter, set_role_filter) = signal(Option::<String>::None);

    let filtered = {
        let champions = champions.clone();
        move || {
            let q = query.get().to_lowercase();
            let role = role_filter.get();
            champions
                .iter()
                .filter(|c| c.name.to_lowercase().contains(&q))
                .filter(|c| match &role {
                    None => true,
                    Some(r) => role_tags(r)
                        .iter()
                        .any(|tag| c.tags.contains(&tag.to_string())),
                })
                .cloned()
                .collect::<Vec<_>>()
        }
    };

    let roles = ["All", "Top", "Jungle", "Mid", "ADC", "Support"];

    view! {
        <div class="flex flex-col gap-2">
            // Role filter tabs
            <div class="flex gap-1 flex-wrap">
                {roles.iter().map(|&role| {
                    let role_str = role.to_string();
                    let role_val = if role == "All" { None } else { Some(role.to_string()) };
                    let role_val_for_class = role_val.clone();
                    let role_static: Option<&'static str> = if role == "All" { None } else { Some(role) };
                    view! {
                        <button
                            title=role_str.clone()
                            class=move || {
                                let active = role_filter.get() == role_val_for_class.clone();
                                if active {
                                    "w-8 h-8 flex items-center justify-center bg-accent text-accent-contrast font-bold rounded transition-colors"
                                } else {
                                    "w-8 h-8 flex items-center justify-center bg-overlay hover:bg-overlay-strong text-muted hover:text-gray-200 rounded transition-colors"
                                }
                            }
                            on:click=move |_| set_role_filter.set(role_val.clone())
                        >
                            {if role == "All" {
                                view! { <span class="text-xs">"All"</span> }.into_any()
                            } else {
                                view! {
                                    <img
                                        src=role_icon_url(role)
                                        alt=role
                                        class=move || if role_filter.get().as_deref() == role_static {
                                            "w-5 h-5 brightness-0"
                                        } else {
                                            "w-5 h-5 invert opacity-75"
                                        }
                                    />
                                }.into_any()
                            }}
                        </button>
                    }
                }).collect_view()}
            </div>

            // Search box
            <input
                type="text"
                placeholder="Search champion..."
                class="bg-elevated border border-outline rounded px-3 py-2 text-primary placeholder-gray-400 focus:outline-none focus:border-accent"
                on:input=move |ev| set_query.set(event_target_value(&ev))
            />

            // Champion grid
            <div class="max-h-52 overflow-y-auto" style="display:grid;grid-template-columns:repeat(auto-fill,minmax(2.75rem,1fr));gap:2px;">
                <For
                    each=filtered
                    key=|c| c.id.clone()
                    children={
                        let used_champions = used_champions.clone();
                        move |champion: Champion| {
                            let is_used = used_champions.contains(&champion.id);
                            let champ_for_click = champion.clone();
                            let champ_for_drag = champion.clone();
                            let icon_url = champion.image_full.clone();
                            view! {
                                <button
                                    class=if is_used {
                                        "relative overflow-hidden rounded border border-divider w-11 h-11 flex-shrink-0 grayscale opacity-40 cursor-not-allowed"
                                    } else {
                                        "relative overflow-hidden rounded border border-divider hover:border-accent w-11 h-11 flex-shrink-0 transition-colors cursor-grab"
                                    }
                                    disabled=is_used
                                    draggable="true"
                                    on:dragstart=move |ev: web_sys::DragEvent| {
                                        if !is_used {
                                            if let Some(dt) = ev.data_transfer() {
                                                let _ = dt.set_data("text/plain", &champ_for_drag.id);
                                            }
                                        }
                                    }
                                    on:click=move |_| {
                                        if !is_used {
                                            on_select.run(champ_for_click.clone());
                                        }
                                    }
                                >
                                    <img
                                        src=icon_url
                                        alt=champion.name.clone()
                                        class="w-full h-full object-cover"
                                    />
                                    <div class="absolute inset-0 bg-black/75 opacity-0 hover:opacity-100 flex items-end justify-center pb-1 transition-opacity pointer-events-none">
                                        <span class="text-primary text-xs font-medium text-center leading-tight px-0.5 truncate w-full">
                                            {champion.name.clone()}
                                        </span>
                                    </div>
                                </button>
                            }
                        }
                    }
                />
            </div>
        </div>
    }
}
