//! Champion/role chrome primitives: ChampPortrait, ChampTile, RoleIcon, Icon.
//! These are visual identity wrappers — region-neutral (same structure both regions).
//! Per CLAUDE.md "no raw hex in components" rule.

use leptos::prelude::*;

/// Strip spaces and special characters from a champion name to get the DDragon key.
/// "Aurelion Sol" → "AurelionSol", "Cho'Gath" → "ChoGath", "Kai'Sa" → "KaiSa".
fn champ_key(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}

/// Champion portrait image from Data Dragon CDN.
/// `kind`: "square" (default), "loading", or "splash".
/// `size`: pixel dimensions (default 64).
#[component]
pub fn ChampPortrait(
    name: String,
    #[prop(optional, default = 64)] size: u32,
    #[prop(optional, default = "square")] kind: &'static str,
) -> impl IntoView {
    let key = champ_key(&name);
    let src = match kind {
        "loading" => format!(
            "https://ddragon.leagueoflegends.com/cdn/img/champion/loading/{key}_0.jpg"
        ),
        "splash" => format!(
            "https://ddragon.leagueoflegends.com/cdn/img/champion/centered/{key}_0.jpg"
        ),
        _ => format!(
            "https://ddragon.leagueoflegends.com/cdn/14.24.1/img/champion/{key}.png"
        ),
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
    let container_style = format!("position: relative; width: {}px; height: {}px; flex-shrink: 0;", size, size);

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
                                        stroke="var(--color-danger, var(--color-accent))"
                                        stroke-width="3"
                                        stroke-linecap="round"
                                    />
                                </svg>
                            }.into_any()
                        } else {
                            view! {}.into_any()
                        }}
                        {if locked {
                            view! {
                                // Accent dot top-right
                                <div
                                    style="position: absolute; top: 3px; right: 3px; width: 7px; height: 7px; border-radius: 50%; background: var(--color-accent);"
                                ></div>
                            }.into_any()
                        } else {
                            view! {}.into_any()
                        }}
                    </div>
                }.into_any()
            } else {
                view! {
                    <div
                        class="bg-elevated border border-outline/30"
                        style=format!("width: {}px; height: {}px;", size, size)
                    ></div>
                }.into_any()
            }}
        </div>
    }
}

/// Role icon using CommunityDragon SVG via CSS mask-image.
/// `role`: "top" | "jungle" | "mid" | "adc" | "support"
/// Fills with `bg-accent` color via CSS mask. Default size = 24.
/// Touch target wrapper at 44px per UI-SPEC accessibility note (G-12).
#[component]
pub fn RoleIcon(
    role: String,
    #[prop(optional, default = 24)] size: u32,
) -> impl IntoView {
    let icon_url = format!(
        "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-static-assets/global/default/svg/position-{}.svg",
        role
    );
    let mask_style = format!(
        "-webkit-mask: url({}) center/contain no-repeat; \
         mask: url({}) center/contain no-repeat; \
         background-color: var(--color-accent); \
         width: {}px; height: {}px; display: inline-block;",
        icon_url, icon_url, size, size
    );
    // 44px minimum touch target wrapping the icon
    let wrapper_size = size.max(44);
    let wrapper_style = format!(
        "display: inline-flex; align-items: center; justify-content: center; \
         width: {}px; height: {}px; flex-shrink: 0;",
        wrapper_size, wrapper_size
    );
    view! {
        <span style=wrapper_style>
            <span style=mask_style aria-label=role></span>
        </span>
    }
}

/// Inline SVG icon dispatch by name.
/// Supported: "home", "user", "swords", "tree", "chart", "calendar", "sword", "shield"
/// Default size = 18. Stroke uses `currentColor` so callers can color via `text-*`.
/// Unknown names return an empty span (no crash).
#[component]
pub fn Icon(
    name: String,
    #[prop(optional, default = 18)] size: u32,
) -> impl IntoView {
    let vb = "0 0 24 24";
    let stroke_attrs = r#"fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round""#;

    let inner_svg = match name.as_str() {
        "home" => Some(format!(
            r#"<svg viewBox="{vb}" width="{size}" height="{size}" {stroke_attrs}><path d="M3 12L12 3l9 9"/><path d="M9 21V12h6v9"/><path d="M3 12v9h18v-9"/></svg>"#
        )),
        "user" => Some(format!(
            r#"<svg viewBox="{vb}" width="{size}" height="{size}" {stroke_attrs}><circle cx="12" cy="8" r="4"/><path d="M4 20c0-4 3.6-7 8-7s8 3 8 7"/></svg>"#
        )),
        "swords" => Some(format!(
            r#"<svg viewBox="{vb}" width="{size}" height="{size}" {stroke_attrs}><path d="M14.5 17.5L3 6V3h3l11.5 11.5"/><path d="M13 19l6-6"/><path d="M2 2l20 20"/><path d="M11 5l6 6"/></svg>"#
        )),
        "tree" => Some(format!(
            r#"<svg viewBox="{vb}" width="{size}" height="{size}" {stroke_attrs}><path d="M12 22v-7"/><path d="M12 15 8 11H5l7-7 7 7h-3z"/></svg>"#
        )),
        "chart" => Some(format!(
            r#"<svg viewBox="{vb}" width="{size}" height="{size}" {stroke_attrs}><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>"#
        )),
        "calendar" => Some(format!(
            r#"<svg viewBox="{vb}" width="{size}" height="{size}" {stroke_attrs}><rect x="3" y="4" width="18" height="18" rx="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/></svg>"#
        )),
        "sword" => Some(format!(
            r#"<svg viewBox="{vb}" width="{size}" height="{size}" {stroke_attrs}><path d="M20 4L4 20"/><path d="M4 4l16 16"/><path d="M12 4v4M4 12h4"/></svg>"#
        )),
        "shield" => Some(format!(
            r#"<svg viewBox="{vb}" width="{size}" height="{size}" {stroke_attrs}><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>"#
        )),
        _ => None,
    };

    if let Some(svg_html) = inner_svg {
        view! {
            <span inner_html=svg_html style="display: inline-flex; align-items: center;"></span>
        }.into_any()
    } else {
        view! { <span></span> }.into_any()
    }
}
