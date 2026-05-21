//! Region-aware interactive primitives: Btn, Badge, ModeToggle.
//! Btn is region-branching (Demacia: gilt gradient + Cinzel; Pandemonium: flat accent + mono + offset shadow).
//! Badge is region-neutral (same shape both regions).
//! ModeToggle is region-branching: Demacia renders a segmented control with gilt borders;
//!   Pandemonium renders a stamped tab-pull with bracket corner styling.
//!
//! G-12 compliance: all interactive elements use focus-visible:ring-2 focus-visible:ring-accent/50.
//! No outline:none without ring replacement.
//! Per CLAUDE.md "no raw hex in components" rule.

use leptos::prelude::*;

/// Region-aware button primitive.
/// `variant`: "primary" | "secondary" | "outline" | "ghost" (default: "primary")
/// `size`: "sm" | "md" | "lg" (default: "md")
/// - Demacia primary: gilt gradient background + Cinzel uppercase font
/// - Pandemonium primary: flat accent bg + mono uppercase + offset box-shadow
/// - All variants: focus-visible:ring-2 ring-accent/50 for G-12 compliance
///
/// Uses `ChildrenFn` because both region arms call `children()`.
#[component]
pub fn Btn(
    region: String,
    #[prop(optional, into)] variant: Option<String>,
    #[prop(optional, into)] size: Option<String>,
    #[prop(optional)] on_click: Option<Callback<()>>,
    children: ChildrenFn,
) -> impl IntoView {
    let is_pandemonium = region == "pandemonium";
    let var = variant.unwrap_or_else(|| "primary".to_string());
    let sz = size.unwrap_or_else(|| "md".to_string());

    // Size classes (shared between regions)
    let size_classes = match sz.as_str() {
        "sm" => "px-3 py-1.5 text-[11px]",
        "lg" => "px-6 py-3.5 text-[15px]",
        _ => "px-4 py-2.5 text-[13px]", // md default
    };

    // Focus ring (G-12 — mandatory on all interactive elements)
    let focus_classes = "focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none";

    move || if is_pandemonium {
        let (bg_class, extra_style) = match var.as_str() {
            "primary" => (
                "bg-accent text-accent-contrast rounded-none",
                "box-shadow: 3px 3px 0 var(--accent-2, var(--color-accent)), 6px 6px 0 var(--accent-3, var(--color-accent));",
            ),
            "secondary" => ("bg-elevated text-secondary border border-outline/50 rounded-none", ""),
            "outline" => ("bg-transparent text-accent border border-accent rounded-none", ""),
            "ghost" => ("bg-transparent text-secondary hover:text-primary rounded-none", ""),
            _ => ("bg-accent text-accent-contrast rounded-none", ""),
        };
        let btn_class = format!(
            "inline-flex items-center gap-2 {} font-mono uppercase tracking-[0.12em] cursor-pointer transition-colors {} {}",
            size_classes, bg_class, focus_classes
        );
        view! {
            <button
                type="button"
                class=btn_class
                style=extra_style
                on:click=move |_| { if let Some(cb) = on_click { cb.run(()); } }
            >
                {children()}
            </button>
        }.into_any()
    } else {
        // Demacia
        let (btn_class, btn_style) = match var.as_str() {
            "primary" => (
                format!(
                    "inline-flex items-center gap-2 {} font-imperial uppercase tracking-[0.14em] rounded cursor-pointer transition-colors {}",
                    size_classes, focus_classes
                ),
                "background: linear-gradient(180deg, var(--gold-1, var(--color-accent)) 0%, var(--gold-2, var(--color-accent)) 50%, var(--gold-3, var(--color-accent)) 100%); color: var(--ink, var(--t-accent-contrast)); border: 1px solid var(--gold-deep, var(--border-outline));".to_string(),
            ),
            "secondary" => (
                format!(
                    "inline-flex items-center gap-2 {} font-imperial uppercase tracking-[0.14em] rounded cursor-pointer bg-elevated text-secondary border border-outline/50 transition-colors {}",
                    size_classes, focus_classes
                ),
                String::new(),
            ),
            "outline" => (
                format!(
                    "inline-flex items-center gap-2 {} font-imperial uppercase tracking-[0.14em] rounded cursor-pointer bg-transparent text-accent border border-accent transition-colors {}",
                    size_classes, focus_classes
                ),
                String::new(),
            ),
            "ghost" => (
                format!(
                    "inline-flex items-center gap-2 {} font-ui rounded cursor-pointer bg-transparent text-secondary hover:text-primary transition-colors {}",
                    size_classes, focus_classes
                ),
                String::new(),
            ),
            _ => (
                format!(
                    "inline-flex items-center gap-2 {} font-imperial uppercase tracking-[0.14em] rounded cursor-pointer transition-colors {}",
                    size_classes, focus_classes
                ),
                "background: linear-gradient(180deg, var(--gold-1, var(--color-accent)) 0%, var(--gold-2, var(--color-accent)) 50%, var(--gold-3, var(--color-accent)) 100%); color: var(--ink, var(--t-accent-contrast)); border: 1px solid var(--gold-deep, var(--border-outline));".to_string(),
            ),
        };
        view! {
            <button
                type="button"
                class=btn_class
                style=btn_style
                on:click=move |_| { if let Some(cb) = on_click { cb.run(()); } }
            >
                {children()}
            </button>
        }.into_any()
    }
}

/// Region-aware mode toggle for /draft, /solo, /team/dashboard routes.
///
/// Demacia: segmented control with gilt rounded pill border, Cinzel uppercase labels.
/// Pandemonium: flat tab-pull with bracket corners, mono uppercase labels with underscores.
///
/// Props:
/// - `region`: "demacia" | "pandemonium"
/// - `current`: `ReadSignal<String>` — active mode value (signal-driven for optimistic updates)
/// - `options`: `Vec<(value, demacia_label, pandemonium_label)>`
/// - `on_select`: `Callback<String>` — fired when user picks a mode
///
/// G-12 compliant: all buttons have focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none.
#[component]
pub fn ModeToggle(
    region: String,
    /// Current mode value — signal-driven so active state stays in sync after server fn updates
    current: ReadSignal<String>,
    /// Mode options as (value, demacia_label, pandemonium_label) tuples
    #[prop(into)]
    options: Vec<(String, String, String)>,
    /// Callback fired when the user picks a mode
    on_select: Callback<String>,
) -> impl IntoView {
    let is_pandemonium = region == "pandemonium";
    // Store options in a StoredValue so both move closures can access without ownership issues
    let opts = StoredValue::new(options);

    move || {
        let options_snapshot = opts.get_value();
        if is_pandemonium {
            view! {
                <div class="inline-flex items-stretch gap-0 bg-elevated rounded-none border border-accent/30">
                    {options_snapshot.into_iter().map(|(value, _dem, pan)| {
                        let v_for_class = value.clone();
                        let v_for_aria = value.clone();
                        let v_for_click = value.clone();
                        view! {
                            <button
                                type="button"
                                class=move || {
                                    let active = current.get() == v_for_class;
                                    if active {
                                        "px-4 py-2 font-mono text-[12px] uppercase tracking-[0.12em] bg-accent text-accent-contrast rounded-none cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                    } else {
                                        "px-4 py-2 font-mono text-[12px] uppercase tracking-[0.12em] text-muted hover:text-secondary cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                    }
                                }
                                aria-pressed=move || (current.get() == v_for_aria).to_string()
                                on:click=move |_| on_select.run(v_for_click.clone())
                            >
                                {pan}
                            </button>
                        }
                    }).collect_view()}
                </div>
            }.into_any()
        } else {
            // Demacia: segmented control with gilt pill border
            view! {
                <div
                    class="inline-flex items-stretch gap-0 rounded-full border border-outline/50 overflow-hidden p-px"
                    style="background: linear-gradient(180deg, var(--gold-1, var(--color-accent)) 0%, var(--gold-2, var(--color-accent)) 50%, var(--gold-3, var(--color-accent)) 100%);"
                >
                    <div class="inline-flex items-stretch gap-0 rounded-full bg-base">
                        {options_snapshot.into_iter().map(|(value, dem, _pan)| {
                            let v_for_class = value.clone();
                            let v_for_aria = value.clone();
                            let v_for_click = value.clone();
                            view! {
                                <button
                                    type="button"
                                    class=move || {
                                        let active = current.get() == v_for_class;
                                        if active {
                                            "px-4 py-1.5 rounded-full font-imperial text-[12px] uppercase tracking-[0.14em] bg-accent text-accent-contrast cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                        } else {
                                            "px-4 py-1.5 rounded-full font-imperial text-[12px] uppercase tracking-[0.14em] text-muted hover:text-secondary cursor-pointer focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:outline-none"
                                        }
                                    }
                                    aria-pressed=move || (current.get() == v_for_aria).to_string()
                                    on:click=move |_| on_select.run(v_for_click.clone())
                                >
                                    {dem}
                                </button>
                            }
                        }).collect_view()}
                    </div>
                </div>
            }.into_any()
        }
    }
}

/// Region-neutral tone badge.
/// `tone`: "neutral" | "accent" | "success" | "warning" | "danger" | "info"
/// Uses JetBrains Mono, uppercase, wide tracking. Same appearance both regions.
///
/// Note: "success", "warning", "info" tokens may not be defined in input.css — fallback to
/// "bg-elevated text-muted" for missing tones with a comment below.
#[component]
pub fn Badge(
    #[prop(optional, into)] tone: Option<String>,
    children: ChildrenFn,
) -> impl IntoView {
    // Map tone to semantic token bg+text classes.
    // success/warning/info: use token if defined in input.css; else fallback to bg-elevated text-muted.
    // (As of Phase 17, input.css defines --color-danger but not --color-success/warning/info.
    //  Fallback class is used until those tokens are added.)
    let tone_class = match tone.as_deref().unwrap_or("neutral") {
        "accent" => "bg-accent text-accent-contrast",
        "danger" => "bg-danger/15 text-danger",
        "success" => "bg-elevated text-muted",   // fallback: --color-success not yet defined
        "warning" => "bg-elevated text-muted",   // fallback: --color-warning not yet defined
        "info" => "bg-elevated text-muted",      // fallback: --color-info not yet defined
        _ => "bg-elevated text-muted",           // neutral
    };

    view! {
        <span class=format!(
            "inline-flex items-center gap-1 px-2 py-0.5 font-mono text-[10px] uppercase tracking-[0.12em] rounded-sm {}",
            tone_class
        )>
            {children()}
        </span>
    }
}
