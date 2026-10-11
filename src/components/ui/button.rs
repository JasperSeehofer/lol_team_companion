//! Button: the one control for actions (and, with `href`, for links that
//! look like actions).

use leptos::prelude::*;

use super::text::join;

/// Visual weight of a [`Button`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    /// Chalk fill. At most one per view: the next thing to do.
    Primary,
    /// Outline. The default for everything else.
    #[default]
    Secondary,
    /// No edge until hovered: toolbars, inline actions, "Cancel".
    Quiet,
    /// Destructive. Outline and text in danger; on hover a dark crimson
    /// fill with primary text (never a pastel fill).
    Danger,
}

impl ButtonVariant {
    pub fn class(self) -> &'static str {
        match self {
            ButtonVariant::Primary => {
                "bg-accent border-accent text-accent-ink font-semibold hover:bg-accent-hover hover:border-accent-hover data-[force=hover]:bg-accent-hover data-[force=hover]:border-accent-hover active:bg-accent data-[force=active]:bg-accent"
            }
            ButtonVariant::Secondary => {
                "bg-transparent border-outline text-primary hover:bg-ink-3 data-[force=hover]:bg-ink-3 active:bg-ink-2 data-[force=active]:bg-ink-2"
            }
            ButtonVariant::Quiet => {
                "bg-transparent border-transparent text-secondary hover:bg-ink-3 hover:text-primary data-[force=hover]:bg-ink-3 data-[force=hover]:text-primary active:bg-ink-2 data-[force=active]:bg-ink-2"
            }
            ButtonVariant::Danger => {
                "bg-transparent border-danger text-danger hover:bg-danger-deep hover:text-primary data-[force=hover]:bg-danger-deep data-[force=hover]:text-primary active:bg-danger-deep/80 data-[force=active]:bg-danger-deep/80"
            }
        }
    }
}

/// Height and padding of a [`Button`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    /// 28px: dense rows and toolbars.
    Sm,
    /// 32px: the default.
    #[default]
    Md,
    /// 40px: the one primary action on a form page.
    Lg,
}

impl ButtonSize {
    pub fn class(self) -> &'static str {
        match self {
            ButtonSize::Sm => "h-7 px-2.5 text-label",
            ButtonSize::Md => "h-8 px-3 text-label",
            ButtonSize::Lg => "h-10 px-4 text-body",
        }
    }
}

/// The `type` attribute of a [`Button`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonType {
    #[default]
    Button,
    Submit,
    Reset,
}

impl ButtonType {
    pub fn as_str(self) -> &'static str {
        match self {
            ButtonType::Button => "button",
            ButtonType::Submit => "submit",
            ButtonType::Reset => "reset",
        }
    }
}

/// A forced interaction state, used by the gallery so screenshots show
/// hover, focus and pressed without a mouse. Never set it in a page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Force {
    Hover,
    Focus,
    Active,
}

impl Force {
    pub fn as_str(self) -> &'static str {
        match self {
            Force::Hover => "hover",
            Force::Focus => "focus",
            Force::Active => "active",
        }
    }
}

pub(crate) const BASE: &str = "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-ctl border font-medium transition-colors cursor-pointer select-none no-underline disabled:cursor-not-allowed disabled:opacity-50 aria-busy:cursor-progress";

/// A button, or a link styled as one when `href` is set.
///
/// A11y: a native `<button type="button">` by default (Enter and Space
/// work, focus is visible through the global 2px chalk outline). While
/// `loading` it is disabled and sets `aria-busy`; the spinner is decorative
/// and the label stays. `shortcut` draws a keycap and sets
/// `aria-keyshortcuts`; the keycap itself is hidden from screen readers.
/// An icon is decorative: icon-only buttons must pass `aria_label`.
#[component]
pub fn Button(
    #[prop(optional)] variant: ButtonVariant,
    #[prop(optional)] size: ButtonSize,
    #[prop(optional)] kind: ButtonType,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] loading: Signal<bool>,
    #[prop(optional)] on_click: Option<Callback<leptos::ev::MouseEvent>>,
    /// Render an `<a href>` instead of a `<button>`.
    #[prop(optional, into)]
    href: Option<String>,
    /// Leading icon (decorative).
    #[prop(optional, into)]
    icon: Option<ViewFn>,
    /// Keycap hint, for example "Enter" or "P".
    #[prop(optional)]
    shortcut: Option<&'static str>,
    /// Accessible name for icon-only buttons.
    #[prop(optional, into)]
    aria_label: Option<String>,
    #[prop(optional)] force: Option<Force>,
    /// Stretch to the container width.
    #[prop(optional)]
    full_width: bool,
    children: Children,
) -> impl IntoView {
    let class = join(&[
        BASE,
        size.class(),
        variant.class(),
        if full_width { "w-full" } else { "" },
    ]);
    let force = force.map(Force::as_str);
    let icon_view = icon.map(|i| {
        view! { <span class="inline-flex shrink-0" aria-hidden="true">{i.run()}</span> }
    });
    let keycap = shortcut.map(|k| {
        view! {
            <kbd
                class="ml-1 inline-grid place-items-center min-w-[1.5em] h-[1.5em] px-1 rounded-ctl border border-current/40 border-b-2 text-caption font-medium leading-none opacity-80"
                aria-hidden="true"
            >
                {k}
            </kbd>
        }
    });

    match href {
        Some(href) => view! {
            <a
                href=href
                class=class
                data-force=force
                aria-label=aria_label
                aria-keyshortcuts=shortcut
            >
                {icon_view}
                {children()}
                {keycap}
            </a>
        }
        .into_any(),
        None => view! {
            <button
                type=kind.as_str()
                class=class
                data-force=force
                aria-label=aria_label
                aria-keyshortcuts=shortcut
                disabled=move || disabled.get() || loading.get()
                aria-busy=move || loading.get().then_some("true")
                on:click=move |ev| {
                    if let Some(cb) = on_click {
                        cb.run(ev);
                    }
                }
            >
                {move || loading.get().then(|| view! {
                    <span
                        class="size-3.5 shrink-0 rounded-full border-2 border-current border-r-transparent animate-spin motion-reduce:animate-none"
                        aria-hidden="true"
                    ></span>
                })}
                {icon_view}
                {children()}
                {keycap}
            </button>
        }
        .into_any(),
    }
}

/// A keycap, for shortcut legends next to controls.
///
/// A11y: a `<kbd>`; screen readers read its text. Do not use it as the only
/// way to learn a shortcut that a control already exposes through
/// `aria-keyshortcuts`.
#[component]
pub fn Keycap(keys: &'static str) -> impl IntoView {
    view! {
        <kbd class="inline-grid place-items-center min-w-[1.6em] h-[1.6em] px-1.5 rounded-ctl border border-outline border-b-2 bg-ink-2 text-caption font-medium leading-none text-secondary">
            {keys}
        </kbd>
    }
}
