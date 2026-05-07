use crate::components::icon::Icon;
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
        <div class="flex flex-col gap-3">
            // Header strip — Champion roster label + role filter pills
            <div class="flex items-center justify-between flex-wrap gap-3">
                <span class="font-imperial uppercase tracking-[0.18em] text-xs text-secondary">
                    "Champion roster"
                </span>
                // Role filter pills (Demacia: bg-elevated container with thin pills)
                <div class="bg-elevated rounded-lg p-0.5 flex gap-0.5">
                    {roles.iter().map(|&role| {
                        let role_str = role.to_string();
                        let role_val = if role == "All" { None } else { Some(role.to_string()) };
                        let role_val_for_class = role_val.clone();
                        let role_static: Option<&'static str> = if role == "All" { None } else { Some(role) };
                        view! {
                            <button
                                title=role_str.clone()
                                aria-label=role_str.clone()
                                class=move || {
                                    let active = role_filter.get() == role_val_for_class.clone();
                                    if active {
                                        "px-3 py-1.5 flex items-center justify-center gap-1.5 bg-accent text-accent-contrast font-imperial uppercase tracking-[0.14em] text-[10px] rounded-md transition-colors cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                    } else {
                                        "px-3 py-1.5 flex items-center justify-center gap-1.5 text-secondary hover:text-primary hover:bg-overlay font-imperial uppercase tracking-[0.14em] text-[10px] rounded-md transition-colors cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                    }
                                }
                                on:click=move |_| set_role_filter.set(role_val.clone())
                            >
                                {if role == "All" {
                                    view! { <span>"All"</span> }.into_any()
                                } else {
                                    view! {
                                        <img
                                            src=role_icon_url(role)
                                            alt=role
                                            class=move || if role_filter.get().as_deref() == role_static {
                                                "w-4 h-4 brightness-0"
                                            } else {
                                                "w-4 h-4 invert opacity-75"
                                            }
                                        />
                                        <span>{role}</span>
                                    }.into_any()
                                }}
                            </button>
                        }
                    }).collect_view()}
                </div>
            </div>

            // Search bar with leading icon
            <div class="relative">
                <span
                    class="absolute left-3 top-1/2 -translate-y-1/2 text-muted pointer-events-none"
                    aria-hidden="true"
                >
                    <Icon name="search" size=16 />
                </span>
                <input
                    type="text"
                    placeholder="Search champion..."
                    aria-label="Search champion"
                    class="bg-surface/50 border border-outline/50 rounded-lg pl-10 pr-4 py-3 w-full text-primary text-sm placeholder-dimmed focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none focus:border-accent/40 transition-colors"
                    on:input=move |ev| set_query.set(event_target_value(&ev))
                />
            </div>

            // Champion grid — auto-fill 56px tiles with gilt outline on hover
            <div
                class="overflow-y-auto max-h-80 p-1"
                style="display:grid;grid-template-columns:repeat(auto-fill,minmax(56px,1fr));gap:8px;"
            >
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
                                    aria-label=champion.name.clone()
                                    title=champion.name.clone()
                                    class=if is_used {
                                        "relative overflow-hidden rounded-md border border-transparent w-full aspect-square grayscale opacity-40 cursor-not-allowed pointer-events-none"
                                    } else {
                                        "relative overflow-hidden rounded-md border border-transparent hover:border-accent/40 w-full aspect-square cursor-grab transition-all hover:shadow-[0_0_8px_var(--color-accent-soft)] focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
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
                                    // Hover overlay with champion name (Demacia: gold-accent over base/80)
                                    <div class="absolute inset-0 bg-overlay-strong opacity-0 hover:opacity-100 flex items-end justify-center pb-1 transition-opacity pointer-events-none">
                                        <span class="text-primary text-[10px] font-display italic text-center leading-tight px-0.5 truncate w-full">
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
