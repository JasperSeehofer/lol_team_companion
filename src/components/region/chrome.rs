//! Champion portraits: ChampPortrait, ChampTile. Images come from the app's
//! own `/img/champion/...` endpoint (cached server-side), never a CDN.
//! Batch 3's `ChampionAvatar` replaces both.

use leptos::prelude::*;

/// Strip spaces and special characters from a champion name to get the DDragon key.
/// "Aurelion Sol" → "AurelionSol", "Cho'Gath" → "ChoGath", "Kai'Sa" → "KaiSa".
fn champ_key(name: &str) -> String {
    name.chars().filter(|c| c.is_alphanumeric()).collect()
}

/// Champion portrait image served by the app.
/// `kind`: "square" (default) or "loading" (the tall loading-screen art).
/// `size`: pixel dimensions (default 64).
#[component]
pub fn ChampPortrait(
    name: String,
    #[prop(optional, default = 64)] size: u32,
    #[prop(optional, default = "square")] kind: &'static str,
) -> impl IntoView {
    let key = champ_key(&name);
    let src = match kind {
        "loading" => crate::models::ddragon::champion_portrait_url(&key),
        _ => crate::models::ddragon::champion_icon_url(&key),
    };
    let alt = name.clone();
    view! {
        <img
            src=src
            alt=alt
            loading="lazy"
            style=format!(
                "display: block; width: {}px; height: {}px; object-fit: cover; object-position: center top;",
                size, size
            )
        />
    }
}

/// Champion tile — a 56×56 (default) portrait with ban/lock/dim state overlays.
/// If `name` is None, renders an empty elevated placeholder box.
/// If `banned`, applies grayscale filter and a diagonal bar overlay.
/// If `locked`, renders a small accent-colored dot in the top-right corner.
/// If `dimmed`, applies opacity-50.
#[component]
pub fn ChampTile(
    #[prop(optional)] name: Option<String>,
    #[prop(optional, default = 56)] size: u32,
    #[prop(optional, default = false)] banned: bool,
    #[prop(optional, default = false)] locked: bool,
    #[prop(optional, default = false)] dimmed: bool,
) -> impl IntoView {
    let opacity_class = if dimmed { "opacity-50" } else { "" };
    let filter_style = if banned { "filter: grayscale(1);" } else { "" };
    let container_style = format!(
        "position: relative; width: {}px; height: {}px; flex-shrink: 0;",
        size, size
    );

    view! {
        <div style=container_style class=opacity_class>
            {if let Some(champ_name) = name {
                view! {
                    <div style=format!("width: {}px; height: {}px; overflow: hidden; position: relative;", size, size)>
                        <div style=filter_style>
                            <ChampPortrait name=champ_name size=size kind="square" />
                        </div>
                        {if banned {
                            view! {
                                // Diagonal ban bar overlay
                                <svg
                                    viewBox="0 0 56 56"
                                    style=format!("position: absolute; inset: 0; width: {}px; height: {}px;", size, size)
                                >
                                    <line
                                        x1="4" y1="4" x2="52" y2="52"
                                        stroke="var(--color-danger)"
                                        stroke-width="3"
                                        stroke-linecap="round"
                                    />
                                </svg>
                            }.into_any()
                        } else {
                            ().into_any()
                        }}
                        {if locked {
                            view! {
                                // Locked marker, top-right
                                <div
                                    style="position: absolute; top: 3px; right: 3px; width: 7px; height: 7px; background: var(--color-accent);"
                                ></div>
                            }.into_any()
                        } else {
                            ().into_any()
                        }}
                    </div>
                }.into_any()
            } else {
                view! {
                    <div
                        class="bg-ink-2 border border-dashed border-rule"
                        style=format!("width: {}px; height: {}px;", size, size)
                    ></div>
                }.into_any()
            }}
        </div>
    }
}
